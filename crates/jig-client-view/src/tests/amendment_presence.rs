//! Selected amendments are independent of displayed source facts (client.md, 9).
use super::*;
use crate::objects::Body;
use crate::tests::fixtures::Fixture;

fn empty() -> Amendment {
    Amendment { addressed: None, spec: None, instructions: None, wake: None, dependencies: None, authority: None }
}
fn sent(out: &[Request]) -> &Amendment {
    out.iter()
        .find_map(|request| match request {
            Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } => {
                match &**ask {
                    Ask::Amend { change, .. } => Some(change),
                    _ => None,
                }
            }
            _ => None,
        })
        .expect("the selected amendment is sent")
}
fn open(row: Task) -> (Fixture, Id<Object>) {
    let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
    drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::Amend } });
    drive(&mut domain, Event::Act { action: Action::Edit { field: Purpose::Reason, text: text(b"") } });
    (domain, id)
}

#[test]
fn amendment_single_field_inputs_select_only_whole_property_and_explicit_empty_instructions() {
    let mut row = task_doc(&[ActKind::Amend]);
    row.chip.phase = Phase::Active;
    row.person_revision = Some(Revision { value: 12 });
    *row.chip.details = ChipPhase::Person { addressed: Recipient::Party(1), taken: None, since: None };
    row.spec.specification.parameters = list([
        ParameterDocument::Number { name: 7, value: u64::MAX },
        ParameterDocument::Bytes { name: 8, value: DocumentText::complete(text(UTF8_TEXTS[0])) },
    ]);
    row.spec.specification.inputs = list([u64::MAX]);
    let mut specification = row.spec.whole().unwrap();
    specification.words = text(b"replacement");
    let mut budget = row.authority.whole().unwrap();
    budget.spend = 25;
    let mut deadline = row.authority.whole().unwrap();
    deadline.deadline = crate::confirmation::amendment::deadline(b"2000-02-29 12:34");
    for (label, value, selected) in [
        (
            b"New specification".as_slice(),
            b"replacement".as_slice(),
            Amendment { spec: Some(specification), ..empty() },
        ),
        (b"New instructions", b"", Amendment { instructions: Some(text(b"")), ..empty() }),
        (b"New budget", b"25", Amendment { authority: Some(budget), ..empty() }),
        (
            b"New deadline (UTC, YYYY-MM-DD HH:MM)",
            b"2000-02-29 12:34",
            Amendment { authority: Some(deadline), ..empty() },
        ),
        (
            b"Address to party number",
            b"9",
            Amendment {
                addressed: Some(AddressChange { revision: Revision { value: 12 }, address: PersonAddress::Party(9) }),
                ..empty()
            },
        ),
        (
            b"Address to role number in this project",
            b"7",
            Amendment {
                addressed: Some(AddressChange { revision: Revision { value: 12 }, address: PersonAddress::Role(7) }),
                ..empty()
            },
        ),
    ] {
        let (mut domain, _) = open(row.clone());
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
        let tree = dialog(&domain, words(), &mut bindings);
        assert!(contains(&tree, b"whole task instructions"));
        assert!(!contains(&tree, b"Address to task number"), "only actual person and role destinations are offered");
        assert!(domain.confirmation().unwrap().choices.change.is_none());
        let action = input(&tree, label, value, &mut bindings);
        drive(&mut domain, Event::Act { action });
        assert_eq!(domain.confirmation().unwrap().choices.change.as_ref(), Some(&selected));
        let out = drive(&mut domain, Event::Act { action: Action::Confirm });
        assert_eq!(sent(&out), &selected);
        if let Some(spec) = &sent(&out).spec {
            assert_eq!(spec.parameters.capacity(), 2);
            assert_eq!(spec.inputs.capacity(), 1);
        }
    }
}

