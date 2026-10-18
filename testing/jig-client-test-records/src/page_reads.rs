//! Existing client page wire literals, shared without changing their values.
use super::w;
use skein_lib::List;

#[must_use]
pub fn read_wire_history_page() -> w::HistoryPage {
    let lim = w::CEILINGS;
    w::HistoryPage::new(&lim, w::HistoryPageParts { cut: false, rows: Box::from([]), before: None, after: None })
        .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_inbox_page() -> w::InboxPage {
    let lim = w::CEILINGS;
    w::InboxPage::new(&lim, w::InboxPageParts { cut: false, rows: Box::from([]), before: None, after: None })
        .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_spend_page() -> w::SpendPage {
    let lim = w::CEILINGS;
    w::SpendPage::new(&lim, w::SpendPageParts { cut: false, rows: Box::from([]), before: None, after: None })
        .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_note_page() -> w::NotePage {
    let lim = w::CEILINGS;
    w::NotePage::new(&lim, w::NotePageParts { cut: false, rows: Box::from([]), before: None, after: None })
        .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_object_page() -> w::ObjectPage {
    let lim = w::CEILINGS;
    w::ObjectPage::new(&lim, w::ObjectPageParts { cut: false, rows: Box::from([]), before: None, after: None })
        .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_owner_ended_page() -> w::OwnerEndedPage {
    let lim = w::CEILINGS;
    w::OwnerEndedPage::new(&lim, w::OwnerEndedPageParts { rows: Box::from([]), before: None, after: None, cut: false })
        .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_owner_inbox_page() -> w::OwnerInboxPage {
    let lim = w::CEILINGS;
    w::OwnerInboxPage::new(
        &lim,
        w::OwnerInboxPageParts { selection: None, cut: false, rows: Box::from([]), before: None, after: None },
    )
    .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_owner_spend_page() -> w::OwnerSpendPage {
    let lim = w::CEILINGS;
    w::OwnerSpendPage::new(
        &lim,
        w::OwnerSpendPageParts { rows: List::with_capacity(0), before: None, after: None, cut: false },
    )
    .expect("literal fixture fits")
}

#[must_use]
pub fn read_wire_activity_page() -> w::ActivityPage {
    let lim = w::CEILINGS;
    w::ActivityPage::new(&lim, w::ActivityPageParts { cut: false, rows: Box::from([]), before: None, after: None })
        .expect("literal fixture fits")
}
