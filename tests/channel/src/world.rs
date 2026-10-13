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
use skein_lib::{Duration, ReplyTo, Token};
use skein_world::domain::Verdict;
use std::{cell::RefCell, rc::Rc};

pub(crate) type Store = jig_fake_store::Store<root::Key, root::Record>;

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub seed: u64,
    pub config: skein_sim::Config,
}
#[derive(Debug)]
pub struct Script {
    pub words: Box<[u8]>,
    pub result: Box<[u8]>,
    pub leading_turns: u32,
    pub latency: Duration,
    pub story: crate::stories::Story,
    pub recorded: Option<Recorded>,
}
#[derive(Clone, Debug)]
pub struct Recorded {
    pub(crate) turn: jig_protocol_worker_wire::Turn,
    pub(crate) answer: jig_protocol_worker_wire::Answer,
}
#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent protocol observations for story assertions")]
pub struct Outcome {
    pub verdict: Verdict,
    pub turns_committed: u32,
    pub answers_acknowledged: u32,
    pub calls_answered: u32,
    pub messages_ended: u32,
    pub refusal: Option<u16>,
    pub reset_injected: bool,
    pub adopting_hellos: u32,
    pub agent_starts: u32,
    pub activity_seen: bool,
    pub text_bytes: usize,
    pub start_messages: u32,
    pub waited: bool,
    pub windows: u32,
    pub resumed_turns: Option<u32>,
    pub identity_kept: bool,
    pub heartbeats: u32,
    pub channel_closed: bool,
    pub lost: u32,
    pub shut_down: bool,
    pub contact_cancelled: bool,
    pub shutdown_cancelled: bool,
    pub refused_messages: Vec<jig_protocol_worker_wire::MessageRefusal>,
    pub recorded: Option<Recorded>,
    pub refusals_kept: bool,
    pub engine_drained: bool,
    pub commits_waiting_at_drain: usize,
}

/// Run one person's chat through the actual agent and all channel layers.
#[must_use]
pub fn run(mut settings: Settings, script: Script) -> Outcome {
    if matches!(script.story, crate::stories::Story::MidFrame) {
        settings.config.buffer = 64;
    }
    let store = Rc::new(RefCell::new(Store::new()));
    let views = Rc::new(RefCell::new(Views::default()));
    let referee = make_referee(&script, store.clone(), views.clone());
    let mut machine = skein_fake_machine::Machine::new();
    let root = (!matches!(script.story, crate::stories::Story::Opening(_))).then(|| machine.lay(&[]));
    let mut harness = skein_world::World::new(settings.seed, settings.config, referee, skein_world::Memory::Unchecked)
        .with_machine(Machine(machine));
    launch(&mut harness, root, &script);
    let scenario = script.story;
    drop(script);
    let mut reset_injected = false;
    let settled = harness.run_with_faults(|at, _, submissions| {
        let reset = !reset_injected && (matches!(scenario, crate::stories::Story::MidFrame) && at == 0
            && submissions.iter().any(|submission| matches!(&submission.kind,
                skein_io::kernel::Op::Send { bytes, from, .. }
                if *from > 8 && bytes.len() >= 8 && skein_channel::parse_header(&bytes[..8]).is_ok_and(|head| head.kind == 0x0180))) || matches!(scenario, crate::stories::Story::Reconnect) && at == 1 && submissions.iter().any(|submission| matches!(&submission.kind, skein_io::kernel::Op::Send { bytes, .. } if bytes.len() >= 8 && skein_channel::parse_header(&bytes[..8]).is_ok_and(|head| head.kind == 0x0104))));
        if reset { reset_injected = true; }
        Some(skein_sim::Faults { reset: if reset { 1000 } else { 0 }, ..settings.config.faults })
    });
    outcome(&settled.procs, &store.borrow(), &views.borrow(), reset_injected, settled.killed.len())
}

