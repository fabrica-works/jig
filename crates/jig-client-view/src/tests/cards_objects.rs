#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
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
            admission: Admission::Refused(Refusal::Authority {
                lacking: Lacking::Budget { needed: 40, left: 12 },
                proposable: None,
            }),
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
        ending: Ending::Failed { reason: Some(text(b"whole failure")) },
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
    let tree = b.finish();
    super::cards_access::check(&tree);
    assert!(tree.len() <= crate::card_nodes(&CLIENT).unwrap().checked_add(1).unwrap());
    let facts = d.objects().get(id).unwrap();
    match facts.card {
        jig_client::objects::Card::Deciding { .. } => {
            assert!(tree.nodes().iter().any(|node| node.styles.as_slice().contains(&crate::JigStyle::Card.token())
                && node.states.as_slice().contains(&State::Busy)));
        }
        jig_client::objects::Card::Open
        | jig_client::objects::Card::Refused { .. }
        | jig_client::objects::Card::Unknown { .. }
        | jig_client::objects::Card::Leaving { .. } => {}
    }
    tree
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

fn whole_object_cards_keep_facts_while_confirmation_request_refusal_and_leaving_change(words: &crate::words::Words) {
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
        let tree = render(&d, id, words, &mut bindings);
        assert!(contains(&tree, whole));
        assert!(contains(&tree, b"Your pool has 12 left; this needs 40"));
        assert!(!tree.nodes().iter().any(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Pass up")));
        let action = press(&tree, label, &mut bindings);
        drive(&mut d, Event::Act { act: action });
        let tree = render(&d, id, words, &mut bindings);
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
        let tree = render(&d, id, words, &mut bindings);
        assert!(contains(&tree, b"Being decided"));
        assert!(contains(&tree, whole));
        drive(
            &mut d,
            Event::Answered {
                request,
                answer: Answer::Refused(Refusal::Moved { now: Revision { value: 19 }, what: Moved::PersonTask }),
            },
        );
        let tree = render(&d, id, words, &mut bindings);
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
        assert!(contains(&render(&d, id, words, &mut bindings), whole));
    }
}

fn person_contracts_and_result_values_keep_choices_slots_and_full_payloads(words: &crate::words::Words) {
    let part = Part::Result { connector: 2, task: 8 };
    for contract in [
        PersonContract::Choice {
            options: list([OptionLine {
                number: 3,
                words: text(b"Third choice"),
                words_max: 64,
                followups: 0,
                fields: list([]),
            }]),
        },
        PersonContract::Approval { state: text(b"Named state"), part: part.clone() },
        PersonContract::Words,
    ] {
        let (mut d, id) = loaded(ObjectKey::PersonTask(8), ReadValue::PersonTask(Box::new(person(contract))));
        let mut bindings = JigBindings::new(64, CLIENT.text);
        let before = render(&d, id, words, &mut bindings);
        for node in before.nodes() {
            if node.role == Role::Choice {
                assert!(node.states.as_slice().contains(&State::Disabled));
            }
        }
        let action = press(&before, b"Answer", &mut bindings);
        drive(&mut d, Event::Act { act: action });
        let tree = render(&d, id, words, &mut bindings);
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
    let tree = render(&result_domain, id, words, &mut bindings);
    assert!(contains(&tree, b"whole failure"));
    drive(&mut result_domain, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::PassUp } });
    let tree = render(&result_domain, id, words, &mut bindings);
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
    let tree = render(&result_domain, id, words, &mut bindings);
    assert!(tree.nodes().iter().any(|n| n.role == Role::Status && n.name.as_deref() == Some(b"Leaving")));
    assert!(contains(&tree, b"whole failure"));
    assert!(contains(&tree, b"whole report"));
    result_value_payloads(words, &part);
}

