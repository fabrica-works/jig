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
            whole_budget_fits(&row);
            let conversation = ConversationLine {
                cached_replies: 0,
                chip: row.chip.clone(),
                project: row.project,
                first_words: DocumentText::complete(text(b"first")),
                unread: Some(Count { value: 0, limited: false }),
                seen: 0,
                last_reply: 0,
                changed: Some(Wall::EPOCH),
                runtime: Some(row.context.as_ref().expect("supplied task context").runtime.clone()),
                delegates: 0,
                budget: budget.clone(),
            };
            whole_budget_fits(&conversation);
            let goal = GoalLine {
                chip: row.chip,
                priority: u32::MAX,
                progress: OwnerPhaseCounts {
                    waiting: row.context.as_ref().expect("supplied task context").impact.tasks.waiting,
                    running: row.context.as_ref().expect("supplied task context").impact.tasks.running,
                    held: row.context.as_ref().expect("supplied task context").impact.tasks.held,
                    closing: row.context.as_ref().expect("supplied task context").impact.tasks.closing,
                    done: row.context.as_ref().expect("supplied task context").impact.tasks.done,
                    failed: row.context.as_ref().expect("supplied task context").impact.tasks.failed,
                    cancelled: row.context.as_ref().expect("supplied task context").impact.tasks.cancelled,
                    idle: 0,
                    starting: 0,
                    backing_off: 0,
                },
                budget,
                held_below: u32::MAX,
                changed: Some(Wall::EPOCH),
            };
            whole_budget_fits(&goal);
        }
    }
}

#[test]
fn owner_row_parent_admission_prices_full_text_optional_runtime_and_allocated_window_without_trimming() {
    let mut nested = List::with_capacity(2);
    nested.push(u64::MAX).unwrap();
    let row = ConversationLine {
        chip: task().chip,
        project: u32::MAX,
        first_words: DocumentText { bytes: text(UTF8_TEXTS[0]), cut: true },
        unread: Some(Count { value: u32::MAX, limited: true }),
        cached_replies: u32::MAX,
        delegates: u32::MAX,
        seen: u64::MAX,
        last_reply: u64::MAX,
        runtime: Some(Runtime::Waiting { dependencies: nested, hold: None }),
        changed: Some(Wall::from_nanos(u64::MAX)),
        budget: task().budget,
    };
    whole_budget_fits(&row);
    let bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    let mut absent = row.clone();
    absent.runtime = None;
    absent.changed = None;
    absent.unread = None;
    assert_eq!(row.heap(&bounds).unwrap() - absent.heap(&bounds).unwrap(), List::<u64>::worst_case(2).unwrap());
    whole_budget_fits(&absent);
    for capacity in [1, 2] {
        let mut rows = List::with_capacity(capacity);
        rows.push(row.clone()).unwrap();
        let page = ReadValue::Conversations(Box::new(ConversationPage {
            rows,
            cut: true,
            before: None,
            after: Some(Cursor::new(Box::from(UTF8_TEXTS[1]), 1024).expect("bounded opaque cursor")),
        }));
        whole_budget_fits(&page);
        let mut too_short = bounds;
        too_short.text = 31;
        assert!(!payload::fits(&page, &too_short));
    }
}

#[test]
fn owner_goal_payload_admits_all_independent_full_width_stages_and_whole_budget_at_exact_bound() {
    let row = GoalLine {
        chip: task().chip,
        priority: u32::MAX,
        held_below: u32::MAX,
        changed: None,
        progress: OwnerPhaseCounts {
            waiting: u32::MAX,
            idle: u32::MAX,
            starting: u32::MAX,
            running: u32::MAX,
            backing_off: u32::MAX,
            closing: u32::MAX,
            held: u32::MAX,
            done: u32::MAX,
            failed: u32::MAX,
            cancelled: u32::MAX,
        },
        budget: OwnerBudget {
            numbers: OwnerNumbers { budget: u64::MAX, spent: u64::MAX, spent_below: u64::MAX, reserved: u64::MAX },
            left: u64::MAX,
            funder: Funder::Recurring { project: u32::MAX, task: u64::MAX, period: u64::MAX },
            deadline: Some(Wall::from_nanos(u64::MAX)),
        },
    };
    whole_budget_fits(&row);
    let page = ReadValue::Goals(Box::new(GoalPage {
        rows: list(vec![row]),
        cut: true,
        before: None,
        after: Some(Cursor::new(Box::from(UTF8_TEXTS[2]), 1024).expect("bounded opaque cursor")),
    }));
    whole_budget_fits(&page);
}

const UTF8_TEXTS: [&[u8]; 3] = [&[b'~'; 32], b"\x7e\0opaque", b"\x7e\0opaque-goal"];
