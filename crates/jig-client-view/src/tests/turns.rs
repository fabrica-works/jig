#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary fixtures use public events and inspect semantic nodes"
)]
use super::{
    cards::{contains, drive, text, words},
    pages::{build, deliver, named_binding},
    work_pages::{entered, list, part},
};
use crate::{Key, Role, Tree, words::Words};
use jig_client::{
    Domain, Event, Request,
    action::Act,
    address::{Address, TaskSection},
    contract::*,
    fields::Purpose,
};
use skein_lib::{Token, Wall};
fn read(out: &[Request]) -> Token {
    out.iter()
        .find_map(|request| match request {
            Request::Read { read, what: Read::Activity { .. } } => Some(*read),
            _ => None,
        })
        .unwrap()
}
fn ready() -> (Domain, Token, Token, Token) {
    ready_task(8)
}
fn ready_task(task: u64) -> (Domain, Token, Token, Token) {
    let (mut d, out) = entered(Address::Task { task, section: TaskSection::Activity });
    let watch = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Task { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let mut doc = super::cards_objects::task_doc(&[ActKind::Write]);
    doc.chip.task = task;
    doc.chip.executor = Executor::Agent { charter: 2 };
    doc.chip.phase = Phase::Running;
    *doc.chip.details = ChipPhase::Running { attempt: 2, since: Wall::EPOCH, tool: None };
    doc.runtime = Runtime::Running { attempt: 2, since: Wall::EPOCH, tool: None };
    doc.hold = None;
    doc.activity = 2;
    let out = deliver(&mut d, watch, Delivery::Task(Box::new(doc)));
    let run = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Run { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    (d, watch, run, read(&out))
}
fn call() -> Call {
    Call {
        number: 1,
        tool: text(b"ReadFact"),
        input: text(b"whole private input"),
        outcome: Some(text(b"whole tool result")),
        failed: false,
        objects: list([ObjectKey::Task(8)]),
        parts: list([part()]),
    }
}
fn turn() -> Turn {
    Turn {
        messages: list([
            Message {
                speaker: Speaker::Assistant,
                blocks: list([Block::Text(text(b"# committed\n**reply**")), Block::Call(call())]),
            },
            Message {
                speaker: Speaker::Tool,
                blocks: list([
                    Block::Opaque { dialect: text(b"provider-fragment"), size: 91 },
                    Block::Result { call: 1, summary: text(b"result summary"), failed: true },
                ]),
            },
        ]),
        used: Usage { input: 2, output: 3, cached: 4, cache_written: 5 },
        spent: 7,
        read_through: Some(11),
        cut: true,
    }
}
fn live(d: &mut Domain, run: Token, number: u32) {
    let value = text(if number == 1 { b"# provisional first" } else { b"later provisional" });
    let length = u32::try_from(value.bytes().len()).unwrap();
    deliver(
        d,
        run,
        Delivery::Run(Box::new(RunText {
            attempt: 2,
            began: Some(Wall::EPOCH),
            turn: TurnName { attempt: 2, number },
            from: 0,
            through: length,
            text: value,
            cut: true,
            call: if number == 1 { Some(LiveCall::Decoded(call())) } else { None },
        })),
    );
}
fn keys(tree: &Tree) -> Vec<Key> {
    tree.nodes().iter().filter_map(|node| node.key).filter(|key| key.kind == 24).collect()
}
fn access(tree: &Tree) {
    assert!(
        tree.nodes().iter().filter(|n| n.binding.is_some()).all(|n| n.name.as_ref().is_some_and(|n| !n.is_empty()))
    );
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::Heading { level: 1 }).count(), 1);
    assert!(tree.nodes().iter().any(|n| n.role == Role::Log && n.name.as_ref().is_some_and(|v| !v.is_empty())));
}
fn replacement(words: &Words) {
    let (mut d, task, run, read) = ready();
    live(&mut d, run, 1);
    let (first, mut bindings) = build(&d, 3, words);
    access(&first);
    assert!(contains(&first, b"Provisional"));
    assert!(!contains(&first, b"whole private input"));
    let key = keys(&first)[0];
    let act =
        crate::decode(&mut bindings, named_binding(&first, b"Open input and result for ReadFact"), crate::Act::Press)
            .unwrap();
    drive(&mut d, Event::Act { act });
    assert!(contains(&build(&d, 3, words).0, b"whole private input"));
    live(&mut d, run, 2);
    let later = build(&d, 3, words).0;
    assert_eq!(keys(&later).len(), 2);
    assert_eq!(keys(&later)[0], key);
    drive(
        &mut d,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                rows: list([
                    ActivityLine {
                        position: 2,
                        at: Wall::EPOCH,
                        event: Activity::Turn { name: TurnName { attempt: 2, number: 1 }, turn: turn() },
                    },
                    ActivityLine {
                        position: 1,
                        at: Wall::EPOCH,
                        event: Activity::Words {
                            by: Party { number: 1, kind: PartyKind::Person, name: text(b"Pat") },
                            message: 11,
                            words: text(b"whole words ## kept"),
                        },
                    },
                ]),
                before: Some(Cursor { value: text(b"older") }),
                after: None,
            }))),
        },
    );
    let (committed, mut bindings) = build(&d, 3, words);
    access(&committed);
    assert_eq!(keys(&committed), keys(&later));
    for label in [
        b"committed".as_slice(),
        b"reply",
        b"later provisional",
        b"whole words ## kept",
        b"Read",
        b"whole private input",
        b"whole tool result",
        b"provider-fragment",
        b"Opaque bytes: 91",
        b"Some turn content did not fit",
        b"Cache tokens read: 4",
        b"Cache tokens written: 5",
        b"Turn spent: 7",
    ] {
        assert!(contains(&committed, label), "{label:?}");
    }
    let words_at = committed
        .nodes()
        .iter()
        .position(|n| n.text.iter().any(|run| run.text.as_ref() == b"whole words ## kept"))
        .unwrap();
    let turn_at = committed.nodes().iter().position(|n| n.key == Some(key)).unwrap();
    assert!(words_at < turn_at);
    assert_eq!(committed.nodes().iter().filter(|n| n.role == Role::Slot).count(), 1);
    let closed = crate::decode(
        &mut bindings,
        named_binding(&committed, b"Fold input and result for ReadFact"),
        crate::Act::Press,
    )
    .unwrap();
    drive(&mut d, Event::Act { act: closed });
    assert!(!contains(&build(&d, 3, words).0, b"whole private input"));
    let mut doc = super::cards_objects::task_doc(&[ActKind::Write]);
    doc.chip.executor = Executor::Agent { charter: 2 };
    doc.chip.title = text(b"Updated live task");
    doc.activity = 2;
    deliver(&mut d, task, Delivery::Task(Box::new(doc)));
    assert!(contains(&build(&d, 3, words).0, b"Updated live task"));
}
fn words_states(words: &Words) {
    let (mut d, _, _, read) = ready();
    drive(
        &mut d,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                rows: list([]),
                before: None,
                after: None,
            }))),
        },
    );
    drive(
        &mut d,
        Event::Act { act: Act::Edit { field: Purpose::Composer { task: 8 }, text: text(b"whole new words") } },
    );
    drive(&mut d, Event::Act { act: Act::Submit { field: Purpose::Composer { task: 8 } } });
    let out = drive(&mut d, Event::Act { act: Act::Confirm });
    let request = out
        .iter()
        .find_map(|r| match r {
            Request::Send { request, .. } => Some(*request),
            _ => None,
        })
        .unwrap();
    assert!(contains(&build(&d, 3, words).0, b"Being sent"));
    drive(
        &mut d,
        Event::Answered { request, answer: Answer::Done(Outcome::Made { what: Made::Words { task: 8, message: 12 } }) },
    );
    assert!(contains(&build(&d, 3, words).0, b"Sent"));
    assert!(contains(&build(&d, 3, words).0, b"whole new words"));
}
fn activities(words: &Words) {
    let (mut d, _, _, mut token) = ready();
    let result = Result {
        task: super::work_pages::chip(9),
        ending: Ending::Done,
        value: Some(ResultValue::Report(DocumentText::complete(text(b"whole heard result")))),
        at: Some(Wall::EPOCH),
        acts: list([]),

        details: None,
    };
    let values = [
        Activity::RunBegan { attempt: 1, how: RunBeginning::Resumed, why: text(b"resume reason") },
        Activity::RunBegan { attempt: 2, how: RunBeginning::Fresh { tail: 4 }, why: text(b"fresh reason") },
        Activity::RunEnded { attempt: 1, how: RunEnding::Parked, spent: 1 },
        Activity::RunEnded { attempt: 2, how: RunEnding::Finished, spent: 2 },
        Activity::RunEnded { attempt: 3, how: RunEnding::Cancelled, spent: 3 },
        Activity::RunEnded {
            attempt: 4,
            how: RunEnding::Failed { class: FailureClass::HostLost, reason: text(b"lost reason") },
            spent: 4,
        },
        Activity::Result { result },
        Activity::News { topic: 9, summary: text(b"whole news"), part: Some(part()) },
        Activity::Decision { object: ObjectKey::Task(8), by: Actor::Party(1), how: Decision::Accepted },
        Activity::Words {
            by: Party { number: 2, kind: PartyKind::Person, name: text(b"Other") },
            message: 10,
            words: text(b"durable sent words"),
        },
    ];
    let expected: [&[u8]; 5] =
        [b"Resumed its conversation", b"Run finished", b"Run cancelled", b"whole heard result", b"durable sent words"];
    for (index, rows) in values.chunks(2).enumerate() {
        let out = drive(
            &mut d,
            Event::Read {
                read: token,
                result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                    rows: list(rows.iter().enumerate().map(|(offset, event)| {
                        ActivityLine {
                            position: u64::try_from(
                                10_usize
                                    .checked_sub(index.checked_mul(2).unwrap().checked_add(offset).unwrap())
                                    .unwrap(),
                            )
                            .unwrap(),
                            at: Wall::EPOCH,
                            event: event.clone(),
                        }
                    })),
                    before: Some(Cursor { value: text(b"older") }),
                    after: Some(Cursor { value: text(b"newer") }),
                }))),
            },
        );
        for request in out {
            if let Request::Read { read, what: Read::Result { .. } } = request {
                let value = Result {
                    task: super::work_pages::chip(9),
                    ending: Ending::Done,
                    value: Some(ResultValue::Report(DocumentText::complete(text(b"whole heard result")))),
                    at: Some(Wall::EPOCH),
                    acts: list([]),

                    details: None,
                };
                drive(&mut d, Event::Read { read, result: ReadResult::Value(ReadValue::Result(Box::new(value))) });
            }
        }
        let (tree, mut bindings) = build(&d, 3, words);
        access(&tree);
        assert!(contains(&tree, expected[index]), "{:?}", expected[index]);
        let more = crate::decode(&mut bindings, named_binding(&tree, b"Read earlier"), crate::Act::Press).unwrap();
        token = read(&drive(&mut d, Event::Act { act: more }));
    }
}
#[test]
fn conversation_keeps_word_states_activity_call_folds_and_provisional_turn_identity() {
    let words = words();
    replacement(&words);
    words_states(&words);
    activities(&words);
}

