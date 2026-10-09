#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary tests inspect trees and script public domain requests"
)]
use super::cards::{CLIENT, contains, document, drive, limits, start, text, words};
use crate::{Act, Builder, JigBindings, Role, State, Tree, cards, decode};
use jig_client::{Domain, Event, Request, action::Act as Action, contract::*, fields::Purpose, objects::Object};
use skein_lib::{Duration, Id, List, Wall};

fn list<T>(rows: impl IntoIterator<Item = T>) -> List<T> {
    let rows: Vec<T> = rows.into_iter().collect();
    let mut out = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for row in rows {
        assert!(out.push(row).is_ok());
    }
    out
}
fn chip() -> Chip {
    Chip {
        task: 8,
        title: text(b"Review"),
        executor: Executor::Party { party: 1 },
        phase: Phase::Held,
        details: Box::new(ChipPhase::Held {
            reason: HoldReason::Budget { spent: 10, limit: 10 },
            since: Wall::EPOCH,
            revision: Revision { value: 4 },
        }),
    }
}
fn authority() -> Authority {
    Authority {
        budget: 10,
        grants: list([]),
        task_kinds: list([Executor::Party { party: 1 }]),
        depth: 2,
        tasks: 3,
        deadline: Some(Wall::EPOCH),
        tools: list([2]),
        note_scopes: list([NoteScope::Deployment]),
    }
}
fn offered(act: ActKind) -> List<Offered> {
    list([
        Offered { act, admission: Admission::Allowed },
        Offered {
            act: ActKind::PassUp,
            admission: Admission::Refused(Refusal::Authority { lacking: Lacking::Budget { needed: 40, left: 12 } }),
        },
    ])
}
fn proposal(action: ProposalAction) -> Proposal {
    Proposal {
        key: ProposalKey { proposer: 8, number: 1 },
        project: 4,
        proposer: Actor::Task(8),
        why: text(b"**whole reason**"),
        action,
        needs: authority(),
        funding: Budget { total: 20, own: 0, ended: 0, reserved: 0, left: 20, funder: Actor::Party(1) },
        waiting: Recipient::Role { project: 4, role: 2 },
        trail: list([Passed { by: Actor::Party(3), at: Wall::EPOCH, reason: text(b"whole trail"), lacking: None }]),
        since: Wall::EPOCH,
        acts: offered(ActKind::Accept),
        ending: None,
    }
}
fn question() -> Question {
    Question {
        task: chip(),
        message: 1,
        words: text(b"whole question"),
        since: Wall::EPOCH,
        acts: offered(ActKind::Answer),
        ending: None,
    }
}
fn person(contract: PersonContract) -> PersonTask {
    PersonTask {
        task: chip(),
        revision: Revision { value: 4 },
        requester: Actor::Task(1),
        spec: text(b"whole specification"),
        contract,
        inputs: list([result()]),
        addressed: Recipient::Party(1),
        taken: Some(document().party),
        since: Some(Wall::EPOCH),
        stall: Some(Duration::from_secs(240)),
        deadline: None,
        acts: offered(ActKind::Answer),
        ending: None,
    }
}
fn result() -> Result {
    Result {
        task: chip(),
        ending: Ending::Failed { reason: text(b"whole failure") },
        value: Some(ResultValue::Report(text(b"whole report"))),
        at: Wall::EPOCH,
        acts: offered(ActKind::MarkSeen),
    }
}
fn loaded_with_watch(key: ObjectKey, value: ReadValue) -> (Domain, Id<Object>, skein_lib::Token) {
    let (mut d, frame) = start();
    let out = drive(
        &mut d,
        Event::Watch { watch: frame, event: WatchEvent::Delivered(Delivery::Frame(Box::new(document()))) },
    );
    let inbox = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Inbox { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let out = drive(
        &mut d,
        Event::Watch {
            watch: inbox,
            event: WatchEvent::Delivered(Delivery::Inbox(Box::new(Inbox {
                entries: list([InboxEntry {
                    reply: None,
                    lineage: list([]),
                    goal: None,
                    recipient: None,
                    stall: None,
                    object: key,
                    project: 4,
                    group: InboxGroup::Questions,
                    position: 1,
                    at: Wall::EPOCH,
                }]),
                needs: document().needs,
                updates: Count { value: 0, limited: false },
                results_seen: 0,
                replies_seen: list([]),
                more: false,
            }))),
        },
    );
    let read = out
        .iter()
        .find_map(|r| match r {
            Request::Read { read, .. } => Some(*read),
            _ => None,
        })
        .unwrap();
    drive(&mut d, Event::Read { read, result: ReadResult::Value(value) });
    let id = d.objects().find(key).unwrap();
    (d, id, inbox)
}
fn loaded(key: ObjectKey, value: ReadValue) -> (Domain, Id<Object>) {
    let (d, id, _) = loaded_with_watch(key, value);
    (d, id)
}
fn render(d: &Domain, id: Id<Object>, words: &crate::words::Words, bindings: &mut JigBindings) -> Tree {
    bindings.begin();
    let mut b = Builder::new(&limits());
    b.open(Role::Page);
    cards::card(&mut b, d, id, &cards::Place::Elsewhere, words, bindings);
    b.close();
    b.finish()
}
fn press(tree: &Tree, label: &[u8], bindings: &mut JigBindings) -> Action {
    let token = tree
        .nodes()
        .iter()
        .find(|n| n.role == Role::Button && n.name.as_deref() == Some(label))
        .unwrap()
        .binding
        .unwrap()
        .token;
    decode(bindings, token, Act::Press).unwrap()
}

#[test]
fn whole_object_cards_keep_facts_while_confirmation_request_refusal_and_leaving_change() {
    let words = words();
    let p = proposal(ProposalAction::Widen { before: authority(), after: authority() });
    for (key, value, label, whole) in [
        (
            ObjectKey::Proposal(p.key),
            ReadValue::Proposal(Box::new(p)),
            b"Accept".as_slice(),
            b"whole reason".as_slice(),
        ),
        (
            ObjectKey::Question { task: 8, message: 1 },
            ReadValue::Question(Box::new(question())),
            b"Answer",
            b"whole question",
        ),
        (
            ObjectKey::PersonTask(8),
            ReadValue::PersonTask(Box::new(person(PersonContract::Words))),
            b"Answer",
            b"whole specification",
        ),
    ] {
        let (mut d, id) = loaded(key, value);
        let mut bindings = JigBindings::new(64, CLIENT.text);
        let tree = render(&d, id, &words, &mut bindings);
        assert!(contains(&tree, whole));
        assert!(contains(&tree, b"Your pool has 12 left; this needs 40"));
        assert!(!tree.nodes().iter().any(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Pass up")));
        let action = press(&tree, label, &mut bindings);
        drive(&mut d, Event::Act { act: action });
        let tree = render(&d, id, &words, &mut bindings);
        assert!(contains(&tree, whole));
        assert!(contains(&tree, b"review the confirmation"));
        if label == b"Answer" {
            let field = tree
                .nodes()
                .iter()
                .find(|n| n.role == Role::TextArea && n.name.as_deref() == Some(b"Your answer"))
                .unwrap()
                .binding
                .unwrap()
                .token;
            let edit =
                decode(&mut bindings, field, Act::Input { value: Box::from(b"complete answer".as_slice()) }).unwrap();
            drive(&mut d, Event::Act { act: edit });
            assert_eq!(d.fields().get(Purpose::Answer).unwrap().text.bytes(), b"complete answer");
        }
        let out = drive(&mut d, Event::Act { act: Action::Confirm });
        let request = out
            .iter()
            .find_map(|r| match r {
                Request::Send { request, .. } => Some(*request),
                _ => None,
            })
            .unwrap();
        let tree = render(&d, id, &words, &mut bindings);
        assert!(contains(&tree, b"Being decided"));
        assert!(contains(&tree, whole));
        drive(
            &mut d,
            Event::Answered {
                request,
                answer: Answer::Refused(Refusal::Moved { now: Revision { value: 19 }, what: Moved::PersonTask }),
            },
        );
        let tree = render(&d, id, &words, &mut bindings);
        assert!(contains(&tree, b"19"));
        assert!(contains(&tree, whole));
        let action = press(&tree, label, &mut bindings);
        drive(&mut d, Event::Act { act: action });
        if label == b"Answer" {
            drive(&mut d, Event::Act { act: Action::Edit { field: Purpose::Answer, text: text(b"answer") } });
        }
        let out = drive(&mut d, Event::Act { act: Action::Confirm });
        let request = out
            .iter()
            .find_map(|r| match r {
                Request::Send { request, .. } => Some(*request),
                _ => None,
            })
            .unwrap();
        drive(
            &mut d,
            Event::Answered {
                request,
                answer: Answer::Done(Outcome::DecidedFirst {
                    by: document().party,
                    how: Decision::Accepted,
                    at: Wall::EPOCH,
                }),
            },
        );
        assert!(contains(&render(&d, id, &words, &mut bindings), whole));
    }
}

#[test]
fn person_contracts_and_result_values_keep_choices_slots_and_full_payloads() {
    let words = words();
    let part = Part::Result { connector: 2, task: 8 };
    for contract in [
        PersonContract::Choice { options: list([OptionLine { number: 3, words: text(b"Third choice") }]) },
        PersonContract::Approval { state: text(b"Named state"), part: part.clone() },
        PersonContract::Words,
    ] {
        let (mut d, id) = loaded(ObjectKey::PersonTask(8), ReadValue::PersonTask(Box::new(person(contract))));
        let mut bindings = JigBindings::new(64, CLIENT.text);
        let before = render(&d, id, &words, &mut bindings);
        for node in before.nodes() {
            if node.role == Role::Choice {
                assert!(node.states.as_slice().contains(&State::Disabled));
            }
        }
        let action = press(&before, b"Answer", &mut bindings);
        drive(&mut d, Event::Act { act: action });
        let tree = render(&d, id, &words, &mut bindings);
        assert!(contains(&tree, b"whole report"));
        assert!(contains(&tree, b"Becomes news for its requester after 4 min"));
        for node in tree.nodes() {
            if let Some(slot) = node.slot {
                assert_eq!(bindings.part(slot.token), Some(&part));
                assert_eq!(decode(&mut bindings, slot.token, Act::Press), None);
            }
            if node.role == Role::Choice {
                let act = decode(&mut bindings, node.binding.unwrap().token, Act::Press).unwrap();
                assert_eq!(act, Action::Choose { option: 3 });
                drive(&mut d, Event::Act { act });
            }
        }
        assert!(contains(&tree, b"Cannot do it"));
    }
    let (mut result_domain, id, watch) = loaded_with_watch(ObjectKey::Result(8), ReadValue::Result(Box::new(result())));
    let mut bindings = JigBindings::new(64, CLIENT.text);
    let tree = render(&result_domain, id, &words, &mut bindings);
    assert!(contains(&tree, b"whole failure"));
    drive(&mut result_domain, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::PassUp } });
    let tree = render(&result_domain, id, &words, &mut bindings);
    assert!(contains(&tree, b"Your pool has 12 left; this needs 40"));
    assert!(contains(&tree, b"whole report"));
    let out = drive(
        &mut result_domain,
        Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Inbox(Box::new(Inbox {
                entries: list([]),
                needs: document().needs,
                updates: Count { value: 0, limited: false },
                results_seen: 0,
                replies_seen: list([]),
                more: false,
            }))),
        },
    );
    let read = out
        .iter()
        .find_map(|r| match r {
            Request::Read { read, .. } => Some(*read),
            _ => None,
        })
        .unwrap();
    drive(&mut result_domain, Event::Read { read, result: ReadResult::Value(ReadValue::Result(Box::new(result()))) });
    let tree = render(&result_domain, id, &words, &mut bindings);
    assert!(tree.nodes().iter().any(|n| n.role == Role::Status && n.name.as_deref() == Some(b"Leaving")));
    assert!(contains(&tree, b"whole failure"));
    assert!(contains(&tree, b"whole report"));
    let (d, _) = start();
    for (value, whole) in [
        (ResultValue::Report(text(b"full report")), b"full report".as_slice()),
        (ResultValue::Words(text(b"full words")), b"full words"),
        (
            ResultValue::Verdict { verdict: 2, fields: list([text(b"full field")]), tasks: list([]), proposal: None },
            b"full field",
        ),
        (ResultValue::Connector { summary: text(b"full connector"), part: part.clone() }, b"full connector"),
        (ResultValue::Choice { option: 3, by: document().party, reason: text(b"full reason") }, b"full reason"),
        (ResultValue::Approval { state: text(b"full state"), approved: false, by: document().party }, b"full state"),
    ] {
        let mut b = Builder::new(&limits());
        b.open(Role::Page);
        let mut bindings = JigBindings::new(64, CLIENT.text);
        cards::result::value_nodes(&mut b, &d, &value, &words, &mut bindings);
        b.close();
        assert!(contains(&b.finish(), whole));
    }
}

