use super::*;
use jig_client::address::{Address, TaskSection};

#[test]
fn task_context_view_preserves_supplied_runtime_spend_slots_and_impact_without_claiming_missing_facts() {
    let reference = Part::Panel { connector: 2, task: u64::MAX };
    for present in [false, true] {
        let mut row = task_doc(&[ActKind::Cancel]);
        row.chip.executor = Executor::Agent { charter: 1 };
        row.chip.title.cut = true;
        row.delegates = List::with_capacity(0);
        let context = row.context.as_mut().unwrap();
        context.runtime =
            Runtime::Running { attempt: u64::MAX, since: Wall::EPOCH, tool: Some(text(b"supplied work")) };
        context.parts = list([reference.clone()]);
        context.spend = list([SpendLine { task: u64::MAX, attempt: Some(u64::MAX), spent: u64::MAX }]);
        context.writer_slots =
            list([WriterSlot { resource: reference.clone(), task: u64::MAX, attempt: Some(u64::MAX) }]);
        context.impact.tasks.running = u32::MAX;
        if !present {
            row.context = None;
        }
        for section in [TaskSection::Overview, TaskSection::Activity, TaskSection::Budget, TaskSection::Panels] {
            let (mut domain, out) = super::super::work_pages::entered(Address::Task { task: 8, section });
            let watch = out
                .iter()
                .find_map(|request| match request {
                    Request::Open { watch, subject: Subject::Task { .. } } => Some(*watch),
                    _ => None,
                })
                .unwrap();
            super::super::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(row.clone())));
            let (tree, bindings) = super::super::pages::build(&domain, 3, &words());
            super::super::cards_access::check(&tree);
            assert!(tree.len() <= crate::task_nodes(&CLIENT, 0).unwrap());
            if section == TaskSection::Activity {
                assert_eq!(contains(&tree, b"supplied work"), present);
            }
            if section == TaskSection::Budget {
                let caption = words().say(
                    JigPhrase::SpentRun,
                    &[Hole::Number(u64::MAX), Hole::Number(u64::MAX), Hole::Amount(u64::MAX)],
                );
                assert_eq!(contains(&tree, &caption), present);
                assert!(contains(&tree, b"Budget: 20"));
            }
            if section == TaskSection::Overview || section == TaskSection::Panels {
                let parts: Vec<_> =
                    tree.nodes().iter().filter_map(|node| node.slot).map(|slot| bindings.part(slot.token)).collect();
                assert_eq!(parts, if present { vec![Some(reference.clone())] } else { vec![] });
            }
        }
        let (mut domain, id, _) = loaded_with_watch(ObjectKey::Task(8), ReadValue::Task(Box::new(row.clone())));
        drive(&mut domain, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::Cancel } });
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
        let tree = dialog(&domain, &words(), &mut bindings);
        let caption = words().say(JigPhrase::TasksRunning, &[Hole::Number(u64::from(u32::MAX))]);
        assert_eq!(contains(&tree, &caption), present);
        let Some(jig_client::objects::Body::Task(retained)) = &domain.objects().get(id).unwrap().body else {
            panic!("whole task");
        };
        assert_eq!(retained.context, row.context);
        assert_eq!(retained.chip.title.cut, true);
    }
}