fn result_value_payloads(words: &crate::words::Words, part: &Part) {
    let (d, _) = start();
    for (value, whole) in [
        (ResultValue::Report(text(b"full report")), b"full report".as_slice()),
        (ResultValue::Words(text(b"full words")), b"full words"),
        (
            ResultValue::Verdict {
                verdict: 2,
                words: text(b"explanation"),
                fields: list([ResultField {
                    name: b"field".as_slice().into(),
                    value: b"full field".as_slice().into(),
                }]),
                tasks: list([]),
                proposal: None,
            },
            b"full field",
        ),
        (ResultValue::Connector { summary: text(b"full connector"), part: part.clone() }, b"full connector"),
        (ResultValue::Choice { option: 3, by: document().party, reason: text(b"full reason") }, b"full reason"),
        (
            ResultValue::Approved { state: b"full state".as_slice().into(), words: Some(text(b"approved words")) },
            b"full state",
        ),
        (ResultValue::Failure(text(b"full failure")), b"full failure"),
    ] {
        let mut b = Builder::new(&limits());
        b.open(Role::Page);
        let mut bindings = JigBindings::new(64, CLIENT.text);
        cards::result::value_nodes(&mut b, &d, &value, words, &mut bindings);
        b.close();
        assert!(contains(&b.finish(), whole));
    }
}

fn proposal_kinds_chip_phases_and_live_replies_keep_facts_and_slots(words: &crate::words::Words) {
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
                    addressed: None,
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
        cards::proposal::action(&mut b, &action, words, &mut bindings);
        b.close();
        let tree = b.finish();
        super::cards_access::check(&tree);
        assert!(contains(&tree, whole));
        for node in tree.nodes() {
            if let Some(slot) = node.slot {
                assert_eq!(bindings.part(slot.token), Some(&part));
            }
        }
    }
    all_chip_phases(words);
    live_reply_and_dependency_chips(words);
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
                since: Some(Wall::EPOCH),
            },
            b"Taken by Pat",
        ),
        (ChipPhase::Closing(Closing { delegates: 2, connectors: list([1]) }), b"Closing: 2 delegates"),
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
                how: Ending::Cancelled { by: Actor::Party(1), reason: Some(text(b"actual cancellation")), at: None },
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
                Ending::Done | Ending::Decided { .. } | Ending::Deleted => Phase::Done,
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
        super::cards_access::check(&tree);
        assert!(contains(&tree, expected), "phase text {expected:?}");
        assert!(contains(&tree, b"Task 8: Review"));
        for node in tree.nodes() {
            if let Some(slot) = node.slot {
                let expected =
                    if value.phase == Phase::Closing { Part::Connector { connector: 1 } } else { part.clone() };
                assert_eq!(bindings.part(slot.token), Some(&expected));
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
    super::cards_access::check(&tree);
    assert!(contains(&tree, b"Using tool actual tool"));
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::Region).count(), 2);
    bindings.begin();
    assert_eq!(decode(&mut bindings, link.token, Act::Press), None);
}

