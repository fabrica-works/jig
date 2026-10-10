use super::*;
use crate::Bounds;
use crate::Decision as ApplicationDecision;
use jig_client::contract::Decision;
use jig_client::contract::*;
use skein_lib::List;
mod policy;
mod recurring;
fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), 4096).expect("fixture text fits")
}
fn list<T>(items: Vec<T>) -> List<T> {
    let mut out = List::with_capacity(u32::try_from(items.len()).expect("fixture list fits"));
    for item in items {
        assert!(out.push(item).is_ok());
    }
    out
}
fn party(number: u64) -> Party {
    Party { number, kind: PartyKind::Person, name: text(b"party") }
}
fn budget() -> Budget {
    Budget { total: 100, own: 0, ended: 0, reserved: 0, left: 100, funder: Actor::Deployment }
}
fn frame(number: u64) -> Frame {
    Frame {
        party: party(number),
        projects: list(vec![ProjectRole {
            project: 1,
            name: text(b"project"),
            roles: list(vec![1]),
            chats: list(vec![Charter { number: 1, name: text(b"chat") }]),
            goals: list(vec![]),
            pool: budget(),
            chat_budget: 100,
        }]),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        updates: Count { value: 0, limited: false },

        retention: Duration::from_secs(1),
        deployment_role: Some(1),
    }
}
fn authority() -> Authority {
    Authority {
        budget: 100,
        grants: list(vec![]),
        task_kinds: list(vec![]),
        depth: 10,
        tasks: 10,
        deadline: None,
        tools: list(vec![]),
        note_scopes: list(vec![]),
    }
}
fn script() -> Script {
    let mut truth = Truth::default();
    truth.frames.insert(1, frame(1));
    truth.frames.insert(2, frame(2));
    truth.sign_ins.insert(11, 1);
    truth.sign_ins.insert(12, 1);
    truth.sign_ins.insert(22, 2);
    Script { truth, ..Script::default() }
}
fn send(engine: &mut Engine, now: u64, sign_in: u64, key: u8, token: u64, ask: Ask) {
    engine
        .send(Time::from_nanos(now), Wall::from_nanos(now), sign_in, [key; 16], Token::new(token), ask)
        .expect("typed request fits");
}
fn final_answer(up: &Up) -> &Answer {
    match up {
        Up::Answer { envelope: Envelope::Final, answer: Some(answer), .. } => answer,
        Up::ApplicationAnswer { .. }
        | Up::ApplicationRead { .. }
        | Up::App { .. }
        | Up::Read { .. }
        | Up::Watch { .. }
        | Up::Ended { .. }
        | Up::Answer { .. } => {
            panic!("expected a final answer: {up:?}")
        }
    }
}
fn proposal() -> Proposal {
    Proposal {
        key: ProposalKey { proposer: Proposer::Task(8), number: 1 },
        project: 1,
        proposer: Proposer::Task(8),
        why: text(b"wider"),
        action: ProposalAction::Batch { tasks: list(vec![]), own: false },
        needs: authority(),
        funding: budget(),
        waiting: Recipient::Role { project: 1, role: 1 },
        trail: list(vec![]),
        since: Wall::EPOCH,
        acts: list(vec![Offered { act: ActKind::Accept, admission: Admission::Allowed }]),
        ending: None,
    }
}
#[test]
fn keyed_contract_durability_sign_ins_conflict_and_decided_first() {
    let mut setup = script();
    let p = proposal();
    setup.truth.objects.insert(ObjectKey::Proposal(p.key), ReadValue::Proposal(Box::new(p.clone())));
    let ask = Ask::DecideProposal { project: 1, proposal: p.key, decision: Decision::Accepted };
    let mut engine = Engine::new(setup, 1).expect("valid script");
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    send(&mut engine, 1, 12, 1, 2, ask.clone());
    send(&mut engine, 2, 22, 2, 3, ask.clone());
    assert_eq!(engine.next_due(), Some(Time::from_nanos(10_000_000)));
    assert!(engine.poll(Time::from_nanos(9_999_999)).is_empty());
    assert!(engine.commits().is_empty());
    let up = engine.poll(Time::from_nanos(10_000_002));
    assert_eq!(up.len(), 3);
    assert_eq!(final_answer(&up[0]), final_answer(&up[1]));
    assert_eq!(
        final_answer(&up[2]),
        &Answer::Done(Outcome::DecidedFirst { by: party(1), how: Decision::Accepted, at: Wall::EPOCH })
    );
    assert_eq!(engine.commits().len(), 2);
    assert_eq!(engine.commits()[0].durable, Some(Time::from_nanos(10_000_000)));
    send(
        &mut engine,
        20_000_000,
        11,
        1,
        4,
        Ask::DecideProposal { project: 1, proposal: p.key, decision: Decision::PassedUp },
    );
    assert_eq!(final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]), &Answer::Refused(Refusal::KeyConflict));
    send(&mut engine, 20_000_000, 11, 1, 5, ask);
    assert_eq!(final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]), final_answer(&up[0]));
    assert_eq!(engine.commits().len(), 2);
}
fn nested_chip_payload_bounds(chip: &Chip) {
    for details in [
        ChipPhase::Running {
            attempt: 1,
            since: Wall::EPOCH,
            tool: Some(Text::new(vec![0; 4097].into_boxed_slice(), 5000).unwrap()),
        },
        ChipPhase::Idle {
            wake: WakePolicy {
                words: true,
                results: true,
                topics: List::with_capacity(33),
                subscriptions: list(vec![]),
            },
        },
    ] {
        let mut invalid_setup = script();
        let mut invalid_chip = chip.clone();
        *invalid_chip.details = details;
        invalid_setup.truth.object_projects.insert(ObjectKey::Question { task: 8, message: 1 }, 1);
        invalid_setup.truth.objects.insert(
            ObjectKey::Question { task: 8, message: 1 },
            ReadValue::Question(Box::new(Question {
                task: invalid_chip,
                message: 1,
                words: text(b"question"),
                since: Wall::EPOCH,
                acts: list(vec![]),
                ending: None,
            })),
        );
        assert_eq!(
            Engine::new(invalid_setup, 1).err(),
            Some(Invalid::Payload),
            "nested chip phase facts are admitted whole"
        );
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "whole revision and retention lifecycle retains all independent assertions")]
fn revision_refusals_scripts_retention_and_whole_payload_admission() {
    bounded_offer_payloads_and_predictions();
    let mut setup = script();
    let chip = Chip {
        task: 8,
        title: text(b"held"),
        executor: Executor::Agent { charter: 1 },
        phase: Phase::Held,
        details: Box::new(ChipPhase::Held {
            reason: HoldReason::Budget { spent: 100, limit: 100 },
            since: Wall::EPOCH,
            revision: Revision { value: 2 },
            was: None,
        }),
    };
    nested_chip_payload_bounds(&chip);
    let escalation = Escalation {
        ending: None,
        task: chip,
        hold: Hold {
            revision: Revision { value: 2 },
            reason: HoldReason::Budget { spent: 100, limit: 100 },
            before: Phase::Running,
            since: Wall::EPOCH,
            result: None,
            spend: list(vec![SpendLine { task: 8, attempt: None, spent: 100 }]),
        },
        steering: Box::new(Steering {
            person_revision: None,
            taken_by: None,
            addressed: None,
            revision: Revision { value: 1 },
            spec: specification(text(b"held work")),
            instructions: Some(DocumentText::complete(text(b""))),
            wake: WakePolicy {
                words: true,
                results: true,
                topics: List::with_capacity(0),
                subscriptions: List::with_capacity(0),
            },
            dependencies: List::with_capacity(0),
            authority: authority(),
            budget: Budget { total: 100, own: 100, ended: 0, reserved: 0, left: 0, funder: Actor::Party(1) },
        }),
        impact: Impact {
            tasks: PhaseCounts { waiting: 0, running: 0, held: 1, closing: 0, done: 0, failed: 0, cancelled: 0 },
            live_runs: 0,
            proposals: 0,
            effects_pending: 0,
            effects_settling: 0,
            takeover_budget: 0,
        },
        waiting: Recipient::Party(1),
        trail: list(vec![]),
        acts: list(vec![Offered { act: ActKind::Release, admission: Admission::Allowed }]),
    };
    nested_escalation_payload_bounds(&escalation);
    setup.truth.object_projects.insert(ObjectKey::Escalation(8), 1);
    setup.truth.objects.insert(ObjectKey::Escalation(8), ReadValue::Escalation(Box::new(escalation)));
    let mut engine = Engine::new(setup, 2).expect("valid script");
    let ask = Ask::DecideEscalation {
        project: 1,
        task: 8,
        revision: Revision { value: 1 },
        decision: Decision::Released { again: None },
    };
    send(&mut engine, 0, 11, 1, 1, ask);
    assert_eq!(
        final_answer(&engine.poll(Time::from_nanos(10_000_000))[0]),
        &Answer::Refused(Refusal::Moved {
            now: Revision { value: 2 },
            what: Moved::Hold {
                reason: Some(CurrentHold::Document(Box::new(HoldReason::Budget { spent: 100, limit: 100 })))
            }
        })
    );
    let mut setup = script();
    let ask = Ask::StartChat { project: 1, words: text(b"hello") };
    setup.rules.push(Rule {
        party: None,
        ask: ask.clone(),
        answer: Answer::Refused(Refusal::Authority {
            lacking: Lacking::Budget { needed: 100, left: 5 },
            proposable: None,
        }),
        changes: vec![],
    });
    let mut engine = Engine::new(setup, 3).expect("valid script");
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    assert!(matches!(
        final_answer(&engine.poll(Time::from_nanos(10_000_000))[0]),
        Answer::Refused(Refusal::Authority { .. })
    ));
    send(&mut engine, 2_000_000_000, 11, 1, 2, ask);
    drop(engine.poll(Time::from_nanos(2_010_000_000)));
    assert_eq!(engine.commits().len(), 2, "retention expired before reuse");
    let mut setup = script();
    setup.bounds.document_bytes = 1;
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Payload));
    let mut setup = script();
    setup.truth.frames.get_mut(&1).expect("frame").projects = List::with_capacity(33);
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Payload), "unused list capacity is part of a whole payload");
    let mut engine = Engine::new(script(), 1).expect("valid script");
    let ask = Ask::StartChat {
        project: 1,
        words: Text::new(vec![0; 4097].into_boxed_slice(), 5000).expect("larger producer permits text"),
    };
    assert_eq!(engine.send(Time::ZERO, Wall::EPOCH, 11, [1; 16], Token::new(1), ask), Err(Invalid::Payload));
    assert!(engine.commits().is_empty());
}
fn counts() -> PhaseCounts {
    PhaseCounts { waiting: 0, running: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 }
}
fn task() -> Task {
    Task {
        chip: Chip {
            task: 8,
            title: text(b"chat"),
            executor: Executor::Agent { charter: 1 },
            phase: Phase::Running,
            details: Box::new(ChipPhase::Running { attempt: 1, since: Wall::EPOCH, tool: None }),
        },
        goal: false,
        priority: None,
        lineage_chips: list(vec![]),
        inputs: list(vec![]),
        project: 1,
        requester: Actor::Party(1),
        lineage: list(vec![]),
        spec: specification(text(b"spec")),
        spec_revision: Revision { value: 1 },
        person_revision: None,
        taken_by: None,
        instructions: Some(DocumentText::complete(text(b"instructions"))),
        wake: WakePolicy { words: true, results: true, topics: list(vec![]), subscriptions: list(vec![]) },
        result_contract: ResultContract::Report,
        dependencies: list(vec![]),
        references: list(vec![]),
        resources: list(vec![]),
        budget: budget(),
        authority: authority(),
        hold: None,
        escalation: None,
        proposals: list(vec![]),
        activity: 0,
        deadline: None,
        changed: Wall::EPOCH,
        acts: list(vec![]),
        parts: list(vec![]),
        closing: None,
        spend: list(vec![]),
        result: None,
        runtime: Runtime::Running { attempt: 1, since: Wall::EPOCH, tool: None },
        impact: Impact {
            tasks: counts(),
            live_runs: 1,
            proposals: 0,
            effects_pending: 0,
            effects_settling: 0,
            takeover_budget: 0,
        },
        delegates: list(vec![]),
        writer_slots: list(vec![]),
    }
}
fn live_script() -> Script {
    let mut setup = script();
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(task())));
    setup.truth.trees.insert(8, Tree { task: 8, nodes: list(vec![]), folded: counts(), held_below: 0 });
    setup.truth.deployment = Some(Deployment {
        hosts: list(vec![]),
        accounts: list(vec![]),
        capacity: list(vec![]),
        proposals: 0,
        refusals: list(vec![]),
        parts: list(vec![]),
    });
    setup
}
fn at(nanos: u64) -> Time {
    Time::from_nanos(nanos)
}
fn scenario_pages_and_read_barriers() {
    let mut setup = live_script();
    setup.bounds.rows = 1;
    setup.truth.sources.push(Source {
        party: 1,
        read: Read::History { task: 8, subtree: false, after: None },
        rows: Rows::History(
            (0..3)
                .map(|position| HistoryLine {
                    position,
                    task: 8,
                    by: Actor::Party(1),
                    at: Wall::EPOCH,
                    change: HistoryChange::Released,
                })
                .collect(),
        ),
    });
    let mut engine = Engine::new(setup, 4).expect("source rows span pages");
    let mut changed = task();
    changed.chip.title = text(b"latest");
    engine
        .act(
            Time::ZERO,
            Wall::EPOCH,
            Change::Object { project: 1, key: ObjectKey::Task(8), value: ReadValue::Task(Box::new(changed.clone())) },
        )
        .expect("whole task");
    engine.read(Time::ZERO, 11, Token::new(1), Read::Task { task: 8 }).expect("read");
    assert!(engine.poll(at(9_999_999)).is_empty());
    assert_eq!(
        engine.poll(at(10_000_000)),
        vec![Up::Read { token: Token::new(1), terminal: ReadResult::Value(ReadValue::Task(Box::new(changed))) }]
    );
    assert_eq!(engine.read(at(10_000_000), 11, Token::new(1), Read::Frame), Err(Invalid::Identity));
    let mut cursor = None;
    for position in 0..3 {
        engine
            .read(
                at(10_000_000),
                11,
                Token::new(10 + position),
                Read::History { task: 8, subtree: false, after: cursor },
            )
            .expect("page");
        let up = engine.poll(at(10_000_000));
        let Up::Read { terminal: ReadResult::Value(ReadValue::History(page)), .. } = &up[0] else {
            panic!("history page");
        };
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows.iter().next().expect("row").position, position);
        assert_eq!(page.before.is_some(), position != 0);
        assert_eq!(page.after.is_some(), position != 2);
        cursor = page.after.clone();
    }
    engine
        .read(at(10_000_000), 11, Token::new(20), Read::History { task: 8, subtree: false, after: None })
        .expect("page");
    let up = engine.poll(at(10_000_000));
    let Up::Read { terminal: ReadResult::Value(ReadValue::History(page)), .. } = &up[0] else {
        panic!("history page");
    };
    let cursor = page.after.clone();
    engine
        .read(at(10_000_000), 22, Token::new(21), Read::History { task: 8, subtree: false, after: cursor })
        .expect("foreign continuation");
    assert_eq!(
        engine.poll(at(10_000_000)),
        vec![Up::Read { token: Token::new(21), terminal: ReadResult::None(ReadNone::Gone) }]
    );
    engine.read(at(10_000_000), 999, Token::new(22), Read::Frame).expect("signed out");
    assert!(matches!(engine.poll(at(10_000_000))[0], Up::Read { terminal: ReadResult::None(ReadNone::SignedOut), .. }));
    let mut frame = frame(2);
    frame.projects = list(vec![]);
    engine.act(at(10_000_000), Wall::EPOCH, Change::Frame(frame)).expect("roles removed");
    engine.read(at(10_000_000), 22, Token::new(23), Read::Task { task: 8 }).expect("hidden read");
    assert_eq!(
        engine.poll(at(20_000_000)),
        vec![Up::Read { token: Token::new(23), terminal: ReadResult::None(ReadNone::Unknown) }]
    );
}
fn scenario_watch_subjects_coalescing_and_lifecycle() {
    let mut engine = Engine::new(live_script(), 5).expect("live truth");
    let subjects = vec![
        Subject::Frame,
        Subject::Task { task: 8 },
        Subject::Tree { task: 8, window: 1 },
        Subject::Chips { tasks: list(vec![8]) },
        Subject::Inbox { window: 1 },
        Subject::Goals { project: 1 },
        Subject::Tasks {
            project: 1,
            filter: TaskFilter { phase: None, executor: None, goal: None, asked_by_me: false },
            window: 1,
        },
        Subject::Conversations,
        Subject::Run { task: 8, turn: None, offset: 0 },
        Subject::Deployment,
    ];
    for (index, subject) in subjects.into_iter().enumerate() {
        engine.open(Time::ZERO, 11, Token::new(index as u64), subject).expect("subject");
    }
    let up = engine.poll(Time::ZERO);
    assert_eq!(up.iter().filter(|event| matches!(event, Up::Watch { event: WatchEvent::Opened, .. })).count(), 10);
    assert_eq!(
        up.iter().filter(|event| matches!(event, Up::Watch { event: WatchEvent::Delivered(_), .. })).count(),
        9,
        "idle run has no fabricated turn"
    );
    let mut changed = task();
    changed.chip.title = text(b"one");
    engine
        .act(
            Time::ZERO,
            Wall::EPOCH,
            Change::Object { project: 1, key: ObjectKey::Task(8), value: ReadValue::Task(Box::new(changed.clone())) },
        )
        .expect("one");
    changed.chip.title = text(b"two");
    engine
        .act(
            at(1),
            Wall::EPOCH,
            Change::Object { project: 1, key: ObjectKey::Task(8), value: ReadValue::Task(Box::new(changed.clone())) },
        )
        .expect("two");
    assert!(engine.poll(at(10_000_001)).is_empty(), "in-flight coalesces");
    engine.taken(Token::new(1));
    let up = engine.poll(at(10_000_001));
    assert_eq!(
        up,
        vec![Up::Watch { token: Token::new(1), event: WatchEvent::Delivered(Delivery::Task(Box::new(changed))) }]
    );
    engine.taken(Token::new(1));
    assert!(engine.poll(at(10_000_001)).is_empty());
    let up = engine.poll(at(1_010_000_001));
    assert_eq!(up.iter().filter(|event| matches!(event, Up::Watch { event: WatchEvent::Alive, .. })).count(), 10);
    engine.close(at(1_010_000_001), Token::new(1));
    engine.close(at(1_010_000_001), Token::new(1));
    assert_eq!(engine.poll(at(1_010_000_001)), vec![Up::Ended { token: Token::new(1), why: WatchEnd::Closed }]);
    let mut changed = task();
    changed.chip.phase = Phase::Done;
    *changed.chip.details = ChipPhase::Ended { how: Ending::Done, at: Wall::EPOCH };
    changed.runtime = Runtime::Ended;
    engine
        .act(
            at(1_010_000_001),
            Wall::EPOCH,
            Change::Object { project: 1, key: ObjectKey::Task(8), value: ReadValue::Task(Box::new(changed)) },
        )
        .expect("ended");
    let up = engine.poll(at(1_020_000_001));
    for id in [2, 3, 8] {
        assert!(up.contains(&Up::Ended { token: Token::new(id), why: WatchEnd::SubjectEnded }));
    }
    let mut setup = live_script();
    setup.bounds.watches = 1;
    let mut engine = Engine::new(setup, 1).expect("limited engine");
    engine.open(Time::ZERO, 11, Token::new(1), Subject::Frame).expect("first");
    engine.open(Time::ZERO, 11, Token::new(2), Subject::Frame).expect("over limit");
    assert!(engine.poll(Time::ZERO).contains(&Up::Ended { token: Token::new(2), why: WatchEnd::Busy }));
    engine.act(Time::ZERO, Wall::EPOCH, Change::SignedOut { sign_in: 11 }).expect("signed out");
    assert_eq!(engine.poll(at(10_000_000)), vec![Up::Ended { token: Token::new(1), why: WatchEnd::SignedOut }]);
}
fn scenario_run_offsets_cut_names_and_activity() {
    let mut setup = live_script();
    setup.bounds.text = 32;
    let mut engine = Engine::new(setup, 6).expect("run bound");
    let name = TurnName { attempt: 1, number: 0 };
    engine.open(Time::ZERO, 11, Token::new(1), Subject::Run { task: 8, turn: None, offset: 0 }).expect("run watch");
    drop(engine.poll(Time::ZERO));
    engine.write(Time::ZERO, 8, name, Wall::EPOCH, &text(b"abcdefghijklmnopqrst"), None).expect("first chunk");
    let up = engine.poll(Time::ZERO);
    let Up::Watch { event: WatchEvent::Delivered(Delivery::Run(run)), .. } = &up[0] else {
        panic!("run text");
    };
    assert_eq!((run.from, run.through, run.cut), (0, 20, false));
    assert_eq!(run.text.bytes(), b"abcdefghijklmnopqrst");
    engine.write(at(1), 8, name, Wall::EPOCH, &text(b"uvwxyz0123456789ABCD"), None).expect("second chunk");
    assert!(engine.poll(at(1)).is_empty());
    engine.continue_run(Token::new(1), Some(name), 20).expect("same watch cursor");
    let up = engine.poll(at(1));
    let Up::Watch { event: WatchEvent::Delivered(Delivery::Run(run)), .. } = &up[0] else {
        panic!("run text");
    };
    assert_eq!((run.from, run.through, run.cut), (20, 40, true));
    assert_eq!(run.text.bytes(), b"uvwxyz0123456789ABCD");
    engine.continue_run(Token::new(1), Some(name), 0).expect("lagging offset");
    let up = engine.poll(at(1));
    let Up::Watch { event: WatchEvent::Delivered(Delivery::Run(run)), .. } = &up[0] else {
        panic!("cut text");
    };
    assert_eq!((run.from, run.through), (8, 40));
    let second = TurnName { attempt: 1, number: 1 };
    engine.write(at(2), 8, second, Wall::EPOCH, &text(b"second"), None).expect("new turn");
    engine.continue_run(Token::new(1), Some(name), 40).expect("old turn position");
    let up = engine.poll(at(2));
    let Up::Watch { event: WatchEvent::Delivered(Delivery::Run(run)), .. } = &up[0] else {
        panic!("new turn text");
    };
    assert_eq!((run.turn, run.from, run.through), (second, 0, 6));
    engine
        .act(
            at(2),
            Wall::EPOCH,
            Change::Source(Source {
                party: 1,
                read: Read::Activity { task: 8, before: None },
                rows: Rows::Activity(vec![ActivityLine {
                    position: 1,
                    at: Wall::EPOCH,
                    event: Activity::Turn {
                        name: second,
                        turn: Turn {
                            messages: list(vec![]),
                            used: Usage { input: 1, output: 1, cached: 0, cache_written: 0 },
                            spent: 1,
                            read_through: None,
                            cut: false,
                        },
                    },
                }]),
            }),
        )
        .expect("turn committed through activity");
    assert_eq!(
        engine.truth().objects.get(&ObjectKey::Task(8)).and_then(|value| if let ReadValue::Task(task) = value {
            Some(task.activity)
        } else {
            None
        }),
        Some(1)
    );
    engine.read(at(2), 11, Token::new(2), Read::Activity { task: 8, before: None }).expect("activity");
    assert!(engine.poll(at(10_000_001)).is_empty());
    let up = engine.poll(at(10_000_002));
    assert!(
        matches!(&up[0],Up::Read { terminal:ReadResult::Value(ReadValue::Activity(page)),.. } if matches!(page.rows.iter().next().expect("turn").event,Activity::Turn { name,.. } if name == second))
    );
    assert!(!engine.runs.contains_key(&8), "durable matching activity replaces the provisional turn");
}
fn scenario_byte_bound_pages_and_watch_windows() {
    let mut setup = live_script();
    setup.bounds.document_bytes = 3000;
    setup.truth.sources.push(Source {
        party: 1,
        read: Read::History { task: 8, subtree: false, after: None },
        rows: Rows::History(
            (0..2)
                .map(|position| HistoryLine {
                    position,
                    task: 8,
                    by: Actor::Party(1),
                    at: Wall::EPOCH,
                    change: HistoryChange::Amended { before: text(&[b'a'; 1100]), after: text(&[b'b'; 1100]) },
                })
                .collect(),
        ),
    });
    let mut engine = Engine::new(setup, 8).expect("each retained row fits");
    engine
        .read(Time::ZERO, 11, Token::new(1), Read::History { task: 8, subtree: false, after: None })
        .expect("byte bounded page");
    let up = engine.poll(Time::ZERO);
    let Up::Read { terminal: ReadResult::Value(value @ ReadValue::History(page)), .. } = &up[0] else {
        panic!("history page");
    };
    assert_eq!(page.rows.len(), 1, "byte bound shrinks below the row bound");
    assert!(page.after.is_some());
    assert!(payload::fits(value, &engine.bounds));
    let mut setup = live_script();
    setup.bounds.document_bytes = 3000;
    let entries = (0..2)
        .map(|position| {
            let line = ConversationLine {
                runtime: Runtime::Idle,
                delegates: 0,
                budget: budget(),
                chip: task().chip,
                project: 1,
                first_words: text(&[b'x'; 1100]),
                unread: Count { value: 1, limited: false },
                seen: 0,
                last_reply: 1,
                changed: Wall::EPOCH,
            };
            InboxEntry {
                reply: Some(Box::new(line)),
                lineage: list(vec![]),
                goal: None,
                recipient: None,
                stall: None,
                object: ObjectKey::Task(8),
                project: 1,
                group: InboxGroup::Replies,
                position,
                at: Wall::EPOCH,
            }
        })
        .collect();
    setup.truth.sources.push(Source {
        party: 1,
        read: Read::Inbox {
            group: jig_client::contract::InboxReadGroup::Updates,
            project: None,
            kind: None,
            goal: None,
            after: None,
        },
        rows: Rows::Inbox(entries),
    });
    let mut engine = Engine::new(setup, 9).expect("each whole reply row fits");
    engine.open(Time::ZERO, 11, Token::new(2), Subject::Inbox { window: 2 }).expect("inbox");
    let up = engine.poll(Time::ZERO);
    let Up::Watch { event: WatchEvent::Delivered(value @ Delivery::Inbox(inbox)), .. } = &up[1] else {
        panic!("inbox window");
    };
    assert_eq!(inbox.entries.len(), 1, "whole nested replies shrink the byte window");
    assert!(inbox.more);
    assert!(inbox.entries.iter().next().expect("entry").reply.is_some());
    assert!(payload::fits(value, &engine.bounds));
}
fn scenario_all_read_shapes_and_reverse_activity() {
    let mut setup = live_script();
    setup.bounds.rows = 1;
    setup.truth.sources.push(Source {
        party: 1,
        read: Read::Activity { task: 8, before: None },
        rows: Rows::Activity(
            (0..3)
                .map(|position| ActivityLine {
                    position,
                    at: Wall::EPOCH,
                    event: Activity::RunBegan { attempt: 1, how: RunBeginning::Resumed, why: text(b"run") },
                })
                .collect(),
        ),
    });
    let mut engine = Engine::new(setup, 7).expect("all reads");
    let mut cursor = None;
    for position in (0..3).rev() {
        engine
            .read(Time::ZERO, 11, Token::new(10 + position), Read::Activity { task: 8, before: cursor })
            .expect("reverse page");
        let up = engine.poll(Time::ZERO);
        let Up::Read { terminal: ReadResult::Value(ReadValue::Activity(page)), .. } = &up[0] else {
            panic!("activity");
        };
        assert_eq!(page.rows.iter().next().expect("row").position, position);
        assert_eq!(page.before.is_some(), position != 0);
        cursor = page.before.clone();
    }
    let reads = vec![
        Read::Frame,
        Read::Task { task: 8 },
        Read::Proposal { proposal: ProposalKey { proposer: Proposer::Task(8), number: 1 } },
        Read::Escalation { task: 8 },
        Read::Question { task: 8, message: 1 },
        Read::PersonTask { task: 8 },
        Read::Result { task: 8 },
        Read::Ended { task: 8, after: None },
        Read::History { task: 8, subtree: true, after: None },
        Read::Inbox {
            group: jig_client::contract::InboxReadGroup::Updates,
            kind: Some(InboxGroup::Replies),
            goal: None,
            project: None,
            after: None,
        },
        Read::Inbox {
            group: jig_client::contract::InboxReadGroup::Needs,
            project: Some(1),
            kind: Some(InboxGroup::Questions),
            goal: None,
            after: None,
        },
        Read::Conversations { after: None },
        Read::Goals { project: 1, after: None },
        Read::Spend { project: 1, after: None },
        Read::Project { project: 1 },
        Read::Notes { scope: NoteScope::Project(1), description: Some(text(b"match")), after: None },
        Read::Deployment,
    ];
    for (index, read) in reads.into_iter().enumerate() {
        let token = Token::new(100 + index as u64);
        engine.read(Time::ZERO, 11, token, read).expect("contract read shape");
        let up = engine.poll(Time::ZERO);
        assert_eq!(up.len(), 1);
        assert!(matches!(up[0],Up::Read { token:id,.. } if id == token));
        assert!(engine.poll(Time::ZERO).is_empty());
    }
    scenario_reply_identity();
}
fn scenario_reply_identity() {
    let mut setup = live_script();
    let line = ConversationLine {
        runtime: Runtime::Idle,
        delegates: 0,
        budget: budget(),
        chip: task().chip,
        project: 1,
        first_words: text(b"whole reply"),
        unread: Count { value: 1, limited: false },
        seen: 0,
        last_reply: 1,
        changed: Wall::EPOCH,
    };
    let entry = InboxEntry {
        reply: Some(Box::new(line)),
        lineage: list(vec![]),
        goal: None,
        recipient: None,
        stall: None,
        object: ObjectKey::Task(9),
        project: 1,
        group: InboxGroup::Replies,
        position: 1,
        at: Wall::EPOCH,
    };
    setup.truth.sources.push(Source {
        party: 1,
        read: Read::Inbox {
            group: jig_client::contract::InboxReadGroup::Updates,
            project: None,
            kind: None,
            goal: None,
            after: None,
        },
        rows: Rows::Inbox(vec![entry]),
    });
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Identity), "reply task identity is whole and authoritative");
}
#[test]
fn reads_watches_and_run_scenarios() {
    scenario_byte_bound_pages_and_watch_windows();
    scenario_all_read_shapes_and_reverse_activity();
    scenario_pages_and_read_barriers();
    scenario_watch_subjects_coalescing_and_lifecycle();
    scenario_run_offsets_cut_names_and_activity();
}

