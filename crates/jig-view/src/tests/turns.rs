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
use crate::words::Words;
use jig_domain_client::{
    Domain, Event, Request,
    action::Act,
    address::{Address, TaskSection},
    contract::*,
    fields::Purpose,
};
use skein_lib::{Token, Wall};
use skein_ui::{Key, Role, Tree};
fn read(out: &[Request]) -> Token {
    out.iter()
        .find_map(|request| match request {
            Request::Read { read, what: Read::Activity { .. } } => Some(*read),
            _ => None,
        })
        .unwrap()
}
pub(super) fn ready() -> (Domain, Token, Token, Token) {
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
    doc.context.as_mut().expect("supplied task context").runtime =
        Runtime::Running { attempt: 2, since: Wall::EPOCH, tool: None };
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
            turn: ProvisionalTurnName { retry: 0, attempt: 2, number },
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
    let act = crate::decode(
        &mut bindings,
        named_binding(&first, b"Open input and result for ReadFact"),
        skein_ui::Act::Press,
    )
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
                cut: false,
                rows: list([
                    ActivityLine {
                        position: 2,
                        at: Wall::EPOCH,
                        event: Activity::Turn {
                            name: TurnName { attempt: 2, number: 1 },
                            turn: TurnRead::Decoded(turn()),
                        },
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
                before: Some(Cursor::new(Box::from(b"older".as_slice()), 1024).expect("bounded opaque cursor")),
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
        skein_ui::Act::Press,
    )
    .unwrap();
    drive(&mut d, Event::Act { act: closed });
    assert!(!contains(&build(&d, 3, words).0, b"whole private input"));
    let mut doc = super::cards_objects::task_doc(&[ActKind::Write]);
    doc.chip.executor = Executor::Agent { charter: 2 };
    doc.chip.title = DocumentText::complete(text(b"Updated live task"));
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
                cut: false,
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
                    cut: false,
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
                    before: Some(Cursor::new(Box::from(b"older".as_slice()), 1024).expect("bounded opaque cursor")),
                    after: Some(Cursor::new(Box::from(b"newer".as_slice()), 1024).expect("bounded opaque cursor")),
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
        let more = crate::decode(&mut bindings, named_binding(&tree, b"Read earlier"), skein_ui::Act::Press).unwrap();
        token = read(&drive(&mut d, Event::Act { act: more }));
    }
}
#[test]
fn conversation_keeps_word_states_activity_call_folds_and_provisional_turn_identity() {
    let words = words();
    replacement(words);
    words_states(words);
    activities(words);
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
                turn: ProvisionalTurnName { retry: 0, attempt, number: u32::MAX },
                from: 0,
                through: 4,
                text: text(b"live"),
                cut: false,
                call: None,
            })),
        );
    }
    let before = build(&d, 3, words).0;
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
                cut: false,
                rows: list([
                    ActivityLine {
                        position: 2,
                        at: Wall::EPOCH,
                        event: Activity::Turn {
                            name: TurnName { attempt: u64::MAX, number: u32::MAX },
                            turn: TurnRead::Decoded(turn()),
                        },
                    },
                    ActivityLine {
                        position: 1,
                        at: Wall::EPOCH,
                        event: Activity::Turn {
                            name: TurnName { attempt: u64::MAX - 1, number: u32::MAX },
                            turn: TurnRead::Decoded(turn()),
                        },
                    },
                ]),
                before: None,
                after: None,
            }))),
        },
    );
    assert_eq!(keys(&build(&d, 3, words).0), prior);
    assert!(d.conversation().live.is_empty());
    let (other, _, _, _) = ready_task(u64::MAX - 1);
    let other = build(&other, 3, words).0;
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
                cut: false,
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
            turn: ProvisionalTurnName { retry: 0, attempt: u64::MAX, number: 1 },
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
        doc.context.as_mut().expect("supplied task context").runtime = runtime;
        doc.activity = 2;
        deliver(&mut d, task_watch, Delivery::Task(Box::new(doc)));
        assert!(!d.conversation().run_finished);
        assert_eq!(d.conversation().live.len(), 1);
    }
    let mut doc = super::cards_objects::task_doc(&[]);
    doc.chip.executor = Executor::Agent { charter: 2 };
    doc.context.as_mut().expect("supplied task context").runtime = Runtime::Ended;
    doc.activity = 2;
    deliver(&mut d, task_watch, Delivery::Task(Box::new(doc)));
    assert!(d.conversation().run_finished);
    assert!(d.conversation().live.is_empty());
}