#[test]
fn proposal_kinds_chip_phases_and_live_replies_keep_facts_and_slots() {
    let words = words();
    let part = Part::Effect { connector: 2, proposal: ProposalKey { proposer: 8, number: 9 } };
    for (action, whole) in [
        (
            ProposalAction::Batch {
                own: true,
                tasks: list([ProposedTask {
                    local: 3,
                    spec: text(b"full batch spec"),
                    executor: Executor::Party { party: 1 },
                    budget: 40,
                    deadline: Some(Wall::EPOCH),
                    dependencies: list([2]),
                    grants: list([]),
                }]),
            },
            b"full batch spec".as_slice(),
        ),
        (ProposalAction::Effect { part: part.clone() }, b"Effect"),
        (ProposalAction::Widen { before: authority(), after: authority() }, b"After"),
        (
            ProposalAction::Amend {
                task: 8,
                before: text(b"full before"),
                change: Amendment {
                    words: Some(text(b"full after")),
                    instructions: None,
                    wake: None,
                    dependencies: None,
                    authority: None,
                },
            },
            b"full after",
        ),
    ] {
        let mut b = Builder::new(&limits());
        b.open(Role::Page);
        let mut bindings = JigBindings::new(64, CLIENT.text);
        cards::proposal::action(&mut b, &action, &words, &mut bindings);
        b.close();
        let tree = b.finish();
        assert!(contains(&tree, whole));
        for node in tree.nodes() {
            if let Some(slot) = node.slot {
                assert_eq!(bindings.part(slot.token), Some(&part));
            }
        }
    }
    all_chip_phases(&words);
    live_reply_and_dependency_chips(&words);
}

