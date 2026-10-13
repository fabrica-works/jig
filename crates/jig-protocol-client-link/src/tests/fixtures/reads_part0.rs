use super::*;
use skein_lib::List;

pub fn read_wire_history_page() -> w::HistoryPage {
    let lim = w::CEILINGS;
    w::HistoryPage::new(&lim, w::HistoryPageParts { cut: false, rows: Box::from([]), before: None, after: None })
        .unwrap()
}

pub fn read_wire_inbox_page() -> w::InboxPage {
    let lim = w::CEILINGS;
    w::InboxPage::new(&lim, w::InboxPageParts { cut: false, rows: Box::from([]), before: None, after: None }).unwrap()
}

pub fn read_wire_conversation_page() -> w::ConversationPage {
    let lim = w::CEILINGS;
    w::ConversationPage::new(
        &lim,
        w::ConversationPageParts { cut: false, rows: Box::from([]), before: None, after: None },
    )
    .unwrap()
}

pub fn read_wire_goal_page() -> w::GoalPage {
    let lim = w::CEILINGS;
    w::GoalPage::new(&lim, w::GoalPageParts { cut: false, rows: Box::from([]), before: None, after: None }).unwrap()
}

pub fn read_wire_spend_page() -> w::SpendPage {
    let lim = w::CEILINGS;
    w::SpendPage::new(&lim, w::SpendPageParts { cut: false, rows: Box::from([]), before: None, after: None }).unwrap()
}

pub fn read_wire_note_page() -> w::NotePage {
    let lim = w::CEILINGS;
    w::NotePage::new(&lim, w::NotePageParts { cut: false, rows: Box::from([]), before: None, after: None }).unwrap()
}

pub fn read_wire_object_page() -> w::ObjectPage {
    let lim = w::CEILINGS;
    w::ObjectPage::new(&lim, w::ObjectPageParts { cut: false, rows: Box::from([]), before: None, after: None }).unwrap()
}

pub fn read_wire_owner_ended_page() -> w::OwnerEndedPage {
    let lim = w::CEILINGS;
    w::OwnerEndedPage::new(&lim, w::OwnerEndedPageParts { rows: Box::from([]), before: None, after: None, cut: false })
        .unwrap()
}

pub fn read_wire_owner_inbox_page() -> w::OwnerInboxPage {
    let lim = w::CEILINGS;
    w::OwnerInboxPage::new(
        &lim,
        w::OwnerInboxPageParts { selection: None, cut: false, rows: Box::from([]), before: None, after: None },
    )
    .unwrap()
}

pub fn read_wire_owner_spend_page() -> w::OwnerSpendPage {
    let lim = w::CEILINGS;
    w::OwnerSpendPage::new(
        &lim,
        w::OwnerSpendPageParts { rows: List::with_capacity(0), before: None, after: None, cut: false },
    )
    .unwrap()
}

pub fn read_wire_proposal() -> w::Proposal {
    let lim = w::CEILINGS;
    w::Proposal::new(
        &lim,
        w::ProposalParts {
            key: read_wire_proposal_key(),
            project: 7,
            proposer: read_wire_proposer(),
            why: Box::from([]),
            action: read_wire_proposal_action(),
            needs: read_wire_authority(),
            funding: read_wire_budget(),
            waiting: read_wire_recipient(),
            trail: Box::from([]),
            since: 7,
            acts: Box::from([]),
            ending: None,
        },
    )
    .unwrap()
}

pub fn read_wire_proposal_key() -> w::ProposalKey {
    jig_test_client_records::read_wire_proposal_key()
}

pub fn read_wire_proposer() -> w::Proposer {
    jig_test_client_records::read_wire_proposer()
}

pub fn read_wire_proposal_action() -> w::ProposalAction {
    w::ProposalAction::Batch(read_wire_proposal_action_batch())
}

pub fn read_wire_activity_page() -> w::ActivityPage {
    let lim = w::CEILINGS;
    w::ActivityPage::new(&lim, w::ActivityPageParts { cut: false, rows: Box::from([]), before: None, after: None })
        .unwrap()
}
