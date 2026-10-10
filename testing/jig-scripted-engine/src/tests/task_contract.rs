use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

#[test]
fn task_contract_payloads_price_reserved_choices_fields_and_whole_opaque_approval_bytes() {
    let mut fields = List::with_capacity(2);
    fields.push(FieldRule { name: vec![0xff; 32].into_boxed_slice(), max: u32::MAX }).unwrap();
    let mut choices = List::with_capacity(2);
    choices.push(VerdictChoice { code: u32::MAX, words: u32::MAX, followups: u32::MAX, fields }).unwrap();
    let mut path = List::with_capacity(2);
    path.push(vec![0xff; 32].into_boxed_slice()).unwrap();
    for contract in [
        TaskContractDocument::Report { words: u32::MAX },
        TaskContractDocument::Approval {
            state: vec![0xff; 32].into_boxed_slice(),
            part: OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Resource, key: OwnerPartKey::Path(path) },
        },
        TaskContractDocument::Verdict { choices, cut: true },
        TaskContractDocument::Change { connector: u16::MAX, kind: u16::MAX, words: u32::MAX },
    ] {
        let mut row = task();
        row.result_contract = contract;
        row.goal = Some(u64::MAX);
        let mut bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
        let before = row.clone();
        let whole = u64::try_from(size_of::<Task>()).unwrap() + row.heap(&bounds).unwrap();
        bounds.document_bytes = whole;
        assert!(payload::fits(&row, &bounds));
        bounds.document_bytes = whole - 1;
        assert!(!payload::fits(&row, &bounds));
        assert_eq!(row, before);
    }
    let mut row = task();
    row.result_contract = TaskContractDocument::Verdict { choices: List::with_capacity(3), cut: false };
    let bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    assert!(!payload::fits(&row, &bounds), "reserved omitted choices still count");
    let mut fields = List::with_capacity(3);
    fields.push(FieldRule { name: Box::from(*b"field"), max: u32::MAX }).unwrap();
    row.result_contract = TaskContractDocument::Verdict {
        choices: list(vec![VerdictChoice { code: 0, words: 0, followups: 0, fields }]),
        cut: false,
    };
    assert!(!payload::fits(&row, &bounds), "reserved nested field rules still count");
    row.result_contract = TaskContractDocument::Approval {
        state: vec![0; 33].into_boxed_slice(),
        part: OwnerPart { connector: 0, kind: OwnerPartKind::Result, key: OwnerPartKey::Number(0) },
    };
    assert!(!payload::fits(&row, &bounds), "opaque state is admitted whole");
}

#[test]
fn task_windows_filter_the_supplied_served_goal_without_reconstructing_it_from_lineage() {
    for supplied in [None, Some(0), Some(8), Some(u64::MAX)] {
        for wanted in [None, Some(0), Some(8), Some(9), Some(u64::MAX)] {
            let mut setup = script();
            let mut row = task();
            row.goal = supplied;
            row.lineage = list(vec![9]);
            let mut ancestor = task();
            ancestor.chip.task = 9;
            ancestor.project = 2;
            ancestor.goal = Some(9);
            setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
            setup.truth.objects.insert(ObjectKey::Task(9), ReadValue::Task(Box::new(ancestor)));
            let mut engine = Engine::new(setup, 5).unwrap();
            engine
                .open(
                    Time::ZERO,
                    11,
                    Token::new(3),
                    Subject::Tasks {
                        project: 1,
                        filter: TaskFilter { phase: None, executor: None, goal: wanted, asked_by_me: false },
                        window: 2,
                    },
                )
                .unwrap();
            let up = engine.poll(Time::ZERO);
            let rows: Vec<_> = up
                .iter()
                .filter_map(|event| match event {
                    Up::Watch { event: WatchEvent::Delivered(Delivery::Tasks(value)), .. } => Some(&value.rows),
                    Up::Answer { .. }
                    | Up::Read { .. }
                    | Up::Watch { .. }
                    | Up::ApplicationAnswer { .. }
                    | Up::ApplicationRead { .. }
                    | Up::App { .. }
                    | Up::Ended { .. } => None,
                })
                .collect();
            assert_eq!(rows.len(), 1);
            if wanted.is_none() || wanted == supplied {
                assert_eq!(rows[0].len(), 1);
                assert_eq!(rows[0].get(0).unwrap().chip.task, 8);
                assert_eq!(rows[0].get(0).unwrap().goal, supplied);
            } else {
                assert!(rows[0].is_empty());
            }
        }
    }
}
