//! Two reference services joined by loopback, with independent parties and durable rows.
//! Contract: protocol/channel.md, 12; the temporary store port lasts until 27.3.
use crate::{
    fixtures,
    processes::{self, Machine, Proc},
};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_fake_parties as parties;
use jig_test_domain as root;
use jig_test_service::{PartyOutput, StoreAnswer, StoreRequest};
use skein_lib::{Duration, ReplyTo, Time, Token};
use skein_world::domain::Verdict;
use std::{cell::RefCell, rc::Rc};

type Store = jig_fake_store::Store<root::Key, root::Record>;

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub seed: u64,
    pub config: skein_sim::Config,
}
#[derive(Debug)]
pub struct Script {
    pub words: Box<[u8]>,
    pub result: Box<[u8]>,
}
#[derive(Debug)]
pub struct Outcome {
    pub verdict: Verdict,
    pub turns_committed: u32,
    pub answers_acknowledged: u32,
    pub calls_answered: u32,
    pub messages_ended: u32,
}

/// Run one person's chat through the actual agent and all channel layers.
#[must_use]
pub fn run(settings: Settings, script: Script) -> Outcome {
    let store = Rc::new(RefCell::new(Store::new()));
    let referee = Referee {
        store: store.clone(),
        party: parties::Party::new(
            1,
            parties::Role::Owner,
            100,
            Box::new([
                parties::Act::SignIn { provider: 1, subject: b"owner-a".as_slice().into() },
                parties::Act::Request { key: [1; 16], ask: parties::Ask::Chat { words: script.words } },
            ]),
        ),
        period: false,
        passed: false,
    };
    let mut machine = skein_fake_machine::Machine::new();
    let root = machine.lay(&[]);
    let mut harness = skein_world::World::new(settings.seed, settings.config, referee, skein_world::Memory::Unchecked)
        .with_machine(Machine(machine));
    harness.host(skein_world::HostedProgram {
        program: b"agent-program".as_slice().into(),
        make: processes::agent,
        instances: 1,
        operations: smith_agent_process_world::limits().routes.checked_add(2).expect("agent operations"),
    });
    harness.spawn_restartable(Vec::new(), processes::engine);
    harness.spawn_root(skein_sim::Handle::new(root.raw()), processes::worker);
    harness.spawn(|| {
        Proc::Peer(Box::new(smith_agent_process_world::fake::configured(
            Box::new([skein_fake_llm_domain::api::Script {
                cue: Box::default(),
                turns: Box::new([skein_fake_llm_domain::api::Turn {
                    lines: Box::new([skein_fake_llm_domain::api::Line::Call {
                        name: b"finish".as_slice().into(),
                        arguments: script.result,
                    }]),
                    finish: skein_fake_llm_domain::api::Finish::ToolCalls,
                    tokens: 1,
                }]),
            }]),
            skein_llm::Credential { access_token: fixtures::TOKEN.into(), account_id: b"acc".as_slice().into() },
            Duration::ZERO,
        )))
    });
    let settled = harness.run();
    let worker = worker(&settled.procs);
    let store = store.borrow();
    let turns = turns(&store);
    Outcome {
        verdict: Verdict::Passed,
        turns_committed: u32::try_from(turns.len()).expect("bounded turns"),
        answers_acknowledged: u32::try_from(worker.wire.acknowledgements.len()).expect("bounded answers"),
        calls_answered: worker.wire.calls_answered,
        messages_ended: u32::try_from(turns.iter().filter(|turn| turn.read.is_some()).count()).expect("read messages"),
    }
}

