use super::*;
use jig_client::address::{Address, TaskSection};

#[test]
fn task_pages_keep_owner_requesters_and_live_retained_stored_links_without_filling_missing_facts() {
    for (requester, caption) in [
        (OwnerRequester::Task(u64::MAX), b"Requested by task 18446744073709551615".as_slice()),
        (OwnerRequester::Person(u64::MAX), b"Requested by person 18446744073709551615".as_slice()),
        (OwnerRequester::Deployment { project: u32::MAX }, b"Requested by deployment in project 4294967295".as_slice()),
    ] {
        let mut supplied_chip = chip();
        supplied_chip.task = u64::MAX;
        supplied_chip.title = DocumentText::complete(text(b"Only supplied linked title"));
        for link in [
            OwnerTaskLink::Live(supplied_chip.clone()),
            OwnerTaskLink::Retained { chip: supplied_chip.clone(), at: None },
            OwnerTaskLink::Retained { chip: supplied_chip.clone(), at: Some(Wall::from_nanos(120_000_000_000)) },
            OwnerTaskLink::Stored { number: u64::MAX, status: None },
            OwnerTaskLink::Stored { number: u64::MAX, status: Some(OwnerTaskStatus::Done) },
            OwnerTaskLink::Stored { number: u64::MAX, status: Some(OwnerTaskStatus::Failed) },
            OwnerTaskLink::Stored { number: u64::MAX, status: Some(OwnerTaskStatus::Cancelled) },
        ] {
            let (mut domain, out) = super::super::work_pages::entered(Address::task(8));
            let watch = out
                .iter()
                .find_map(|request| match request {
                    Request::Open { watch, subject: Subject::Task { .. } } => Some(*watch),
                    _ => None,
                })
                .unwrap();
            let mut row = task_doc(&[]);
            row.requester = requester;
            row.inputs = list([link.clone()]);
            row.dependencies = list([link.clone()]);
            row.references = list([link.clone()]);
            row.cut = true;
            super::super::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(row)));
            let words = words();
            let (tree, mut bindings) = super::super::pages::build(&domain, 3, &words);
            super::super::cards_access::check(&tree);
            assert!(contains(&tree, caption));
            assert!(contains(&tree, b"Some task lists are incomplete"));
            assert!(tree.len() <= crate::task_nodes(&CLIENT, 0).unwrap());
            let mut routes = 0_u32;
            for node in tree.nodes() {
                if let Some(binding) = node.binding
                    && let Some(Action::Go {
                        address: Address::Task { task: u64::MAX, section: TaskSection::Overview },
                    }) = decode(&mut bindings, binding.token, Act::Press)
                {
                    routes += 1;
                }
            }
            assert_eq!(routes, 3, "each supplied input, dependency and reference links by its number");
            match &link {
                OwnerTaskLink::Live(_) => assert!(contains(&tree, b"Live task")),
                OwnerTaskLink::Retained { at, .. } => {
                    assert!(contains(&tree, b"Retained ended task"));
                    assert_eq!(contains(&tree, b"Ended time unavailable"), at.is_none());
                    assert_eq!(contains(&tree, b"Ended at 01:02"), at.is_some());
                }
                OwnerTaskLink::Stored { status, .. } => {
                    assert!(contains(&tree, b"Stored task 18446744073709551615"));
                    assert_eq!(contains(&tree, b"Stored status unavailable"), status.is_none());
                    assert!(!contains(&tree, b"Only supplied linked title"), "a stored reference supplies no title");
                }
            }
        }
    }
}

#[test]
fn aggregate_task_loss_prevents_partial_dependency_replacement_and_complete_links_keep_named_numbers() {
    for cut in [false, true] {
        let mut row = task_doc(&[ActKind::Amend]);
        row.cut = cut;
        row.dependencies = list([
            OwnerTaskLink::Stored { number: u64::MAX, status: None },
            OwnerTaskLink::Retained { chip: chip(), at: None },
        ]);
        let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
        drive(&mut domain, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::Amend } });
        let words = words();
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
        let tree = dialog(&domain, &words, &mut bindings);
        assert_eq!(contains(&tree, b"Some task lists are incomplete"), cut);
        let mut selected = Vec::new();
        for node in tree.nodes() {
            if let Some(binding) = node.binding
                && let Some(Action::Change { change }) = decode(&mut bindings, binding.token, Act::Press)
                && let Some(dependencies) = change.dependencies
            {
                selected.push(dependencies);
            }
        }
        if cut {
            assert!(selected.is_empty(), "aggregate loss cannot supply a whole dependency replacement");
            let changed = input(&tree, b"New instructions", b"", &mut bindings);
            drive(&mut domain, Event::Act { act: changed });
            assert_eq!(domain.confirmation().unwrap().choices.change.as_ref().unwrap().instructions, Some(text(b"")));
        } else {
            assert_eq!(selected, vec![list([8]), list([u64::MAX])]);
        }
    }
}
