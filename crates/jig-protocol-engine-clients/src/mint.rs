//! Application-verified identities mint credentials only after durability (client.md, 8).
use super::{
    Box, Component, Env, Kept, Limits, Queue, ReplyTo, Request, SecretOwner, SecretsRequest, Token, Waiting, core,
    people, routes, sign_in, w,
};
use crate::MintRefusal;
impl Component {
    pub(super) fn mint(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        identity: people::Identity,
        out: &mut Queue<Request>,
    ) {
        let why = if self.stopping() {
            Some(MintRefusal::NotReady)
        } else if !application_provider(&self.settings.providers, identity.key.provider) {
            Some(MintRefusal::NotApplication)
        } else if !identity_fits(&identity, env.limits.identity_bytes) {
            Some(MintRefusal::Bound)
        } else if self.pending.mints.len() == env.limits.mints || self.admitted() >= env.limits.exchanges {
            Some(MintRefusal::Busy)
        } else {
            None
        };
        if let Some(why) = why {
            out.push(Request::MintRefused { exchange, why });
            return;
        }
        let name = self.name();
        self.pending.mints.insert(name, exchange).expect("admitted mint");
        self.pending.exchanges.insert(name, Waiting::Core).expect("admitted exchange");
        out.push(Request::Core(Box::new(core::Event::SignIn { reply_to: ReplyTo::new(name), identity })));
    }
    pub(super) fn mint_refused(&mut self, name: Token, why: MintRefusal, out: &mut Queue<Request>) -> bool {
        if let Some(exchange) = self.pending.mints.remove(&name) {
            self.pending.exchanges.remove(&name);
            out.push(Request::MintRefused { exchange, why });
            true
        } else {
            false
        }
    }
    pub(super) fn sign_in_reply(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        reply: people::Reply,
        out: &mut Queue<Request>,
    ) {
        if self.stopping() && self.mint_refused(exchange, MintRefusal::NotReady, out) {
            return;
        }
        match reply {
            people::Reply::SignedIn { person, sign_in, expires } => {
                let token = sign_in::token(&mut self.rng, sign_in);
                let digest = sign_in::digest(token.expose());
                let answer = if self.pending.mints.contains_key(&exchange) {
                    sign_in::Answer::Minted { credential: token, expires }
                } else {
                    let value = w::SignedIn::new(
                        &env.limits.wire,
                        w::SignedInParts { token, party: person, expires: expires.as_nanos() },
                    )
                    .expect("40-byte token admitted");
                    sign_in::Answer::SignedIn(value)
                };
                self.pending
                    .exchanges
                    .insert(exchange, Waiting::Keep { sign_in, owner: SecretOwner::SignIn(sign_in), answer })
                    .expect("same slot");
                out.push(Request::Secrets(SecretsRequest::Keep {
                    owner: SecretOwner::SignIn(sign_in),
                    value: Kept { digest, expires },
                }));
            }
            people::Reply::Again(why) => {
                let refusal = match why {
                    people::Again::NotReady => MintRefusal::NotReady,
                    people::Again::Busy => MintRefusal::Busy,
                };
                if !self.mint_refused(exchange, refusal, out) {
                    let answer = if refusal == MintRefusal::NotReady {
                        routes::sign_in_not_ready()
                    } else {
                        routes::refused(&env.limits.wire)
                    };
                    self.respond(env, exchange, answer);
                }
            }
            people::Reply::SignedOut | people::Reply::Outcome(_) | people::Reply::Refused(_) => {
                if !self.mint_refused(exchange, MintRefusal::Bound, out) {
                    self.respond(env, exchange, routes::refused(&env.limits.wire));
                }
            }
        }
    }
}
#[expect(clippy::disallowed_methods, reason = "the protocol validates the application identity text (client.md, 8)")]
fn identity_fits(identity: &people::Identity, bound: u32) -> bool {
    let length = identity.key.subject.len().checked_add(identity.login.len());
    let length = match length {
        Some(length) => length.checked_add(identity.name.len()),
        None => None,
    };
    match length {
        Some(length) => {
            length <= usize::try_from(bound).expect("u32 fits")
                && ::core::str::from_utf8(&identity.login).is_ok()
                && ::core::str::from_utf8(&identity.name).is_ok()
        }
        None => false,
    }
}

fn application_provider(providers: &skein_lib::List<u16>, number: u16) -> bool {
    for provider in providers {
        if *provider == number {
            return true;
        }
    }
    false
}
