//! The live layer's person-driven sign-in, bounded answer and confirmed sign-out.
use crate::support::{self, Person, Process};
use jig_client::{Asker, Handle, Key, contract as c};
use jig_client_protocol_world::native::{NativeClient, Received, Script};
use skein_io::kernel::Signal;
use skein_lib::{Duration, Queue, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, end_to_end::Streams, real};
use std::{net::ToSocketAddrs, path::Path};

const SIGN_IN: Key = Key { asker: Asker(77), number: 1 };
const SIGN_OUT: Key = Key { asker: Asker(77), number: 2 };
const ACCOUNTS: Handle = Handle { asker: Asker(77), number: 1 };
const PROVIDER: u32 = 1;
const MODEL: &str = "gpt-6-luna";
// configuration.md's small profile: 300 seconds, plus exchange/observation margin.
const SIGN_IN_WAIT: Duration = Duration::from_secs(330);
const WHOLE: Duration = Duration::from_secs(600);

pub fn configuration(directory: &Path) -> crate::codec::Limits {
    let address = ("api.openai.com", 443)
        .to_socket_addrs()
        .expect("resolve public Responses endpoint")
        .find(std::net::SocketAddr::is_ipv4)
        .expect("public Responses IPv4 endpoint")
        .to_string();
    configuration_at(directory, &address)
}

pub fn configuration_at(directory: &Path, address: &str) -> crate::codec::Limits {
    let mut value = support::configuration_value(directory);
    let path = directory.join("config.json");
    let engine = &mut value["engine"];
    engine["llm"]["providers"][0] = serde_json::json!({
        "name":"ChatGPT", "number":PROVIDER, "models":[MODEL],
        "signed_in_plan":true, "manage_usage":"https://chatgpt.com/settings/usage",
        "sources":{"sign_in":{
            "issuer":"https://auth.openai.com", "client":"dynamic_agent_client",
            "authorization":"https://auth.openai.com/api/accounts/authorize",
            "token":"https://auth.openai.com/api/accounts/oauth/token",
            "revocation":"https://auth.openai.com/api/accounts/oauth/revoke",
            "keys":"https://auth.openai.com/.well-known/jwks.json",
            "transport":"tls", "server_name":"auth.openai.com",
            "scopes":["openid","profile","email","offline_access","resource.invoke","chatgpt.tokens.use.direct"],
            "must_grant":["chatgpt.tokens.use.direct"], "resource":"https://api.openai.com/v1",
            "name_hint":"jig", "redirect_path":"auth/callback", "redirect_port":1455
        }}
    });
    let endpoint = &mut engine["llm"]["endpoints"][0];
    endpoint["provider"] = "openai".into();
    endpoint["dialect"] = 3.into();
    endpoint["address"] = serde_json::json!({"socket":address});
    endpoint["transport"] = "tls".into();
    endpoint["server_name"] = "api.openai.com".into();
    endpoint["headers"] = serde_json::json!([]);
    endpoint.as_object_mut().expect("endpoint").remove("trust");
    engine["llm"]["models"][0]["name"] = MODEL.into();
    engine["llm"]["models"][0]["output"] = 4096.into();
    engine["llm"]["models"][0]["efforts"] = serde_json::json!(["low"]);
    engine["charters"][0]["model"] = MODEL.into();
    engine["charters"][0]["turns"] = 5.into();
    engine["charters"][0]["time_ms"] = 240_000.into();
    engine["charters"][0]["waiting_ms"] = 10_000.into();
    let bytes = serde_json::to_vec(&value).expect("live config JSON");
    support::private(&path, &bytes);
    let limits = crate::store_limits(&bytes);
    let path = directory.join("agent.json");
    let mut agent = support::agent_value(directory);
    // The 64 KiB reasoning item and complete trace exceed standard's memory ceiling.
    agent["profile"]["declared"]["memory"] = (3_u64 << 30).into();
    let endpoint = &mut agent["endpoints"][0];
    endpoint["provider"] = "openai".into();
    endpoint["dialect"] = 3.into();
    endpoint["address"] = address.into();
    endpoint["transport"] = "tls".into();
    endpoint["server_name"] = "api.openai.com".into();
    endpoint.as_object_mut().expect("endpoint").remove("trust_der");
    endpoint["identity"] = "plain".into();
    endpoint["headers"] = serde_json::json!([]);
    endpoint["connect"] = 10_000.into();
    endpoint["handshake"] = 10_000.into();
    endpoint["models"][0]["name"] = MODEL.into();
    endpoint["models"][0]["output"] = 4096.into();
    endpoint["models"][0]["efforts"] = serde_json::json!(["low"]);
    endpoint["models"][0]["reasoning_item"] = 65_536.into();
    endpoint["models"][0]["window"] = 272_000.into();
    endpoint["models"][0]["idle"] = 300_000.into();
    agent["trace"] = support::trace::settings(directory);
    support::private(&path, &serde_json::to_vec(&agent).expect("live agent JSON"));
    support::private(&directory.join("charters/chat.txt"), crate::INSTRUCTIONS.as_bytes());
    eprintln!("Live model={MODEL}; Responses={address}; TLS server_name=api.openai.com; machine trust");
    limits
}

