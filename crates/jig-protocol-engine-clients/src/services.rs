//! Service credentials and sign-in digests use separate owners (client.md, 8).
use super::{
    Box, Component, Env, Kept, Limits, Queue, ReplyTo, Request, SecretOwner, SecretsAnswer, SecretsRequest, Token,
    Waiting, core, people, routes, sign_in,
};
use crate::sign_in::{Answer, CarriedRequest};
use skein_lib::{Secret, Wall};
impl Component {
    pub(super) fn secrets(&mut self, env: &Env<Limits>, answer: SecretsAnswer, out: &mut Queue<Request>) {
        match answer {
            SecretsAnswer::Kept { owner } => {
                let mut found = None;
                for (exchange, waiting) in &self.pending.exchanges {
                    if let Waiting::Keep { owner: wanted, .. } = waiting
                        && *wanted == owner
                    {
                        found = Some(*exchange);
                        break;
                    }
                }
                if let Some(exchange) = found
                    && let Some(Waiting::Keep { answer, .. }) = self.pending.exchanges.remove(&exchange)
                {
                    match answer {
                        Answer::SignedIn(value) => {
                            self.respond(env, exchange, routes::signed_in(&env.limits.wire, value));
                        }
                        Answer::Minted { credential, expires } => {
                            let exchange = self.pending.mints.remove(&exchange).expect("mint owner");
                            out.push(Request::Minted { exchange, credential, expires });
                        }
                        Answer::Service(value) => {
                            self.answer(env, exchange, Ok(value), out);
                            self.replayed(env, owner, out);
                        }
                    }
                }
            }
            SecretsAnswer::Got { owner, value } => {
                // Gets answer in the same iterate; waiting gets of one owner can use it.
                for _ in 0..env.limits.exchanges {
                    let mut found = None;
                    for (exchange, waiting) in &self.pending.exchanges {
                        let wanted = match waiting {
                            Waiting::Get { owner, .. } => Some(*owner),
                            Waiting::Made { party, .. } => Some(SecretOwner::Service(*party)),
                            Waiting::Core
                            | Waiting::SignOut { .. }
                            | Waiting::Watch { .. }
                            | Waiting::Keep { .. }
                            | Waiting::Ask { .. }
                            | Waiting::Carry { .. }
                            | Waiting::Read { .. }
                            | Waiting::ApplicationRead { .. }
                            | Waiting::Checked { .. } => None,
                        };
                        if wanted == Some(owner) {
                            found = Some(*exchange);
                            break;
                        }
                    }
                    let Some(exchange) = found else { break };
                    match self.pending.exchanges.remove(&exchange).expect("found get") {
                        Waiting::Get { digest, body, route, .. } => {
                            if sign_in::accepted(value.as_ref(), &digest, env.wall) {
                                match owner {
                                    SecretOwner::SignIn(sign_in) => {
                                        self.pending
                                            .exchanges
                                            .insert(exchange, Waiting::Checked { sign_in, body, route })
                                            .expect("same get slot");
                                    }
                                    SecretOwner::Service(party) => self.service_sign_in(env, exchange, party, out),
                                    SecretOwner::Carried(_) => unreachable!("carried values are only erased"),
                                }
                            } else {
                                match owner {
                                    SecretOwner::SignIn(_) => self.denied(env, exchange, route),
                                    SecretOwner::Service(_) => {
                                        self.respond(env, exchange, routes::refused(&env.limits.wire));
                                    }
                                    SecretOwner::Carried(_) => unreachable!("carried values are only erased"),
                                }
                            }
                        }
                        Waiting::Made { sign_in, party } => {
                            self.made(env, exchange, sign_in, party, value.as_ref(), out);
                        }
                        Waiting::Core
                        | Waiting::SignOut { .. }
                        | Waiting::Watch { .. }
                        | Waiting::Keep { .. }
                        | Waiting::Ask { .. }
                        | Waiting::Carry { .. }
                        | Waiting::Read { .. }
                        | Waiting::ApplicationRead { .. }
                        | Waiting::Checked { .. } => {}
                    }
                }
            }
            SecretsAnswer::Carried { exchange, number } => self.carried(env, exchange, number, out),
            SecretsAnswer::Failed => self.abort(out),
            SecretsAnswer::Erased { owner } => self.erased(env, owner),
        }
    }
    /// The store kept a request's carried value: the core sees the request
    /// now; one whose exchange is gone, or not kept, is answered or erased.
    fn carried(&mut self, env: &Env<Limits>, exchange: Token, number: Option<u64>, out: &mut Queue<Request>) {
        let Some(slot) = self.pending.carried.get_mut(&exchange) else {
            unreachable!("the store answers each carry once")
        };
        let waiting = self.pending.exchanges.remove(&exchange);
        let Some(number) = number else {
            self.pending.carried.remove(&exchange);
            if let Some(waiting) = waiting {
                if let Waiting::Carry { .. } = waiting {
                    self.respond(env, exchange, routes::busy());
                } else {
                    self.pending.exchanges.insert(exchange, waiting).expect("same slot");
                }
            }
            return;
        };
        *slot = Some(number);
        match waiting {
            Some(Waiting::Carry { sign_in, family, version, request }) => {
                let request = match *request {
                    CarriedRequest::Application(mut request) => {
                        match &mut request {
                            crate::ToApplication::Request { carried: Some(carried), .. } => {
                                carried.number = number;
                            }
                            crate::ToApplication::Request { carried: None, .. }
                            | crate::ToApplication::Read { .. }
                            | crate::ToApplication::Watch { .. } => unreachable!("only a carrying request is kept"),
                        }
                        Request::Application(request)
                    }
                    CarriedRequest::AccountKey { key, ask, digest } => Request::Core(Box::new(core::Event::Key {
                        reply_to: ReplyTo::new(exchange),
                        sign_in,
                        key,
                        ask,
                        carried: core::Carried { number, digest },
                    })),
                };
                self.pending
                    .exchanges
                    .insert(exchange, Waiting::Ask { sign_in, family, version })
                    .expect("same carry slot");
                out.push(request);
            }
            Some(waiting) => {
                self.pending.exchanges.insert(exchange, waiting).expect("same slot");
                self.unused(exchange, out);
            }
            None => self.unused(exchange, out),
        }
    }
    fn service_sign_in(&mut self, env: &Env<Limits>, exchange: Token, party: u64, out: &mut Queue<Request>) {
        if self.stopping() {
            self.respond(env, exchange, routes::sign_in_not_ready());
            return;
        }
        if self.admitted() >= env.limits.exchanges {
            self.respond(env, exchange, routes::refused(&env.limits.wire));
            return;
        }
        let identity = people::Identity {
            key: people::IdentityKey {
                provider: self.settings.deployment_provider,
                subject: Box::new(party.to_be_bytes()),
            },
            login: Box::new([]),
            name: Box::new([]),
        };
        self.pending.exchanges.insert(exchange, Waiting::Core).expect("same get slot");
        out.push(Request::Core(Box::new(core::Event::SignIn { reply_to: ReplyTo::new(exchange), identity })));
    }
    fn made(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        party: u64,
        kept: Option<&Kept>,
        out: &mut Queue<Request>,
    ) {
        let owner = SecretOwner::Service(party);
        let mut keeping = false;
        for (_, waiting) in &self.pending.exchanges {
            if let Waiting::Keep { owner: wanted, .. } = waiting
                && *wanted == owner
            {
                keeping = true;
                break;
            }
        }
        if keeping {
            self.pending.exchanges.insert(exchange, Waiting::Made { sign_in, party }).expect("same made slot");
            return;
        }
        if kept.is_some() {
            self.answer(
                env,
                exchange,
                crate::translate::answers::service(party, Secret::new(Box::new([])), &env.limits.wire),
                out,
            );
            return;
        }
        let credential = sign_in::token(&mut self.rng, party);
        let digest = sign_in::digest(credential.expose());
        let answer = crate::translate::answers::service(party, credential, &env.limits.wire);
        match answer {
            Ok(answer) => {
                self.pending
                    .exchanges
                    .insert(exchange, Waiting::Keep { sign_in, owner, answer: Answer::Service(answer) })
                    .expect("same made slot");
                out.push(Request::Secrets(SecretsRequest::Keep {
                    owner,
                    value: Kept { digest, expires: Wall::from_nanos(u64::MAX) },
                }));
            }
            Err(problem) => self.answer(env, exchange, Err(problem), out),
        }
    }
    fn replayed(&mut self, env: &Env<Limits>, owner: SecretOwner, out: &mut Queue<Request>) {
        for _ in 0..env.limits.exchanges {
            let mut found = None;
            for (exchange, waiting) in &self.pending.exchanges {
                if let Waiting::Made { party, .. } = waiting
                    && owner == SecretOwner::Service(*party)
                {
                    found = Some((*exchange, *party));
                    break;
                }
            }
            let Some((exchange, party)) = found else { break };
            self.pending.exchanges.remove(&exchange);
            self.answer(
                env,
                exchange,
                crate::translate::answers::service(party, Secret::new(Box::new([])), &env.limits.wire),
                out,
            );
        }
    }
}