fn nested_escalation_payload_bounds(escalation: &Escalation) {
    for text_or_capacity in [true, false] {
        let mut invalid_setup = script();
        let mut row = escalation.clone();
        if text_or_capacity {
            row.steering.spec = specification(Text::new(vec![0; 4097].into_boxed_slice(), 5000).unwrap());
        } else {
            row.hold.spend = List::with_capacity(33);
        }
        invalid_setup.truth.object_projects.insert(ObjectKey::Escalation(8), 1);
        invalid_setup.truth.objects.insert(ObjectKey::Escalation(8), ReadValue::Escalation(Box::new(row)));
        assert_eq!(
            Engine::new(invalid_setup, 1).err(),
            Some(Invalid::Payload),
            "whole nested steering and hold spend are admitted"
        );
    }
}

mod application;
mod faults;
#[test]
fn restart_faults_and_application_hooks_keep_one_durable_truth() {
    faults::run();
    application::run();
}

fn bounded_offer_payloads_and_predictions() {
    use jig_client::contract::Bounds as DraftBounds;
    closing_connector_payload_bounds();
    let mut direct = authority();
    direct.budget = 110;
    let mut ceiling = direct.clone();
    ceiling.budget = 200;
    let within = DraftBounds::Amend {
        budget_left: 10,
        grants: list(vec![]),
        authority: Box::new(direct),
        ceiling: Box::new(ceiling),
    };
    let offer = Admission::Bounded { within, beyond: Beyond::Proposed { to: Recipient::Party(2) } };
    for (budget, answer, valid) in [
        (110, Answer::Done(Outcome::Changed), true),
        (
            111,
            Answer::Done(Outcome::Proposed { proposal: ProposalKey { proposer: Proposer::Task(8), number: 1 } }),
            true,
        ),
        (
            110,
            Answer::Done(Outcome::Proposed { proposal: ProposalKey { proposer: Proposer::Task(8), number: 1 } }),
            false,
        ),
        (111, Answer::Done(Outcome::Changed), false),
    ] {
        let mut setup = live_script();
        let mut row = task();
        row.acts = list(vec![Offered { act: ActKind::Amend, admission: offer.clone() }]);
        setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
        let mut after = authority();
        after.budget = budget;
        let ask = Ask::Amend {
            project: 1,
            task: 8,
            revision: Revision { value: 1 },
            release: None,
            change: Amendment {
                addressed: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority: Some(after),
                instructions: None,
            },
        };
        setup.rules.push(Rule { party: Some(1), ask: ask.clone(), answer: answer.clone(), changes: vec![] });
        let mut engine = Engine::new(setup, 1).unwrap();
        let result = engine.send(Time::ZERO, Wall::EPOCH, 11, [1; 16], Token::new(1), ask);
        if valid {
            result.unwrap();
            assert_eq!(final_answer(&engine.poll(at(10_000_000))[0]), &answer);
        } else {
            assert_eq!(result, Err(Invalid::Script));
            assert!(engine.commits().is_empty());
        }
    }
    for variant in 0..5 {
        let mut setup = live_script();
        let mut row = task();
        let within = if variant == 0 {
            DraftBounds::Goal {
                pool_left: 1,
                procedures: List::with_capacity(0),
                charters: List::with_capacity(33),
                latest: None,
                budget: 1,
                maximum_budget: 2,
                maximum_deadline: None,
            }
        } else {
            let mut direct = authority();
            let mut ceiling = authority();
            let mut grants = list(vec![]);
            if variant == 1 {
                grants = List::with_capacity(33);
            }
            if variant == 2 {
                direct.tools = List::with_capacity(33);
            }
            if variant == 3 {
                ceiling.grants = list(vec![Grant {
                    connector: 1,
                    kind: 1,
                    pattern: Text::new(vec![0; 4097].into_boxed_slice(), 5000).unwrap(),
                }]);
            }
            if variant == 4 {
                ceiling.note_scopes = List::with_capacity(33);
            }
            DraftBounds::Amend { budget_left: 1, grants, authority: Box::new(direct), ceiling: Box::new(ceiling) }
        };
        row.acts = list(vec![Offered {
            act: ActKind::Amend,
            admission: Admission::Bounded { within, beyond: Beyond::Proposed { to: Recipient::Party(2) } },
        }]);
        setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
        assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Payload), "all nested bounded facts are priced");
    }
}

