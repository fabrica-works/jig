//! Whole owner authority remains independent of selected changes and document loss.
use super::*;
use jig_client::address::{Address, TaskSection};

fn supplied() -> OwnerAuthorityDocument {
    let mut segments = List::with_capacity(CLIENT.rows);
    segments.push(Box::from(*b"a/b")).unwrap();
    segments.push(Box::from(*b"\xff\0")).unwrap();
    let pattern = OwnerPattern { segments, last: OwnerPatternEnd::Open(Box::from(*b"\xff/")) };
    let mut grants = List::with_capacity(CLIENT.rows);
    grants.push(OwnerGrant { connector: u16::MAX, kind: u16::MAX, pattern: pattern.clone() }).unwrap();
    let mut kinds = List::with_capacity(CLIENT.rows);
    kinds.push(OwnerAuthorityExecutor::Charter(u32::MAX)).unwrap();
    kinds.push(OwnerAuthorityExecutor::Role(u32::MAX)).unwrap();
    let mut note_resources = List::with_capacity(CLIENT.rows);
    note_resources.push(OwnerResourceScope { connector: u16::MAX, pattern }).unwrap();
    OwnerAuthorityDocument {
        authority: OwnerAuthority {
            tools: u64::MAX,
            grants,
            delegation: OwnerDelegation { kinds, tasks: u32::MAX, depth: u32::MAX },
            spend: 10,
            deadline: None,
            notes: u8::MAX,
            note_resources,
        },
        cut: false,
    }
}

#[test]
fn task_authority_forms_preserve_all_permissions_and_selected_delta_without_reconstructing_loss() {
    for cut in [false, true] {
        let mut row = task_doc(&[ActKind::Amend]);
        row.authority = supplied();
        row.authority.cut = cut;
        let before = row.clone();
        let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
        drive(&mut domain, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::Amend } });
        let words = words();
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
        let tree = dialog(&domain, &words, &mut bindings);
        assert!(contains(&tree, b"Tool bits: 18446744073709551615"));
        assert!(contains(&tree, b"Note scope bits: 255"));
        assert!(contains(&tree, b"Role: 4294967295"));
        assert!(contains(&tree, b"a/b"));
        assert!(contains(&tree, b"\\xFF\\x00"));
        assert_eq!(contains(&tree, b"Authority document is cut"), cut);
        assert_eq!(tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"New budget")), !cut);
        assert_eq!(
            tree.nodes().iter().any(|node| node.name.as_deref() == Some(b"New deadline (UTC, YYYY-MM-DD HH:MM)")),
            !cut
        );
        let action = input(&tree, b"New instructions", b"", &mut bindings);
        drive(&mut domain, Event::Act { act: action });
        let tree = dialog(&domain, &words, &mut bindings);
        if !cut {
            let action = input(&tree, b"New budget", b"25", &mut bindings);
            drive(&mut domain, Event::Act { act: action });
        }
        let selected = domain.confirmation().unwrap().choices.change.as_ref().unwrap();
        assert_eq!(selected.instructions, Some(text(b"")));
        assert!(selected.spec.is_none());
        assert!(selected.wake.is_none());
        if cut {
            assert!(selected.authority.is_none());
        } else {
            let mut expected = before.authority.authority.clone();
            expected.spend = 25;
            assert_eq!(selected.authority, Some(expected));
            assert_eq!(selected.authority.as_ref().unwrap().grants.capacity(), CLIENT.rows);
            assert_eq!(
                selected.authority.as_ref().unwrap().grants.get(0).unwrap().pattern.segments.capacity(),
                CLIENT.rows
            );
        }
        let expected = selected.clone();
        let out = drive(&mut domain, Event::Act { act: Action::Confirm });
        let sent: Vec<_> = out
            .iter()
            .filter_map(|request| match request {
                Request::Send { ask, .. } => match &**ask {
                    Ask::Amend { change, .. } => Some(change),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        assert_eq!(sent, vec![&expected]);
        let Some(jig_client::objects::Body::Task(actual)) = &domain.objects().get(id).unwrap().body else {
            panic!("whole source remains");
        };
        assert_eq!(**actual, before);
    }
}

#[test]
fn task_budget_section_renders_exact_bitmap_delegation_and_literal_authority_loss_within_bounds() {
    for cut in [false, true] {
        for kind in [
            OwnerAuthorityExecutor::Charter(u32::MAX),
            OwnerAuthorityExecutor::Procedure(u32::MAX),
            OwnerAuthorityExecutor::Role(u32::MAX),
        ] {
            let (mut domain, out) =
                super::super::work_pages::entered(Address::Task { task: 8, section: TaskSection::Budget });
            let watch = out
                .iter()
                .find_map(|request| match request {
                    Request::Open { watch, subject: Subject::Task { .. } } => Some(*watch),
                    _ => None,
                })
                .unwrap();
            let mut row = task_doc(&[]);
            row.chip.executor = Executor::Agent { charter: 1 };
            row.authority = supplied();
            row.authority.cut = cut;
            *row.authority.authority.delegation.kinds.get_mut(0).unwrap() = kind;
            super::super::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(row.clone())));
            let (tree, _) = super::super::pages::build(&domain, 3, &words());
            super::super::cards_access::check(&tree);
            assert!(contains(&tree, b"Tool bits: 18446744073709551615"));
            assert!(contains(&tree, b"Note scope bits: 255"));
            assert!(contains(&tree, b"Delegate depth: 4294967295"));
            assert!(contains(&tree, b"Kind: 65535"));
            assert!(contains(&tree, b"a/b"));
            assert!(contains(&tree, b"\\xFF\\x00"));
            assert!(contains(&tree, b"Open ending"));
            assert_eq!(contains(&tree, b"Authority document is cut"), cut);
            assert!(tree.len() <= crate::task_nodes(&CLIENT, 0).unwrap());
            let id = domain.task_page().header.unwrap();
            let Some(jig_client::objects::Body::Task(actual)) = &domain.objects().get(id).unwrap().body else {
                panic!("whole source remains");
            };
            assert_eq!(**actual, row);
        }
    }
}
