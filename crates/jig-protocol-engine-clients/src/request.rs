//! Keyed request admission and version envelopes (protocol/client.md, 5 and 9).
use super::{Box, Component, Env, Limits, Queue, ReplyTo, Request, Token, Waiting, core, people, routes, sign_in, w};
impl Component {
    pub(super) fn request(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        if self.stopping() {
            self.respond(env, exchange, routes::not_ready());
            return;
        }
        let Some(owner) = self.owner(exchange) else {
            return;
        };
        let active = self.active(sign_in);
        if active >= env.limits.exchanges_per_client || self.admitted() >= env.limits.exchanges {
            self.respond(env, exchange, routes::busy());
            return;
        }
        let keyed = match w::Keyed::decode(&env.limits.wire, &mut skein_lib::Reader::new(body)) {
            Ok(value) => value.into_parts(),
            Err(_) => {
                self.not_a_record(env, owner);
                return;
            }
        };
        let family = match &keyed.owner {
            w::Owner::Jig => None,
            w::Owner::Family(value) => Some(value.number()),
        };
        if !self.serves(family, keyed.version) {
            let answer =
                w::KeyedAnswer::NotServed(crate::served::not_served(&self.settings, keyed.owner, &env.limits.wire));
            self.respond(env, exchange, routes::encoded(&answer));
            return;
        }
        let request = match keyed.owner {
            w::Owner::Jig if keyed.value.is_some() => {
                // jig's own requests carry no value yet: the key's source is
                // later work (llm.md), so nothing is kept.
                self.not_a_record(env, owner);
                return;
            }
            w::Owner::Jig => {
                let ask = match w::Ask::decode(&env.limits.wire, &mut skein_lib::Reader::new(&keyed.request)) {
                    Ok(value) => {
                        if let Ok(Some(answer)) = crate::translate::operations::refused(&value, &env.limits.wire) {
                            self.respond(env, exchange, routes::encoded(&answer));
                            return;
                        }
                        crate::translate::asks::ask(value, &env.limits.wire)
                    }
                    Err(problem) => Err(crate::translate::Untranslatable { path: problem.path }),
                };
                let Ok(ask) = ask else {
                    self.not_a_record(env, owner);
                    return;
                };
                Request::Core(Box::new(core::Event::People(people::Event::Ask {
                    reply_to: ReplyTo::new(exchange),
                    sign_in,
                    key: keyed.key,
                    ask,
                })))
            }
            w::Owner::Family(family) => {
                let Ok(record) =
                    w::FamilyRequest::decode(&env.limits.wire, &mut skein_lib::Reader::new(&keyed.request))
                else {
                    self.not_a_record(env, owner);
                    return;
                };
                let scope = crate::served::scope(record.scope());
                let (digest, carried) = match &keyed.value {
                    None => (sign_in::digest(record.record()).expose().try_into().expect("32-byte digest"), None),
                    Some(value) => (
                        sign_in::carrying(record.record(), value),
                        Some(core::Carried { number: 0, digest: skein_lib::sha256::digest(value.expose()) }),
                    ),
                };
                let request = crate::ToApplication::Request {
                    reply_to: ReplyTo::new(exchange),
                    sign_in,
                    key: keyed.key,
                    family: family.number(),
                    version: keyed.version,
                    scope,
                    record: record.into_parts().record,
                    digest,
                    carried,
                };
                if let Some(value) = keyed.value {
                    self.carry(env, exchange, sign_in, request, value, out);
                    return;
                }
                Request::Application(request)
            }
        };
        self.pending
            .exchanges
            .insert(exchange, Waiting::Ask { sign_in, family, version: keyed.version })
            .expect("get released slot");
        out.push(request);
    }
    fn not_a_record(&mut self, env: &Env<Limits>, owner: Token) {
        self.connections.get_mut(&owner).expect("exchange owner").respond(crate::Reply::NotARecord, &env.limits);
    }
    /// Keeps the value a request carried before the core sees the
    /// request, which names it by number and digest only
    /// (protocol/client.md, 5; store.md 9).
    fn carry(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        request: crate::ToApplication,
        value: skein_lib::Secret,
        out: &mut Queue<Request>,
    ) {
        if self.pending.carried.len() == self.pending.carried.capacity() {
            self.respond(env, exchange, routes::busy());
            return;
        }
        let crate::ToApplication::Request { family, version, .. } = &request else {
            unreachable!("only a request carries a value")
        };
        let waiting = Waiting::Carry { sign_in, family: *family, version: *version, request: Box::new(request) };
        self.pending.carried.insert(exchange, None).expect("carried below its capacity");
        self.pending.exchanges.insert(exchange, waiting).expect("get released slot");
        out.push(Request::Secrets(crate::SecretsRequest::Carry { exchange, value }));
    }
}
