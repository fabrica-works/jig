use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

#[test]
fn task_link_payloads_price_whole_chips_and_every_reserved_reference_list() {
    for requester in [
        OwnerRequester::Task(u64::MAX),
        OwnerRequester::Person(u64::MAX),
        OwnerRequester::Deployment { project: u32::MAX },
    ] {
        let mut row = task();
        row.requester = requester;
        row.cut = true;
        row.inputs = List::with_capacity(2);
        row.inputs.push(OwnerTaskLink::Live(row.chip.clone())).unwrap();
        row.dependencies = List::with_capacity(2);
        row.dependencies.push(OwnerTaskLink::Retained { chip: row.chip.clone(), at: None }).unwrap();
        row.references = List::with_capacity(2);
        row.references.push(OwnerTaskLink::Stored { number: u64::MAX, status: None }).unwrap();
        let mut bounds = Bounds { rows: 2, document_bytes: u64::MAX, ..Bounds::default() };
        let before = row.clone();
        let whole = u64::try_from(size_of::<Task>()).unwrap() + row.heap(&bounds).unwrap();
        bounds.document_bytes = whole;
        assert!(payload::fits(&row, &bounds));
        bounds.document_bytes = whole - 1;
        assert!(!payload::fits(&row, &bounds));
        bounds.document_bytes = whole;
        assert_eq!(row.requester.heap(&bounds), Some(0));
        assert_eq!(row.references.get(0).unwrap().heap(&bounds), Some(0));
        assert_eq!(row, before);
        for field in 0..3 {
            let mut oversized = row.clone();
            match field {
                0 => oversized.inputs = List::with_capacity(3),
                1 => oversized.dependencies = List::with_capacity(3),
                2 => oversized.references = List::with_capacity(3),
                _ => unreachable!(),
            }
            bounds.document_bytes = u64::MAX;
            assert!(!payload::fits(&oversized, &bounds), "allocated empty reference rows still count");
        }
        let mut nested = row.clone();
        let mut large_chip = nested.chip.clone();
        large_chip.title.bytes = Text::new(vec![b't'; 4097].into_boxed_slice(), 4097).unwrap();
        nested.inputs = list(vec![OwnerTaskLink::Live(large_chip)]);
        assert!(!payload::fits(&nested, &bounds));
    }
}

#[test]
fn task_list_filters_do_not_alias_person_task_and_deployment_requesters_with_equal_numbers() {
    for (requester, asked_by_person) in [
        (OwnerRequester::Person(1), true),
        (OwnerRequester::Task(1), false),
        (OwnerRequester::Deployment { project: 1 }, false),
    ] {
        for asked_by_me in [false, true] {
            let mut setup = script();
            let mut row = task();
            row.requester = requester;
            setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
            let mut engine = Engine::new(setup, 5).unwrap();
            engine
                .open(
                    Time::ZERO,
                    11,
                    Token::new(3),
                    Subject::Tasks {
                        project: 1,
                        filter: TaskFilter { phase: None, executor: None, goal: None, asked_by_me },
                        window: 2,
                    },
                )
                .unwrap();
            let up = engine.poll(Time::ZERO);
            let deliveries: Vec<_> = up
                .iter()
                .filter_map(|event| match event {
                    Up::Watch { event: WatchEvent::Delivered(Delivery::Tasks(rows)), .. } => Some(rows),
                    _ => None,
                })
                .collect();
            assert_eq!(deliveries.len(), 1);
            let rows = &deliveries[0].rows;
            if asked_by_me && !asked_by_person {
                assert!(rows.is_empty());
            } else {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows.get(0).unwrap().requester, requester);
                assert_eq!(rows.get(0).unwrap().asked_by_me, asked_by_person);
            }
        }
    }
}