fn all_chip_phases(words: &crate::words::Words) {
    let (d, _) = start();
    let part = Part::Panel { connector: 2, task: 8 };
    for (details, expected) in [
        (
            ChipPhase::Waiting { dependencies: list([99]), hold: Some(part.clone()) },
            b"Loading task 99 phase facts".as_slice(),
        ),
        (
            ChipPhase::Idle {
                wake: WakePolicy { words: true, results: true, topics: list([3]), subscriptions: list([part.clone()]) },
            },
            b"Subscribed topic 3",
        ),
        (ChipPhase::Starting, b"Starting"),
        (
            ChipPhase::Running { attempt: 2, since: Wall::EPOCH, tool: Some(text(b"search tool")) },
            b"Using tool search tool",
        ),
        (ChipPhase::Running { attempt: 2, since: Wall::EPOCH, tool: None }, b"Writing"),
        (ChipPhase::WaitingForWords { attempt: 3, since: Wall::EPOCH }, b"Waiting for your words"),
        (
            ChipPhase::BackingOff {
                at: Wall::EPOCH,
                attempt: 2,
                tries: 3,
                class: FailureClass::HostLost,
                reason: text(b"actual retry reason"),
            },
            b"actual retry reason",
        ),
        (ChipPhase::Procedure { panel: part.clone() }, b"Task 8: Review"),
        (
            ChipPhase::Person {
                addressed: Recipient::Role { project: 4, role: 2 },
                taken: Some(document().party),
                since: Wall::EPOCH,
            },
            b"Taken by Pat",
        ),
        (ChipPhase::Closing(Closing { delegates: 2, effects: 1 }), b"2 delegates and 1 effects settling"),
        (
            ChipPhase::Held {
                reason: HoldReason::Stopped { by: document().party, at: Wall::EPOCH },
                since: Wall::EPOCH,
                revision: Revision { value: 7 },
            },
            b"Held: stopped by Pat",
        ),
        (
            ChipPhase::Ended {
                how: Ending::Cancelled { by: Actor::Party(1), reason: text(b"actual cancellation") },
                at: Wall::EPOCH,
            },
            b"actual cancellation",
        ),
    ] {
        let mut value = chip();
        value.phase = match &details {
            ChipPhase::Waiting { .. } => Phase::Waiting,
            ChipPhase::Idle { .. } => Phase::Idle,
            ChipPhase::Starting => Phase::Due,
            ChipPhase::Running { .. } | ChipPhase::WaitingForWords { .. } => Phase::Running,
            ChipPhase::BackingOff { .. } => Phase::BackingOff,
            ChipPhase::Procedure { .. } | ChipPhase::Person { .. } => Phase::Active,
            ChipPhase::Closing(_) => Phase::Closing,
            ChipPhase::Held { .. } => Phase::Held,
            ChipPhase::Ended { how, .. } => match how {
                Ending::Done | Ending::Decided { .. } => Phase::Done,
                Ending::Failed { .. } => Phase::Failed,
                Ending::Cancelled { .. } | Ending::Withdrawn { .. } => Phase::Cancelled,
            },
        };
        *value.details = details;
        let mut b = Builder::new(&limits());
        b.open(Role::Page);
        let mut bindings = JigBindings::new(64, CLIENT.text);
        cards::chip::chip(&mut b, &d, &value, words, &mut bindings);
        b.close();
        let tree = b.finish();
        assert!(contains(&tree, expected), "phase text {expected:?}");
        assert!(contains(&tree, b"Task 8: Review"));
        for node in tree.nodes() {
            if let Some(slot) = node.slot {
                assert_eq!(bindings.part(slot.token), Some(&part));
            }
        }
        if contains(&tree, b"Attempt") {
            assert!(contains(&tree, b"For 0 min"));
        }
    }
}