fn impact() -> Impact {
    Impact {
        tasks: PhaseCounts { waiting: 1, running: 2, held: 3, closing: 4, done: 5, failed: 6, cancelled: 7 },
        live_runs: 8,
        proposals: 9,
        effects_pending: 10,
        effects_settling: 11,
        takeover_budget: 12,
    }
}
fn escalation_doc(reason: HoldReason, acts: &[ActKind]) -> Escalation {
    let mut task = chip();
    task.details =
        Box::new(ChipPhase::Held { reason: reason.clone(), since: Wall::EPOCH, revision: Revision { value: 4 } });
    Escalation {
        task,
        hold: Hold {
            revision: Revision { value: 4 },
            reason,
            before: Phase::Closing,
            since: Wall::EPOCH,
            result: Some(ResultValue::Report(text(b"kept closing result"))),
            spend: list([
                SpendLine { task: 8, attempt: Some(2), spent: 7 },
                SpendLine { task: 9, attempt: None, spent: 3 },
            ]),
        },
        steering: Box::new(Steering {
            addressed: Some(Recipient::Role { project: 4, role: 2 }),
            revision: Revision { value: 6 },
            spec: text(b"held specification"),
            instructions: text(b"held instructions"),
            wake: WakePolicy { words: true, results: true, topics: list([]), subscriptions: list([]) },
            dependencies: list([9]),
            authority: authority(),
            budget: Budget { total: 10, own: 10, ended: 0, reserved: 0, left: 0, funder: Actor::Party(1) },
        }),
        impact: impact(),
        waiting: Recipient::Task(1),
        trail: list([Passed {
            by: Actor::Party(3),
            at: Wall::EPOCH,
            reason: text(b"previous holder reason"),
            lacking: None,
        }]),
        acts: list(acts.iter().map(|act| Offered { act: *act, admission: Admission::Allowed })),
        ending: None,
    }
}
fn dialog(d: &Domain, words: &crate::words::Words, bindings: &mut JigBindings) -> Tree {
    bindings.begin();
    let mut b = Builder::new(&limits());
    b.open(Role::Page);
    crate::confirmation::confirmation(&mut b, d, words, bindings);
    b.close();
    let tree = b.finish();
    super::cards_access::check(&tree);
    assert!(tree.len() <= crate::confirmation_nodes(&CLIENT).unwrap().checked_add(1).unwrap());
    tree
}
pub(super) fn task_doc(acts: &[ActKind]) -> Task {
    Task {
        chip: chip(),
        goal: true,
        priority: Some(1),
        lineage_chips: list([]),
        inputs: list([]),
        project: 4,
        requester: Actor::Party(1),
        lineage: list([]),
        spec: text(b"whole task specification"),
        spec_revision: Revision { value: 6 },
        instructions: text(b"whole task instructions"),
        wake: WakePolicy { words: true, results: true, topics: list([]), subscriptions: list([]) },
        result_contract: ResultContract::Report,
        dependencies: list([]),
        references: list([]),
        resources: list([]),
        budget: Budget { total: 20, own: 0, ended: 0, reserved: 0, left: 20, funder: Actor::Party(1) },
        authority: authority(),
        hold: Some(escalation_doc(HoldReason::Budget { spent: 10, limit: 10 }, &[]).hold),
        escalation: None,
        proposals: list([]),
        activity: 0,
        deadline: None,
        changed: Wall::EPOCH,
        acts: list(acts.iter().map(|act| Offered { act: *act, admission: Admission::Allowed })),
        parts: list([]),
        closing: None,
        spend: list([]),
        result: None,
        runtime: Runtime::Held,
        impact: impact(),
        delegates: list([LiveDelegate { chip: chip(), takeover_budget: 13 }]),
        writer_slots: list([]),
    }
}
fn input(tree: &Tree, label: &[u8], value: &[u8], bindings: &mut JigBindings) -> Action {
    let node = tree.nodes().iter().find(|n| n.name.as_deref() == Some(label) && n.binding.is_some()).unwrap();
    decode(bindings, node.binding.unwrap().token, Act::Input { value: Box::from(value) }).unwrap()
}
fn escalation_reasons_confirmation_choices_changed_revisions_and_notices_keep_semantic_facts(
    words: &crate::words::Words,
) {
    use crate::confirmation::amendment::deadline;
    use ActKind::{AmendRelease, Cancel, Release};
    let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
    let part = Part::Hold { connector: 2, task: 8 };
    let reasons = [
        (
            HoldReason::Tries {
                failures: list([AttemptFailure {
                    attempt: 2,
                    class: FailureClass::Permanent,
                    reason: text(b"attempt refusal"),
                    at: Wall::EPOCH,
                }]),
            },
            vec![Release, AmendRelease, Cancel],
            b"attempt refusal".as_slice(),
        ),
        (
            HoldReason::Stalled { since: Wall::EPOCH, waiting: text(b"the person") },
            vec![Release, AmendRelease, Cancel],
            b"the person".as_slice(),
        ),
        (
            HoldReason::Dependency { task: 9, how: Ending::Failed { reason: Some(text(b"dependency refusal")) } },
            vec![AmendRelease, Cancel],
            b"dependency refusal".as_slice(),
        ),
        (HoldReason::Drift { part: part.clone() }, vec![Release, Cancel], b"system changed".as_slice()),
        (HoldReason::EffectFailed { part: part.clone() }, vec![Release, Cancel], b"failed for good".as_slice()),
        (HoldReason::EffectUncertain { part: part.clone() }, vec![Release, Cancel], b"may or may not".as_slice()),
        (HoldReason::Budget { spent: 10, limit: 10 }, vec![AmendRelease, Cancel], b"spent 10".as_slice()),
        (HoldReason::Deadline { at: Wall::EPOCH }, vec![AmendRelease, Cancel], b"past deadline".as_slice()),
        (
            HoldReason::Stopped { by: document().party, at: Wall::EPOCH },
            vec![Release, AmendRelease, Cancel],
            b"stopped by Pat".as_slice(),
        ),
        (
            HoldReason::Resource { resource: part.clone(), holder: Actor::Task(9) },
            vec![Release, AmendRelease, Cancel],
            b"past its bound".as_slice(),
        ),
        (HoldReason::Pool { part: part.clone() }, vec![Release, Cancel], b"pool lost".as_slice()),
        (
            HoldReason::Procedure { part: part.clone() },
            vec![Release, AmendRelease, Cancel],
            b"procedure needs".as_slice(),
        ),
    ];
    for (reason, acts, expected) in reasons {
        escalation_row_scenario(reason, &acts, expected, words, &mut bindings);
    }
    confirmation_act_scenarios(words, &mut bindings);
    goal_dialog_scenario(words, &mut bindings);
    notice_scenarios(words);
    assert_eq!(deadline(b"1970-01-01 00:00"), Some(Wall::EPOCH));
    assert_eq!(deadline(b"2000-02-29 12:34").unwrap().as_nanos(), 951_827_640_000_000_000);
    assert!(deadline(b"2100-02-29 00:00").is_none());
    assert!(deadline(b"2000-02-30 00:00").is_none());
    assert!(deadline(b"1969-12-31 23:59").is_none());
    assert!(deadline(b"9999-12-31 00:00").is_none());
}
fn confirmation_act_scenarios(words: &crate::words::Words, bindings: &mut JigBindings) {
    for act in [
        ActKind::Write,
        ActKind::Take,
        ActKind::HandBack,
        ActKind::Release,
        ActKind::LeaveHeld,
        ActKind::Amend,
        ActKind::AmendRelease,
        ActKind::Stop,
        ActKind::Cancel,
        ActKind::Close,
        ActKind::TakeOver,
        ActKind::Prioritise,
        ActKind::SetGoal,
        ActKind::MarkSeen,
        ActKind::Adopt,
        ActKind::ChangePolicy,
        ActKind::SetRoles,
        ActKind::MakeService,
        ActKind::SetPool,
    ] {
        let task = task_doc(&[act]);
        let (mut d, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(task)));
        drive(&mut d, Event::Act { act: Action::Intend { object: id.token(), act } });
        let tree = dialog(&d, words, bindings);
        assert!(contains(&tree, &words.say(cards::common::act_phrase(act), &[])));
        if act == ActKind::Cancel {
            assert!(contains(&tree, b"Dangerous action"));
            assert!(contains(&tree, b"11 begun effects will settle"));
            assert!(tree.nodes().iter().any(|n| n.role == Role::Button
                && n.styles.as_slice().contains(&crate::JigStyle::Danger.token())
                && n.states.as_slice().contains(&State::Disabled)));
            let edit = input(&tree, b"Your reason", b"stop it", bindings);
            drive(&mut d, Event::Act { act: edit });
            let tree = dialog(&d, words, bindings);
            assert_eq!(press(&tree, b"Confirm", bindings), Action::Confirm);
        }
        if act == ActKind::Close {
            let choose = press(&tree, b"Take over delegate 8, reserving 13", bindings);
            drive(&mut d, Event::Act { act: choose });
            assert_eq!(d.confirmation().unwrap().choices.disposals.as_slice(), &[Disposal::TakeOver { task: 8 }]);
            let tree = dialog(&d, words, bindings);
            assert_eq!(press(&tree, b"Confirm", bindings), Action::Confirm);
        }
        if act == ActKind::Prioritise {
            let change = input(&tree, b"New goal priority", b"3", bindings);
            drive(&mut d, Event::Act { act: change });
            assert_eq!(d.confirmation().unwrap().choices.priority, Some(3));
        }
    }
    for act in [ActKind::Accept, ActKind::Reject, ActKind::PassUp] {
        let mut p = proposal(ProposalAction::Batch { tasks: list([]), own: true });
        p.acts = list([Offered { act, admission: Admission::Proposed { to: Recipient::Task(2) } }]);
        let (mut d, id) = loaded(ObjectKey::Proposal(p.key), ReadValue::Proposal(Box::new(p)));
        drive(&mut d, Event::Act { act: Action::Intend { object: id.token(), act } });
        let tree = dialog(&d, words, bindings);
        assert!(contains(&tree, b"will be proposed"));
        assert!(contains(&tree, b"Waiting for task 2"));
        if act == ActKind::Accept {
            assert!(contains(&tree, b"accepted tasks will be yours"));
            assert!(contains(&tree, b"Funds 10; 20 left now, 10 after"));
        }
    }
    let p = person(PersonContract::Words);
    let (mut d, id) = loaded(ObjectKey::PersonTask(8), ReadValue::PersonTask(Box::new(p)));
    drive(&mut d, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::Answer } });
    let tree = dialog(&d, words, bindings);
    assert!(tree.nodes().iter().any(|n| n.role == Role::TextArea && n.name.as_deref() == Some(b"Your answer")));
    let choose = press(&tree, b"Cannot do it", bindings);
    drive(&mut d, Event::Act { act: choose });
    assert!(d.confirmation().unwrap().choices.cannot);
    for act in [ActKind::EditNote, ActKind::DeleteNote] {
        let note = Note {
            number: 1,
            revision: Revision { value: 2 },
            scope: NoteScope::Project(4),
            description: text(b"note title"),
            body: text(b"note body"),
            by: Actor::Party(1),
            tasks: list([]),
            changed: Wall::EPOCH,
            acts: offered(act),
        };
        let (mut d, id) = note_loaded(note);
        drive(&mut d, Event::Act { act: Action::Intend { object: id.token(), act } });
        let tree = dialog(&d, words, bindings);
        assert!(contains(&tree, b"note title"));
        if act == ActKind::EditNote {
            assert!(tree.nodes().iter().any(|n| n.value.as_ref().is_some_and(|v| v.text.as_ref() == b"note body")));
        }
    }
    changed_dialog_scenario(words, bindings);
}
fn goal_dialog_scenario(words: &crate::words::Words, bindings: &mut JigBindings) {
    use jig_client::address::{Address, Place, ProjectSection};
    let (mut d, frame) = start();
    let mut doc = document();
    doc.projects.get_mut(0).unwrap().goals = list([Charter { number: 1, name: text(b"lead") }]);
    drive(&mut d, Event::Watch { watch: frame, event: WatchEvent::Delivered(Delivery::Frame(Box::new(doc))) });
    let out = drive(
        &mut d,
        Event::Went { place: Place::Jig(Address::Project { project: 4, section: ProjectSection::Goals }) },
    );
    let read = out
        .iter()
        .find_map(|r| match r {
            Request::Read { read, what: Read::Project { .. } } => Some(*read),
            _ => None,
        })
        .unwrap();
    let budget = Budget { total: 20, own: 0, ended: 0, reserved: 0, left: 20, funder: Actor::Party(1) };
    let project = Project {
        number: 4,
        name: text(b"One"),
        period: 1,
        spend_lines: list([]),
        policy: Policy {
            ceiling: authority(),
            period: 1,
            spend: 0,
            roles: list([]),
            requirements: list([]),
            deciders: list([]),
        },
        roles: list([]),
        spend: budget,
        pools: list([Pool { party: 1, period: 1, budget }]),
        resources: list([]),
        services: list([]),
        recurring: list([]),
        parts: list([]),
        acts: list([Offered {
            act: ActKind::SetGoal,
            admission: Admission::Proposed { to: Recipient::Role { project: 4, role: 2 } },
        }]),
    };
    drive(&mut d, Event::Read { read, result: ReadResult::Value(ReadValue::Project(Box::new(project))) });
    drive(&mut d, Event::Act { act: Action::Edit { field: Purpose::NewGoal, text: text(b"whole new goal") } });
    drive(
        &mut d,
        Event::Act {
            act: Action::GoalForm {
                form: jig_client::pages::project::goals::GoalForm {
                    project: 4,
                    charter: 1,
                    budget: 21,
                    deadline: Some(Wall::EPOCH),
                    priority: 2,
                },
            },
        },
    );
    drive(&mut d, Event::Act { act: Action::Submit { field: Purpose::NewGoal } });
    let tree = dialog(&d, words, bindings);
    assert!(contains(&tree, b"whole new goal"));
    assert!(contains(&tree, b"Your pool has 20 left; this needs 21"));
    assert!(contains(&tree, b"will be proposed"));
    assert!(contains(&tree, b"Charter 1"));
    assert_eq!(press(&tree, b"Confirm", bindings), Action::Confirm);
}
fn notice_scenarios(words: &crate::words::Words) {
    use jig_client::notices::Kind;
    for kind in [
        Kind::Link(jig_client::link::Link::Offline),
        Kind::Link(jig_client::link::Link::Live),
        Kind::Answer { object: ObjectKey::Task(8), answer: Answer::Done(Outcome::Changed) },
        Kind::PageAnswer {
            answer: Answer::Refused(Refusal::Authority {
                lacking: Lacking::Budget { needed: 40, left: 12 },
                proposable: None,
            }),
        },
        Kind::Unknown { object: ObjectKey::Escalation(8), act: ActKind::Release },
        Kind::PageUnknown { purpose: jig_client::pages::IntentPurpose::SetGoal },
        Kind::Refused(Limit::Objects),
    ] {
        let mut b = Builder::new(&limits());
        crate::notices::facts(&mut b, &kind, words);
        let tree = b.finish();
        super::cards_access::check(&tree);
        assert_eq!(tree.nodes()[0].role, Role::Status);
        assert!(tree.nodes()[0].name.is_some());
        assert!(tree.len() > 1);
    }
    let (mut d, _) = start();
    jig_client::notices::show(&mut d, Kind::Refused(Limit::Objects));
    let mut b = Builder::new(&limits());
    let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
    crate::notices::notices(&mut b, &d, words, &mut bindings);
    let tree = b.finish();
    super::cards_access::check(&tree);
    assert!(contains(&tree, b"Working set limit"));
    let dismiss = press(&tree, b"Dismiss", &mut bindings);
    drive(&mut d, Event::Act { act: dismiss });
    assert!(d.notices().ids().is_empty());
}