fn closing_connector_payload_bounds() {
    let mut setup = live_script();
    let mut row = task();
    row.chip.phase = Phase::Closing;
    row.chip.details = Box::new(ChipPhase::Closing(Closing {
        delegates: 1,
        connectors: Some(List::with_capacity(33)),
        stage: None,
        ending: None,
    }));
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Payload));
}

#[test]
fn recurring_optional_boundaries_keep_whole_value_and_allocated_list_limits() {
    let mut bounds = Bounds { rows: 2, ..Bounds::default() };
    for next in [None, Some(Wall::from_nanos(u64::MAX))] {
        let mut chip = task().chip;
        chip.title = text(b"job");
        chip.details = Box::new(ChipPhase::Starting);
        let mut row = Recurring {
            chip,
            key: u32::MAX,
            batch: list(vec![]),
            budget: 20,
            overlap: RecurringOverlap::Wait,
            last_period: 8,
            pending_period: Some(9),
            last_batch: list(vec![7, 8]),
            cut: true,
            current_period: 9,
            began: Some(Wall::from_nanos(1)),
            next,
        };
        let bytes = u64::try_from(core::mem::size_of::<Recurring>()).unwrap()
            + u64::try_from(core::mem::size_of::<ChipPhase>()).unwrap()
            + List::<u64>::worst_case(2).unwrap()
            + 3;
        bounds.document_bytes = bytes;
        assert!(crate::payload::fits(&row, &bounds));
        bounds.document_bytes = bytes.checked_sub(1).unwrap();
        assert!(!crate::payload::fits(&row, &bounds));
        bounds.document_bytes = u64::MAX;
        row.last_batch = List::with_capacity(3);
        assert!(!crate::payload::fits(&row, &bounds));
    }
}

