use super::*;
use skein_lib::List;
pub fn read_wire_owner_historical() -> w::OwnerHistorical {
    let lim = w::CEILINGS;
    w::OwnerHistorical::new(
        &lim,
        w::OwnerHistoricalParts {
            key: read_wire_object_key(),
            value: read_wire_owner_historical_value(),
            at: None,
            by: None,
        },
    )
    .unwrap()
}

pub fn read_wire_object_key() -> w::ObjectKey {
    w::ObjectKey::Task(read_wire_object_key_task())
}

pub fn read_wire_object_key_task() -> w::ObjectKeyTask {
    let lim = w::CEILINGS;
    w::ObjectKeyTask::new(&lim, w::ObjectKeyTaskParts { value: 7 }).unwrap()
}

pub fn read_wire_owner_historical_value() -> w::OwnerHistoricalValue {
    w::OwnerHistoricalValue::Task(read_wire_owner_historical_value_task())
}

pub fn read_wire_owner_historical_value_task() -> w::OwnerHistoricalValueTask {
    let lim = w::CEILINGS;
    w::OwnerHistoricalValueTask::new(
        &lim,
        w::OwnerHistoricalValueTaskParts { document: task_record(), result: read_wire_result() },
    )
    .unwrap()
}

pub fn read_value_history_page() -> c::HistoryPage {
    c::HistoryPage { cut: false, rows: List::with_capacity(0), before: None, after: None }
}

pub fn read_value_inbox_page() -> c::InboxPage {
    c::InboxPage { cut: false, rows: List::with_capacity(0), before: None, after: None }
}

pub fn read_value_conversation_page() -> c::ConversationPage {
    c::ConversationPage { cut: false, rows: List::with_capacity(0), before: None, after: None }
}

pub fn read_value_goal_page() -> c::GoalPage {
    c::GoalPage { cut: false, rows: List::with_capacity(0), before: None, after: None }
}

pub fn read_value_spend_page() -> c::SpendPage {
    c::SpendPage { cut: false, rows: List::with_capacity(0), before: None, after: None }
}

pub fn read_value_note_page() -> c::NotePage {
    c::NotePage { cut: false, rows: List::with_capacity(0), before: None, after: None }
}

pub fn read_value_object_page() -> c::ObjectPage {
    c::ObjectPage { cut: false, rows: List::with_capacity(0), before: None, after: None }
}

pub fn read_value_owner_ended_page() -> c::OwnerEndedPage {
    c::OwnerEndedPage { rows: List::with_capacity(0), before: None, after: None, cut: false }
}

pub fn read_value_owner_inbox_page() -> c::OwnerInboxPage {
    c::OwnerInboxPage { selection: None, cut: false, rows: List::with_capacity(0), before: None, after: None }
}

pub fn read_value_owner_spend_page() -> c::OwnerSpendPage {
    c::OwnerSpendPage { rows: List::with_capacity(0), before: None, after: None, cut: false }
}

pub fn read_value_activity_page() -> c::ActivityPage {
    c::ActivityPage { cut: false, rows: List::with_capacity(0), before: None, after: None }
}