struct Referee {
    /// Owned outside every engine process, so restart cannot drop durable rows.
    store: Rc<RefCell<Store>>,
    party: parties::Party,
    period: bool,
    passed: bool,
}
impl skein_world::Referee<Proc> for Referee {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        for proc in procs {
            match proc {
                Proc::Engine(host) => {
                    let engine = host.service.engine().expect("engine layer");
                    let mut store = self.store.borrow_mut();
                    answer_store(engine, &mut store);
                    answer_parties(engine, &mut self.party);
                    if !self.period && store.applied > 0 {
                        engine.parties_in().push(core::Event::Period {
                            project: 1,
                            period: 1,
                            budget: 100,
                            next: None,
                        });
                        self.period = true;
                    }
                    if self.period
                        && !self.passed
                        && let Some(up) = self.party.tick(now)
                    {
                        engine.parties_in().push(party_event(up));
                    }
                    if self.passed && !host.stopping {
                        host.stopping = true;
                        host.service.shutdown();
                        host.service.shutdown();
                    }
                }
                Proc::Worker(host) if self.passed && !host.stopping => {
                    host.stopping = true;
                    host.service.shutdown();
                }
                Proc::Peer(peer) if self.passed => peer.shutdown(),
                Proc::Worker(_) | Proc::Peer(_) | Proc::Agent { .. } => {}
            }
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Proc]) {
        let store = self.store.borrow();
        let turns = turns(&store);
        assert!(turns.len() <= 1, "one committed turn for the chat");
        let worker = worker(procs);
        let engine = procs
            .iter()
            .find_map(|proc| match proc {
                Proc::Engine(engine) => Some(engine),
                Proc::Worker(_) | Proc::Agent { .. } | Proc::Peer(_) => None,
            })
            .expect("engine process");
        assert!(worker.wire.acknowledgements.len() <= 1, "one received answer acknowledgement");
        assert!(engine.wire.acknowledgements.len() <= 1, "one sent answer acknowledgement");
        for &(task, attempt) in &worker.wire.acknowledgements {
            assert!(
                terminal(&store, task, attempt),
                "acknowledgement follows a durable successful terminal: {task}/{attempt}; diagnostics {}",
                diagnostics(procs)
            );
        }
        let replied = procs.iter().any(|proc| match proc {
            Proc::Peer(peer) => smith_agent_process_world::fake::replied(peer),
            Proc::Engine(_) | Proc::Worker(_) | Proc::Agent { .. } => false,
        });
        if self.party.quiescent() && turns.len() == 1 && worker.wire.acknowledgements.len() == 1 && replied {
            self.passed = true;
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.passed).then_some(Time::from_nanos(5_000_000_000))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.passed && now >= Time::from_nanos(5_000_000_000)).then(|| {
            format!("chat completes once; replies {:?}, rows {:?}", self.party.replies, self.store.borrow().rows)
        })
    }
    fn passed(&self) -> bool {
        self.passed
    }
}