#[test]
fn amendment_dependency_and_wake_controls_preserve_only_prior_selected_changes() {
    let mut row = task_doc(&[ActKind::Amend]);
    row.dependencies = list([OwnerTaskLink::Live(chip())]);
    row.wake.notices = WakeRule::Batch { count: 11, age: Duration::from_nanos(22) };
    row.wake.news = WakeRule::Immediate;
    row.wake.questions = true;
    row.subscriptions.entries =
        list([Subscription { number: 11, kind: SubscriptionKind::Topic { connector: 2, topic: 22 } }]);
    let initial_wake = row.wake;
    let (mut domain, _) = open(row);
    let words = words();
    let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
    let tree = dialog(&domain, words, &mut bindings);
    drive(&mut domain, Event::Act { action: press(&tree, b"Drop dependency 8", &mut bindings) });
    let mut expected = Amendment { dependencies: Some(list([])), ..empty() };
    assert_eq!(
        domain.confirmation().unwrap().choices.change.as_ref().unwrap().dependencies.as_ref().unwrap().capacity(),
        1
    );
    assert_eq!(domain.confirmation().unwrap().choices.change.as_ref(), Some(&expected));
    let tree = dialog(&domain, words, &mut bindings);
    drive(&mut domain, Event::Act { action: press(&tree, b"Wakes on inbox messages", &mut bindings) });
    let mut wake = initial_wake;
    wake.words = WakeRule::Never;
    expected.wake = Some(wake);
    assert_eq!(domain.confirmation().unwrap().choices.change.as_ref(), Some(&expected));
    let tree = dialog(&domain, words, &mut bindings);
    drive(&mut domain, Event::Act { action: press(&tree, b"Wakes on a delegate result", &mut bindings) });
    wake.results = ResultsWake::Never;
    expected.wake = Some(wake);
    let tree = dialog(&domain, words, &mut bindings);
    assert!(!tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"Drop dependency 8")));
    drive(&mut domain, Event::Act { action: input(&tree, b"New instructions", b"", &mut bindings) });
    expected.instructions = Some(text(b""));
    let out = drive(&mut domain, Event::Act { action: Action::Confirm });
    assert_eq!(sent(&out), &expected);
}

#[test]
fn amendment_loss_in_specification_does_not_select_or_block_independent_fields() {
    for mask in 1_u8..8 {
        let mut row = task_doc(&[ActKind::Amend]);
        row.spec.specification.words.cut = mask & 1 != 0;
        row.spec.specification.parameters = list([ParameterDocument::Bytes {
            name: 8,
            value: DocumentText { bytes: text(UTF8_TEXTS[0]), cut: mask & 2 != 0 },
        }]);
        row.spec.cut = mask & 4 != 0;
        let (mut domain, _) = open(row);
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
        let tree = dialog(&domain, words(), &mut bindings);
        assert!(!tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"New specification")));
        assert!(contains(&tree, b"The specification is incomplete"));
        assert!(contains(&tree, b"whole task specification"));
        assert!(contains(&tree, UTF8_TEXTS[0]));
        assert_eq!(contains(&tree, b"Only part of this text was supplied"), mask & 3 != 0);
        drive(&mut domain, Event::Act { action: input(&tree, b"New instructions", b"", &mut bindings) });
        let out = drive(&mut domain, Event::Act { action: Action::Confirm });
        assert_eq!(sent(&out), &Amendment { instructions: Some(text(b"")), ..empty() });
    }
}

#[test]
fn amendment_rebuild_keeps_selected_authority_and_distinct_person_spec_and_hold_revisions() {
    let row = escalation_doc(HoldReason::Budget { spent: 10, limit: 10 }, &[ActKind::AmendRelease]);
    let (mut domain, id) = loaded(ObjectKey::Escalation(8), ReadValue::Escalation(Box::new(row.clone())));
    drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::AmendRelease } });
    drive(&mut domain, Event::Act { action: Action::Edit { field: Purpose::Reason, text: text(b"") } });
    let words = words();
    let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
    for (label, value) in [
        (b"New budget".as_slice(), b"25".as_slice()),
        (b"New deadline (UTC, YYYY-MM-DD HH:MM)", b"2000-02-29 12:34"),
        (b"New instructions", b""),
    ] {
        let tree = dialog(&domain, words, &mut bindings);
        drive(&mut domain, Event::Act { action: input(&tree, label, value, &mut bindings) });
    }
    let mut authority = row.steering.authority.whole().unwrap();
    authority.spend = 25;
    authority.deadline = crate::confirmation::amendment::deadline(b"2000-02-29 12:34");
    let expected = Amendment { authority: Some(authority), instructions: Some(text(b"")), ..empty() };
    let open = domain.confirmation().unwrap();
    assert_eq!(open.choices.change.as_ref(), Some(&expected));
    assert_eq!(open.revision, Some(Revision { value: 6 }));
    assert_eq!(open.release_revision, Some(Revision { value: 4 }));
    let projected = domain.objects().get(id).unwrap();
    let Body::Escalation(retained) = projected.body.as_ref().unwrap() else {
        panic!("retained escalation");
    };
    assert_eq!(retained.steering.person_revision, Some(Revision { value: 12 }));
    assert_eq!(retained.steering.instructions.as_ref().unwrap().bytes(), b"held instructions");
    let out = drive(&mut domain, Event::Act { action: Action::Confirm });
    assert_eq!(sent(&out), &expected);
    let ask = out
        .iter()
        .find_map(|request| match request {
            Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } => {
                Some(&**ask)
            }
            _ => None,
        })
        .expect("sent amendment request");
    let Ask::Amend { revision, release, .. } = ask else {
        panic!("sent amendment request");
    };
    assert_eq!(*revision, Revision { value: 6 });
    assert_eq!(*release, Some(Revision { value: 4 }));
}