#[test]
fn raw_live_call_rendering_omits_unsupplied_wall_and_preserves_escaped_literals_loss_and_folds() {
    let words = words();
    let (mut d, _, run, _) = ready();
    let name = ProvisionalTurnName { retry: 0, attempt: u64::MAX, number: u32::MAX };
    let raw = CurrentCall {
        identity: current_identity(UTF8_TEXTS[0]),
        began: None,
        tool: text(UTF8_TEXTS[1]),
        input: text(UTF8_TEXTS[2]),
        cut: true,
    };
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
    let (folded, bindings) = build(&d, 3, words);
    access(&folded);
    assert!(contains(&folded, UTF8_TEXTS[1]));
    assert!(!contains(&folded, UTF8_TEXTS[2]));
    assert!(contains(&folded, UTF8_TEXTS[0]));
    let missing_nodes = folded.nodes().len();
    drop(bindings);
    value.began = Some(Wall::EPOCH);
    deliver(&mut d, run, Delivery::Run(Box::new(value.clone())));
    let supplied = build(&d, 3, words).0;
    assert_eq!(supplied.nodes().len(), missing_nodes + 2, "an actual epoch gets its own time and text nodes");
    assert_eq!(d.conversation().live.get(0).unwrap().began, Some(Wall::EPOCH));
    drive(&mut d, Event::Act { act: Act::FoldCurrentCall { identity: raw.identity.clone(), open: true } });
    let expanded = build(&d, 3, words).0;
    access(&expanded);
    assert!(contains(&expanded, UTF8_TEXTS[2]));
    assert!(expanded.nodes().iter().any(|node| node.states.as_slice().contains(&skein_ui::State::Expanded)));
    assert_eq!(d.conversation().parts(4).len(), 0);
    match value.call.as_mut().unwrap() {
        LiveCall::Raw(call) => call.cut = false,
        LiveCall::Decoded(_) => unreachable!(),
    }
    value.began = None;
    deliver(&mut d, run, Delivery::Run(Box::new(value)));
    let uncut = build(&d, 3, words).0;
    assert_eq!(uncut.nodes().len() + 4, expanded.nodes().len(), "wall and raw-call loss are independent leaves");
    assert!(contains(&uncut, UTF8_TEXTS[2]));
    let mut first_seen = RunText {
        attempt: name.attempt,
        began: None,
        turn: name,
        from: 0,
        through: 4,
        text: text(b"live"),
        cut: false,
        call: Some(LiveCall::Raw(CurrentCall { began: Some(Wall::EPOCH), cut: false, ..raw })),
    };
    deliver(&mut d, run, Delivery::Run(Box::new(first_seen.clone())));
    let observed = build(&d, 3, words).0;
    assert_eq!(observed.nodes().len(), uncut.nodes().len() + 2);
    assert_eq!(d.conversation().live.get(0).unwrap().began, None);
    if let Some(LiveCall::Raw(call)) = &mut first_seen.call {
        call.began = None;
    }
    deliver(&mut d, run, Delivery::Run(Box::new(first_seen)));
    assert_eq!(build(&d, 3, words).0.nodes().len(), uncut.nodes().len());
}

