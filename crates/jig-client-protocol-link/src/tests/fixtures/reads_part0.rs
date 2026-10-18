use super::*;

pub fn read_wire_history_page() -> w::HistoryPage {
    jig_client_test_records::read_wire_history_page()
}

pub fn read_wire_inbox_page() -> w::InboxPage {
    jig_client_test_records::read_wire_inbox_page()
}

pub fn read_wire_spend_page() -> w::SpendPage {
    jig_client_test_records::read_wire_spend_page()
}

pub fn read_wire_note_page() -> w::NotePage {
    jig_client_test_records::read_wire_note_page()
}

pub fn read_wire_object_page() -> w::ObjectPage {
    jig_client_test_records::read_wire_object_page()
}

pub fn read_wire_owner_ended_page() -> w::OwnerEndedPage {
    jig_client_test_records::read_wire_owner_ended_page()
}

pub fn read_wire_owner_inbox_page() -> w::OwnerInboxPage {
    jig_client_test_records::read_wire_owner_inbox_page()
}

pub fn read_wire_owner_spend_page() -> w::OwnerSpendPage {
    jig_client_test_records::read_wire_owner_spend_page()
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
    jig_client_test_records::read_wire_proposal_key()
}

pub fn read_wire_proposer() -> w::Proposer {
    jig_client_test_records::read_wire_proposer()
}

pub fn read_wire_proposal_action() -> w::ProposalAction {
    w::ProposalAction::Batch(read_wire_proposal_action_batch())
}

pub fn read_wire_activity_page() -> w::ActivityPage {
    jig_client_test_records::read_wire_activity_page()
}
