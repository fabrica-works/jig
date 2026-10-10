use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

fn whole_budget_fits<T: Payload + Clone + PartialEq + core::fmt::Debug>(row: T) {
    let before = row.clone();
    let mut bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    let whole = u64::try_from(size_of::<T>()).unwrap() + row.heap(&bounds).unwrap();
    bounds.document_bytes = whole;
    assert!(payload::fits(&row, &bounds));
    bounds.document_bytes = whole - 1;
    assert!(!payload::fits(&row, &bounds));
    assert_eq!(row, before);
}

#[test]
fn current_budget_parent_payloads_price_whole_task_conversation_and_goal_facts_without_recomputing_them() {
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
            whole_budget_fits(row.clone());
            let conversation = ConversationLine {
                chip: row.chip.clone(),
                project: row.project,
                first_words: text(b"first"),
                unread: Count { value: 0, limited: false },
                seen: 0,
                last_reply: 0,
                changed: Wall::EPOCH,
                runtime: row.runtime.clone(),
                delegates: 0,
                budget: budget.clone(),
            };
            whole_budget_fits(conversation);
            let goal = GoalLine {
                chip: row.chip,
                priority: u32::MAX,
                progress: row.impact.tasks,
                budget,
                held_below: u32::MAX,
                changed: Wall::EPOCH,
            };
            whole_budget_fits(goal);
        }
    }
}