#[test]
fn maximum_raw_live_calls_fit_whole_page_and_escaped_phrase_capacities() {
    use super::cards::CLIENT;
    let words = words();
    let (mut d, _, run, _) = ready();
    let tool = full_raw_byte(UTF8_TEXTS[3][0]);
    let input = full_raw_byte(UTF8_TEXTS[4][0]);
    for number in 1..=CLIENT.rows {
        let name = ProvisionalTurnName { retry: 0, attempt: u64::MAX, number };
        let call = CurrentCall {
            identity: CallIdentity {
                sequence: number,
                id: full_raw_byte(UTF8_TEXTS[5][0]),
                ..current_identity(UTF8_TEXTS[0])
            },
            began: Some(Wall::EPOCH),
            tool: tool.clone(),
            input: input.clone(),
            cut: true,
        };
        deliver(
            &mut d,
            run,
            Delivery::Run(Box::new(RunText {
                attempt: name.attempt,
                began: Some(Wall::from_nanos(u64::MAX)),
                turn: name,
                from: 0,
                through: CLIENT.text,
                text: full_raw_byte(b'x'),
                cut: true,
                call: Some(LiveCall::Raw(call.clone())),
            })),
        );
        drive(&mut d, Event::Act { act: Act::FoldCurrentCall { identity: call.identity.clone(), open: true } });
    }
    assert_eq!(d.conversation().live.len(), CLIENT.rows);
    assert_eq!(d.conversation().current_opened.len(), CLIENT.rows);
    let limits = crate::limits::of_words(&CLIENT, words).unwrap();
    let mut b = skein_ui::Builder::new(&limits);
    let mut bindings = crate::JigBindings::new(limits.bindings, CLIENT.text);
    b.open(Role::Page);
    assert!(crate::jig(
        &mut b,
        &d,
        skein_ui::SlotRef { owner: (crate::Owner::Application { number: 1 }).opaque(), token: Token::new(1) },
        words,
        &mut bindings
    ));
    b.close();
    let tree = b.finish();
    access(&tree);
    assert!(u32::try_from(tree.nodes().len()).unwrap() <= limits.nodes);
    let mut escaped = Vec::new();
    for _ in 0..CLIENT.text {
        escaped.push(UTF8_TEXTS[4][0]);
    }
    assert!(contains(&tree, &escaped));
    match d.conversation().live.get(0).unwrap().call.as_ref().unwrap() {
        LiveCall::Raw(call) => {
            assert_eq!(call.identity.owner, Token::new(u64::MAX));
            assert_eq!(call.identity.id, full_raw_byte(UTF8_TEXTS[5][0]));
        }
        LiveCall::Decoded(_) => panic!("expected the supplied raw current call"),
    }
    assert!(limits.text_per_node >= CLIENT.text * 4);
    let (mut reloaded, _, run, _) = ready();
    assert!(reloaded.conversation().live.is_empty());
    deliver(
        &mut reloaded,
        run,
        Delivery::Run(Box::new(RunText {
            attempt: u64::MAX,
            began: None,
            turn: ProvisionalTurnName { retry: 0, attempt: u64::MAX, number: 1 },
            from: 0,
            through: 0,
            text: text(b""),
            cut: false,
            call: Some(LiveCall::Raw(CurrentCall {
                identity: current_identity(UTF8_TEXTS[0]),
                began: None,
                tool,
                input,
                cut: false,
            })),
        })),
    );
    assert_eq!(reloaded.conversation().live.get(0).unwrap().began, None);
}