fn live_reply_and_dependency_chips(words: &crate::words::Words) {
    let (mut d, frame) = start();
    let out = drive(
        &mut d,
        Event::Watch { watch: frame, event: WatchEvent::Delivered(Delivery::Frame(Box::new(document()))) },
    );
    let inbox = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Inbox { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let mut dependency = chip();
    dependency.task = 9;
    dependency.title = text(b"Live dependency");
    dependency.phase = Phase::Running;
    *dependency.details = ChipPhase::Running { attempt: 2, since: Wall::EPOCH, tool: Some(text(b"actual tool")) };
    let conversation = ConversationLine {
        chip: dependency,
        project: 4,
        first_words: text(b"whole first reply"),
        unread: Count { value: 2, limited: false },
        seen: 5,
        last_reply: 7,
        changed: Wall::EPOCH,
        runtime: Runtime::Running { attempt: 2, since: Wall::EPOCH, tool: Some(text(b"actual tool")) },
        delegates: 0,
        budget: Budget { total: 0, own: 0, ended: 0, reserved: 0, left: 0, funder: Actor::Party(1) },
    };
    drive(
        &mut d,
        Event::Watch {
            watch: inbox,
            event: WatchEvent::Delivered(Delivery::Inbox(Box::new(Inbox {
                entries: list([InboxEntry {
                    reply: Some(Box::new(conversation)),
                    lineage: list([]),
                    goal: None,
                    recipient: None,
                    stall: None,
                    object: ObjectKey::Task(9),
                    project: 4,
                    group: InboxGroup::Replies,
                    position: 7,
                    at: Wall::EPOCH,
                }]),
                needs: document().needs,
                updates: Count { value: 1, limited: false },
                results_seen: 0,
                replies_seen: list([]),
                more: false,
            }))),
        },
    );
    let id = d.objects().find(ObjectKey::Task(9)).unwrap();
    let mut bindings = JigBindings::new(64, CLIENT.text);
    let tree = render(&d, id, words, &mut bindings);
    assert!(contains(&tree, b"Live dependency"));
    assert!(contains(&tree, b"whole first reply"));
    let link = tree
        .nodes()
        .iter()
        .find(|n| {
            n.role == Role::Link && n.name.as_deref().is_some_and(|name| name.starts_with(b"Open at first reply"))
        })
        .unwrap()
        .binding
        .unwrap();
    assert_eq!(
        bindings.address(link.token),
        Some(jig_client::address::Address::Task { task: 9, section: jig_client::address::TaskSection::Activity })
    );
    assert_eq!(decode(&mut bindings, link.token, Act::Press), Some(Action::OpenChat { task: 9 }));
    let mut root = chip();
    root.phase = Phase::Waiting;
    *root.details = ChipPhase::Waiting { dependencies: list([9, 9]), hold: None };
    let mut b = Builder::new(&limits());
    b.open(Role::Page);
    bindings.begin();
    cards::chip::chip(&mut b, &d, &root, words, &mut bindings);
    b.close();
    let tree = b.finish();
    assert!(contains(&tree, b"Using tool actual tool"));
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::Region).count(), 2);
    bindings.begin();
    assert_eq!(decode(&mut bindings, link.token, Act::Press), None);
}