#[derive(Debug)]
enum Phase {
    Admission,
    Address,
    Signing { source: u32 },
    Answer { source: u32 },
    SignOut,
    Closing,
    Ended,
}
struct Judge {
    phase: Phase,
    deadline: Time,
    sign_in_deadline: Option<Time>,
    received: usize,
    accounts: Option<c::OwnerAccounts>,
    state: String,
}
fn client(procs: &mut [Process]) -> &mut NativeClient {
    let Process::Person(person) = &mut procs[1] else { panic!("native person") };
    person.client.as_mut().expect("native client")
}
fn progress(previous: &mut String, document: &c::OwnerAccounts, source: u32) {
    let state = if let Some(end) = document.last_end.as_ref().filter(|end| end.source == source) {
        format!("{:?}", end.end)
    } else if document.signing.iter().any(|signing| signing.source == source) {
        "signing".into()
    } else {
        return;
    };
    let facts = document
        .accounts
        .iter()
        .find_map(|account| account.sources.iter().find(|grant| grant.number == source).map(|grant| (account, grant)));
    let state = if let Some((account, grant)) = facts {
        format!(
            "{state}; account={}; plan={}; state={:?}; usable={}",
            account.number, account.plan, grant.state, grant.usable
        )
    } else {
        state
    };
    if *previous != state {
        eprintln!("ChatGPT sign-in source={source}: {state}");
        *previous = state;
    }
}
impl Judge {
    fn receive(&mut self, native: &NativeClient) -> Option<c::Answer> {
        let mut answer = None;
        for received in &native.received[self.received..] {
            match received {
                Received::Delivery(c::Delivery::OwnerAccounts(document)) => {
                    if let Phase::Signing { source } = self.phase {
                        progress(&mut self.state, document, source);
                    }
                    self.accounts = Some(document.as_ref().clone());
                }
                Received::Answer { answer: value, .. } => answer = Some(value.clone()),
                Received::SignedIn { .. } | Received::Sent { .. } | Received::Delivery(_) | Received::Read(_) => {}
            }
        }
        self.received = native.received.len();
        answer
    }
    fn signing(&mut self, procs: &mut [Process], source: u32) {
        let Some(document) = &self.accounts else { return };
        progress(&mut self.state, document, source);
        let Some(end) = document.last_end.as_ref().filter(|end| end.source == source) else { return };
        match end.end {
            c::OwnerAccountEnd::Made { account, declined } | c::OwnerAccountEnd::Renewed { account, declined } => {
                assert!(!declined, "ChatGPT sign-in made, but plan usage was declined");
                let account = document.accounts.iter().find(|line| line.number == account).expect("made account");
                let grant = account.sources.iter().find(|grant| grant.number == source).expect("signed-in source");
                assert!(account.plan, "ChatGPT sign-in made without plan usage");
                if grant.usable && grant.state == c::OwnerAccountState::Granted {
                    assert_eq!(account.holder, c::OwnerAccountHolder::Deployment);
                    assert_eq!(grant.kind, c::OwnerAccountKind::SignIn);
                    self.sign_in_deadline = None;
                    let native = client(procs);
                    native.account_request(jig_client::Event::Unwatch { handle: ACCOUNTS });
                    native.hold_after_script();
                    native.resume_after_account_setup();
                    self.phase = Phase::Answer { source };
                }
            }
            c::OwnerAccountEnd::Failed { failure } => panic!("ChatGPT sign-in failed: {failure:?}"),
            c::OwnerAccountEnd::Refused { refusal } => panic!("ChatGPT sign-in refused: {refusal:?}"),
        }
    }
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        let Process::Person(person) = &mut procs[1] else { panic!("native person") };
        let Some(native) = &mut person.client else { return };
        let answer = self.receive(native);
        match self.phase {
            Phase::Admission => {
                if native.signed_in == 1 && native.account_sign_in_ready() {
                    native.account_request(jig_client::Event::Watch {
                        handle: ACCOUNTS,
                        subject: jig_client::Watchable::Jig(c::Subject::Accounts),
                    });
                    native.ask_account_sign_in(
                        SIGN_IN,
                        c::Ask::SignIn { holder: c::AccountHolder::Deployment, provider: PROVIDER },
                    );
                    self.sign_in_deadline = Some(now.saturating_add(SIGN_IN_WAIT));
                    self.phase = Phase::Address;
                }
            }
            Phase::Address => {
                if let Some(answer) = answer {
                    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignInStarted { source, address })) =
                        answer
                    else {
                        panic!("ChatGPT sign-in start: {answer:?}");
                    };
                    eprintln!("Sign in at: {}", std::str::from_utf8(&address).expect("authorization URL"));
                    eprintln!(
                        "(the browser must reach 127.0.0.1:1455 on this machine: ssh -L 1455:127.0.0.1:1455 HOST)"
                    );
                    self.phase = Phase::Signing { source };
                    self.signing(procs, source);
                }
            }
            Phase::Signing { source } => self.signing(procs, source),
            Phase::Answer { source } => {
                if native.script_finished() {
                    native.hold_for_account_sign_in();
                    native.ask_account_sign_in(
                        SIGN_OUT,
                        c::Ask::SignOut { holder: c::AccountHolder::Deployment, source },
                    );
                    self.phase = Phase::SignOut;
                }
            }
            Phase::SignOut => {
                if let Some(answer) = answer {
                    assert_eq!(
                        answer,
                        c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignedOut {
                            confirmed: true,
                            failed_hosts: Box::new([])
                        })),
                        "ChatGPT sign-out confirmed by the issuer and every host"
                    );
                    eprintln!("ChatGPT sign-out confirmed=true; failed_hosts=[]");
                    native.finish_account_sign_in();
                    self.phase = Phase::Closing;
                }
            }
            Phase::Closing => {
                if native.is_empty() {
                    let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
                    binary.signal(Signal::Kill);
                    self.phase = Phase::Ended;
                }
            }
            Phase::Ended => {}
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        let Process::Binary(binary) = &procs[0] else { panic!("binary") };
        assert!(
            binary.exit_status().is_none() || matches!(self.phase, Phase::Ended),
            "jig-test failed {:?}: {}",
            binary.exit_status(),
            String::from_utf8_lossy(binary.stderr())
        );
    }
    fn next_deadline(&self) -> Option<Time> {
        if matches!(self.phase, Phase::Ended) {
            None
        } else {
            Some(self.sign_in_deadline.map_or(self.deadline, |at| at.min(self.deadline)))
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        self.next_deadline()
            .filter(|at| now >= *at)
            .map(|_| format!("live ChatGPT phase {:?}: {}", self.phase, self.state))
    }
    fn passed(&self) -> bool {
        matches!(self.phase, Phase::Ended)
    }
}

pub fn run(directory: &Path, started: Time) -> (u32, u32) {
    let clock = Clock::new();
    let mut binary = support::start(directory);
    let streams = binary.take_streams().expect("binary pipes");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let client_directory = tempfile::tempdir_in(directory).expect("native client directory");
    let root = skein_shell::open_root(client_directory.path()).expect("client root");
    let mut world = real::World::new(Judge {
        phase: Phase::Admission,
        deadline: started.saturating_add(Duration::from_secs(590)),
        sign_in_deadline: None,
        received: 0,
        accounts: None,
        state: String::new(),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds.into_iter().chain([root]).collect(), || {
        Process::Person(Box::new(Person {
            client: None,
            root: Some(root),
            script: Some(Script::answered_report(crate::WORDS.as_bytes(), support::ANSWER, Duration::from_secs(240))),
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
    let remaining = started.saturating_add(WHOLE).saturating_since(clock.now().now);
    let outcome = world.run(&clock, remaining);
    assert!(outcome.procs.iter().all(Host::is_empty), "all live sign-in io settles");
    let Process::Person(person) = &outcome.procs[1] else { panic!("native person") };
    let native = person.client.as_ref().expect("signed-in client");
    (native.signed_in, native.chats_made)
}