fn note_loaded(note: Note) -> (Domain, Id<Object>) {
    use jig_client::address::{Address, Place, ProjectSection};
    let (mut d, frame) = start();
    drive(&mut d, Event::Watch { watch: frame, event: WatchEvent::Delivered(Delivery::Frame(Box::new(document()))) });
    let out = drive(
        &mut d,
        Event::Went { place: Place::Jig(Address::Project { project: 4, section: ProjectSection::Notes }) },
    );
    let read = out
        .iter()
        .find_map(|r| match r {
            Request::Read { read, what: Read::Notes { .. } } => Some(*read),
            _ => None,
        })
        .unwrap();
    drive(
        &mut d,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Notes(Box::new(NotePage {
                rows: list([note]),
                before: None,
                after: None,
            }))),
        },
    );
    let id = d.objects().find(ObjectKey::Note { project: 4, note: 1 }).unwrap();
    (d, id)
}

#[expect(
    clippy::match_like_matches_macro,
    reason = "ordinary scenario patterns use matches with an explicit catch-all; the macro is disallowed"
)]
fn escalation_row_scenario(
    reason: HoldReason,
    acts: &[ActKind],
    expected: &[u8],
    words: &crate::words::Words,
    bindings: &mut JigBindings,
) {
    let row = escalation_doc(reason, acts);
    let (mut d, id) = loaded(ObjectKey::Escalation(8), ReadValue::Escalation(Box::new(row.clone())));
    let tree = render(&d, id, words, bindings);
    assert!(contains(&tree, expected));
    assert!(contains(&tree, b"kept closing result"));
    assert!(contains(&tree, b"Task 8, attempt 2: spent 7"));
    assert!(contains(&tree, b"Task 9: spent 3"));
    assert!(contains(&tree, b"previous holder reason"));
    for act in acts {
        assert!(
            tree.nodes().iter().any(|n| n.role == Role::Button
                && n.name.as_deref() == Some(&words.say(cards::common::act_phrase(*act), &[])))
        );
    }
    drive(&mut d, Event::Act { act: Action::Intend { object: id.token(), act: acts[0] } });
    let tree = dialog(&d, words, bindings);
    assert!(tree.nodes().iter().any(|n| n.role == Role::Dialog && n.name.is_some()));
    if match row.hold.reason {
        HoldReason::EffectUncertain { .. } => true,
        _ => false,
    } {
        assert!(contains(&tree, b"engine will do as you choose"));
        let choose = press(&tree, b"Release without making the effect again; tell its executor so", bindings);
        drive(&mut d, Event::Act { act: choose });
        assert_eq!(d.confirmation().unwrap().choices.again, Some(false));
        let tree = dialog(&d, words, bindings);
        let send = press(&tree, b"Confirm", bindings);
        let out = drive(&mut d, Event::Act { act: send });
        assert!(out.iter().any(|r| match r {
            Request::Send { ask, .. } => match **ask {
                Ask::Release { again: Some(false), .. } => true,
                _ => false,
            },
            _ => false,
        }));
    }
    if acts[0] == ActKind::AmendRelease {
        assert!(contains(&tree, b"held specification"));
        assert!(contains(&tree, b"Hold revision 4"));
        assert_eq!(d.confirmation().unwrap().release_revision, Some(Revision { value: 4 }));
        let initial_writes = d.confirmation().unwrap().writes;
        let written = tree
            .nodes()
            .iter()
            .find(|n| n.name.as_deref() == Some(b"New instructions"))
            .unwrap()
            .value
            .as_ref()
            .unwrap()
            .written;
        assert_eq!(written, initial_writes);
        let change = input(&tree, b"New budget", b"25", bindings);
        drive(&mut d, Event::Act { act: change });
        assert_eq!(d.confirmation().unwrap().choices.change.as_ref().unwrap().authority.as_ref().unwrap().budget, 25);
        let tree = dialog(&d, words, bindings);
        let change = input(&tree, b"New instructions", b"try another way", bindings);
        drive(&mut d, Event::Act { act: change });
        let open = d.confirmation().unwrap();
        assert_eq!(open.choices.change.as_ref().unwrap().instructions.as_ref().unwrap().bytes(), b"try another way");
        assert_eq!(open.choices.change.as_ref().unwrap().authority.as_ref().unwrap().budget, 25);
        assert_eq!(open.revision, Some(Revision { value: 6 }));
        let tree = dialog(&d, words, bindings);
        let change = input(&tree, b"Address to role number in this project", b"7", bindings);
        drive(&mut d, Event::Act { act: change });
        assert_eq!(
            d.confirmation().unwrap().choices.change.as_ref().unwrap().addressed,
            Some(Recipient::Role { project: 4, role: 7 })
        );
    }
    if expected == b"spent 10" {
        let initial_writes = d.confirmation().unwrap().writes;
        drive(&mut d, Event::Act { act: Action::Dismiss });
        drive(&mut d, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::AmendRelease } });
        assert!(d.confirmation().unwrap().writes > initial_writes);
        let tree = dialog(&d, words, bindings);
        let value = tree
            .nodes()
            .iter()
            .find(|n| n.name.as_deref() == Some(b"New instructions"))
            .unwrap()
            .value
            .as_ref()
            .unwrap();
        assert_eq!(value.text.as_ref(), b"held instructions");
        assert_eq!(value.written, d.confirmation().unwrap().writes);
    }
}