#[test]
fn raw_live_call_controls_use_selected_normal_language_and_checked_capacity() {
    use super::cards::CLIENT;
    use crate::live_call_words::LiveCallPhrase;
    use crate::phrases::english;
    use skein_ui::phrases::{Form, PhraseLimits, Phrasebook, PluralClass, Segment};

    let first = english::language();
    let mut selected = english::language();
    for phrase in &mut selected.phrases {
        let label: Option<&[u8]> = if phrase.id == LiveCallPhrase::OpenCurrentCallInput.id() {
            Some(b"Selected language opens actual current input")
        } else if phrase.id == LiveCallPhrase::FoldCurrentCallInput.id() {
            Some(b"Selected language folds actual current input")
        } else {
            None
        };
        if let Some(label) = label {
            phrase.forms =
                Box::new([Form { class: PluralClass::Other, segments: Box::new([Segment::Text(label.into())]) }]);
        }
    }
    let book = Phrasebook::load(
        crate::phrases::jig::shapes(CLIENT.text),
        Box::new([first, selected]),
        &PhraseLimits {
            languages: 2,
            phrases: crate::owner_words::phrase_count(),
            holes: 4,
            segments: 8000,
            text_bytes: 100_000,
            hole_bytes: CLIENT.text,
        },
    )
    .unwrap();
    let words = Words { book, language: 1, offset_minutes: 0 };
    let (mut d, _, run, _) = ready();
    deliver(
        &mut d,
        run,
        Delivery::Run(Box::new(RunText {
            attempt: 2,
            began: None,
            turn: ProvisionalTurnName { retry: 0, attempt: 2, number: 1 },
            from: 0,
            through: 0,
            text: text(b""),
            cut: false,
            call: Some(LiveCall::Raw(CurrentCall {
                identity: current_identity(UTF8_TEXTS[0]),
                began: None,
                tool: text(b"literal/tool"),
                input: text(b"input"),
                cut: false,
            })),
        })),
    );
    let (folded, mut bindings) = build(&d, 3, &words);
    assert!(contains(&folded, b"Selected language opens actual current input"));
    assert!(!contains(&folded, b"Open current call input"));
    let act = crate::decode(
        &mut bindings,
        named_binding(&folded, b"Selected language opens actual current input"),
        skein_ui::Act::Press,
    )
    .unwrap();
    drive(&mut d, Event::Act { act });
    let opened = build(&d, 3, &words).0;
    assert!(contains(&opened, b"Selected language folds actual current input"));
    assert!(contains(&opened, b"input"));
    let limits = crate::limits::of_words(&CLIENT, &words).unwrap();
    assert!(limits.text_per_node >= words.book.capacity(LiveCallPhrase::OpenCurrentCallInput.id()).unwrap());
}

fn full_raw_byte(byte: u8) -> Text {
    let mut bytes = Vec::new();
    bytes.resize(usize::try_from(super::cards::CLIENT.text).unwrap(), byte);
    Text::new(bytes.into_boxed_slice(), super::cards::CLIENT.text).unwrap()
}

fn current_identity(id: &[u8]) -> CallIdentity {
    CallIdentity { owner: Token::new(u64::MAX), sequence: u32::MAX, retry: u32::MAX, id: text(id) }
}