fn outcome(procs: &[Proc], store: &Store, views: &Views, reset_injected: bool, killed: usize) -> Outcome {
    let worker = procs.iter().find_map(|proc| match proc {
        Proc::Worker(host) => Some(&host.wire),
        Proc::Probe(peer) => Some(&peer.wire),
        Proc::Engine(_) | Proc::Agent { .. } | Proc::Peer(_) => None,
    });
    let refusal = procs.iter().find_map(|proc| if let Proc::Probe(peer) = proc { peer.refused } else { None });
    let turns = turns(store);
    let assignments: Vec<_> = worker
        .into_iter()
        .flat_map(|wire| wire.frames.iter())
        .filter(|(_, kind, _)| *kind == 0x0180)
        .map(|(_, _, bytes)| {
            jig_protocol_worker_wire::Assignment::decode(
                &jig_protocol_worker_wire::CEILINGS,
                &mut skein_lib::Reader::new(bytes),
            )
            .expect("received assignment")
        })
        .collect();
    Outcome {
        engine_drained: views.engine_drained,
        commits_waiting_at_drain: views.commits_waiting_at_drain,
        recorded: recorded(worker),
        refusals_kept: views.refusals_kept,
        refused_messages: collect_refusals(procs),
        contact_cancelled: cancelled(worker, &jig_protocol_worker_wire::CancelReason::Contact),
        shutdown_cancelled: cancelled(worker, &jig_protocol_worker_wire::CancelReason::Shutdown),
        lost: u32::try_from(store.rows.values().filter(|row| matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(answer))) if answer.end == jig_domain_tasks::End::Failed(jig_domain_tasks::Class::Lost))).count()).expect("lost terminals"),
        shut_down: procs.iter().any(|proc| matches!(proc, Proc::Worker(host) if host.stopping)),
        heartbeats: procs.iter().find_map(|proc| if let Proc::Probe(peer) = proc { Some(peer.heartbeats) } else { None }).unwrap_or(0),
        channel_closed: procs.iter().any(|proc| matches!(proc, Proc::Engine(host) if host.wire.closed)),
        windows: u32::try_from(turns.iter().filter(|turn| turn.opens_window).count().saturating_sub(1)).expect("window count"),
        resumed_turns: assignments.last().map(|assignment| transcript_count(assignment.transcript())),
        identity_kept: assignments.last().is_some_and(|next| assignments[0].identity() == next.identity()),
        activity_seen: views.activity_seen,
        text_bytes: views.text_bytes,
        verdict: Verdict::Passed,
        turns_committed: u32::try_from(turns.len()).expect("bounded turns"),
        answers_acknowledged: worker.map_or(0, |wire| u32::try_from(wire.acknowledgements.len()).expect("bounded answers")),
        calls_answered: worker.map_or(0, |wire| wire.calls_answered),
        refusal,
        reset_injected,
        adopting_hellos: worker.map_or(0, |wire| wire.adopting_hellos),
        agent_starts: u32::try_from(killed + procs.iter().filter(|proc| matches!(proc, Proc::Agent { .. })).count()).expect("agent count"),
        messages_ended: u32::try_from(views.messages_ended).expect("bounded messages"),
        start_messages: worker.map_or(0, |wire| wire.start_messages),
        waited: worker.is_some_and(|wire| wire.waited),
    }
}

pub(crate) fn turns(store: &Store) -> Vec<&core::TurnRecord> {
    store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) => Some(turn),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .collect()
}
pub(crate) fn terminal(store: &Store, task: u64, attempt: u64, scenario: crate::stories::Story) -> bool {
    store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(answer)))
        if answer.task == task && answer.attempt == attempt && if matches!(scenario, crate::stories::Story::InvalidCharter) || matches!(scenario, crate::stories::Story::InvalidTranscript) && attempt != 1 {
            answer.end == jig_domain_tasks::End::Failed(jig_domain_tasks::Class::Invalid)
        } else if matches!(scenario, crate::stories::Story::Window | crate::stories::Story::InvalidTranscript) && (attempt == 1 || matches!(scenario, crate::stories::Story::Window) && attempt == 2) {
            answer.end == jig_domain_tasks::End::Parked
        } else if matches!(scenario, crate::stories::Story::BeyondGrace | crate::stories::Story::Shutdown) { answer.end == jig_domain_tasks::End::Failed(jig_domain_tasks::Class::Agent) } else { matches!(answer.end, jig_domain_tasks::End::Finished { .. }) }
    ))
}
pub(crate) fn pending_turn(store: &Store) -> bool {
    store.pending.iter().any(|pending| {
        pending.writes.iter().any(|write| {
            matches!(
                write,
                jig_fake_store::Write::Save {
                    row: root::Record::Core(core::Record::Core(core::CoreRecord::Turn(_))),
                    ..
                }
            )
        })
    })
}