fn changed_dialog_scenario(words: &crate::words::Words, bindings: &mut JigBindings) {
    let row = escalation_doc(HoldReason::Budget { spent: 10, limit: 10 }, &[ActKind::Release]);
    let (mut d, id, _watch) = loaded_with_watch(ObjectKey::Escalation(8), ReadValue::Escalation(Box::new(row.clone())));
    drive(&mut d, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::Release } });
    let out = drive(&mut d, Event::Act { act: Action::Confirm });
    let request = out
        .iter()
        .find_map(|r| match r {
            Request::Send { request, .. } => Some(*request),
            _ => None,
        })
        .unwrap();
    let out = drive(
        &mut d,
        Event::Answered {
            request,
            answer: Answer::Refused(Refusal::Moved { now: Revision { value: 5 }, what: Moved::Hold { reason: None } }),
        },
    );
    let mut moved = row;
    moved.hold.revision.value = 5;
    let read = out
        .iter()
        .find_map(|r| match r {
            Request::Read { read, .. } => Some(*read),
            _ => None,
        })
        .unwrap();
    drive(&mut d, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::Release } });
    drive(&mut d, Event::Read { read, result: ReadResult::Value(ReadValue::Escalation(Box::new(moved))) });
    let tree = dialog(&d, words, bindings);
    assert!(contains(&tree, b"revision this action names changed"));
    assert!(!tree.nodes().iter().any(|n| n.name.as_deref() == Some(b"Confirm") && n.binding.is_some()));
}

