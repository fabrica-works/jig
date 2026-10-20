//! LLM account sign-in against the shipped process and the existing independent issuer/browser.
use crate::account_sign_in_visitor::Visitor;
use crate::support::{Person, private, start};
use jig_client::{Asker, Key, contract as c};
use jig_client_protocol_world::native::Received;
use jig_client_protocol_world::native::{NativeClient, Script};
use jig_protocol_accounts_world::{fixtures as accounts, world::Story};
use skein_fake_peers::{Transport, oauth};
use skein_io::kernel::{Complete, Signal, Submit};
use skein_lib::{Duration, Queue, Time, Wall};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Streams},
    real,
};
use std::{fs, path::Path};

const ACCOUNT_SIGN_IN: Key = Key { asker: Asker(7), number: 1 };
const ACCOUNT_SIGN_OUT: Key = Key { asker: Asker(7), number: 2 };

pub fn configuration(directory: &Path) {
    crate::support::configuration(directory);
    let path = directory.join("config.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["engine"]["llm"]["providers"] = serde_json::json!([{
        "name":"Fake", "signed_in_plan":true, "manage_usage":"https://usage.example/", "number":3, "models":["fake"], "sources":{"sign_in":{
            "issuer":"http://127.0.0.1:31000", "client":"client", "scopes":["openid","read"],
            "name_hint":"jig-test", "authorization":"http://127.0.0.1:31000/authorize",
            "token":"http://127.0.0.1:31000/token", "revocation":"http://127.0.0.1:31000/revoke",
            "keys":"http://127.0.0.1:31000/keys", "transport":"plaintext",
            "must_grant":["openid","read"], "resource":"api", "redirect_path":"callback"
        }}
    }]);
    private(&path, &serde_json::to_vec(&value).expect("configuration JSON"));
}
fn issuer() -> oauth::Peer {
    let profile = accounts::limits();
    let limits = skein_fake_oauth::Limits {
        document: profile.client.document,
        uri_bytes: 256,
        request_bytes: 1024,
        codes: 1,
        rotations: 2,
        carried: 5,
        plans: 2,
    };
    let response = oauth::least_response(&limits).expect("issuer response");
    let mut peer = oauth::Peer::new(
        "127.0.0.1:31000".parse().expect("issuer socket"),
        Transport::Plaintext,
        skein_fake_peers::Limits {
            io: profile.io,
            connections: 2,
            queue: accounts::ROOM,
            plaintext: 32768,
            ciphertext: 32768,
            observations: 64,
            observation_bytes: 4096,
        },
        skein_fake_oauth::Config {
            terms: accounts::issuer_terms(Story::AccountSignInVaryingPort),
            authorization_url: accounts::AUTHORIZE.into(),
            token_endpoint: accounts::TOKEN.into(),
            client_id: b"client".as_slice().into(),
            client_secret: None,
            redirect_uri: accounts::PARTY_REDIRECT.into(),
            refresh_token: b"seed".as_slice().into(),
        },
        limits,
        skein_http::server::Limits { response, ..profile.server },
    )
    .expect("issuer");
    for plan in accounts::issuer_plans(Story::AccountSignInVaryingPort) {
        peer.queue(plan).expect("issuer plan");
    }
    peer
}
enum Process {
    Binary(Box<Binary>),
    Person(Box<Person>),
    Issuer(Box<oauth::Peer>),
    Visitor(Box<Visitor>),
}
impl Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Binary(x) => x.as_ref(),
            Self::Person(x) => x.as_ref(),
            Self::Issuer(x) => x.as_ref(),
            Self::Visitor(x) => x.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Binary(x) => x.as_mut(),
            Self::Person(x) => x.as_mut(),
            Self::Issuer(x) => x.as_mut(),
            Self::Visitor(x) => x.as_mut(),
        }
    }
}
impl Host for Process {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.host().next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.host().worst_case()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}
#[derive(Debug, Default)]
enum Phase {
    #[default]
    AccountSignIn,
    Address,
    AccountSignOut {
        source: u32,
    },
    Confirmed,
    Closing,
    Ended,
}
struct Judge {
    status: String,
    phase: Phase,
    deadline: Time,
}
fn client(procs: &mut [Process]) -> &mut NativeClient {
    let Process::Person(person) = &mut procs[1] else { panic!("native person") };
    person.client.as_mut().expect("native client")
}
fn answer(procs: &[Process]) -> Option<c::Answer> {
    let Process::Person(person) = &procs[1] else { panic!("native person") };
    person.client.as_ref()?.received.iter().rev().find_map(|event| match event {
        Received::Answer { answer, .. } => Some(answer.clone()),
        Received::SignedIn { .. } | Received::Sent { .. } | Received::Delivery(_) | Received::Read(_) => None,
    })
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        match self.phase {
            Phase::AccountSignIn => {
                let Process::Person(person) = &mut procs[1] else { panic!("native person") };
                if let Some(client) = &mut person.client
                    && client.signed_in == 1
                    && client.account_sign_in_ready()
                {
                    client.ask_account_sign_in(
                        ACCOUNT_SIGN_IN,
                        c::Ask::SignIn { holder: c::AccountHolder::Own, provider: 3 },
                    );
                    self.phase = Phase::Address;
                }
            }
            Phase::Address => {
                if let Some(answer) = answer(procs) {
                    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignInStarted { source, address })) =
                        answer
                    else {
                        panic!("account sign-in answer: {answer:?}")
                    };
                    assert!(!address.is_empty());
                    let Process::Visitor(visitor) = &mut procs[3] else { panic!("browser") };
                    visitor.hand(&address);
                    self.phase = Phase::AccountSignOut { source };
                }
            }
            Phase::AccountSignOut { source } => {
                let Process::Visitor(visitor) = &procs[3] else { panic!("browser") };
                let Process::Issuer(issuer) = &procs[2] else { panic!("issuer") };
                // The browser's callback finishes after the driver's exchange. Wait for its
                // sockets too, so sign-out cannot race the account sign-in exchange teardown.
                let token = issuer.observations().iter().find_map(|event| match event {
                    oauth::Observation::Post { connection } => Some(*connection),
                    oauth::Observation::Accepted { .. }
                    | oauth::Observation::Authorize { .. }
                    | oauth::Observation::Document { .. }
                    | oauth::Observation::Revoke { .. }
                    | oauth::Observation::Redirect { .. }
                    | oauth::Observation::Answered { .. }
                    | oauth::Observation::Closed { .. } => None,
                });
                if visitor.is_empty()
                    && token.is_some_and(|token| {
                        issuer.observations().iter().any(
                            |event| matches!(event, oauth::Observation::Closed { connection } if *connection == token),
                        )
                    })
                {
                    client(procs).ask_account_sign_in(
                        ACCOUNT_SIGN_OUT,
                        c::Ask::SignOut { holder: c::AccountHolder::Own, source },
                    );
                    self.phase = Phase::Confirmed;
                }
            }
            Phase::Confirmed => {
                if let Some(answer) = answer(procs)
                    && !matches!(
                        &answer,
                        c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignInStarted { .. }))
                    )
                {
                    assert_eq!(
                        answer,
                        c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignedOut { confirmed: true }))
                    );
                    client(procs).finish_account_sign_in();
                    self.phase = Phase::Closing;
                }
            }
            Phase::Closing => {
                if client(procs).is_empty() {
                    let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
                    binary.signal(Signal::Kill);
                    let Process::Issuer(issuer) = &mut procs[2] else { panic!("issuer") };
                    issuer.shutdown();
                    self.phase = Phase::Ended;
                }
            }
            Phase::Ended => {}
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        let Process::Visitor(visitor) = &procs[3] else { panic!("browser") };
        let Process::Issuer(issuer) = &procs[2] else { panic!("issuer") };
        let events: Vec<_> = issuer
            .observations()
            .iter()
            .map(|event| match event {
                oauth::Observation::Accepted { .. } => "accepted",
                oauth::Observation::Authorize { .. } => "authorize",
                oauth::Observation::Document { .. } => "document",
                oauth::Observation::Post { .. } => "post",
                oauth::Observation::Revoke { .. } => "revoke",
                oauth::Observation::Redirect { .. } => "redirect",
                oauth::Observation::Answered { .. } => "answered",
                oauth::Observation::Closed { .. } => "closed",
            })
            .collect();
        let Process::Person(person) = &procs[1] else { panic!("native person") };
        self.status = format!(
            "phase {:?}, signed in {:?}, browser handed {}, empty {}, status {:?}, issuer {:?}, answer {:?}",
            self.phase,
            person.client.as_ref().map(|c| c.signed_in),
            visitor.handed(),
            visitor.is_empty(),
            visitor.status(),
            events,
            answer(procs)
        );
        let Process::Binary(binary) = &procs[0] else { panic!("binary") };
        assert!(
            binary.exit_status().is_none() || matches!(self.phase, Phase::Ended),
            "jig-test failed: {}",
            String::from_utf8_lossy(binary.stderr())
        );
    }
    fn next_deadline(&self) -> Option<Time> {
        (!matches!(self.phase, Phase::Ended)).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline && !matches!(self.phase, Phase::Ended))
            .then(|| format!("account sign-in process story did not finish: {}", self.status))
    }
    fn passed(&self) -> bool {
        matches!(self.phase, Phase::Ended)
    }
}
pub fn run(directory: &Path) {
    let clock = Clock::new();
    let mut binary = start(directory);
    let streams = binary.take_streams().expect("binary pipes");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let client_directory = tempfile::tempdir_in(directory).expect("client directory");
    let root = skein_shell::open_root(client_directory.path()).expect("client root");
    let mut world = real::World::new(Judge {
        status: String::new(),
        phase: Phase::default(),
        deadline: clock.now().now.saturating_add(Duration::from_secs(10)),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds.into_iter().chain([root]).collect(), || {
        Process::Person(Box::new(Person {
            client: None,
            root: Some(root),
            script: Some(Script::account_sign_in()),
            account_sign_in: true,
            llm_key: None,
            input: Some(input),
            output: Some(output),
            reading: false,
            closing: 0,
            bytes: Vec::new(),
            completions: Queue::with_capacity(512),
            submissions: Queue::with_capacity(512),
        }))
    });
    world.spawn(|| Process::Issuer(Box::new(issuer())));
    world.spawn(|| Process::Visitor(Box::default()));
    let outcome = world.run(&clock, Duration::from_secs(15));
    assert!(outcome.procs.iter().all(Host::is_empty), "all real account sign-in io settles");
}
