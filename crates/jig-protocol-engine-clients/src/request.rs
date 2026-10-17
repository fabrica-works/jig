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
                self.connections.get_mut(&owner).expect("exchange owner").respond(400, Box::new([]), &env.limits);
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
                    self.connections.get_mut(&owner).expect("exchange owner").respond(400, Box::new([]), &env.limits);
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
                    self.connections.get_mut(&owner).expect("exchange owner").respond(400, Box::new([]), &env.limits);
                    return;
                };
                let scope = crate::served::scope(record.scope());
                let digest = sign_in::digest(record.record()).expose().try_into().expect("32-byte digest");
                Request::Application(crate::ToApplication::Request {
                    reply_to: ReplyTo::new(exchange),
                    sign_in,
                    key: keyed.key,
                    family: family.number(),
                    version: keyed.version,
                    scope,
                    record: record.into_parts().record,
                    digest,
                })
            }
        };
        self.pending
            .exchanges
            .insert(exchange, Waiting::Ask { sign_in, family, version: keyed.version })
            .expect("get released slot");
        out.push(request);
    }
}