#[test]
fn whole_run_call_only_is_visible_once_without_a_turn_and_uses_normal_document_ack() {
    let words = words();
    let (mut d, _, run, _) = ready();
    let raw = CurrentCall {
        identity: current_identity(UTF8_TEXTS[0]),
        began: None,
        tool: text(b"call-only-tool"),
        input: text(b"call-only-input"),
        cut: true,
    };
    let mut state = RunState { task: 8, attempt: Some(2), committed_turn: 0, writing: None, call: Some(raw.clone()) };
    let out = deliver(&mut d, run, Delivery::RunState(Box::new(state.clone())));
    for request in &out {
        if let Request::Continue { .. } = request {
            panic!("normal document never invents a stream continuation");
        }
    }
    assert!(d.conversation().live.is_empty());
    assert_eq!(d.conversation().current, None);
    assert_eq!(d.conversation().offset(), 0);
    assert_eq!(d.conversation().current_run, Some(state.clone()));
    let folded = build(&d, 3, words).0;
    access(&folded);
    assert!(keys(&folded).is_empty(), "call-only has no invented provisional turn key");
    assert!(contains(&folded, b"call-only-tool"));
    assert!(!contains(&folded, b"call-only-input"));
    let label: &[u8] = b"Observed current tool call";
    assert_eq!(folded.nodes().iter().filter(|node| node.name.as_deref() == Some(label)).count(), 1);
    drive(&mut d, Event::Act { act: Act::FoldCurrentCall { identity: raw.identity.clone(), open: true } });
    let opened = build(&d, 3, words).0;
    assert!(contains(&opened, b"call-only-input"));
    let without_wall = opened.nodes().len();
    state.call.as_mut().unwrap().began = Some(Wall::EPOCH);
    deliver(&mut d, run, Delivery::RunState(Box::new(state.clone())));
    assert_eq!(build(&d, 3, words).0.nodes().len(), without_wall + 2);
    state.writing = Some(RunText {
        attempt: 2,
        began: None,
        turn: ProvisionalTurnName { retry: 0, attempt: 2, number: 1 },
        from: 0,
        through: 4,
        text: text(b"live"),
        cut: false,
        call: Some(LiveCall::Raw(raw)),
    });
    deliver(&mut d, run, Delivery::RunState(Box::new(state.clone())));
    let writing = build(&d, 3, words).0;
    assert_eq!(keys(&writing).len(), 1);
    assert_eq!(
        writing.nodes().iter().filter(|node| node.name.as_deref() == Some(label)).count(),
        1,
        "the full snapshot's independent call is not duplicated inside writing"
    );
    state.writing = None;
    state.committed_turn = 1;
    deliver(&mut d, run, Delivery::RunState(Box::new(state.clone())));
    assert_eq!(d.conversation().current_run, Some(state.clone()));
    assert_eq!(build(&d, 3, words).0.nodes().iter().filter(|node| node.name.as_deref() == Some(label)).count(), 1);
    state.call = None;
    deliver(&mut d, run, Delivery::RunState(Box::new(state)));
    let cleared = build(&d, 3, words).0;
    assert!(!contains(&cleared, b"call-only-tool"), "absent whole-document call clears the retained current call");
}

#[test]
fn maximum_independent_current_call_fits_page_with_all_existing_provisional_rows() {
    use super::cards::CLIENT;
    let words = words();
    let (mut d, _, run, _) = ready();
    for number in 1..=CLIENT.rows {
        live(&mut d, run, number);
    }
    let call = CurrentCall {
        identity: CallIdentity { id: full_raw_byte(UTF8_TEXTS[5][0]), ..current_identity(b"id") },
        began: Some(Wall::from_nanos(u64::MAX)),
        tool: full_raw_byte(UTF8_TEXTS[3][0]),
        input: full_raw_byte(UTF8_TEXTS[4][0]),
        cut: true,
    };
    deliver(
        &mut d,
        run,
        Delivery::RunState(Box::new(RunState {
            task: 8,
            attempt: Some(2),
            committed_turn: 0,
            writing: None,
            call: Some(call.clone()),
        })),
    );
    drive(&mut d, Event::Act { act: Act::FoldCurrentCall { identity: call.identity, open: true } });
    let limits = crate::limits::of_words(&CLIENT, words).unwrap();
    let mut b = skein_ui::Builder::new(&limits);
    let mut bindings = crate::JigBindings::new(limits.bindings, CLIENT.text);
    b.open(Role::Page);
    assert!(crate::jig(
        &mut b,
        &d,
        skein_ui::SlotRef { owner: (crate::Owner::Application { number: 1 }).opaque(), token: Token::new(1) },
        words,
        &mut bindings
    ));
    b.close();
    let tree = b.finish();
    access(&tree);
    assert!(u32::try_from(tree.nodes().len()).unwrap() <= limits.nodes);
    let mut escaped = Vec::new();
    for _ in 0..CLIENT.text {
        escaped.push(UTF8_TEXTS[4][0]);
    }
    assert!(contains(&tree, &escaped));
    assert_eq!(d.conversation().live.len(), CLIENT.rows);
}