pub(crate) fn answer_store(
    engine: &mut jig_test_service::Engine,
    store: &mut Store,
    rules: &mut crate::referee::Rules,
    hold: bool,
    scenario: crate::stories::Story,
    now: skein_lib::Time,
) {
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
                let mut rows: Box<[core::TurnRecord]> = turns(store)
                    .into_iter()
                    .filter(|row| row.task == task && from.is_some_and(|at| (row.attempt, row.turn) >= at))
                    .cloned()
                    .collect();
                if matches!(scenario, crate::stories::Story::InvalidTranscript) {
                    for row in &mut rows {
                        row.transcript[0] = 0xff;
                    }
                }
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
        let pending = store.pending.front().expect("pending commit");
        if hold
            && pending.writes.iter().any(|write| {
                matches!(
                    write,
                    jig_fake_store::Write::Save {
                        row: root::Record::Core(core::Record::Core(core::CoreRecord::Turn(_))),
                        ..
                    }
                )
            })
        {
            break;
        }
        for write in &pending.writes {
            if let jig_fake_store::Write::Save { row, .. } = write {
                match row {
                    root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) => {
                        rules.turned(turn.task, turn.attempt, turn.turn);
                        rules.read(turn.task, turn.attempt, turn.read);
                    }
                    root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(answer))) => {
                        if answer.end == jig_domain_tasks::End::Failed(jig_domain_tasks::Class::Lost) {
                            rules.lost_ended(answer.task, answer.attempt, now);
                        } else {
                            rules.answered(answer.task, answer.attempt);
                        }
                    }
                    root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Live(task))) => {
                        for &name in &task.refused {
                            rules.message_refused(task.number, task.attempt, name);
                        }
                    }
                    root::Record::Core(_) | root::Record::Connector { .. } => {}
                }
            }
        }
        let number = store.tick(false).expect("fault-free store").expect("commit completed");
        engine.store_answers().push(StoreAnswer::Committed { number, fill: 0 });
    }
}
pub(crate) fn restore(store: &Store, step: core::RestartStep) -> Box<[root::Record]> {
    match step {
        core::RestartStep::LoadCore => core::RestoreRange::LIVE
            .iter()
            .flat_map(|range| {
                store.rows.iter().filter_map(move |(key, row)| match key {
                    root::Key::Core(key) if range.admits(key) => Some(row.clone()),
                    root::Key::Core(_) | root::Key::Connector { .. } => None,
                })
            })
            .collect(),
        core::RestartStep::RestoreConnector { connector } => store
            .rows
            .iter()
            .filter_map(|(key, row)| match key {
                root::Key::Connector { number, .. } if *number == connector => Some(row.clone()),
                root::Key::Core(_) | root::Key::Connector { .. } => None,
            })
            .collect(),
        core::RestartStep::LoadInbox
        | core::RestartStep::AdoptRuns
        | core::RestartStep::ReadAfresh { .. }
        | core::RestartStep::SettleOutbox { .. }
        | core::RestartStep::Open => Box::default(),
    }
}