#[test]
fn collective_policy_recipients_accept_any_listed_role_and_price_the_whole_list() {
    let recipient = Recipient::Roles { project: 1, roles: list(vec![7, 1]) };
    let mut setup = script();
    setup.truth.frames.get_mut(&2).unwrap().projects.get_mut(0).unwrap().roles = list(vec![99]);
    assert!(crate::requests::recipient(&setup.truth, 1, &recipient));
    assert!(!crate::requests::recipient(&setup.truth, 2, &recipient));
    let mut row = proposal();
    row.waiting = recipient.clone();
    setup.truth.objects.insert(ObjectKey::Proposal(row.key), ReadValue::Proposal(Box::new(row.clone())));
    let mut engine = Engine::new(setup, 1).unwrap();
    send(&mut engine, 0, 11, 1, 1, Ask::DecideProposal { project: 1, proposal: row.key, decision: Decision::Accepted });
    let up = engine.poll(Time::from_nanos(10_000_000));
    assert_eq!(
        final_answer(&up[0]),
        &Answer::Done(Outcome::Decided { by: Actor::Party(1), how: Decision::Accepted, at: Wall::EPOCH })
    );
    let mut bounds = Bounds { rows: 2, ..Bounds::default() };
    bounds.document_bytes =
        u64::try_from(core::mem::size_of::<Recipient>()).unwrap() + List::<u32>::worst_case(2).unwrap();
    assert!(crate::payload::fits(&recipient, &bounds));
    bounds.document_bytes = bounds.document_bytes.checked_sub(1).unwrap();
    assert!(!crate::payload::fits(&recipient, &bounds));
    for bounded in [false, true] {
        let mut setup = script();
        let mut row = row.clone();
        let recipient = Recipient::Roles { project: 1, roles: List::with_capacity(33) };
        row.acts.get_mut(0).unwrap().admission = if bounded {
            Admission::Bounded {
                within: jig_client::contract::Bounds::Goal {
                    pool_left: 10,
                    procedures: List::with_capacity(0),
                    charters: list(vec![1]),
                    latest: None,
                    budget: 0,
                    maximum_budget: 100,
                    maximum_deadline: None,
                },
                beyond: Beyond::Proposed { to: recipient },
            }
        } else {
            Admission::Proposed { to: recipient }
        };
        setup.truth.objects.insert(ObjectKey::Proposal(row.key), ReadValue::Proposal(Box::new(row)));
        assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Payload));
    }
}