#[test]
fn full_width_attempt_turn_keys_survive_commit_under_the_actual_task_log_scope() {
    let words = words();
    let (mut d, _, run, read) = ready_task(u64::MAX);
    for attempt in [u64::MAX - 1, u64::MAX] {
        deliver(
            &mut d,
            run,
            Delivery::Run(Box::new(RunText {
                attempt,
                began: Some(Wall::EPOCH),
                turn: TurnName { attempt, number: u32::MAX },
                from: 0,
                through: 4,
                text: text(b"live"),
                cut: false,
                call: None,
            })),
        );
    }
    let before = build(&d, 3, &words).0;
    let prior = keys(&before);
    assert_eq!(prior.len(), 2);
    assert_ne!(prior[0], prior[1]);
    let scope =
        before.nodes().iter().position(|n| n.key.is_some_and(|key| key.kind == 28 && key.a == u64::MAX)).unwrap();
    assert_eq!(before.nodes()[scope].role, Role::Item);
    assert_eq!(before.nodes()[scope + 1].role, Role::Log);
    assert!(before.nodes()[scope + 1].key.is_none());
    drive(
        &mut d,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                rows: list([
                    ActivityLine {
                        position: 2,
                        at: Wall::EPOCH,
                        event: Activity::Turn { name: TurnName { attempt: u64::MAX, number: u32::MAX }, turn: turn() },
                    },
                    ActivityLine {
                        position: 1,
                        at: Wall::EPOCH,
                        event: Activity::Turn {
                            name: TurnName { attempt: u64::MAX - 1, number: u32::MAX },
                            turn: turn(),
                        },
                    },
                ]),
                before: None,
                after: None,
            }))),
        },
    );
    assert_eq!(keys(&build(&d, 3, &words).0), prior);
    assert!(d.conversation().live.is_empty());
    let (other, _, _, _) = ready_task(u64::MAX - 1);
    let other = build(&other, 3, &words).0;
    let log = |tree: &Tree| tree.nodes().iter().find_map(|n| n.key.filter(|key| key.kind == 28));
    assert_ne!(log(&before), log(&other));
}
#[test]
fn held_and_closing_owner_facts_do_not_finish_an_unobserved_live_attempt() {
    let (mut d, task_watch, run, read) = ready();
    drive(
        &mut d,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                rows: list([ActivityLine {
                    position: 2,
                    at: Wall::EPOCH,
                    event: Activity::RunBegan {
                        attempt: u64::MAX,
                        how: RunBeginning::Fresh { tail: 0 },
                        why: text(b"actual activation"),
                    },
                }]),
                before: None,
                after: None,
            }))),
        },
    );
    deliver(
        &mut d,
        run,
        Delivery::Run(Box::new(RunText {
            attempt: u64::MAX,
            began: Some(Wall::EPOCH),
            turn: TurnName { attempt: u64::MAX, number: 1 },
            from: 0,
            through: 4,
            text: text(b"live"),
            cut: false,
            call: None,
        })),
    );
    let active = ActiveRun {
        stage: RunStage::Running { attempt: u64::MAX },
        phase_since: Wall::EPOCH,
        attempt: Some(u64::MAX),
        placement: None,
        retry: None,
        observation: None,
    };
    for runtime in [
        Runtime::Active(active.clone()),
        Runtime::Held { was: None },
        Runtime::Held { was: Some(HeldRuntime::Active(active)) },
        Runtime::Closing(Closing { delegates: 0, connectors: None, stage: None, ending: None }),
        Runtime::Closing(Closing {
            delegates: 0,
            connectors: Some(list([])),
            stage: Some(ClosingStage::Run { attempt: u64::MAX }),
            ending: None,
        }),
    ] {
        let mut doc = super::cards_objects::task_doc(&[]);
        doc.chip.executor = Executor::Agent { charter: 2 };
        doc.runtime = runtime;
        doc.activity = 2;
        deliver(&mut d, task_watch, Delivery::Task(Box::new(doc)));
        assert!(!d.conversation().run_finished);
        assert_eq!(d.conversation().live.len(), 1);
    }
    let mut doc = super::cards_objects::task_doc(&[]);
    doc.chip.executor = Executor::Agent { charter: 2 };
    doc.runtime = Runtime::Ended;
    doc.activity = 2;
    deliver(&mut d, task_watch, Delivery::Task(Box::new(doc)));
    assert!(d.conversation().run_finished);
    assert!(d.conversation().live.is_empty());
}