#[test]
fn person_address_inputs_preserve_full_width_targets_and_the_named_person_revision_with_other_selected_edits() {
    for (label, bytes, address) in [
        (b"Address to party number".as_slice(), b"18446744073709551615".as_slice(), PersonAddress::Party(u64::MAX)),
        (b"Address to role number in this project".as_slice(), b"4294967295".as_slice(), PersonAddress::Role(u32::MAX)),
    ] {
        let row = escalation_doc(HoldReason::Budget { spent: 10, limit: 10 }, &[ActKind::AmendRelease]);
        let (mut domain, id) = loaded(ObjectKey::Escalation(8), ReadValue::Escalation(Box::new(row.clone())));
        drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::AmendRelease } });
        drive(&mut domain, Event::Act { action: Action::Edit { field: Purpose::Reason, text: text(b"") } });
        let words = words();
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
        let tree = dialog(&domain, words, &mut bindings);
        drive(
            &mut domain,
            Event::Act { action: input(&tree, b"New specification", b"selected whole words", &mut bindings) },
        );
        let tree = dialog(&domain, words, &mut bindings);
        assert!(contains(&tree, b"Person-task revision 12"));
        if address == PersonAddress::Role(u32::MAX) {
            let token =
                tree.nodes().iter().find(|node| node.name.as_deref() == Some(label)).unwrap().binding.unwrap().token;
            assert_eq!(decode(&mut bindings, token, Act::Input { value: Box::from(b"4294967296".as_slice()) }), None);
            assert!(domain.confirmation().unwrap().choices.change.as_ref().unwrap().addressed.is_none());
        }
        drive(&mut domain, Event::Act { action: input(&tree, label, bytes, &mut bindings) });
        let mut specification = row.steering.spec.whole().unwrap();
        specification.words = text(b"selected whole words");
        let expected = Amendment {
            addressed: Some(AddressChange { revision: Revision { value: 12 }, address }),
            spec: Some(specification),
            ..empty()
        };
        assert_eq!(domain.confirmation().unwrap().choices.change.as_ref(), Some(&expected));
        let out = drive(&mut domain, Event::Act { action: Action::Confirm });
        assert_eq!(sent(&out), &expected);
        let ask = out
            .iter()
            .find_map(|request| match request {
                Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } => {
                    Some(&**ask)
                }
                _ => None,
            })
            .unwrap();
        let Ask::Amend { project, revision, release, .. } = ask else {
            panic!("sent amendment");
        };
        assert_eq!(*project, 4);
        assert_eq!(*revision, Revision { value: 6 });
        assert_eq!(*release, Some(Revision { value: 4 }));
    }
}

#[test]
fn address_controls_require_actual_person_revision_and_keep_task_recipients_outside_the_closed_destination() {
    for (revision, recipient) in [
        (None, Some(Recipient::Party(9))),
        (Some(Revision { value: 12 }), Some(Recipient::Task(9))),
        (Some(Revision { value: 12 }), Some(Recipient::Roles { project: 4, roles: list([7, 2]) })),
        (Some(Revision { value: 12 }), Some(Recipient::Role { project: 99, role: 7 })),
    ] {
        let mut row = escalation_doc(HoldReason::Budget { spent: 10, limit: 10 }, &[ActKind::AmendRelease]);
        row.steering.person_revision = revision;
        row.steering.addressed = recipient;
        let (mut domain, id) = loaded(ObjectKey::Escalation(8), ReadValue::Escalation(Box::new(row)));
        drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::AmendRelease } });
        drive(&mut domain, Event::Act { action: Action::Edit { field: Purpose::Reason, text: text(b"") } });
        let tree = dialog(&domain, words(), &mut JigBindings::new(limits().bindings, CLIENT.text));
        assert!(!tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"Address to party number")));
        assert!(
            !tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"Address to role number in this project"))
        );
        assert!(!tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"Address to task number")));
        assert!(
            tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"New budget")),
            "absent address facts do not block independent properties"
        );
    }
    let mut row = task_doc(&[ActKind::Amend]);
    row.person_revision = None;
    let (domain, _) = open(row);
    let tree = dialog(&domain, words(), &mut JigBindings::new(limits().bindings, CLIENT.text));
    assert!(!tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"Address to party number")));
    assert!(tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"New budget")));
}

const UTF8_TEXTS: [&[u8]; 1] = [b"\0~"];