#[test]
fn amendment_release_checks_the_named_hold_separately_from_the_spec() {
    let change = Change::Request(Box::new(Ask::MarkSeen { seen: Seen::Results { through: 1 } }));
    let mut exact = Bounds {
        document_bytes: u64::try_from(core::mem::size_of::<Box<Ask>>())
            .unwrap()
            .checked_add(u64::try_from(core::mem::size_of::<Ask>()).unwrap())
            .unwrap(),
        ..Bounds::default()
    };
    assert_eq!(change.validate(&exact), Ok(()));
    exact.document_bytes = exact.document_bytes.checked_sub(1).unwrap();
    assert_eq!(change.validate(&exact), Err(Invalid::Payload));
    for stale_hold in [false, true] {
        let mut setup = script();
        let mut row = task();
        row.hold = Some(Hold {
            revision: Revision { value: 7 },
            reason: HoldReason::Stopped { by: party(1), at: Wall::EPOCH },
            before: Phase::Running,
            since: Wall::EPOCH,
            result: None,
            spend: list(vec![]),
        });
        row.acts = list(vec![Offered { act: ActKind::AmendRelease, admission: Admission::Allowed }]);
        setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
        let mut engine = Engine::new(setup, 1).unwrap();
        send(
            &mut engine,
            0,
            11,
            1,
            1,
            Ask::Amend {
                project: 1,
                task: 8,
                revision: Revision { value: if stale_hold { 1 } else { 2 } },
                release: Some(Revision { value: if stale_hold { 6 } else { 7 } }),
                change: Amendment {
                    addressed: None,
                    spec: Some(specification(text(b"new spec")).whole().unwrap()),
                    wake: None,
                    dependencies: None,
                    authority: None,
                    instructions: None,
                },
            },
        );
        assert_eq!(
            final_answer(&engine.poll(Time::from_nanos(10_000_000))[0]),
            &Answer::Refused(Refusal::Moved {
                now: Revision { value: if stale_hold { 7 } else { 1 } },
                what: if stale_hold {
                    Moved::Hold {
                        reason: Some(CurrentHold::Document(Box::new(HoldReason::Stopped {
                            by: party(1),
                            at: Wall::EPOCH,
                        }))),
                    }
                } else {
                    Moved::Spec
                },
            })
        );
    }
}

#[test]
fn moved_hold_refusal_prices_the_complete_reason_allocation_and_each_nested_failure() {
    let failures = list(vec![
        AttemptFailure { attempt: 1, class: FailureClass::HostLost, reason: text(b"first"), at: Wall::EPOCH },
        AttemptFailure { attempt: 2, class: FailureClass::Temporary, reason: text(b"second"), at: Wall::EPOCH },
    ]);
    let answer = Answer::Refused(Refusal::Moved {
        now: Revision { value: 8 },
        what: Moved::Hold { reason: Some(CurrentHold::Document(Box::new(HoldReason::Tries { failures }))) },
    });
    let mut bounds = Bounds {
        document_bytes: u64::try_from(core::mem::size_of::<Answer>())
            .unwrap()
            .checked_add(u64::try_from(core::mem::size_of::<HoldReason>()).unwrap())
            .unwrap()
            .checked_add(List::<AttemptFailure>::worst_case(2).unwrap())
            .unwrap()
            .checked_add(11)
            .unwrap(),
        ..Bounds::default()
    };
    assert!(crate::payload::fits(&answer, &bounds));
    bounds.document_bytes = bounds.document_bytes.checked_sub(1).unwrap();
    assert!(!crate::payload::fits(&answer, &bounds));
    bounds.document_bytes = u64::MAX;
    bounds.rows = 1;
    assert!(!crate::payload::fits(&answer, &bounds));
    bounds.rows = 2;
    bounds.text = 5;
    assert!(!crate::payload::fits(&answer, &bounds));
}

