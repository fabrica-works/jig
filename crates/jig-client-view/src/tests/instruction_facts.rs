//! Instruction source presence and independent amendment choices (objects.md, 11).
use super::*;
use crate::{
    address::{Address, TaskSection},
    objects::Body,
};

fn sources() -> [Option<DocumentText>; 6] {
    [
        Some(DocumentText::complete(text(b"supplied local effective text"))),
        None,
        Some(DocumentText::complete(text(b""))),
        Some(DocumentText { bytes: text(b""), cut: true }),
        Some(DocumentText { bytes: text(b"supplied partial text"), cut: true }),
        Some(DocumentText::complete(text(b"supplied charter effective text"))),
    ]
}
fn sent(out: &[Request]) -> &Amendment {
    out.iter()
        .find_map(|request| match request {
            Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } => {
                match &**ask {
                    Ask::Amend { change, .. } => Some(change.as_ref()),
                    Ask::Compound { .. } => crate::tests::fixtures::selected_change(ask),
                    _ => None,
                }
            }
            _ => None,
        })
        .unwrap()
}
fn facts(tree: &Tree, source: Option<&DocumentText>) {
    assert_eq!(contains(tree, b"Instructions were not supplied"), source.is_none());
    assert_eq!(
        contains(tree, b"Instructions are empty"),
        source.is_some_and(|value| value.bytes().is_empty() && !value.cut)
    );
    assert_eq!(contains(tree, b"Only part of this text was supplied"), source.is_some_and(|value| value.cut));
    let Some(value) = source else {
        return;
    };
    if !value.bytes().is_empty() {
        assert!(contains(tree, value.bytes()));
    }
}
fn baseline(tree: &Tree, source: Option<&DocumentText>) {
    let node = tree.nodes().iter().find(|node| node.name.as_deref() == Some(b"New instructions")).unwrap();
    let expected = source.filter(|value| !value.cut).map_or(b"".as_slice(), DocumentText::bytes);
    assert_eq!(node.value.as_ref().map_or(b"".as_slice(), |value| &*value.text), expected);
}

#[test]
fn task_instruction_updates_distinguish_supplied_effective_empty_unknown_and_partial_text() {
    let (mut domain, out) =
        super::super::work_pages::entered(Address::Task { task: 8, section: TaskSection::Overview });
    let watch =
        out.iter()
            .find_map(|request| match request {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Task { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    for source in sources() {
        let mut row = task_doc(&[]);
        row.chip.executor = Executor::Agent { charter: 2 };
        row.instructions = source.clone();
        super::super::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(row)));
        let tree = super::super::pages::build(&domain, 3, words()).0;
        facts(&tree, source.as_ref());
        if source.is_none() {
            assert!(!contains(&tree, b"supplied local effective text"));
            assert!(!contains(&tree, b"supplied charter effective text"));
        }
        let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
        let projected = domain.objects().get(id).unwrap();
        let Body::Task(retained) = projected.body.as_ref().unwrap() else {
            panic!("task");
        };
        assert_eq!(retained.instructions, source);
    }
}

#[test]
fn task_instruction_amendment_uses_only_complete_source_without_selecting_it() {
    for source in sources() {
        let mut row = task_doc(&[ActKind::Amend]);
        row.instructions = source.clone();
        let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(row.clone())));
        drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::Amend } });
        drive(&mut domain, Event::Act { action: Action::Edit { field: Purpose::Reason, text: text(b"") } });
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
        let tree = dialog(&domain, words(), &mut bindings);
        facts(&tree, source.as_ref());
        baseline(&tree, source.as_ref());
        assert!(domain.confirmation().unwrap().choices.change.is_none());
        drive(&mut domain, Event::Act { action: input(&tree, b"New budget", b"25", &mut bindings) });
        let mut expected = Amendment {
            llm: None,
            addressed: None,
            spec: None,
            instructions: None,
            wake: None,
            dependencies: None,
            authority: Some(row.authority.whole().unwrap()),
            tracked: None,
        };
        expected.authority.as_mut().unwrap().spend = 25;
        assert_eq!(domain.confirmation().unwrap().choices.change.as_ref(), Some(&expected));
        let tree = dialog(&domain, words(), &mut bindings);
        baseline(&tree, source.as_ref());
        drive(&mut domain, Event::Act { action: input(&tree, b"New instructions", b"", &mut bindings) });
        expected.instructions = Some(text(b""));
        let out = drive(&mut domain, Event::Act { action: Action::Confirm });
        assert_eq!(sent(&out), &expected);
        let projected = domain.objects().get(id).unwrap();
        let Body::Task(retained) = projected.body.as_ref().unwrap() else {
            panic!("task");
        };
        assert_eq!(retained.instructions, source);
    }
}

#[test]
fn steering_instruction_unknown_context_and_loss_preserve_explicit_clear_and_revision_selection() {
    for source in sources() {
        let mut row = escalation_doc(HoldReason::Budget { spent: 10, limit: 10 }, &[ActKind::Amend]);
        row.steering.instructions = source.clone();
        let (mut domain, id) = loaded(ObjectKey::Escalation(8), ReadValue::Escalation(Box::new(row.clone())));
        drive(&mut domain, Event::Act { action: Action::AmendRelease { object: id.token() } });
        drive(&mut domain, Event::Act { action: Action::Edit { field: Purpose::Reason, text: text(b"") } });
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
        let tree = dialog(&domain, words(), &mut bindings);
        facts(&tree, source.as_ref());
        baseline(&tree, source.as_ref());
        assert!(domain.confirmation().unwrap().choices.change.is_none());
        drive(&mut domain, Event::Act { action: input(&tree, b"New budget", b"25", &mut bindings) });
        let mut authority = row.steering.authority.whole().unwrap();
        authority.spend = 25;
        let mut expected = Amendment {
            llm: None,
            addressed: None,
            spec: None,
            instructions: None,
            wake: None,
            dependencies: None,
            authority: Some(authority),
            tracked: None,
        };
        assert_eq!(domain.confirmation().unwrap().choices.change.as_ref(), Some(&expected));
        let tree = dialog(&domain, words(), &mut bindings);
        drive(&mut domain, Event::Act { action: input(&tree, b"New instructions", b"", &mut bindings) });
        expected.instructions = Some(text(b""));
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
        let Ask::Compound { operations, .. } = ask else {
            panic!("compound amendment");
        };
        let [Operation::Amend { revision, .. }, Operation::Release { revision: release, .. }] = operations.as_slice()
        else {
            panic!("amend and release");
        };
        assert_eq!(*revision, Some(Revision { value: 6 }));
        assert_eq!(Some(*release), Some(Revision { value: 4 }));
        let projected = domain.objects().get(id).unwrap();
        let Body::Escalation(retained) = projected.body.as_ref().unwrap() else {
            panic!("escalation");
        };
        assert_eq!(retained.steering.person_revision, Some(Revision { value: 12 }));
        assert_eq!(retained.steering.instructions, source);
    }
}