#[test]
fn displayed_retry_replaces_empty_then_resumes_with_exact_actual_watch_position() {
    let words = words();
    let (mut d, _, run, _) = ready();
    live(&mut d, run, 1);
    drive(
        &mut d,
        Event::Act {
            act: Act::FoldCall {
                name: jig_domain_client::conversation::CallName { attempt: 2, turn: 1, call: 1 },
                open: true,
            },
        },
    );
    assert!(contains(&build(&d, 3, words).0, b"whole private input"), "existing rich call detail is still available");
    let name = ProvisionalTurnName { attempt: 2, number: 1, retry: u32::MAX };
    let mut value =
        RunText { attempt: 2, began: None, turn: name, from: 0, through: 0, text: text(b""), cut: false, call: None };
    let out = deliver(&mut d, run, Delivery::Run(Box::new(value.clone())));
    assert!(
        out.contains(&Request::Continue { watch: run, subject: Subject::Run { task: 8, turn: Some(name), offset: 0 } })
    );
    assert_eq!(d.conversation().live.len(), 1);
    assert_eq!(d.conversation().live.get(0).unwrap().name, name);
    assert_eq!(d.conversation().live.get(0).unwrap().text.bytes(), b"");
    let empty = build(&d, 3, words).0;
    access(&empty);
    assert!(!contains(&empty, b"provisional first"));
    assert!(!contains(&empty, b"whole private input"), "a removed retry's rich detail cannot remain visible");
    value.text = text(b"new retry bytes");
    value.through = 15;
    let out = deliver(&mut d, run, Delivery::Run(Box::new(value.clone())));
    assert!(
        out.contains(&Request::Continue {
            watch: run,
            subject: Subject::Run { task: 8, turn: Some(name), offset: 15 }
        })
    );
    assert!(contains(&build(&d, 3, words).0, b"new retry bytes"));
    deliver(&mut d, run, Delivery::Run(Box::new(value)));
    assert_eq!(d.conversation().live.get(0).unwrap().text.bytes(), b"new retry bytes");
    assert_eq!(d.conversation().offset(), 15);
}

#[test]
fn whole_owner_activity_and_fetched_archive_keep_numeric_senders_references_and_separate_spend_facts_in_the_public_log()
{
    let (mut domain, _, _, read) = ready();
    let words = words();
    let source = |position, line, decoded| ActivityLine {
        position,
        at: Wall::EPOCH,
        event: Activity::Owner {
            source: Box::new(OwnerActivityLine { task: 8, position, at: Wall::EPOCH, line }),
            decoded,
        },
    };
    drive(
        &mut domain,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                cut: false,
                rows: list([
                    source(
                        2,
                        OwnerActivity::Turn { reply: u64::MAX, attempt: 2, turn: 1, read: Some(0) },
                        Some(ArchivedTurn {
                            task: 8,
                            name: TurnName { attempt: 2, number: 1 },
                            cumulative: 19,
                            read: None,
                            at: Wall::from_nanos(42),
                            turn: TurnRead::Decoded(turn()),
                        }),
                    ),
                    source(
                        1,
                        OwnerActivity::Words {
                            message: 11,
                            by: OwnerRequester::Task(7),
                            kind: OwnerMessageKind::Proposal {
                                proposer: u64::MAX,
                                proposal: 0,
                                kind: OwnerMessageProposalKind::Amend,
                            },
                            words: text(b"raw owner words"),
                        },
                        None,
                    ),
                ]),
                before: None,
                after: None,
            }))),
        },
    );
    let (tree, _) = build(&domain, 3, words);
    for label in [
        b"raw owner words".as_slice(),
        b"Task: ",
        b"18446744073709551615",
        b"Archive cumulative spend:",
        b"Archive accepted at:",
        b"Archive read position not supplied",
        b"Turn spent:",
        b"committed",
        b"Read",
    ] {
        assert!(contains(&tree, label), "missing {label:?}");
    }
    let limits = crate::limits::of_words(&super::cards::CLIENT, words).unwrap();
    assert!(tree.len() <= limits.nodes);
}

const UTF8_TEXTS: [&[u8]; 6] = [b"id/\x7e", b"raw/tool\x7e", b"a/b\x00\x21", b"~", b"!", b"?"];