#[test]
fn verdict_documents_price_whole_machine_field_arrays_and_bytes() {
    let fields = list(vec![ResultField { name: b"state".as_slice().into(), value: Box::new([0, 255]) }]);
    let value =
        ResultValue::Verdict { verdict: u32::MAX, words: text(b"why"), fields, tasks: list(vec![]), proposal: None };
    let mut bounds = Bounds { text: 8, rows: 1, ..Bounds::default() };
    bounds.document_bytes = u64::try_from(core::mem::size_of::<ResultValue>()).unwrap()
        + List::<ResultField>::worst_case(1).unwrap()
        + List::<Chip>::worst_case(0).unwrap()
        + 10;
    assert!(crate::payload::fits(&value, &bounds));
    bounds.document_bytes -= 1;
    assert!(!crate::payload::fits(&value, &bounds));
    bounds.document_bytes += 1;
    bounds.text = 4;
    assert!(!crate::payload::fits(&value, &bounds), "field name is whole or refused");
    let mut names = List::with_capacity(2);
    names.push(FieldRule { name: b"state".as_slice().into(), max: u32::MAX }).unwrap();
    let option = OptionLine {
        number: u32::MAX,
        words: text(b"choice"),
        words_max: u32::MAX,
        followups: u32::MAX,
        fields: names,
    };
    bounds.text = 8;
    bounds.document_bytes = u64::MAX;
    assert!(!crate::payload::fits(&option, &bounds), "allocated rule capacity also consumes rows");
    bounds.rows = 2;
    assert!(crate::payload::fits(&option, &bounds));
}

#[test]
fn approved_documents_preserve_opaque_state_optional_words_and_exact_byte_bounds() {
    let mut bounds = Bounds { text: 3, document_bytes: u64::MAX, ..Bounds::default() };
    let absent = ResultValue::Approved { state: Box::new([0, 255, 1]), words: None };
    let empty = ResultValue::Approved { state: Box::new([0, 255, 1]), words: Some(text(b"")) };
    assert_ne!(absent, empty);
    for value in [absent, empty, ResultValue::Approved { state: Box::new([0, 255, 1]), words: Some(text(b"yes")) }] {
        let ResultValue::Approved { words, .. } = &value else {
            unreachable!("approval fixture");
        };
        let explanation = words.as_ref().map_or(0, |words| words.bytes().len());
        bounds.document_bytes = u64::try_from(core::mem::size_of::<ResultValue>() + 3 + explanation).unwrap();
        assert!(crate::payload::fits(&value, &bounds));
        bounds.document_bytes -= 1;
        assert!(!crate::payload::fits(&value, &bounds));
        bounds.document_bytes += 1;
        bounds.text = 2;
        assert!(!crate::payload::fits(&value, &bounds), "whole opaque state is refused rather than cut");
        bounds.text = 3;
    }
    let failure = ResultValue::Failure(text(b"bad"));
    bounds.document_bytes = u64::try_from(core::mem::size_of::<ResultValue>() + 3).unwrap();
    assert!(crate::payload::fits(&failure, &bounds));
    bounds.document_bytes -= 1;
    assert!(!crate::payload::fits(&failure, &bounds));
}

fn specification(words: jig_client::contract::Text) -> jig_client::contract::SpecificationDocument {
    jig_client::contract::SpecificationDocument::complete(jig_client::contract::Specification {
        words,
        parameters: List::with_capacity(0),
        inputs: List::with_capacity(0),
    })
}

#[test]
fn whole_specification_prices_parameter_and_input_capacity_and_refuses_partial_values() {
    let mut parameters = List::with_capacity(4);
    parameters.push(Parameter::Number { name: u32::MAX, value: u64::MAX }).unwrap();
    parameters.push(Parameter::Bytes { name: 7, value: text(&[0, 255, 1]) }).unwrap();
    parameters.push(Parameter::Resource { name: 8, connector: u16::MAX, resource: u64::MAX }).unwrap();
    let mut inputs = List::with_capacity(3);
    inputs.push(u64::MAX).unwrap();
    let value = {
        let mut document = SpecificationDocument::complete(Specification { words: text(b"whole"), parameters, inputs });
        document.cut = true;
        document
    };
    let allocated = u64::try_from(core::mem::size_of::<SpecificationDocument>() + 5 + 3).unwrap()
        + List::<ParameterDocument>::worst_case(4).unwrap()
        + List::<u64>::worst_case(3).unwrap();
    let mut bounds = Bounds { text: 5, rows: 4, document_bytes: allocated, ..Bounds::default() };
    assert!(crate::payload::fits(&value, &bounds));
    assert!(value.cut, "actual owner marker remains a fact");
    bounds.document_bytes -= 1;
    assert!(!crate::payload::fits(&value, &bounds));
    bounds.document_bytes = allocated;
    bounds.rows = 3;
    assert!(!crate::payload::fits(&value, &bounds), "allocated parameter slots are bounded");
    bounds.rows = 4;
    bounds.text = 4;
    assert!(!crate::payload::fits(&value, &bounds), "whole words are never clipped");
    bounds.text = 5;
    let mut too_long = value.clone();
    too_long
        .specification
        .parameters
        .get_mut(1)
        .unwrap()
        .clone_from(&ParameterDocument::Bytes { name: 7, value: DocumentText::complete(text(b"longer")) });
    bounds.document_bytes = u64::MAX;
    assert!(!crate::payload::fits(&too_long, &bounds), "whole opaque bytes parameter is never clipped");
}

fn cursor_inbox_entry(number: u64, kind: InboxGroup, at: u64, goal: Option<u64>) -> InboxEntry {
    InboxEntry {
        object: match kind {
            InboxGroup::Questions => ObjectKey::Question { task: number, message: 1 },
            InboxGroup::Results => ObjectKey::Result(number),
            InboxGroup::PersonTasks => ObjectKey::PersonTask(number),
            InboxGroup::Proposals | InboxGroup::Escalations | InboxGroup::Replies => unreachable!("fixture kinds"),
        },
        group: kind,
        project: 1,
        lineage: list(vec![]),
        goal,
        recipient: None,
        stall: None,
        reply: None,
        position: number,
        at: Wall::from_nanos(at),
    }
}

#[test]
fn inbox_group_cursors_keep_exact_filters_and_whole_query_order() {
    let mut setup = live_script();
    setup.bounds.rows = 1;
    for (group, rows) in [
        (
            InboxReadGroup::Needs,
            vec![
                cursor_inbox_entry(6, InboxGroup::PersonTasks, 1, Some(9)),
                cursor_inbox_entry(8, InboxGroup::Questions, 3, Some(9)),
                cursor_inbox_entry(7, InboxGroup::Questions, 2, Some(9)),
                cursor_inbox_entry(5, InboxGroup::Questions, 0, Some(8)),
            ],
        ),
        (
            InboxReadGroup::Updates,
            vec![
                cursor_inbox_entry(10, InboxGroup::Results, 4, Some(9)),
                cursor_inbox_entry(11, InboxGroup::Results, 5, Some(9)),
            ],
        ),
    ] {
        setup.truth.sources.push(Source {
            party: 1,
            read: Read::Inbox { group, project: None, kind: None, goal: None, after: None },
            rows: Rows::Inbox(rows),
        });
    }
    let mut engine = Engine::new(setup, 92).expect("whole owner collections fit");
    let needs = Read::Inbox {
        group: InboxReadGroup::Needs,
        project: Some(1),
        kind: Some(InboxGroup::Questions),
        goal: Some(9),
        after: None,
    };
    let updates = Read::Inbox {
        group: InboxReadGroup::Updates,
        project: Some(1),
        kind: Some(InboxGroup::Results),
        goal: Some(9),
        after: None,
    };
    engine.read(Time::ZERO, 11, Token::new(1), needs.clone()).unwrap();
    let up = engine.poll(Time::ZERO);
    let Up::Read { terminal: ReadResult::Value(ReadValue::Inbox(page)), .. } = &up[0] else {
        panic!("needs page");
    };
    assert_eq!(
        page.rows.get(0).unwrap().object,
        ObjectKey::Question { task: 7, message: 1 },
        "filter before bounded page slicing"
    );
    let needs_cursor = page.after.clone().unwrap();
    engine.read(Time::ZERO, 11, Token::new(2), updates.clone()).unwrap();
    let up = engine.poll(Time::ZERO);
    let Up::Read { terminal: ReadResult::Value(ReadValue::Inbox(page)), .. } = &up[0] else {
        panic!("updates page");
    };
    assert_eq!(page.rows.get(0).unwrap().object, ObjectKey::Result(11));
    let updates_cursor = page.after.clone().unwrap();
    for (token, mut query, cursor, wanted) in [
        (3, needs.clone(), needs_cursor.clone(), ObjectKey::Question { task: 8, message: 1 }),
        (4, updates.clone(), updates_cursor, ObjectKey::Result(10)),
    ] {
        if let Read::Inbox { after, .. } = &mut query {
            *after = Some(cursor);
        }
        engine.read(Time::ZERO, 11, Token::new(token), query).unwrap();
        let up = engine.poll(Time::ZERO);
        let Up::Read { terminal: ReadResult::Value(ReadValue::Inbox(page)), .. } = &up[0] else {
            panic!("independent continuation");
        };
        assert_eq!(page.rows.get(0).unwrap().object, wanted);
        assert!(page.after.is_none());
    }
    let mut wrong = updates;
    if let Read::Inbox { after, .. } = &mut wrong {
        *after = Some(needs_cursor);
    }
    engine.read(Time::ZERO, 11, Token::new(5), wrong).unwrap();
    assert!(matches!(engine.poll(Time::ZERO)[0], Up::Read { terminal: ReadResult::None(ReadNone::Gone), .. }));
    engine.open(Time::ZERO, 11, Token::new(6), Subject::Inbox { window: 1 }).unwrap();
    let up = engine.poll(Time::ZERO);
    let Up::Watch { event: WatchEvent::Delivered(Delivery::Inbox(value)), .. } = &up[1] else {
        panic!("shared live watch");
    };
    assert_eq!(
        value.entries.get(0).unwrap().object,
        ObjectKey::Question { task: 5, message: 1 },
        "live watch remains unfiltered"
    );
    assert!(value.more);
}