#[test]
fn raw_live_call_rendering_omits_unsupplied_wall_and_preserves_escaped_literals_loss_and_folds() {
    let words = words();
    let (mut d, _, run, _) = ready();
    let name = TurnName { attempt: u64::MAX, number: u32::MAX };
    let raw = CurrentCall { number: u64::MAX, tool: text(b"raw/tool\xff"), input: text(b"a/b\x00\xfe"), cut: true };
    let mut value = RunText {
        attempt: name.attempt,
        began: None,
        turn: name,
        from: 0,
        through: 4,
        text: text(b"live"),
        cut: false,
        call: Some(LiveCall::Raw(raw.clone())),
    };
    deliver(&mut d, run, Delivery::Run(Box::new(value.clone())));
    assert_eq!(d.conversation().live.get(0).unwrap().began, None);
    assert_eq!(d.conversation().live.get(0).unwrap().call, value.call);
    let (folded, bindings) = build(&d, 3, &words);
    access(&folded);
    assert!(contains(&folded, b"\\x72\\x61\\x77\\x2F\\x74\\x6F\\x6F\\x6C\\xFF"));
    assert!(!contains(&folded, b"\\x61\\x2F\\x62\\x00\\xFE"));
    let missing_nodes = folded.nodes().len();
    drop(bindings);
    value.began = Some(Wall::EPOCH);
    deliver(&mut d, run, Delivery::Run(Box::new(value.clone())));
    let supplied = build(&d, 3, &words).0;
    assert_eq!(supplied.nodes().len(), missing_nodes + 1, "an actual epoch gets its own wall leaf");
    assert_eq!(d.conversation().live.get(0).unwrap().began, Some(Wall::EPOCH));
    drive(
        &mut d,
        Event::Act {
            act: Act::FoldCall {
                name: jig_client::conversation::CallName { attempt: name.attempt, turn: name.number, call: raw.number },
                open: true,
            },
        },
    );
    let expanded = build(&d, 3, &words).0;
    access(&expanded);
    assert!(contains(&expanded, b"\\x61\\x2F\\x62\\x00\\xFE"));
    assert!(expanded.nodes().iter().any(|node| node.states.as_slice().contains(&crate::State::Expanded)));
    assert_eq!(d.conversation().parts(4).len(), 0);
    match value.call.as_mut().unwrap() {
        LiveCall::Raw(call) => call.cut = false,
        LiveCall::Decoded(_) => unreachable!(),
    }
    value.began = None;
    deliver(&mut d, run, Delivery::Run(Box::new(value)));
    let uncut = build(&d, 3, &words).0;
    assert_eq!(uncut.nodes().len() + 2, expanded.nodes().len(), "wall and raw-call loss are independent leaves");
    assert!(contains(&uncut, b"\\x61\\x2F\\x62\\x00\\xFE"));
}