#[path = "cards_sizes.rs"]
mod sizes;

#[test]
fn jig_object_cards_hold_actions_confirmations_and_notices() {
    let words = words();
    whole_object_cards_keep_facts_while_confirmation_request_refusal_and_leaving_change(&words);
    person_contracts_and_result_values_keep_choices_slots_and_full_payloads(&words);
    proposal_kinds_chip_phases_and_live_replies_keep_facts_and_slots(&words);
    escalation_reasons_confirmation_choices_changed_revisions_and_notices_keep_semantic_facts(&words);
    sizes::scenario(&words);
    closing_connectors_use_their_owners_words(&words);
    person_chip_preserves_absent_readiness(&words);
    authority_refusal_uses_only_supplied_proposal_eligibility(&words);
    collective_recipients_name_every_supplied_role_in_order(&words);
}

fn closing_connectors_use_their_owners_words(words: &crate::words::Words) {
    let mut b = Builder::new(&limits());
    let mut bindings = JigBindings::new(16, CLIENT.text);
    b.open(Role::Page);
    cards::common::closing(&mut b, &Closing { delegates: 2, connectors: list([7, 11]) }, words, &mut bindings);
    b.close();
    let slots = b.slots(0);
    assert_eq!(slots.len(), 2);
    for slot in &slots {
        let connector = match bindings.part(slot.reference.token).unwrap() {
            Part::Connector { connector } => *connector,
            _ => panic!("a name-only connector part"),
        };
        assert_eq!(slot.reference.owner, crate::Owner::Side { connector });
        let mut own = Builder::new(&limits());
        own.open(Role::Region);
        own.text(crate::Run::Plain, if connector == 7 { b"amber" } else { b"blue" });
        own.close();
        b.splice(slot.id, own);
    }
    let tree = b.finish();
    for fact in [b"Closing: 2 delegates".as_slice(), b"amber", b"blue", b"'s effects settling"] {
        assert!(contains(&tree, fact));
    }
    assert!(!contains(&tree, b"1 effects"));
    assert!(!contains(&tree, b"2 effects"));
}