pub(crate) fn party_event(up: parties::Up) -> core::Event {
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
        parties::Up::Request {
            to,
            sign_in,
            project,
            key,
            ask: parties::Ask::Say { task: parties::Task::Number(task), words, release },
        } => core::Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(to)),
            sign_in,
            key,
            ask: people::Ask::Say { project, task, words, release },
        }),
        parties::Up::Request { ask, .. } => panic!("channel script requested {ask:?}"),
    }
}
pub(crate) fn answer_parties(engine: &mut jig_test_service::Engine, party: &mut parties::Party, views: &mut Views) {
    while let Some(output) = engine.parties_out().pop() {
        match output {
            PartyOutput::Held(root::Delivery::Core(core::Held::PeopleReply { to, reply })) => {
                let reply = match reply {
                    people::Reply::SignedIn { person, sign_in, .. } => parties::Reply::SignedIn { person, sign_in },
                    people::Reply::Outcome(people::Outcome::Started { task }) => parties::Reply::Started { task },
                    people::Reply::Outcome(people::Outcome::Said { task, message, released }) => {
                        parties::Reply::Said { task, message, released }
                    }
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
            PartyOutput::Held(root::Delivery::Core(core::Held::Delivery { watcher, document })) => {
                if let core::documents::Document::Task(task) = &*document {
                    views.activity_seen |= task.chip.details.observation.is_some();
                }
                if let core::documents::Document::Run(run) = &*document {
                    assert!(run.writing.is_none(), "process facts contain no provisional text");
                }
                engine.parties_in().push(core::Event::Delivered { watcher });
            }
            PartyOutput::Now(core::Now::RunText { watcher, text, .. }) => {
                views.text_bytes += text.len();
                engine.parties_in().push(core::Event::StreamDelivered { watcher });
            }
            PartyOutput::Busy(event) => {
                let root::Event::Core(event) = *event else { panic!("party port busy with {event:?}") };
                engine.parties_in().push(event);
            }
            PartyOutput::Held(_) | PartyOutput::Now(_) => {}
        }
    }
}

pub(crate) fn note_rows(store: &Store, range: &jig_domain_notes::Range) -> jig_domain_notes::Rows {
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

pub(crate) fn diagnostics(procs: &[Proc]) -> String {
    procs
        .iter()
        .filter_map(|proc| match proc {
            Proc::Agent { errors, .. } => Some(String::from_utf8_lossy(&errors.bytes()).into_owned()),
            Proc::Worker(host) => Some(format!(
                "{:?}",
                host.wire
                    .frames
                    .iter()
                    .filter_map(|(_, kind, bytes)| if *kind == 0x0103 {
                        Some(
                            jig_protocol_worker_wire::Answer::decode(
                                &jig_protocol_worker_wire::CEILINGS,
                                &mut skein_lib::Reader::new(bytes),
                            )
                            .expect("answer")
                            .ending()
                            .clone(),
                        )
                    } else {
                        None
                    })
                    .collect::<Vec<_>>()
            )),
            Proc::Peer(peer) => Some(format!(
                "{:?}",
                peer.observations()
                    .iter()
                    .filter(|o| matches!(o, skein_fake_peers::llm::Observation::Answered { .. }))
                    .collect::<Vec<_>>()
            )),
            Proc::Engine(host) => Some(format!(
                "engine turns {:?}",
                host.wire
                    .frames
                    .iter()
                    .filter_map(|(_, kind, bytes)| if *kind == 0x0101 {
                        let t = jig_protocol_worker_wire::Turn::decode(
                            &jig_protocol_worker_wire::CEILINGS,
                            &mut skein_lib::Reader::new(bytes),
                        )
                        .expect("wire turn");
                        Some((t.task(), t.attempt(), t.number(), t.spent(), *t.read(), t.opens_window()))
                    } else {
                        None
                    })
                    .collect::<Vec<_>>()
            )),
            Proc::Probe(_) => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn worker_wire(procs: &[Proc]) -> &processes::Wire {
    procs
        .iter()
        .find_map(|proc| match proc {
            Proc::Worker(worker) => Some(&worker.wire),
            Proc::Probe(peer) => Some(&peer.wire),
            Proc::Engine(_) | Proc::Agent { .. } | Proc::Peer(_) => None,
        })
        .expect("worker's wire observations")
}

#[derive(Default)]
pub(crate) struct Views {
    pub(crate) activity_seen: bool,
    pub(crate) text_bytes: usize,
    pub(crate) messages_ended: usize,
    pub(crate) refusals_kept: bool,
    pub(crate) engine_drained: bool,
    pub(crate) commits_waiting_at_drain: usize,
}

fn transcript_count(bytes: &[u8]) -> u32 {
    let mut reader = skein_lib::Reader::new(bytes);
    let mut count = 0;
    while !reader.is_empty() {
        let length = reader.u32().expect("transcript piece length");
        reader.skip(length).expect("transcript piece body");
        count += 1;
    }
    count
}

fn make_referee(script: &Script, store: Rc<RefCell<Store>>, views: Rc<RefCell<Views>>) -> crate::referee::Referee {
    crate::referee::Referee {
        store,
        party: parties::Party::new(1, parties::Role::Owner, 100, fixtures::party_script(script)),
        period: false,
        passed: false,
        stopped: false,
        rules: crate::referee::Rules::default(),
        observed: 0,
        expected_turns: match script.story {
            crate::stories::Story::InvalidCharter
            | crate::stories::Story::BeyondGrace
            | crate::stories::Story::Shutdown => 0,
            crate::stories::Story::InvalidTranscript => 1,
            crate::stories::Story::Window => 7,
            crate::stories::Story::Drain
            | crate::stories::Story::Chat
            | crate::stories::Story::Stalled
            | crate::stories::Story::Quiet
            | crate::stories::Story::Replaced
            | crate::stories::Story::Reconnect
            | crate::stories::Story::Refusals(_)
            | crate::stories::Story::Opening(_)
            | crate::stories::Story::Progress
            | crate::stories::Story::Facts
            | crate::stories::Story::Inbox
            | crate::stories::Story::Call
            | crate::stories::Story::Wait
            | crate::stories::Story::MidFrame
            | crate::stories::Story::Restart { .. }
            | crate::stories::Story::HeldCommit
            | crate::stories::Story::Partition { .. } => script.leading_turns + 1,
        },
        expected_answers: if matches!(script.story, crate::stories::Story::Window) { 3 } else { 1 },
        story: script.story,
        injected: false,
        resumed_attempt: 0,
        engine_observed: 0,
        roots_observed: std::collections::BTreeMap::new(),
        formatted: String::new(),
        grants_observed: 0,
        engine_instance: None,
        views,
    }
}

fn launch(
    harness: &mut skein_world::World<Proc, crate::referee::Referee, Machine>,
    root: Option<skein_fake_machine::Opened>,
    script: &Script,
) {
    harness.host(skein_world::HostedProgram {
        program: b"agent-program".as_slice().into(),
        make: if matches!(script.story, crate::stories::Story::BeyondGrace) {
            processes::agent_slow
        } else {
            processes::agent
        },
        instances: 2,
        operations: smith_agent_process_world::limits().routes.checked_add(2).expect("agent operations"),
    });
    let engine_factory = if matches!(
        script.story,
        crate::stories::Story::Refusals(
            crate::stories::MessageRefusal::TooLarge | crate::stories::MessageRefusal::Replay
        )
    ) {
        processes::engine_message
    } else if matches!(script.story, crate::stories::Story::Call) {
        processes::engine_calls
    } else if matches!(script.story, crate::stories::Story::Window) {
        processes::engine_window
    } else {
        processes::engine
    };
    harness.spawn_restartable(Vec::new(), engine_factory);

    if let crate::stories::Story::Restart { before } = script.story {
        harness.cut(0, before, skein_world::Cut::Kill);
    }
    if matches!(script.story, crate::stories::Story::Refusals(crate::stories::MessageRefusal::Replay)) {
        harness.cut(0, 18, skein_world::Cut::Kill);
    }
    spawn_channel_side(harness, root, script);
    if !matches!(
        script.story,
        crate::stories::Story::Opening(_)
            | crate::stories::Story::Stalled
            | crate::stories::Story::Quiet
            | crate::stories::Story::Replaced
            | crate::stories::Story::Refusals(
                crate::stories::MessageRefusal::Full | crate::stories::MessageRefusal::NameInUse
            )
    ) {
        harness.spawn(|| {
            let scripts = Box::new([skein_fake_llm_domain::api::Script {
                cue: Box::default(),
                turns: fixtures::provider_turns(script),
            }]);
            let peer = if matches!(script.story, crate::stories::Story::Window) {
                processes::window_provider(scripts)
            } else {
                smith_agent_process_world::fake::configured(
                    scripts,
                    skein_llm::Credential {
                        access_token: fixtures::TOKEN.into(),
                        account_id: b"acc".as_slice().into(),
                    },
                    script.latency,
                )
            };
            Proc::Peer(Box::new(peer))
        });
    }
}

fn spawn_channel_side(
    harness: &mut skein_world::World<Proc, crate::referee::Referee, Machine>,
    root: Option<skein_fake_machine::Opened>,
    script: &Script,
) {
    if matches!(
        script.story,
        crate::stories::Story::Refusals(
            crate::stories::MessageRefusal::Full | crate::stories::MessageRefusal::NameInUse
        )
    ) {
        harness.spawn(|| {
            Proc::Probe(Box::new(crate::relay::Peer::refusing(
                if matches!(script.story, crate::stories::Story::Refusals(crate::stories::MessageRefusal::Full)) {
                    jig_protocol_worker_wire::MessageRefusal::Full
                } else {
                    jig_protocol_worker_wire::MessageRefusal::NameInUse
                },
                script.recorded.clone().expect("recorded native wire"),
            )))
        });
    } else if matches!(script.story, crate::stories::Story::Replaced) {
        harness.spawn(|| Proc::Probe(Box::new(crate::relay::Peer::silent(false))));
        harness.spawn(|| {
            let mut peer = crate::relay::Peer::silent(false);
            peer.start_at = skein_lib::Time::from_nanos(1_000_000_000);
            Proc::Probe(Box::new(peer))
        });
    } else if matches!(script.story, crate::stories::Story::Stalled | crate::stories::Story::Quiet) {
        harness.spawn(|| {
            Proc::Probe(Box::new(crate::relay::Peer::silent(matches!(script.story, crate::stories::Story::Stalled))))
        });
    } else if let crate::stories::Story::Opening(case) = script.story {
        harness.spawn(|| Proc::Probe(Box::new(crate::relay::Peer::opening(case))));
    } else {
        let factory = if matches!(
            script.story,
            crate::stories::Story::Refusals(
                crate::stories::MessageRefusal::TooLarge | crate::stories::MessageRefusal::Replay
            )
        ) {
            processes::worker_message
        } else if matches!(script.story, crate::stories::Story::InvalidCharter) {
            processes::worker_bad_charter
        } else {
            processes::worker
        };
        harness.spawn_root(skein_sim::Handle::new(root.expect("worker root").raw()), factory);
    }
}

fn cancelled(worker: Option<&processes::Wire>, reason: &jig_protocol_worker_wire::CancelReason) -> bool {
    worker.is_some_and(|wire| wire.frames.iter().any(|(_, kind, bytes)| {
        *kind == 0x0103 && jig_protocol_worker_wire::Answer::decode(&jig_protocol_worker_wire::CEILINGS,
            &mut skein_lib::Reader::new(bytes)).is_ok_and(|answer| matches!(answer.ending(), jig_protocol_worker_wire::Ending::Failed(failed)
                if matches!(failed.failure(), jig_protocol_worker_wire::Failure::Cancelled(value) if value.value() == reason)))
    }))
}

fn collect_refusals(procs: &[Proc]) -> Vec<jig_protocol_worker_wire::MessageRefusal> {
    procs
        .iter()
        .filter_map(|proc| if let Proc::Engine(host) = proc { Some(host) } else { None })
        .flat_map(|host| host.wire.frames.iter())
        .flat_map(|(_, kind, bytes)| match *kind {
            0x0101 => jig_protocol_worker_wire::Turn::decode(
                &jig_protocol_worker_wire::CEILINGS,
                &mut skein_lib::Reader::new(bytes),
            )
            .expect("turn")
            .refused()
            .iter()
            .map(|r| r.reason().clone())
            .collect::<Vec<_>>(),
            0x0103 => jig_protocol_worker_wire::Answer::decode(
                &jig_protocol_worker_wire::CEILINGS,
                &mut skein_lib::Reader::new(bytes),
            )
            .expect("answer")
            .refused()
            .iter()
            .map(|r| r.reason().clone())
            .collect::<Vec<_>>(),
            _ => Vec::new(),
        })
        .collect()
}

fn recorded(worker: Option<&processes::Wire>) -> Option<Recorded> {
    let wire = worker?;
    let turn = wire.frames.iter().find_map(|(_, kind, bytes)| {
        if *kind == 0x0101 {
            Some(
                jig_protocol_worker_wire::Turn::decode(
                    &jig_protocol_worker_wire::CEILINGS,
                    &mut skein_lib::Reader::new(bytes),
                )
                .expect("native turn"),
            )
        } else {
            None
        }
    })?;
    let answer = wire.frames.iter().find_map(|(_, kind, bytes)| {
        if *kind == 0x0103 {
            Some(
                jig_protocol_worker_wire::Answer::decode(
                    &jig_protocol_worker_wire::CEILINGS,
                    &mut skein_lib::Reader::new(bytes),
                )
                .expect("native answer"),
            )
        } else {
            None
        }
    })?;
    Some(Recorded { turn, answer })
}
