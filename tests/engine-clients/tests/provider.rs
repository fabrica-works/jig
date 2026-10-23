#![expect(
    clippy::disallowed_types,
    reason = "the task requires real elapsed measurements; protocol deadlines use only the injected world clock"
)]
//! The application's page and endpoint routes around the real clients' component (client.md, 18).
use jig_client_protocol_wire::v19 as w;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_engine_clients_world::fixtures;
use jig_protocol_config::clients::{Provider, ProviderClient, ProviderUrls, provider_issuer};
use jig_protocol_engine_clients::{
    Component, Event, FromTransport, Kept, Message, ProviderAnswer, Reply, Request, SecretOwner, SecretsAnswer,
    SecretsRequest,
};
use serde_json::{Value, json};
use skein_lib::{
    Duration, Env, Queue, Reader, Secret, Time, Token, Wall, Writer,
    base64::{self, Alphabet},
};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    process::Command,
    sync::OnceLock,
    time::Instant,
};
const REDIRECT: &str = "https://app.example.com/callback";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
const PERSON_PROVIDER: u16 = 3;
// Two code slots, and two token slots: the story reaches both admission bounds independently.
const ROOM: u32 = 2;
struct Keys {
    pem: Secret,
    secret: Secret,
}
fn keys() -> &'static Keys {
    static KEYS: OnceLock<Keys> = OnceLock::new();
    KEYS.get_or_init(|| {
        let scratch = tempfile::tempdir().expect("private provider scratch");
        let path = scratch.path().join("provider.pem");
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path).expect("private key file");
        let started = Instant::now();
        let result = Command::new("openssl")
            .args(["genpkey", "-algorithm", "RSA", "-pkeyopt", "rsa_keygen_bits:3072", "-out"])
            .arg(&path)
            .output()
            .expect("openssl installed");
        assert!(result.status.success(), "key generation succeeds");
        eprintln!("provider world RSA 3072 key generation: {:?}", started.elapsed());
        assert_eq!(fs::metadata(&path).expect("key metadata").permissions().mode() & 0o777, 0o600);
        let pem = Secret::new(fs::read(path).expect("key file").into_boxed_slice());
        let path = scratch.path().join("client.secret");
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path).expect("private secret file");
        let result = Command::new("openssl")
            .args(["rand", "-hex", "-out"])
            .arg(&path)
            .arg("32")
            .output()
            .expect("openssl installed");
        assert!(result.status.success(), "client secret generation succeeds");
        let secret = jig_protocol_config::secret(fs::read(path).expect("client secret file").into_boxed_slice());
        Keys { pem, secret }
    })
}
fn urls() -> ProviderUrls {
    ProviderUrls {
        issuer: Box::from(b"https://app.example.com".as_slice()),
        public_origin: Box::from(b"https://app.example.com".as_slice()),
        authorization_endpoint: Box::from(b"https://app.example.com/authorize".as_slice()),
        token_endpoint: Box::from(b"https://app.example.com/token".as_slice()),
        userinfo_endpoint: Box::from(b"https://app.example.com/userinfo".as_slice()),
        jwks_uri: Box::from(b"https://app.example.com/keys".as_slice()),
    }
}
struct World {
    component: Component,
    env: Env<jig_protocol_engine_clients::Limits>,
    out: Queue<Request>,
    kept: BTreeMap<SecretOwner, Kept>,
    signer: Option<people::Signer>,
    credential: Option<Secret>,
    reply: Option<Reply>,
    endpoint: Option<ProviderAnswer>,
    next: u64,
    key_set: Value,
    secret: Secret,
}
impl World {
    fn new(provider: bool) -> Self {
        let mut limits = fixtures::limits();
        limits.provider_codes = ROOM;
        limits.provider_tokens = ROOM;
        let mut settings = fixtures::settings();
        settings.link = None;
        settings.addresses = Box::new([PERSON_PROVIDER]);
        settings.providers = skein_lib::List::with_capacity(1);
        settings.providers.push(PERSON_PROVIDER).expect("application provider room");
        let key_set = if provider {
            let configuration = Provider {
                key: Box::from(b"provider.pem".as_slice()),
                clients: Box::new([ProviderClient {
                    id: Box::from(b"app".as_slice()),
                    secret: Box::from(b"client.secret".as_slice()),
                    redirect_uri: REDIRECT.as_bytes().into(),
                }]),
                addresses: settings.addresses.clone(),
            };
            let issuer = provider_issuer(
                &configuration,
                &[
                    (configuration.key.clone(), keys().pem.clone()),
                    (configuration.clients[0].secret.clone(), keys().secret.clone()),
                ],
                urls(),
                &limits,
            )
            .expect("startup builds issuer");
            let discovery: Value = serde_json::from_slice(&issuer.discovery()).expect("discovery JSON");
            assert_eq!(discovery["issuer"], "https://app.example.com");
            let key_set = serde_json::from_slice(&issuer.key_set()).expect("key set JSON");
            settings.provider = Some(issuer);
            key_set
        } else {
            Value::Null
        };
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut out = Queue::with_capacity(64);
        let component = Component::new(settings, &env, &mut out).expect("component starts");
        Self {
            component,
            env,
            out,
            kept: BTreeMap::new(),
            signer: None,
            credential: None,
            reply: None,
            endpoint: None,
            next: 1,
            key_set,
            secret: keys().secret.clone(),
        }
    }
    fn send(&mut self, event: Event) {
        self.component.down(&self.env, event, &mut self.out);
        for _ in 0..256 {
            let active = self.component.up(&self.env, &mut self.out);
            let Some(request) = self.out.pop() else {
                if !active {
                    return;
                }
                continue;
            };
            match request {
                Request::Core(event) => {
                    if let core::Event::SignIn { reply_to, identity, .. } = *event {
                        self.signer = Some(people::Signer { identity, kind: people::Kind::Person });
                        self.component.down(
                            &self.env,
                            Event::Reply {
                                to: reply_to,
                                reply: people::Reply::SignedIn {
                                    person: 1,
                                    sign_in: 7,
                                    expires: Wall::from_nanos(u64::MAX),
                                },
                            },
                            &mut self.out,
                        );
                    } else {
                        let core::Event::People(people::Event::Whose { reply_to, .. }) = *event else {
                            panic!("unexpected core event");
                        };
                        self.component.down(
                            &self.env,
                            Event::Reply { to: reply_to, reply: people::Reply::Signer(self.signer.clone()) },
                            &mut self.out,
                        );
                    }
                }
                Request::Secrets(request) => {
                    let answer = match request {
                        SecretsRequest::Keep { owner, value } => {
                            self.kept.insert(owner, value);
                            SecretsAnswer::Kept { owner }
                        }
                        SecretsRequest::Get { owner } => {
                            SecretsAnswer::Got { owner, value: self.kept.get(&owner).cloned() }
                        }
                        SecretsRequest::Erase { owner } => {
                            self.kept.remove(&owner);
                            SecretsAnswer::Erased { owner }
                        }
                        SecretsRequest::Carry { .. } => panic!("no carrying request"),
                    };
                    self.component.down(&self.env, Event::Secrets(answer), &mut self.out);
                }
                Request::Transport(jig_protocol_engine_clients::ToTransport::Reply { reply, .. }) => {
                    self.reply = Some(reply);
                }
                Request::Redeemed { exchange, answer } | Request::Claimed { exchange, answer } => {
                    assert_eq!(exchange, Token::new(99));
                    self.endpoint = Some(answer);
                }
                Request::Minted { credential, .. } => self.credential = Some(credential),
                Request::Application(_)
                | Request::Link { .. }
                | Request::TakeMoved { .. }
                | Request::MintRefused { .. }
                | Request::Closed
                | Request::Transport(_) => panic!("unexpected provider story output"),
            }
        }
        panic!("provider did not settle");
    }
    fn sign_in(&mut self) {
        self.send(Event::Mint {
            exchange: Token::new(98),
            identity: people::Identity {
                key: people::IdentityKey {
                    provider: PERSON_PROVIDER,
                    subject: Box::from(b"alice@example.com".as_slice()),
                },
                login: Box::from(b"alice".as_slice()),
                name: Box::from(b"Alice".as_slice()),
            },
        });
    }
    fn authorize(&mut self, redirect: &str) -> w::AuthorizeAnswer {
        let query = format!(
            "client_id=app&redirect_uri={redirect}&response_type=code&scope=openid&code_challenge_method=S256&code_challenge={CHALLENGE}&state=back&nonce=once"
        );
        self.query(query.as_bytes())
    }
    fn query(&mut self, query: &[u8]) -> w::AuthorizeAnswer {
        let record =
            w::Authorize::new(&w::CEILINGS, w::AuthorizeParts { query: query.into() }).expect("bounded authorization");
        let mut bytes = Writer::new(usize::try_from(record.measure()).expect("short record"));
        record.encode(&mut bytes).expect("measured authorization");
        let exchange = Token::new(self.next);
        self.next += 1;
        self.send(Event::Transport(FromTransport::Exchange {
            exchange,
            credential: self.credential.clone(),
            message: Message::Authorize(bytes.finish()),
        }));
        let Reply::Record(body) = self.reply.take().expect("authorization answered") else {
            panic!("record answer");
        };
        w::AuthorizeAnswer::decode(&w::CEILINGS, &mut Reader::new(&body)).expect("authorization answer")
    }
    fn code(&mut self) -> Secret {
        let w::AuthorizeAnswer::Go(go) = self.authorize(REDIRECT) else {
            panic!("authorization granted");
        };
        let location = std::str::from_utf8(go.location()).expect("redirect UTF-8");
        assert!(location.ends_with("&state=back"));
        let code = location.split("code=").nth(1).expect("code parameter").split('&').next().expect("code");
        assert_eq!(code.len(), 43);
        Secret::new(code.as_bytes().into())
    }
    fn redeem(
        &mut self,
        code: &Secret,
        redirect: &str,
        verifier: &str,
        header: bool,
        correct_secret: bool,
    ) -> ProviderAnswer {
        let mut secret = self.secret.expose().to_vec();
        if !correct_secret {
            secret[0] ^= 1;
        }
        let credentials = if header {
            String::new()
        } else {
            format!("&client_id=app&client_secret={}", std::str::from_utf8(&secret).expect("hex secret"))
        };
        let body = Secret::new(
            format!(
                "grant_type=authorization_code&code={}&redirect_uri={redirect}&code_verifier={verifier}{credentials}",
                std::str::from_utf8(code.expose()).expect("code UTF-8")
            )
            .into_bytes()
            .into_boxed_slice(),
        );
        let authorization = header.then(|| {
            let credentials = [b"app:".as_slice(), secret.as_slice()].concat();
            let encoded = encode(Alphabet::Standard, &credentials);
            Secret::new([b"Basic ".as_slice(), encoded.as_ref()].concat().into_boxed_slice())
        });
        self.send(Event::Redeem {
            exchange: Token::new(99),
            authorization,
            content_type: Some(Box::from(b"application/x-www-form-urlencoded".as_slice())),
            body,
        });
        self.endpoint.take().expect("token answer")
    }
    fn claims(&mut self, token: &str) -> ProviderAnswer {
        self.send(Event::Claims {
            exchange: Token::new(99),
            authorization: Some(Secret::new(format!("Bearer {token}").into_bytes().into_boxed_slice())),
        });
        self.endpoint.take().expect("userinfo answer")
    }
    fn advance(&mut self, seconds: u64) {
        let span = Duration::from_secs(seconds);
        self.env.now = self.env.now.saturating_add(span);
        self.env.wall = Wall::from_nanos(self.env.wall.as_nanos().saturating_add(span.as_nanos()));
    }
}
fn encode(alphabet: Alphabet, bytes: &[u8]) -> Box<[u8]> {
    let length = base64::encoded_len(alphabet, u32::try_from(bytes.len()).expect("short bytes")).expect("encoded size");
    let mut writer = Writer::new(usize::try_from(length).expect("short output"));
    base64::encode(alphabet, bytes, &mut writer).expect("measured base64");
    writer.finish()
}
fn decode(value: &str) -> Box<[u8]> {
    base64::decode(Alphabet::Url, value.as_bytes(), 8192).expect("base64url")
}
fn error(answer: &ProviderAnswer, expected: &str) {
    let body: Value = serde_json::from_slice(answer.body.expose()).expect("error JSON");
    assert_eq!(body["error"].as_str(), Some(expected));
}
fn invalid_grant(answer: &ProviderAnswer) {
    assert_eq!(answer.status, 400);
    error(answer, "invalid_grant");
}
fn refused(answer: w::AuthorizeAnswer, why: &w::AuthorizeWhy) {
    let w::AuthorizeAnswer::Refused(found) = answer else {
        panic!("authorization refused");
    };
    assert_eq!(found.why(), why);
}
#[test]
fn provider_codes_tokens_and_claims_follow_one_signed_in_person() {
    keys();
    let started = Instant::now();
    let mut world = World::new(true);
    refused(world.authorize("https://other.example.com/callback"), &w::AuthorizeWhy::UnknownRedirect);
    refused(world.query(b"%invalid"), &w::AuthorizeWhy::Malformed);
    let w::AuthorizeAnswer::Go(back) = world.query(format!("client_id=app&redirect_uri={REDIRECT}").as_bytes()) else {
        panic!("issuer error sent back");
    };
    assert!(back.location().windows(b"error=invalid_request".len()).any(|part| part == b"error=invalid_request"));
    assert_eq!(world.authorize(REDIRECT), w::AuthorizeAnswer::SignedOut);
    world.sign_in();
    let code = world.code();
    let answer = world.redeem(&code, REDIRECT, VERIFIER, false, true);
    assert_eq!(answer.status, 200);
    let tokens: Value = serde_json::from_slice(answer.body.expose()).expect("token JSON");
    assert_eq!(tokens["token_type"], "Bearer");
    assert_eq!(tokens["expires_in"], 300);
    let jwt = tokens["id_token"].as_str().expect("ID token");
    let parts: Vec<_> = jwt.split('.').collect();
    assert_eq!(parts.len(), 3);
    let jwk = &world.key_set["keys"][0];
    let header: Value = serde_json::from_slice(&decode(parts[0])).expect("JWT header");
    assert_eq!(header["alg"], "RS256");
    assert_eq!(header["kid"], jwk["kid"]);
    let signed = format!("{}.{}", parts[0], parts[1]);
    let n = decode(jwk["n"].as_str().expect("modulus"));
    let e = decode(jwk["e"].as_str().expect("exponent"));
    let signature = decode(parts[2]);
    assert!(skein_tls::verify_rs256(&n, &e, signed.as_bytes(), &signature));
    assert!(!skein_tls::verify_rs256(&n, &e, b"different signing input", &signature));
    let claims: Value = serde_json::from_slice(&decode(parts[1])).expect("JWT claims");
    assert_eq!(
        claims,
        json!({"iss":"https://app.example.com","sub":"3:alice@example.com","aud":"app","exp":300,"iat":0,"nonce":"once","preferred_username":"alice","email":"alice@example.com","email_verified":true,"name":"Alice"})
    );
    let access = tokens["access_token"].as_str().expect("access token");
    let answer = world.claims(access);
    assert_eq!(answer.status, 200);
    let info: Value = serde_json::from_slice(answer.body.expose()).expect("userinfo JSON");
    assert_eq!(
        info,
        json!({"sub":"3:alice@example.com","preferred_username":"alice","email":"alice@example.com","email_verified":true,"name":"Alice"})
    );
    invalid_grant(&world.redeem(&code, REDIRECT, VERIFIER, false, true));
    let expiring = world.code();
    assert_eq!(world.authorize(REDIRECT), w::AuthorizeAnswer::Busy, "live code reserves remaining token slot");
    world.advance(60);
    invalid_grant(&world.redeem(&expiring, REDIRECT, VERIFIER, true, true));
    let code = world.code();
    invalid_grant(&world.redeem(&code, "https://other.example.com/callback", VERIFIER, false, true));
    invalid_grant(&world.redeem(&code, REDIRECT, VERIFIER, false, true));
    let code = world.code();
    invalid_grant(&world.redeem(&code, REDIRECT, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", false, true));
    invalid_grant(&world.redeem(&code, REDIRECT, VERIFIER, false, true));
    let code = world.code();
    for header in [false, true] {
        let answer = world.redeem(&code, REDIRECT, VERIFIER, header, false);
        assert_eq!(answer.status, 401);
        error(&answer, "invalid_client");
        assert_eq!(answer.challenge.as_deref(), header.then_some(b"Basic".as_slice()));
    }
    let answer = world.redeem(&code, REDIRECT, VERIFIER, true, true);
    assert_eq!(answer.status, 200, "Basic accepted; invalid client attempts did not authenticate or spend code");
    assert_eq!(world.authorize(REDIRECT), w::AuthorizeAnswer::Busy, "token table full");
    world.advance(240);
    let answer = world.claims(access);
    assert_eq!(answer.status, 401);
    assert_eq!(answer.challenge.as_deref(), Some(b"Bearer error=\"invalid_token\"".as_slice()));
    assert!(answer.body.is_empty());
    world.signer.as_mut().expect("signed in").kind = people::Kind::Service;
    refused(world.authorize(REDIRECT), &w::AuthorizeWhy::NotAPerson);
    world.signer.as_mut().expect("signed in").kind = people::Kind::Person;
    world.signer.as_mut().expect("signed in").identity.key.subject = Box::new([255]);
    refused(world.authorize(REDIRECT), &w::AuthorizeWhy::NotAPerson);
    world.sign_in();
    world.advance(60);
    let _first = world.code();
    let _second = world.code();
    assert_eq!(world.authorize(REDIRECT), w::AuthorizeAnswer::Busy, "code table full");
    world.advance(60);
    let fresh = world.code();
    let mut restarted = World::new(true);
    invalid_grant(&restarted.redeem(&fresh, REDIRECT, VERIFIER, false, true));
    assert_eq!(restarted.claims(access).status, 401, "restart retains no access token");
    let mut absent = World::new(false);
    let answer = absent.redeem(&code, REDIRECT, VERIFIER, false, true);
    error(&answer, "invalid_client");
    let answer = absent.claims("unknown");
    error(&answer, "invalid_client");
    eprintln!("provider story excluding key generation: {:?}", started.elapsed());
}
