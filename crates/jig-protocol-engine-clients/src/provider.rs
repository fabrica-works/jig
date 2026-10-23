//! Volatile codes and access tokens; authorization order and claims (protocol/client.md, 18).
use super::Component;
use crate::{Limits, ProviderAnswer, Reply, Request, exchange::Exchange, routes, sign_in::Waiting};
use alloc::boxed::Box;
use jig_client_protocol_wire::v19 as w;
use jig_domain_people as people;
use skein_lib::{Decimal, Duration, Env, Map, Queue, Reader, ReplyTo, Secret, Time, Token, Writer, bytes, sha256};
use skein_oauth_issuer::{Authorization, Email, Person, Refusal, TokenError};

const CODE_LIFE: Duration = Duration::from_secs(60);
const TOKEN_LIFE: Duration = Duration::from_secs(300);
#[derive(Debug)]
pub(crate) struct Code {
    until: Time,
    authorization: Authorization,
    person: Person,
}
#[derive(Debug)]
pub(crate) struct Access {
    until: Time,
    person: Person,
}
#[derive(Debug)]
pub(super) struct State {
    codes: Map<[u8; 32], Code>,
    tokens: Map<[u8; 32], Access>,
}
impl State {
    pub(super) fn new(limits: &Limits) -> Self {
        Self { codes: Map::with_capacity(limits.provider_codes), tokens: Map::with_capacity(limits.provider_tokens) }
    }
    fn sweep(&mut self, now: Time) {
        for _ in 0..self.codes.capacity() {
            let mut expired = None;
            for (key, code) in &self.codes {
                if code.until <= now {
                    expired = Some(*key);
                    break;
                }
            }
            let Some(key) = expired else { break };
            self.codes.remove(&key);
        }
        for _ in 0..self.tokens.capacity() {
            let mut expired = None;
            for (key, token) in &self.tokens {
                if token.until <= now {
                    expired = Some(*key);
                    break;
                }
            }
            let Some(key) = expired else { break };
            self.tokens.remove(&key);
        }
    }
}
impl Component {
    /// Check the query before any credential is read, even for a signed-out request.
    pub(super) fn authorize_query(&mut self, env: &Env<Limits>, connection: &mut Exchange, body: &[u8]) -> bool {
        self.provider.sweep(env.now);
        let mut reader = Reader::new(body);
        let Ok(record) = w::Authorize::decode(&env.limits.wire, &mut reader) else {
            connection.respond(Reply::NotARecord, &env.limits);
            return false;
        };
        if !reader.is_empty() {
            connection.respond(Reply::NotARecord, &env.limits);
            return false;
        }
        let answer = match &self.settings.provider {
            None => Some(refused(&env.limits, w::AuthorizeWhy::NoProvider)),
            Some(issuer) => match issuer.authorize(record.query()) {
                Ok(_) => None,
                Err(Refusal::Malformed) => Some(refused(&env.limits, w::AuthorizeWhy::Malformed)),
                Err(Refusal::UnknownClient) => Some(refused(&env.limits, w::AuthorizeWhy::UnknownClient)),
                Err(Refusal::UnknownRedirect) => Some(refused(&env.limits, w::AuthorizeWhy::UnknownRedirect)),
                Err(Refusal::Back { location }) => Some(go(&env.limits, location)),
            },
        };
        if let Some(answer) = answer {
            connection.respond(Reply::Record(routes::authorize_answer(answer)), &env.limits);
            return false;
        }
        true
    }
    pub(super) fn authorize_checked(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        if self.stopping() {
            self.respond(env, exchange, routes::authorize_answer(w::AuthorizeAnswer::NotReady));
            return;
        }
        let issuer = self.settings.provider.as_ref().expect("provider query checked");
        let record = w::Authorize::decode(&env.limits.wire, &mut Reader::new(body)).expect("checked authorize record");
        let authorization = issuer.authorize(record.query()).expect("same immutable issuer and checked query");
        self.pending.exchanges.insert(exchange, Waiting::Authorize { sign_in, authorization }).expect("same slot");
        out.push(Request::Core(Box::new(jig_domain_core::Event::People(people::Event::Whose {
            reply_to: ReplyTo::new(exchange),
            sign_in,
        }))));
    }
    pub(super) fn authorize_reply(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        reply: people::Reply,
        authorization: Authorization,
    ) {
        let answer = match reply {
            people::Reply::Signer(Some(signer)) => self.authorize_person(env, authorization, signer),
            people::Reply::Signer(None) => w::AuthorizeAnswer::SignedOut,
            people::Reply::Full { .. }
            | people::Reply::SignedIn { .. }
            | people::Reply::SignedOut
            | people::Reply::Outcome(_)
            | people::Reply::Again(_)
            | people::Reply::Refused(_) => w::AuthorizeAnswer::NotReady,
        };
        self.respond(env, exchange, routes::authorize_answer(answer));
    }
    fn authorize_person(
        &mut self,
        env: &Env<Limits>,
        authorization: Authorization,
        signer: people::Signer,
    ) -> w::AuthorizeAnswer {
        if self.stopping() {
            return w::AuthorizeAnswer::NotReady;
        }
        let Some(person) = person(signer, &self.settings.addresses, &env.limits) else {
            return refused(&env.limits, w::AuthorizeWhy::NotAPerson);
        };
        self.provider.sweep(env.now);
        // Every live code reserves an access-token slot, including while Whose was outstanding.
        if self.provider.codes.len() >= env.limits.provider_codes
            || self.provider.tokens.len().saturating_add(self.provider.codes.len()) >= env.limits.provider_tokens
        {
            return w::AuthorizeAnswer::Busy;
        }
        let code = draw(&mut self.rng);
        let issuer = self.settings.provider.as_ref().expect("provider query checked");
        let location = issuer.redirect(&authorization, &code).expect("issuer checked authorization and bounded code");
        self.provider
            .codes
            .insert(
                sha256::digest(code.expose()),
                Code { until: env.now.saturating_add(CODE_LIFE), authorization, person },
            )
            .expect("code room checked");
        go(&env.limits, location)
    }
    #[expect(clippy::manual_map, reason = "step code matches optional headers without closures")]
    pub(super) fn redeem(
        &mut self,
        env: &Env<Limits>,
        authorization: Option<&Secret>,
        content_type: Option<&[u8]>,
        body: &[u8],
    ) -> ProviderAnswer {
        self.provider.sweep(env.now);
        let Some(issuer) = &self.settings.provider else {
            return answer(TokenError::InvalidClient { header: authorization.is_some() }.answer());
        };
        let header = match authorization {
            Some(value) => Some(value.expose()),
            None => None,
        };
        let request = match issuer.token_request(header, content_type, body) {
            Ok(request) => request,
            Err(error) => return answer(error.answer()),
        };
        // Removal precedes every remaining check: a presented code is spent even on a bad verifier or redirect.
        let Some(code) = self.provider.codes.remove(&sha256::digest(request.code.expose())) else {
            return answer(TokenError::InvalidGrant.answer());
        };
        if let Err(error) = issuer.redeem(&request, &code.authorization) {
            return answer(error.answer());
        }
        let access = draw(&mut self.rng);
        let body = issuer
            .tokens(&code.authorization, &code.person, &access, TOKEN_LIFE, env.wall)
            .expect("checked issuer, bounded identity and generated token");
        self.provider
            .tokens
            .insert(
                sha256::digest(access.expose()),
                Access { until: env.now.saturating_add(TOKEN_LIFE), person: code.person },
            )
            .expect("every code reserved a token slot");
        ProviderAnswer { status: 200, challenge: None, body: Secret::new(body) }
    }
    #[expect(clippy::manual_map, reason = "step code matches optional headers without closures")]
    pub(super) fn claims(&mut self, env: &Env<Limits>, authorization: Option<&Secret>) -> ProviderAnswer {
        self.provider.sweep(env.now);
        let Some(issuer) = &self.settings.provider else {
            return answer(TokenError::InvalidClient { header: authorization.is_some() }.answer());
        };
        let header = match authorization {
            Some(value) => Some(value.expose()),
            None => None,
        };
        if let Some(token) = issuer.bearer(header)
            && let Some(access) = self.provider.tokens.get(&sha256::digest(token.expose()))
        {
            let body = issuer.userinfo(&access.person).expect("checked identity claims");
            return ProviderAnswer { status: 200, challenge: None, body: Secret::new(body) };
        }
        answer(issuer.userinfo_refused())
    }
}
fn refused(limits: &Limits, why: w::AuthorizeWhy) -> w::AuthorizeAnswer {
    w::AuthorizeAnswer::Refused(
        w::AuthorizeRefused::new(&limits.wire, w::AuthorizeRefusedParts { why }).expect("fixed refusal"),
    )
}
fn go(limits: &Limits, location: Box<[u8]>) -> w::AuthorizeAnswer {
    w::AuthorizeAnswer::Go(
        w::AuthorizeGo::new(&limits.wire, w::AuthorizeGoParts { location })
            .expect("provider location bound checked at startup"),
    )
}
fn draw(rng: &mut skein_lib::CryptoRng) -> Secret {
    let raw = crate::sign_in::draw(rng);
    let mut writer = Writer::new(43);
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Url, raw.expose(), &mut writer)
        .expect("32 bytes encode in 43");
    Secret::new(writer.finish())
}
fn answer(value: skein_oauth_issuer::Answer) -> ProviderAnswer {
    ProviderAnswer { status: value.status, challenge: value.challenge, body: Secret::new(value.body) }
}
#[expect(
    clippy::disallowed_methods,
    reason = "the provider protocol must refuse identities whose claims are not UTF-8 (client.md, 18)"
)]
fn person(signer: people::Signer, addresses: &[u16], limits: &Limits) -> Option<Person> {
    if signer.kind != people::Kind::Person {
        return None;
    }
    let identity = signer.identity;
    let bound = usize::try_from(limits.identity_bytes).expect("u32 fits");
    for value in [&identity.key.subject, &identity.login, &identity.name] {
        if value.len() > bound || core::str::from_utf8(value).is_err() {
            return None;
        }
    }
    let number = Decimal::of(u64::from(identity.key.provider));
    let mut subject = Writer::new(number.as_bytes().len().checked_add(1)?.checked_add(identity.key.subject.len())?);
    subject.put(number.as_bytes()).expect("measured subject");
    subject.put(b":").expect("measured subject");
    subject.put(&identity.key.subject).expect("measured subject");
    let name = if identity.name.is_empty() { None } else { Some(identity.name) };
    let email = if addresses.contains(&identity.key.provider) {
        Some(Email { address: bytes::copy_of(&identity.key.subject), verified: true })
    } else {
        None
    };
    Some(Person { subject: subject.finish(), username: Some(identity.login), email, name })
}
