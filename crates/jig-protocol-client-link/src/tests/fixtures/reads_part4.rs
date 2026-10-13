use super::*;
use skein_lib::List;
pub fn read_wire_owner_historical() -> w::OwnerHistorical {
    jig_test_client_records::read_wire_owner_historical()
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