fn person_chip_preserves_absent_readiness(words: &crate::words::Words) {
    for since in [None, Some(Wall::EPOCH)] {
        let (d, _) = start();
        let mut value = chip();
        value.phase = Phase::Active;
        *value.details = ChipPhase::Person { addressed: Recipient::Party(1), taken: None, since };
        let mut b = Builder::new(&limits());
        let mut bindings = JigBindings::new(16, CLIENT.text);
        b.open(Role::Page);
        cards::chip::chip(&mut b, &d, &value, words, &mut bindings);
        b.close();
        let tree = b.finish();
        assert_eq!(contains(&tree, b"Since"), since.is_some());
    }
}

fn authority_refusal_uses_only_supplied_proposal_eligibility(words: &crate::words::Words) {
    for proposable in [None, Some(true), Some(false)] {
        let mut b = Builder::new(&limits());
        b.open(Role::Page);
        words.refusal(&mut b, &Refusal::Authority { lacking: Lacking::Approver, proposable });
        b.close();
        let tree = b.finish();
        assert!(contains(&tree, b"Proposal approval authority is missing"));
        assert_eq!(contains(&tree, b"Nobody in this project may accept it"), proposable == Some(false));
    }
}

fn collective_recipients_name_every_supplied_role_in_order(words: &crate::words::Words) {
    let mut b = Builder::new(&limits());
    b.open(Role::Page);
    cards::common::recipient(&mut b, &Recipient::Roles { project: 4, roles: list([7, 2]) }, words);
    b.close();
    let tree = b.finish();
    let names: Vec<_> =
        tree.nodes().iter().flat_map(|node| node.text.as_slice().iter().map(|run| run.text.as_ref())).collect();
    assert_eq!(names, [b"Waiting at policy roles in project 4".as_slice(), b"Role 7", b"Role 2"]);
}

