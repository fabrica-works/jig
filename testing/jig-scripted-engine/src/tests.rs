use super::*;
use jig_client::contract::*;
use skein_lib::List;
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
        versions: VersionRange { first: 1, last: 1 },
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
        | Up::ApplicationWatch { .. }
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
        key: ProposalKey { proposer: 8, number: 1 },
        project: 1,
        proposer: Actor::Task(8),
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
fn revision_refusals_scripts_retention_and_whole_payload_admission() {
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
            addressed: None,
            revision: Revision { value: 1 },
            spec: text(b"held work"),
            instructions: text(b""),
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
        &Answer::Refused(Refusal::Moved { now: Revision { value: 2 }, what: Moved::Hold })
    );
    let mut setup = script();
    let ask = Ask::StartChat { project: 1, words: text(b"hello") };
    setup.rules.push(Rule {
        party: None,
        ask: ask.clone(),
        answer: Answer::Refused(Refusal::Authority { lacking: Lacking::Budget { needed: 100, left: 5 } }),
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
        spec: text(b"spec"),
        spec_revision: Revision { value: 1 },
        instructions: text(b"instructions"),
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
        read: Read::Inbox { project: None, kind: None, goal: None, after: None },
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
        Read::Proposal { proposal: ProposalKey { proposer: 8, number: 1 } },
        Read::Escalation { task: 8 },
        Read::Question { task: 8, message: 1 },
        Read::PersonTask { task: 8 },
        Read::Result { task: 8 },
        Read::Ended { task: 8, after: None },
        Read::History { task: 8, subtree: true, after: None },
        Read::InboxGroup { group: InboxGroup::Replies, project: None, after: None },
        Read::Inbox { project: Some(1), kind: Some(InboxGroup::Questions), goal: None, after: None },
        Read::Conversations { project: Some(1), after: None },
        Read::Goals { project: 1, ended: false, after: None },
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
        read: Read::Inbox { project: None, kind: None, goal: None, after: None },
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
            row.steering.spec = Text::new(vec![0; 4097].into_boxed_slice(), 5000).unwrap();
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
fn making_script() -> (Script, Ask) {
    let mut setup = live_script();
    let ask = Ask::StartChat { project: 1, words: text(b"hello") };
    let mut made = task();
    made.chip.title = text(b"made");
    setup.rules.push(Rule {
        party: Some(1),
        ask: ask.clone(),
        answer: Answer::Done(Outcome::Made { what: Made::Chat { task: 8 } }),
        changes: vec![Change::Object { project: 1, key: ObjectKey::Task(8), value: ReadValue::Task(Box::new(made)) }],
    });
    (setup, ask)
}
fn scenario_restart_cuts_offline_and_retained_keys() {
    let (setup, ask) = making_script();
    let mut engine = Engine::new(setup, 10).expect("making script");
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    engine
        .write(Time::ZERO, 8, TurnName { attempt: 1, number: 0 }, Wall::EPOCH, &text(b"provisional"), None)
        .expect("run");
    engine.open(Time::ZERO, 11, Token::new(3), Subject::Task { task: 8 }).expect("watch");
    engine.read(Time::ZERO, 11, Token::new(4), Read::Task { task: 8 }).expect("read");
    engine.fault(at(1), Wall::EPOCH, Fault::Restart { offline: Duration::from_millis(10) });
    assert!(engine.commits().is_empty());
    assert!(engine.runs.is_empty());
    assert!(engine.keys.is_empty());
    assert!(
        matches!(engine.truth().objects.get(&ObjectKey::Task(8)),Some(ReadValue::Task(doc)) if doc.chip.title.bytes()==b"chat")
    );
    let up = engine.poll(at(1));
    assert!(up.contains(&Up::Answer { attempt: Token::new(1), envelope: Envelope::Lost, answer: None }));
    assert!(up.contains(&Up::Read { token: Token::new(4), terminal: ReadResult::None(ReadNone::Unreachable) }));
    assert!(up.contains(&Up::Ended { token: Token::new(3), why: WatchEnd::Dropped }));
    send(&mut engine, 2, 11, 1, 2, ask.clone());
    assert_eq!(
        engine.poll(at(2)),
        vec![Up::Answer { attempt: Token::new(2), envelope: Envelope::Again, answer: None }]
    );
    assert_eq!(engine.next_due(), Some(at(10_000_001)));
    send(&mut engine, 10_000_001, 11, 1, 5, ask.clone());
    let answered = engine.poll(at(20_000_001));
    assert_eq!(engine.commits().len(), 1, "undurable request was remade once");
    assert!(matches!(final_answer(&answered[0]), Answer::Done(Outcome::Made { .. })));
    engine.fault(at(20_000_002), Wall::EPOCH, Fault::Restart { offline: Duration::from_millis(1) });
    send(&mut engine, 21_000_002, 12, 1, 6, ask);
    assert_eq!(final_answer(&engine.poll(at(21_000_002))[0]), final_answer(&answered[0]));
    assert_eq!(engine.commits().len(), 1, "durable key survives the restart across sign-ins");
}
fn scenario_busy_not_ready_loss_sign_out_versions_and_watch_faults() {
    for fault in [Fault::Busy, Fault::NotReady] {
        let (setup, ask) = making_script();
        let mut engine = Engine::new(setup, 11).expect("fault script");
        engine.fault(Time::ZERO, Wall::EPOCH, fault);
        send(&mut engine, 0, 11, 1, 1, ask.clone());
        assert_eq!(
            engine.poll(Time::ZERO),
            vec![Up::Answer { attempt: Token::new(1), envelope: Envelope::Again, answer: None }]
        );
        assert!(engine.keys.is_empty());
        assert!(engine.pending.is_empty());
        send(&mut engine, 1, 11, 1, 2, ask);
        drop(engine.poll(at(10_000_001)));
        assert_eq!(engine.commits().len(), 1);
    }
    let (setup, ask) = making_script();
    let mut engine = Engine::new(setup, 12).expect("loss");
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    engine.fault(at(1), Wall::EPOCH, Fault::Lost);
    assert!(engine.poll(at(9_999_999)).is_empty());
    assert_eq!(
        engine.poll(at(10_000_000)),
        vec![Up::Answer { attempt: Token::new(1), envelope: Envelope::Lost, answer: None }]
    );
    send(&mut engine, 10_000_001, 12, 1, 2, ask.clone());
    let recovered = engine.poll(at(10_000_001));
    assert!(matches!(final_answer(&recovered[0]), Answer::Done(Outcome::Made { .. })));
    assert_eq!(engine.commits().len(), 1);
    let (setup, ask) = making_script();
    let mut engine = Engine::new(setup, 13).expect("sign-out");
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    engine.read(Time::ZERO, 11, Token::new(2), Read::Frame).expect("read");
    engine.open(Time::ZERO, 11, Token::new(3), Subject::Frame).expect("watch");
    engine.fault(at(1), Wall::EPOCH, Fault::SignedOut { sign_in: 11 });
    let up = engine.poll(at(1));
    assert!(up.contains(&Up::Answer { attempt: Token::new(1), envelope: Envelope::SignedOut, answer: None }));
    assert!(up.contains(&Up::Read { token: Token::new(2), terminal: ReadResult::None(ReadNone::SignedOut) }));
    assert!(up.contains(&Up::Ended { token: Token::new(3), why: WatchEnd::SignedOut }));
    drop(engine.poll(at(10_000_000)));
    send(&mut engine, 10_000_001, 12, 1, 4, ask);
    assert!(matches!(final_answer(&engine.poll(at(10_000_001))[0]), Answer::Done(Outcome::Made { .. })));
    assert_eq!(engine.commits().len(), 1);
    let mut engine = Engine::new(live_script(), 14).expect("versions");
    engine.open(Time::ZERO, 11, Token::new(1), Subject::Frame).expect("frame");
    drop(engine.poll(Time::ZERO));
    engine.taken(Token::new(1));
    engine.fault(Time::ZERO, Wall::EPOCH, Fault::Versions);
    assert!(engine.poll(at(9_999_999)).is_empty());
    let up = engine.poll(at(10_000_000));
    assert!(
        matches!(&up[0],Up::Watch { event:WatchEvent::Delivered(Delivery::Frame(frame)),.. } if frame.versions.first==2 && frame.versions.last==2)
    );
    engine.fault(at(10_000_000), Wall::EPOCH, Fault::Drop { watch: Token::new(1) });
    assert_eq!(engine.poll(at(10_000_000)), vec![Up::Ended { token: Token::new(1), why: WatchEnd::Dropped }]);
    engine.open(at(10_000_000), 11, Token::new(2), Subject::Frame).expect("new frame");
    drop(engine.poll(at(10_000_000)));
    engine.stop(at(10_000_000));
    assert_eq!(engine.poll(at(10_000_000)), vec![Up::Ended { token: Token::new(2), why: WatchEnd::Stopping }]);
}
fn scenario_slow_current_documents_preserve_in_flight_and_barriers() {
    let mut engine = Engine::new(live_script(), 15).expect("slow");
    engine.fault(Time::ZERO, Wall::EPOCH, Fault::Slow { by: Duration::from_millis(50) });
    engine.open(Time::ZERO, 11, Token::new(1), Subject::Frame).expect("frame");
    let mut latest = frame(1);
    latest.needs.questions = 7;
    engine.act(at(10_000_000), Wall::EPOCH, Change::Frame(latest.clone())).expect("latest frame");
    assert!(engine.poll(at(49_999_999)).is_empty());
    let up = engine.poll(at(60_000_000));
    assert!(up.contains(&Up::Watch {
        token: Token::new(1),
        event: WatchEvent::Delivered(Delivery::Frame(Box::new(latest)))
    }));
    engine.fault(at(60_000_000), Wall::EPOCH, Fault::Slow { by: Duration::from_millis(1) });
    let mut latest = frame(1);
    latest.needs.questions = 8;
    engine.act(at(60_000_000), Wall::EPOCH, Change::Frame(latest.clone())).expect("changed while in flight");
    assert!(engine.poll(at(70_000_000)).is_empty(), "slow does not acknowledge a delivery the client still holds");
    engine.taken(Token::new(1));
    assert_eq!(
        engine.poll(at(70_000_000)),
        vec![Up::Watch { token: Token::new(1), event: WatchEvent::Delivered(Delivery::Frame(Box::new(latest))) }]
    );
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnDocument {
    revision: u64,
    words: Text,
    rows: List<Text>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnAsk {
    words: Text,
    transport: Text,
}
impl payload::Payload for OwnDocument {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.words.heap(bounds)?.checked_add(self.rows.heap(bounds)?)
    }
}
impl payload::Payload for OwnAsk {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.words.heap(bounds)?.checked_add(self.transport.heap(bounds)?)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnApplication {
    document: OwnDocument,
}
impl Application for OwnApplication {
    type Ask = OwnAsk;
    type Answer = OwnDocument;
    type Read = u32;
    type Subject = u32;
    type Delivery = OwnDocument;
    type Change = OwnDocument;
    fn fits(&self, value: ApplicationValue<'_, Self>, bounds: &Bounds) -> bool {
        match value {
            ApplicationValue::Ask(value) => payload::fits(value, bounds),
            ApplicationValue::Answer(value) | ApplicationValue::Delivery(value) | ApplicationValue::Change(value) => {
                payload::fits(value, bounds)
            }
            ApplicationValue::Read(value) | ApplicationValue::Subject(value) => payload::fits(value, bounds),
        }
    }
    fn decide(
        &self,
        context: Context<'_>,
        ask: &OwnAsk,
    ) -> core::result::Result<ApplicationDecision<OwnDocument, OwnDocument>, Refusal> {
        if ask.words.bytes().is_empty() {
            return Err(Refusal::NotOffered);
        }
        let document =
            OwnDocument { revision: self.document.revision + 1, words: ask.words.clone(), rows: list(vec![]) };
        Ok(ApplicationDecision {
            answer: document.clone(),
            changes: vec![document.clone()],
            jig: vec![Change::Seen { party: context.party, results: document.revision, replies: vec![] }],
        })
    }
    fn read(&self, _: Context<'_>, read: &u32) -> core::result::Result<OwnDocument, ReadNone> {
        if *read == 0 { Ok(self.document.clone()) } else { Err(ReadNone::Unknown) }
    }
    fn watch(&self, _: Context<'_>, subject: &u32) -> core::result::Result<Option<OwnDocument>, WatchEnd> {
        if *subject == 0 { Ok(Some(self.document.clone())) } else { Err(WatchEnd::SubjectEnded) }
    }
    fn apply(&mut self, change: &OwnDocument) -> core::result::Result<(), Invalid> {
        self.document = change.clone();
        Ok(())
    }
}
fn own() -> OwnApplication {
    OwnApplication { document: OwnDocument { revision: 0, words: text(b"initial"), rows: list(vec![]) } }
}
fn own_ask(transport: &[u8]) -> OwnAsk {
    OwnAsk { words: text(b"changed"), transport: text(transport) }
}
#[expect(clippy::too_many_lines, reason = "one scenario follows a shared key through both vocabularies and a restart")]
fn scenario_application_hooks_share_keys_commits_retention_and_restart() {
    let mut engine = Engine::with_application(live_script(), 16, own()).expect("hook kit");
    engine.open_application(Time::ZERO, 11, Token::new(1), 7, 1, 0).expect("own watch");
    drop(engine.poll(Time::ZERO));
    engine
        .send_application(Time::ZERO, Wall::EPOCH, 11, [1; 16], Token::new(2), 7, 1, [9; 32], own_ask(b"one"))
        .expect("own request");
    engine
        .send_application(
            at(1),
            Wall::EPOCH,
            12,
            [1; 16],
            Token::new(3),
            7,
            1,
            [9; 32],
            own_ask(b"different transport"),
        )
        .expect("same digest");
    engine.read_application(at(1), 11, Token::new(4), 7, 1, &0).expect("own read");
    assert!(engine.poll(at(9_999_999)).is_empty());
    engine.taken(Token::new(1));
    let up = engine.poll(at(10_000_000));
    assert_eq!(engine.commits().len(), 1);
    assert_eq!(engine.application().document.revision, 1);
    assert_eq!(engine.truth().results_seen.get(&1), Some(&1));
    let Up::ApplicationAnswer { envelope: Envelope::Final, answer: Some(answer), .. } = &up[0] else {
        panic!("own answer");
    };
    assert_eq!(
        &up[1],
        &Up::ApplicationAnswer { attempt: Token::new(3), envelope: Envelope::Final, answer: Some(answer.clone()) }
    );
    assert!(
        up.contains(&Up::ApplicationRead { token: Token::new(4), terminal: Ok(engine.application().document.clone()) })
    );
    assert!(up.contains(&Up::ApplicationWatch {
        token: Token::new(1),
        event: ApplicationEvent::Delivered(engine.application().document.clone())
    }));
    assert_eq!(engine.commits()[0].application.as_ref().expect("own ledger").digest, Some([9; 32]));
    engine
        .send(
            at(10_000_001),
            Wall::EPOCH,
            11,
            [1; 16],
            Token::new(5),
            Ask::StartChat { project: 1, words: text(b"collision") },
        )
        .expect("jig collision");
    assert!(engine.poll(at(10_000_001)).contains(&Up::Answer {
        attempt: Token::new(5),
        envelope: Envelope::Final,
        answer: Some(Answer::Refused(Refusal::KeyConflict))
    }));
    engine
        .send_application(
            at(10_000_001),
            Wall::EPOCH,
            11,
            [1; 16],
            Token::new(6),
            8,
            1,
            [9; 32],
            own_ask(b"owner conflict"),
        )
        .expect("owner collision");
    assert_eq!(
        engine.poll(at(10_000_001)),
        vec![Up::ApplicationAnswer {
            attempt: Token::new(6),
            envelope: Envelope::Final,
            answer: Some(Err(Refusal::KeyConflict))
        }]
    );
    engine
        .send_application(at(10_000_001), Wall::EPOCH, 11, [2; 16], Token::new(7), 7, 1, [8; 32], own_ask(b"pending"))
        .expect("pending own request");
    assert_eq!(engine.application().document.revision, 2);
    engine.fault(at(10_000_002), Wall::EPOCH, Fault::Restart { offline: Duration::from_millis(1) });
    assert_eq!(engine.application().document.revision, 1);
    assert_eq!(engine.truth().results_seen.get(&1), Some(&1));
    let up = engine.poll(at(10_000_002));
    assert!(up.contains(&Up::ApplicationAnswer { attempt: Token::new(7), envelope: Envelope::Lost, answer: None }));
    assert!(up.contains(&Up::Ended { token: Token::new(1), why: WatchEnd::Dropped }));
    engine
        .send_application(at(11_000_002), Wall::EPOCH, 12, [1; 16], Token::new(8), 7, 1, [9; 32], own_ask(b"retained"))
        .expect("retained own request");
    assert_eq!(
        engine.poll(at(11_000_002)),
        vec![Up::ApplicationAnswer { attempt: Token::new(8), envelope: Envelope::Final, answer: Some(answer.clone()) }]
    );
    engine.fault(at(11_000_002), Wall::EPOCH, Fault::Lost);
    engine
        .send_application(at(11_000_002), Wall::EPOCH, 11, [2; 16], Token::new(9), 7, 1, [8; 32], own_ask(b"retry"))
        .expect("remade own request");
    assert_eq!(
        engine.poll(at(21_000_002)),
        vec![Up::ApplicationAnswer { attempt: Token::new(9), envelope: Envelope::Lost, answer: None }]
    );
    engine
        .send_application(
            at(21_000_003),
            Wall::EPOCH,
            11,
            [2; 16],
            Token::new(10),
            7,
            1,
            [8; 32],
            own_ask(b"after loss"),
        )
        .expect("recover own answer");
    assert!(
        matches!(&engine.poll(at(21_000_003))[0],Up::ApplicationAnswer { answer:Some(Ok(doc)),.. } if doc.revision==2)
    );
    assert_eq!(engine.commits().len(), 2);
    let invalid = OwnDocument { revision: 3, words: text(b"bad"), rows: List::with_capacity(33) };
    assert_eq!(engine.act_application(at(21_000_003), Wall::EPOCH, 7, 1, vec![invalid], vec![]), Err(Invalid::Payload));
    assert_eq!(engine.application().document.revision, 2);
    let next = OwnDocument { revision: 3, words: text(b"story"), rows: list(vec![]) };
    engine.act_application(at(21_000_003), Wall::EPOCH, 7, 1, vec![next.clone()], vec![]).expect("own story");
    engine.read_application(at(21_000_003), 11, Token::new(11), 7, 1, &0).expect("story read");
    assert!(engine.poll(at(31_000_002)).is_empty());
    assert_eq!(engine.poll(at(31_000_003)), vec![Up::ApplicationRead { token: Token::new(11), terminal: Ok(next) }]);
}
fn scenario_hook_limits_visibility_conflicts_and_expired_retention() {
    let mut setup = live_script();
    setup.bounds.watches = 1;
    let mut engine = Engine::with_application(setup, 17, own()).expect("shared limits");
    engine.open_application(Time::ZERO, 11, Token::new(1), 7, 1, 0).expect("own watch");
    engine.open(Time::ZERO, 11, Token::new(2), Subject::Frame).expect("jig over limit");
    assert!(engine.poll(Time::ZERO).contains(&Up::Ended { token: Token::new(2), why: WatchEnd::Busy }));
    engine.close(Time::ZERO, Token::new(1));
    assert_eq!(engine.poll(Time::ZERO), vec![Up::Ended { token: Token::new(1), why: WatchEnd::Closed }]);
    engine.open(Time::ZERO, 11, Token::new(3), Subject::Frame).expect("jig watch");
    engine.open_application(Time::ZERO, 11, Token::new(4), 7, 1, 0).expect("own over limit");
    assert!(engine.poll(Time::ZERO).contains(&Up::Ended { token: Token::new(4), why: WatchEnd::Busy }));
    engine.read_application(Time::ZERO, 11, Token::new(5), 7, 2, &0).expect("hidden project");
    assert_eq!(
        engine.poll(Time::ZERO),
        vec![Up::ApplicationRead { token: Token::new(5), terminal: Err(ReadNone::Unknown) }]
    );
    engine
        .send(
            Time::ZERO,
            Wall::EPOCH,
            11,
            [1; 16],
            Token::new(6),
            Ask::StartChat { project: 1, words: text(b"jig first") },
        )
        .expect("jig key");
    engine
        .send_application(Time::ZERO, Wall::EPOCH, 12, [1; 16], Token::new(7), 7, 1, [9; 32], own_ask(b"own conflict"))
        .expect("jig owns key");
    let up = engine.poll(at(10_000_000));
    assert!(up.contains(&Up::ApplicationAnswer {
        attempt: Token::new(7),
        envelope: Envelope::Final,
        answer: Some(Err(Refusal::KeyConflict))
    }));
    assert_eq!(engine.application().document.revision, 0);
    engine
        .send_application(
            at(1_010_000_001),
            Wall::EPOCH,
            11,
            [1; 16],
            Token::new(8),
            7,
            1,
            [9; 32],
            own_ask(b"expired jig key"),
        )
        .expect("expired key becomes own");
    let up = engine.poll(at(1_020_000_001));
    assert!(up.iter().any(|value| matches!(value,Up::ApplicationAnswer { attempt,answer:Some(Ok(doc)),.. } if *attempt==Token::new(8) && doc.revision==1)));
    engine
        .send_application(
            at(2_020_000_002),
            Wall::EPOCH,
            11,
            [1; 16],
            Token::new(9),
            7,
            1,
            [9; 32],
            own_ask(b"expired own key"),
        )
        .expect("own retention expired");
    let up = engine.poll(at(2_030_000_002));
    assert!(up.iter().any(|value| matches!(value,Up::ApplicationAnswer { attempt,answer:Some(Ok(doc)),.. } if *attempt==Token::new(9) && doc.revision==2)));
    assert_eq!(engine.commits().len(), 3);
}
fn scenario_deployment_parts_dynamic_rules_and_scheduling() {
    let mut setup = live_script();
    setup.truth.frames.get_mut(&1).expect("frame").projects = list(vec![]);
    let mut engine = Engine::with_application(setup, 18, own()).expect("deployment role");
    engine
        .read_scoped_application(Time::ZERO, 11, Token::new(1), 7, ApplicationScope::Deployment, &0)
        .expect("deployment part");
    assert_eq!(
        engine.poll(Time::ZERO),
        vec![Up::ApplicationRead { token: Token::new(1), terminal: Ok(own().document) }]
    );
    engine
        .open_scoped_application(Time::ZERO, 11, Token::new(2), 7, ApplicationScope::Deployment, 0)
        .expect("deployment watch");
    drop(engine.poll(Time::ZERO));
    let mut removed = frame(1);
    removed.deployment_role = None;
    removed.projects = list(vec![]);
    engine.act(Time::ZERO, Wall::EPOCH, Change::Frame(removed)).expect("role removed");
    assert_eq!(engine.poll(at(10_000_000)), vec![Up::Ended { token: Token::new(2), why: WatchEnd::NotAllowed }]);
    let mut engine = Engine::new(live_script(), 19).expect("dynamic rules");
    let ask = Ask::StartChat { project: 1, words: text(b"world generated words") };
    let mut made = task();
    made.chip.task = 9;
    let rule = Rule {
        party: Some(1),
        ask: ask.clone(),
        answer: Answer::Done(Outcome::Made { what: Made::Chat { task: 9 } }),
        changes: vec![Change::Object { project: 1, key: ObjectKey::Task(9), value: ReadValue::Task(Box::new(made)) }],
    };
    engine.script(rule.clone()).expect("world scripts actual words");
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    assert_eq!(engine.pending_commits().next().expect("staged ledger").durable, None);
    assert!(engine.commits().is_empty());
    let first = engine.poll(at(10_000_000));
    let mut invalid = rule.clone();
    invalid.answer = Answer::Refused(Refusal::NotOffered);
    assert_eq!(engine.script(invalid), Err(Invalid::Script));
    let mut replacement = rule;
    replacement.answer = Answer::Refused(Refusal::NotOffered);
    replacement.changes = vec![];
    engine.script(replacement).expect("replace future outcome");
    send(&mut engine, 10_000_001, 11, 1, 2, ask);
    assert_eq!(
        final_answer(&engine.poll(at(10_000_001))[0]),
        final_answer(&first[0]),
        "retained key ignores later script changes"
    );
    engine.fault(at(10_000_001), Wall::EPOCH, Fault::Restart { offline: Duration::from_millis(1) });
    drop(engine.poll(at(10_000_001)));
    send(&mut engine, 12_000_000, 11, 2, 3, Ask::StartChat { project: 1, words: text(b"fresh") });
    assert!(
        engine.next_due().is_some_and(|due| due >= at(12_000_000)),
        "late activity after offline cannot move the world's clock back"
    );
    let mut engine = Engine::new(live_script(), 20).expect("run heartbeat");
    engine.open(Time::ZERO, 11, Token::new(1), Subject::Run { task: 8, turn: None, offset: 0 }).expect("run");
    drop(engine.poll(Time::ZERO));
    let turn = TurnName { attempt: 1, number: 0 };
    engine.write(Time::ZERO, 8, turn, Wall::EPOCH, &text(b"first"), None).expect("write");
    drop(engine.poll(Time::ZERO));
    for nanos in [500_000_000, 1_000_000_000] {
        engine.write(at(nanos), 8, turn, Wall::EPOCH, &text(b"more"), None).expect("still writing");
    }
    assert_eq!(
        engine.poll(at(1_000_000_000)),
        vec![Up::Watch { token: Token::new(1), event: WatchEvent::Alive }],
        "writing does not postpone the slow client's heartbeat"
    );
    let mut engine = Engine::new(live_script(), 21).expect("slow heartbeat");
    engine.fault(Time::ZERO, Wall::EPOCH, Fault::Slow { by: Duration::from_secs(5) });
    engine.open(Time::ZERO, 11, Token::new(1), Subject::Frame).expect("slow frame");
    for seconds in 1..5 {
        assert!(engine.poll(at(seconds * 1_000_000_000)).is_empty());
    }
    assert_eq!(
        engine.due.iter().filter(|due| matches!(due.up, Up::Watch { event: WatchEvent::Alive, .. })).count(),
        1,
        "slow transport coalesces heartbeat output too"
    );
}
#[test]
fn fault_and_application_scenarios() {
    scenario_deployment_parts_dynamic_rules_and_scheduling();
    scenario_hook_limits_visibility_conflicts_and_expired_retention();
    scenario_restart_cuts_offline_and_retained_keys();
    scenario_busy_not_ready_loss_sign_out_versions_and_watch_faults();
    scenario_slow_current_documents_preserve_in_flight_and_barriers();
    scenario_application_hooks_share_keys_commits_retention_and_restart();
}
