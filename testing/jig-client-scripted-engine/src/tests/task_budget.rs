use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

fn whole_budget_fits<T: Payload + Clone + PartialEq + core::fmt::Debug>(row: &T) {
    let before = row.clone();
    let mut bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    let whole = u64::try_from(size_of::<T>()).unwrap() + row.heap(&bounds).unwrap();
    bounds.document_bytes = whole;
    assert!(payload::fits(row, &bounds));
    bounds.document_bytes = whole - 1;
    assert!(!payload::fits(row, &bounds));
    assert_eq!(row, &before);
}

#[test]
fn current_budget_parent_payloads_price_task_and_line_facts_whole() {
    for funder in [
        Funder::Task(u64::MAX),
        Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX },
        Funder::Recurring { project: u32::MAX, task: u64::MAX, period: u64::MAX },
        Funder::Period { project: u32::MAX, period: u64::MAX },
    ] {
        for deadline in [None, Some(Wall::EPOCH), Some(Wall::from_nanos(u64::MAX))] {
            let budget = OwnerBudget {
                numbers: OwnerNumbers { budget: u64::MAX, spent: u64::MAX, spent_below: 7, reserved: 11 },
                left: 17,
                funder,
                deadline,
            };
            let mut row = task();
            row.budget = budget.clone();
            whole_budget_fits(&row);
            let conversation = TaskLine {
                chip: {
                    let mut chip = row.chip.clone();
                    chip.title = DocumentText::complete(text(b"first"));
                    chip
                },
                project: row.project,
                requester: jig_client_domain::contract::OwnerRequester::Person(1),
                tracked: None,
                label: 1,
                flags: jig_client_domain::contract::TaskFlags {
                    conversation: true,
                    listed: false,
                    retention: jig_client_domain::contract::Retention::ForgottenOnceRead,
                },
                changed: Wall::EPOCH,
                last_activity: Wall::EPOCH,
                ended_at: None,
                budget: budget.clone(),
                waits_on: jig_client_domain::contract::WaitsOn::Nothing,
                phases: jig_client_domain::contract::OwnerPhaseCounts {
                    waiting: 0,
                    idle: 0,
                    starting: 0,
                    running: 0,
                    backing_off: 0,
                    closing: 0,
                    held: 0,
                    done: 0,
                    failed: 0,
                    cancelled: 0,
                },
                held_below: 0,
                conversation: Some(jig_client_domain::contract::ConversationRead {
                    seen: 0,
                    unread: Count { value: 0, limited: false },
                }),
            };
            whole_budget_fits(&conversation);
        }
    }
}