#[test]
fn approved_and_failed_results_show_only_their_supplied_owner_payloads() {
    let (domain, _) = start();
    let words = words();
    for explanation in [None, Some(text(b"")), Some(text(b"exact explanation"))] {
        let value = ResultValue::Approved { state: Box::new([0, 255, 1]), words: explanation.clone() };
        let mut builder = Builder::new(&limits());
        builder.open(Role::Page);
        let mut bindings = JigBindings::new(64, CLIENT.text);
        cards::result::value_nodes(&mut builder, &domain, &value, &words, &mut bindings);
        builder.close();
        let tree = builder.finish();
        assert!(contains(&tree, &[0, 255, 1]));
        assert!(!contains(&tree, document().party.name.bytes()));
        if let Some(explanation) = explanation
            && !explanation.bytes().is_empty()
        {
            assert!(contains(&tree, explanation.bytes()));
        }
    }
    let mut builder = Builder::new(&limits());
    builder.open(Role::Page);
    let mut bindings = JigBindings::new(64, CLIENT.text);
    cards::result::value_nodes(
        &mut builder,
        &domain,
        &ResultValue::Failure(text(b"cannot complete")),
        &words,
        &mut bindings,
    );
    builder.close();
    let tree = builder.finish();
    assert!(contains(&tree, b"Failed"));
    assert!(contains(&tree, b"cannot complete"));
}
