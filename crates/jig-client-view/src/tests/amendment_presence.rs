//! Selected amendments are independent of displayed source facts (client.md, 9).
use super::*;
use jig_client::objects::Body;

fn empty() -> Amendment {
    Amendment { addressed: None, spec: None, instructions: None, wake: None, dependencies: None, authority: None }
}
fn sent(out: &[Request]) -> &Amendment {
    out.iter()
        .find_map(|request| match request {
            Request::Send { ask, .. } => match &**ask {
                Ask::Amend { change, .. } => Some(change),
                _ => None,
            },
            _ => None,
        })
        .expect("the selected amendment is sent")
}
fn open(row: Task) -> (Domain, Id<Object>) {
    let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
    drive(&mut domain, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::Amend } });
    (domain, id)
}

#[test]
fn amendment_single_field_inputs_select_only_whole_property_and_explicit_empty_instructions() {
    let mut row = task_doc(&[ActKind::Amend]);
    row.chip.phase = Phase::Active;
    *row.chip.details = ChipPhase::Person { addressed: Recipient::Party(1), taken: None, since: None };
    row.spec.specification.parameters = list([
        ParameterDocument::Number { name: 7, value: u64::MAX },
        ParameterDocument::Bytes { name: 8, value: DocumentText::complete(text(&[0, 255])) },
    ]);
    row.spec.specification.inputs = list([u64::MAX]);
    let mut specification = row.spec.whole().unwrap();
    specification.words = text(b"replacement");
    let mut budget = row.authority.clone();
    budget.budget = 25;
    let mut deadline = row.authority.clone();
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
        (b"Address to party number", b"9", Amendment { addressed: Some(Recipient::Party(9)), ..empty() }),
        (b"Address to task number", b"9", Amendment { addressed: Some(Recipient::Task(9)), ..empty() }),
        (
            b"Address to role number in this project",
            b"7",
            Amendment { addressed: Some(Recipient::Role { project: 4, role: 7 }), ..empty() },
        ),
    ] {
        let (mut domain, _) = open(row.clone());
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
        let tree = dialog(&domain, &words(), &mut bindings);
        assert!(contains(&tree, b"whole task instructions"));
        assert!(domain.confirmation().unwrap().choices.change.is_none());
        let action = input(&tree, label, value, &mut bindings);
        drive(&mut domain, Event::Act { act: action });
        assert_eq!(domain.confirmation().unwrap().choices.change.as_ref(), Some(&selected));
        let out = drive(&mut domain, Event::Act { act: Action::Confirm });
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
    row.dependencies = list([chip()]);
    row.wake.topics = list([11, 22]);
    row.wake.subscriptions = list([Part::Resource { connector: 2, path: text(b"actual subscribed path") }]);
    let initial_wake = row.wake.clone();
    let (mut domain, _) = open(row);
    let words = words();
    let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
    let tree = dialog(&domain, &words, &mut bindings);
    drive(&mut domain, Event::Act { act: press(&tree, b"Drop dependency 8", &mut bindings) });
    let mut expected = Amendment { dependencies: Some(list([])), ..empty() };
    assert_eq!(
        domain.confirmation().unwrap().choices.change.as_ref().unwrap().dependencies.as_ref().unwrap().capacity(),
        1
    );
    assert_eq!(domain.confirmation().unwrap().choices.change.as_ref(), Some(&expected));
    let tree = dialog(&domain, &words, &mut bindings);
    drive(&mut domain, Event::Act { act: press(&tree, b"Wakes on inbox messages", &mut bindings) });
    let mut wake = initial_wake;
    wake.words = false;
    expected.wake = Some(wake.clone());
    assert_eq!(domain.confirmation().unwrap().choices.change.as_ref(), Some(&expected));
    let tree = dialog(&domain, &words, &mut bindings);
    drive(&mut domain, Event::Act { act: press(&tree, b"Wakes on a delegate result", &mut bindings) });
    wake.results = false;
    expected.wake = Some(wake);
    let tree = dialog(&domain, &words, &mut bindings);
    assert!(!tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"Drop dependency 8")));
    drive(&mut domain, Event::Act { act: input(&tree, b"New instructions", b"", &mut bindings) });
    expected.instructions = Some(text(b""));
    let out = drive(&mut domain, Event::Act { act: Action::Confirm });
    assert_eq!(sent(&out), &expected);
}

#[test]
fn amendment_loss_in_specification_does_not_select_or_block_independent_fields() {
    for mask in 1_u8..8 {
        let mut row = task_doc(&[ActKind::Amend]);
        row.spec.specification.words.cut = mask & 1 != 0;
        row.spec.specification.parameters = list([ParameterDocument::Bytes {
            name: 8,
            value: DocumentText { bytes: text(&[0, 255]), cut: mask & 2 != 0 },
        }]);
        row.spec.cut = mask & 4 != 0;
        let (mut domain, _) = open(row);
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
        let tree = dialog(&domain, &words(), &mut bindings);
        assert!(!tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"New specification")));
        assert!(contains(&tree, b"The specification is incomplete"));
        drive(&mut domain, Event::Act { act: input(&tree, b"New instructions", b"", &mut bindings) });
        let out = drive(&mut domain, Event::Act { act: Action::Confirm });
        assert_eq!(sent(&out), &Amendment { instructions: Some(text(b"")), ..empty() });
    }
}

#[test]
fn amendment_rebuild_keeps_selected_authority_and_distinct_person_spec_and_hold_revisions() {
    let row = escalation_doc(HoldReason::Budget { spent: 10, limit: 10 }, &[ActKind::AmendRelease]);
    let (mut domain, id) = loaded(ObjectKey::Escalation(8), ReadValue::Escalation(Box::new(row.clone())));
    drive(&mut domain, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::AmendRelease } });
    let words = words();
    let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
    for (label, value) in [
        (b"New budget".as_slice(), b"25".as_slice()),
        (b"New deadline (UTC, YYYY-MM-DD HH:MM)", b"2000-02-29 12:34"),
        (b"New instructions", b""),
    ] {
        let tree = dialog(&domain, &words, &mut bindings);
        drive(&mut domain, Event::Act { act: input(&tree, label, value, &mut bindings) });
    }
    let mut authority = row.steering.authority;
    authority.budget = 25;
    authority.deadline = crate::confirmation::amendment::deadline(b"2000-02-29 12:34");
    let expected = Amendment { authority: Some(authority), instructions: Some(text(b"")), ..empty() };
    let open = domain.confirmation().unwrap();
    assert_eq!(open.choices.change.as_ref(), Some(&expected));
    assert_eq!(open.revision, Some(Revision { value: 6 }));
    assert_eq!(open.release_revision, Some(Revision { value: 4 }));
    let Body::Escalation(retained) = domain.objects().get(id).unwrap().body.as_ref().unwrap() else {
        panic!("actual escalation");
    };
    assert_eq!(retained.steering.person_revision, Some(Revision { value: 12 }));
    assert_eq!(retained.steering.instructions.bytes(), b"held instructions");
    let out = drive(&mut domain, Event::Act { act: Action::Confirm });
    assert_eq!(sent(&out), &expected);
    assert!(
        out.iter().any(|request| matches!(request, Request::Send { ask, .. }
        if matches!(&**ask, Ask::Amend { revision: Revision { value: 6 }, release: Some(Revision { value: 4 }), .. })))
    );
}