fn turns(store: &Store) -> Vec<&core::TurnRecord> {
    store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) => Some(turn),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .collect()
}
fn terminal(store: &Store, task: u64, attempt: u64) -> bool {
    store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(answer)))
        if answer.task == task && answer.attempt == attempt && matches!(answer.end, jig_domain_tasks::End::Finished { .. })
    ))
}
fn answer_store(engine: &mut jig_test_service::Engine, store: &mut Store) {
    while let Some(request) = engine.store_requests().pop() {
        match request {
            StoreRequest::Commit { number, mut writes } => {
                let mut transaction = Vec::with_capacity(usize::try_from(writes.len()).expect("bounded writes"));
                while let Some(write) = writes.pop() {
                    transaction.push(match write {
                        root::Write::Save(row) => jig_fake_store::Write::Save { key: row.key(), row },
                        root::Write::Erase(key) => jig_fake_store::Write::Erase(key),
                    });
                }
                store.submit(number, transaction.into_boxed_slice(), 0);
            }
            StoreRequest::Restart(step) => {
                let rows = restore(store, step);
                engine.store_answers().push(StoreAnswer::Restored { step, rows });
            }
            StoreRequest::Load(core::Now::StartPreparation { task, from, .. }) => {
                let rows = turns(store)
                    .into_iter()
                    .filter(|row| row.task == task && from.is_some_and(|at| (row.attempt, row.turn) >= at))
                    .cloned()
                    .collect();
                engine.store_answers().push(StoreAnswer::Transcript { task, rows, done: true });
            }
            StoreRequest::Load(other) => panic!("this chat does not request {other:?}"),
            StoreRequest::Read(core::Held::NotesLoad { owner, range }) => {
                engine.store_answers().push(StoreAnswer::Core(Box::new(core::Event::Notes(
                    jig_domain_notes::Event::Loaded { owner, rows: note_rows(store, &range), more: false },
                ))));
            }
            StoreRequest::Read(other) => panic!("this chat does not read {other:?}"),
        }
    }
    while !store.pending.is_empty() {
        let number = store.tick(false).expect("fault-free store").expect("commit completed");
        engine.store_answers().push(StoreAnswer::Committed { number, fill: 0 });
    }
}
fn restore(store: &Store, step: core::RestartStep) -> Box<[root::Record]> {
    store
        .rows
        .iter()
        .filter(|(key, _)| match (step, key) {
            (core::RestartStep::LoadCore, root::Key::Core(key)) => {
                core::RestoreRange::LIVE.iter().any(|range| range.admits(key))
            }
            (core::RestartStep::RestoreConnector { connector }, root::Key::Connector { number, .. }) => {
                connector == *number
            }
            _ => false,
        })
        .map(|(_, row)| row.clone())
        .collect()
}
fn party_event(up: parties::Up) -> core::Event {
    match up {
        parties::Up::SignIn { to, provider, subject } => core::Event::SignIn {
            reply_to: ReplyTo::new(Token::new(to)),
            identity: people::Identity {
                key: people::IdentityKey { provider, subject },
                login: b"person".as_slice().into(),
                name: b"Person".as_slice().into(),
            },
        },
        parties::Up::Request { to, sign_in, project, key, ask: parties::Ask::Chat { words } } => {
            core::Event::People(people::Event::Ask {
                reply_to: ReplyTo::new(Token::new(to)),
                sign_in,
                key,
                ask: people::Ask::StartChat { project, words },
            })
        }
        parties::Up::Request { ask, .. } => panic!("one-chat script requested {ask:?}"),
    }
}
fn answer_parties(engine: &mut jig_test_service::Engine, party: &mut parties::Party) {
    while let Some(output) = engine.parties_out().pop() {
        match output {
            PartyOutput::Held(root::Delivery::Core(core::Held::PeopleReply { to, reply })) => {
                let reply = match reply {
                    people::Reply::SignedIn { person, sign_in, .. } => parties::Reply::SignedIn { person, sign_in },
                    people::Reply::Outcome(people::Outcome::Started { task }) => parties::Reply::Started { task },
                    other @ (people::Reply::SignedOut
                    | people::Reply::Outcome(_)
                    | people::Reply::Again(_)
                    | people::Reply::Refused(_)) => panic!("one-chat reply {other:?}"),
                };
                party.reply(to.into_token().raw(), reply);
            }
            PartyOutput::Now(core::Now::SignInRefused { to }) => {
                party.reply(to.into_token().raw(), parties::Reply::Again(parties::Again::NotReady));
            }
            PartyOutput::Busy(event) => {
                let root::Event::Core(event) = *event else { panic!("party port busy with {event:?}") };
                engine.parties_in().push(event);
            }
            PartyOutput::Held(_) | PartyOutput::Now(_) => {}
        }
    }
}

fn note_rows(store: &Store, range: &jig_domain_notes::Range) -> jig_domain_notes::Rows {
    use jig_domain_notes::{Key, Range};
    let mut records = skein_lib::List::with_capacity(fixtures::NOTES_LOAD_ROWS);
    for row in store.rows.values() {
        if let root::Record::Core(core::Record::Notes(note)) = row {
            let includes = match (range, note.key()) {
                (Range::Entry { name }, Key::Entry { name: found }) => *name == found,
                (Range::Lines { scope, after }, Key::Line { scope: found, name }) => {
                    *scope == found && after.is_none_or(|after| name > after)
                }
                (Range::Entry { .. }, Key::Line { .. }) | (Range::Lines { .. }, Key::Entry { .. }) => false,
            };
            if includes {
                records.push(note.clone()).expect("bounded note load");
            }
        }
    }
    jig_domain_notes::Rows { records }
}

fn diagnostics(procs: &[Proc]) -> String {
    procs
        .iter()
        .filter_map(|proc| match proc {
            Proc::Agent { errors, .. } => Some(String::from_utf8_lossy(&errors.bytes()).into_owned()),
            Proc::Engine(_) | Proc::Worker(_) | Proc::Peer(_) => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn worker(procs: &[Proc]) -> &processes::ServiceHost {
    procs
        .iter()
        .find_map(|proc| match proc {
            Proc::Worker(worker) => Some(worker),
            Proc::Engine(_) | Proc::Agent { .. } | Proc::Peer(_) => None,
        })
        .expect("worker process")
}