#[test]
fn goals_read_from_start_includes_live_and_ended_while_watch_is_live() {
    let mut setup = live_script();
    let goal = |number, ended| {
        let mut chip = task().chip;
        chip.task = number;
        if ended {
            chip.phase = Phase::Done;
            chip.details = Box::new(ChipPhase::Ended { how: Ending::Done, at: Wall::EPOCH });
        }
        GoalLine {
            chip,
            priority: 1,
            progress: PhaseCounts { waiting: 0, running: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 },
            budget: budget(),
            deadline: None,
            held_below: 0,
            changed: Wall::EPOCH,
        }
    };
    setup.truth.sources.push(Source {
        party: 1,
        read: Read::Goals { project: 1, after: None },
        rows: Rows::Goals(vec![goal(8, false), goal(9, true)]),
    });
    let mut engine = Engine::new(setup, 93).unwrap();
    engine.read(Time::ZERO, 11, Token::new(1), Read::Goals { project: 1, after: None }).unwrap();
    let up = engine.poll(Time::ZERO);
    let Up::Read { terminal: ReadResult::Value(ReadValue::Goals(page)), .. } = &up[0] else {
        panic!("one goal list from start");
    };
    assert_eq!(page.rows.len(), 2);
    assert_eq!(page.rows.get(0).unwrap().chip.task, 8);
    assert_eq!(page.rows.get(1).unwrap().chip.phase, Phase::Done);
    engine.open(Time::ZERO, 11, Token::new(2), Subject::Goals { project: 1 }).unwrap();
    let up = engine.poll(Time::ZERO);
    let Up::Watch { event: WatchEvent::Delivered(Delivery::Goals(value)), .. } = &up[1] else {
        panic!("live board");
    };
    assert_eq!(value.rows.len(), 1);
    assert_eq!(value.rows.get(0).unwrap().chip.task, 8);
    assert!(!value.more);
}

#[test]
fn procedure_goal_bounds_price_allocated_pairs_and_judge_the_exact_executor() {
    let exact = GoalExecutor::Procedure { connector: u16::MAX, code: u32::MAX };
    let offer = Admission::Bounded {
        within: jig_client::contract::Bounds::Goal {
            pool_left: 100,
            charters: list(vec![]),
            procedures: list(vec![
                GoalProcedure { connector: u16::MAX, code: u32::MAX },
                GoalProcedure { connector: 7, code: u32::MAX },
            ]),
            latest: None,
            budget: 3,
            maximum_budget: 10,
            maximum_deadline: None,
        },
        beyond: Beyond::Proposed { to: Recipient::Party(2) },
    };
    let bytes = u64::try_from(core::mem::size_of::<Admission>()).unwrap()
        + List::<u32>::worst_case(0).unwrap()
        + List::<GoalProcedure>::worst_case(2).unwrap();
    let mut bounds = Bounds { rows: 2, document_bytes: bytes, ..Bounds::default() };
    assert!(crate::payload::fits(&offer, &bounds));
    bounds.document_bytes = bytes.checked_sub(1).unwrap();
    assert!(!crate::payload::fits(&offer, &bounds));
    bounds.document_bytes = u64::MAX;
    bounds.rows = 1;
    assert!(!crate::payload::fits(&offer, &bounds));
    let mut room = offer.clone();
    let Admission::Bounded { within: jig_client::contract::Bounds::Goal { procedures, .. }, .. } = &mut room else {
        panic!("goal bounds");
    };
    *procedures = List::with_capacity(3);
    bounds.rows = 2;
    assert!(!crate::payload::fits(&room, &bounds), "unused allocated room is bounded too");
    for (executor, budget, answer) in [
        (exact, 3, Answer::Done(Outcome::Changed)),
        (
            exact,
            4,
            Answer::Done(Outcome::Proposed { proposal: ProposalKey { proposer: Proposer::Task(1), number: 2 } }),
        ),
        (GoalExecutor::Procedure { connector: 6, code: u32::MAX }, 3, Answer::Refused(Refusal::NotOffered)),
        (GoalExecutor::Procedure { connector: u16::MAX, code: 7 }, 3, Answer::Refused(Refusal::NotOffered)),
        (GoalExecutor::Charter(1), 3, Answer::Refused(Refusal::NotOffered)),
        (exact, 11, Answer::Refused(Refusal::NotOffered)),
    ] {
        let mut setup = live_script();
        // A project-scoped ask reads only its actual project's offered facts.
        let project = Project {
            number: 1,
            period: 1,
            name: text(b"project"),
            spend: Budget { total: 100, own: 0, ended: 0, reserved: 0, left: 100, funder: Actor::Deployment },
            policy: Policy {
                ceiling: authority(),
                period: 1,
                spend: 0,
                roles: list(vec![]),
                requirements: list(vec![]),
                deciders: list(vec![]),
            },
            roles: list(vec![]),
            pools: list(vec![]),
            resources: list(vec![]),
            services: list(vec![]),
            recurring: list(vec![]),
            parts: list(vec![]),
            spend_lines: list(vec![]),
            acts: list(vec![Offered { act: ActKind::SetGoal, admission: offer.clone() }]),
        };
        setup.truth.projects.insert(1, project);
        let ask =
            Ask::SetGoal { project: 1, spec: text(b"exact procedure"), executor, budget, priority: 8, deadline: None };
        setup.rules.push(Rule { party: Some(1), ask: ask.clone(), answer: answer.clone(), changes: vec![] });
        let mut engine = Engine::new(setup, 1).unwrap();
        send(&mut engine, 0, 11, 1, 1, ask.clone());
        assert_eq!(final_answer(&engine.poll(at(10_000_000))[0]), &answer);
        assert_eq!(engine.commits()[0].ask, Some(ask));
    }
}

#[test]
fn document_text_loss_is_preserved_and_priced_with_whole_document_parameter_capacity() {
    for cut in [false, true] {
        let value = DocumentText { bytes: text(&[0, 255, 1]), cut };
        let mut bounds = Bounds {
            text: 3,
            document_bytes: u64::try_from(core::mem::size_of::<DocumentText>() + 3).unwrap(),
            ..Bounds::default()
        };
        assert!(crate::payload::fits(&value, &bounds));
        assert_eq!(value.whole().is_some(), !cut);
        assert_eq!(value.bytes(), [0, 255, 1]);
        bounds.document_bytes -= 1;
        assert!(!crate::payload::fits(&value, &bounds));
        bounds.document_bytes = u64::MAX;
        bounds.text = 2;
        assert!(!crate::payload::fits(&value, &bounds));
    }
    let mut parameters = List::with_capacity(4);
    parameters.push(Parameter::Bytes { name: u32::MAX, value: text(&[0, 255, 1]) }).unwrap();
    let value = SpecificationDocument::complete(Specification {
        words: text(b"w"),
        parameters,
        inputs: List::with_capacity(2),
    });
    let whole = value.whole().unwrap();
    assert_eq!(whole.parameters.capacity(), 4);
    assert_eq!(whole.inputs.capacity(), 2);
    let size = u64::try_from(core::mem::size_of::<SpecificationDocument>() + 1 + 3).unwrap()
        + List::<ParameterDocument>::worst_case(4).unwrap()
        + List::<u64>::worst_case(2).unwrap();
    let mut bounds = Bounds { text: 3, rows: 4, document_bytes: size, ..Bounds::default() };
    assert!(crate::payload::fits(&value, &bounds));
    bounds.document_bytes -= 1;
    assert!(!crate::payload::fits(&value, &bounds));
    bounds.document_bytes = size;
    bounds.rows = 3;
    assert!(!crate::payload::fits(&value, &bounds));
}

