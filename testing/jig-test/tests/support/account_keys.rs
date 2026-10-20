//! Account key add, replace and remove through the shipped process; answers only.
use crate::support::{Person, private, start};
use jig_client::{Asker, Key, accounts, contract as c};
use jig_client_protocol_world::native::{NativeClient, Received, Script};
use skein_io::kernel::{Complete, Signal, Submit};
use skein_lib::{Duration, Queue, Secret, Time, Wall};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Streams},
    real,
};
use std::{fs, path::Path};
const ADD: Key = Key { asker: Asker(7), number: 41 };
const REPLACE: Key = Key { asker: Asker(7), number: 42 };
const REMOVE: Key = Key { asker: Asker(7), number: 43 };
pub fn configuration(directory: &Path) {
    crate::support::configuration(directory);
    let path = directory.join("config.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["engine"]["llm"]["providers"] =
        serde_json::json!([{"name":"Fake", "number":3, "models":["fake"], "sources":{"keys":true}}]);
    private(&path, &serde_json::to_vec(&value).expect("configuration JSON"));
}
enum Process {
    Binary(Box<Binary>),
    Person(Box<Person>),
}
impl Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Binary(x) => x.as_ref(),
            Self::Person(x) => x.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Binary(x) => x.as_mut(),
            Self::Person(x) => x.as_mut(),
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

#[derive(Clone, Copy, Debug, Default)]
enum Phase {
    #[default]
    Ready,
    Added,
    Replaced {
        source: u32,
    },
    Removed,
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
fn answers(procs: &[Process]) -> Vec<c::Answer> {
    let Process::Person(person) = &procs[1] else { panic!("native person") };
    person.client.as_ref().map_or_else(Vec::new, |client| {
        client
            .received
            .iter()
            .filter_map(|event| match event {
                Received::Answer { answer, .. } => Some(answer.clone()),
                Received::SignedIn { .. } | Received::Sent { .. } | Received::Delivery(_) | Received::Read(_) => None,
            })
            .collect()
    })
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        match self.phase {
            Phase::Ready => {
                let Process::Person(person) = &mut procs[1] else { panic!("native person") };
                if let Some(client) = &mut person.client
                    && client.signed_in == 1
                    && client.account_sign_in_ready()
                {
                    client.account_request(accounts::add_key(
                        ADD,
                        [41; 16],
                        c::AccountHolder::Own,
                        3,
                        None,
                        Secret::new(Box::from(b"first LLM account key".as_slice())),
                    ));
                    self.phase = Phase::Added;
                }
            }
            Phase::Added => {
                if let Some(answer) = answers(procs).first() {
                    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountKeyKept { source })) = answer else {
                        panic!("account key kept: {answer:?}")
                    };
                    let source = *source;
                    assert_ne!(source, 0);
                    client(procs).account_request(accounts::replace_key(
                        REPLACE,
                        [42; 16],
                        c::AccountHolder::Own,
                        source,
                        Secret::new(Box::from(b"replacement LLM account key".as_slice())),
                    ));
                    self.phase = Phase::Replaced { source };
                }
            }
            Phase::Replaced { source } => {
                if let Some(answer) = answers(procs).get(1) {
                    assert_eq!(answer, &c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountKeyKept { source })));
                    client(procs).account_request(accounts::account_change(
                        REMOVE,
                        [43; 16],
                        c::AccountHolder::Own,
                        c::AccountChange::RemoveKey { source },
                    ));
                    self.phase = Phase::Removed;
                }
            }
            Phase::Removed => {
                if let Some(answer) = answers(procs).get(2) {
                    assert_eq!(answer, &c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountChanged)));
                    assert_eq!(answers(procs).len(), 3);
                    client(procs).finish_account_sign_in();
                    self.phase = Phase::Closing;
                }
            }
            Phase::Closing => {
                if client(procs).is_empty() {
                    let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
                    binary.signal(Signal::Kill);
                    self.phase = Phase::Ended;
                }
            }
            Phase::Ended => {}
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        self.status = format!("{:?}: {:?}", self.phase, answers(procs));
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
            .then(|| format!("account key process story did not finish: {}", self.status))
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
            input: Some(input),
            output: Some(output),
            reading: false,
            closing: 0,
            bytes: Vec::new(),
            completions: Queue::with_capacity(512),
            submissions: Queue::with_capacity(512),
        }))
    });
    let outcome = world.run(&clock, Duration::from_secs(15));
    assert!(outcome.procs.iter().all(Host::is_empty), "all real account key io settles");
}