#[test]
fn activation_observations_and_prior_closing_are_admitted_whole_with_their_allocated_bounds() {
    let mut setup = script();
    let mut row = task();
    let mut active = ActiveRun {
        stage: RunStage::Running { attempt: u64::MAX },
        phase_since: Wall::EPOCH,
        attempt: Some(u64::MAX),
        placement: Some(Wall::EPOCH),
        retry: Some(RetryNotice::Refused { count: u32::MAX }),
        observation: Some(RunObservation {
            channel: Token::new(u64::MAX),
            work: RunWork::Tool { call: text(b"call"), tool: text(b"tool") },
            since: Wall::EPOCH,
            reported_at: Wall::EPOCH,
            started: None,
        }),
    };
    *row.chip.details = ChipPhase::Active(active.clone());
    row.runtime = Runtime::Active(active.clone());
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row.clone())));
    assert!(Engine::new(setup.clone(), 1).is_ok());
    active.observation.as_mut().unwrap().work = RunWork::Tool {
        call: Text::new(vec![b'x'; setup.bounds.text as usize + 1].into_boxed_slice(), setup.bounds.text + 1).unwrap(),
        tool: text(b"tool"),
    };
    row.runtime = Runtime::Active(active);
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row.clone())));
    assert_eq!(Engine::new(setup.clone(), 1).err(), Some(Invalid::Payload));
    row.runtime = Runtime::Held {
        was: Some(HeldRuntime::Closing(Closing {
            delegates: 0,
            connectors: Some(List::with_capacity(setup.bounds.rows + 1)),
            stage: Some(ClosingStage::Run { attempt: u64::MAX }),
            ending: None,
        })),
    };
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Payload));
}

#[test]
fn full_width_task_and_party_makers_remain_distinct_in_scripted_reads_decisions_and_keyed_replays() {
    let mut setup = script();
    let keys = [
        ProposalKey { proposer: Proposer::Task(u64::MAX), number: u64::MAX },
        ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX },
    ];
    for (index, key) in keys.iter().enumerate() {
        let mut row = proposal();
        row.key = *key;
        row.proposer = key.proposer;
        row.why = text(if index == 0 { b"task maker" } else { b"party maker" });
        setup.truth.objects.insert(ObjectKey::Proposal(*key), ReadValue::Proposal(Box::new(row)));
    }
    let mut engine = Engine::new(setup, 197).unwrap();
    for (index, key) in keys.iter().enumerate() {
        engine.read(Time::ZERO, 11, Token::new(index as u64 + 1), Read::Proposal { proposal: *key }).unwrap();
        let rows = engine.poll(Time::ZERO);
        let Up::Read { terminal: ReadResult::Value(ReadValue::Proposal(row)), .. } = &rows[0] else {
            panic!("exact typed read")
        };
        assert_eq!(row.key, *key);
        assert_eq!(row.proposer, key.proposer);
        assert_eq!(row.why.bytes(), if index == 0 { b"task maker".as_slice() } else { b"party maker".as_slice() });
    }
    let asks = keys.map(|proposal| Ask::DecideProposal { project: 1, proposal, decision: Decision::Accepted });
    send(&mut engine, 0, 11, 1, 10, asks[0].clone());
    send(&mut engine, 1, 11, 2, 11, asks[1].clone());
    let answers = engine.poll(Time::from_nanos(10_000_001));
    assert_eq!(answers.len(), 2);
    assert_eq!(engine.commits().len(), 2);
    assert_eq!(engine.commits()[0].ask, Some(asks[0].clone()));
    assert_eq!(engine.commits()[1].ask, Some(asks[1].clone()));
    send(&mut engine, 20_000_000, 11, 1, 12, asks[1].clone());
    assert_eq!(final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]), &Answer::Refused(Refusal::KeyConflict));
    send(&mut engine, 20_000_000, 11, 1, 13, asks[0].clone());
    assert_eq!(final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]), final_answer(&answers[0]));
    send(&mut engine, 20_000_000, 11, 2, 14, asks[1].clone());
    assert_eq!(final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]), final_answer(&answers[1]));
    assert_eq!(engine.commits().len(), 2);
}

#[test]
fn note_corrections_price_the_whole_scope_and_allocated_reference_capacity() {
    let mut references = List::with_capacity(4);
    references.push(u64::MAX).unwrap();
    let ask = Ask::EditNote {
        project: 1,
        note: 77,
        scope: NoteScope::Resource { project: 1, connector: u16::MAX, pattern: text(b"a/b") },
        revision: Revision { value: u64::MAX },
        description: text(b"new"),
        body: text(b"body"),
        references,
    };
    let mut bounds = Bounds { text: 4, rows: 4, document_bytes: u64::MAX, ..Bounds::default() };
    bounds.document_bytes =
        u64::try_from(core::mem::size_of::<Ask>() + 3 + 3 + 4).unwrap() + List::<u64>::worst_case(4).unwrap();
    assert!(crate::payload::fits(&ask, &bounds));
    bounds.document_bytes -= 1;
    assert!(!crate::payload::fits(&ask, &bounds));
    bounds.document_bytes += 1;
    bounds.rows = 3;
    assert!(!crate::payload::fits(&ask, &bounds), "allocated reference capacity is bounded as a whole");
    bounds.rows = 4;
    bounds.text = 2;
    assert!(!crate::payload::fits(&ask, &bounds), "the supplied scope and body are refused rather than cut");
}

#[test]
fn optional_instruction_sources_price_whole_task_and_steering_at_byte_and_text_boundaries() {
    use crate::payload::Payload;
    let mut bounds = Bounds { text: 33, document_bytes: u64::MAX, ..Bounds::default() };
    for source in [
        None,
        Some(DocumentText::complete(text(b""))),
        Some(DocumentText { bytes: text(b""), cut: true }),
        Some(DocumentText::complete(text(&[b'x'; 33]))),
        Some(DocumentText { bytes: text(&[b'x'; 33]), cut: true }),
    ] {
        let mut row = task();
        row.instructions = source.clone();
        let mut steering = Steering {
            person_revision: Some(Revision { value: 12 }),
            taken_by: None,
            addressed: None,
            revision: Revision { value: 6 },
            spec: row.spec.clone(),
            instructions: None,
            wake: row.wake.clone(),
            dependencies: List::with_capacity(0),
            authority: row.authority.clone(),
            budget: budget(),
        };
        let mut absent = row.clone();
        absent.instructions = None;
        let bytes = source.as_ref().map_or(0, |value| value.bytes().len()) as u64;
        assert_eq!(row.heap(&bounds).unwrap(), absent.heap(&bounds).unwrap() + bytes);
        let steering_base = steering.heap(&bounds).unwrap();
        steering.instructions = source.clone();
        assert_eq!(steering.heap(&bounds).unwrap(), steering_base + bytes);
        bounds.document_bytes = row.heap(&bounds).unwrap() + u64::try_from(core::mem::size_of::<Task>()).unwrap();
        assert!(crate::payload::fits(&row, &bounds));
        bounds.document_bytes -= 1;
        assert!(!crate::payload::fits(&row, &bounds));
        bounds.document_bytes += 2;
        assert!(crate::payload::fits(&row, &bounds));
        bounds.document_bytes =
            steering.heap(&bounds).unwrap() + u64::try_from(core::mem::size_of::<Steering>()).unwrap();
        assert!(crate::payload::fits(&steering, &bounds));
        bounds.document_bytes -= 1;
        assert!(!crate::payload::fits(&steering, &bounds));
        bounds.document_bytes += 2;
        assert!(crate::payload::fits(&steering, &bounds));
        bounds.document_bytes = u64::MAX;
        bounds.text = 32;
        assert_eq!(crate::payload::fits(&row, &bounds), bytes <= 32);
        assert_eq!(crate::payload::fits(&steering, &bounds), bytes <= 32);
        bounds.text = 34;
        assert!(crate::payload::fits(&row, &bounds));
        assert!(crate::payload::fits(&steering, &bounds));
        bounds.text = 33;
        assert_eq!(row.instructions, source);
        assert_eq!(steering.instructions, source);
    }
}
