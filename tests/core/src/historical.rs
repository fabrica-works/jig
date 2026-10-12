//! Committed proposal decisions loaded by their exact maker and number
//! (`domain/people.md`, section 5.1; `client/domain/contract.md`, section 4).

use crate::Store;
use jig_domain_core as core;
use jig_domain_people::Proposer;
use jig_test_domain as root;
use skein_lib::Token;

pub(crate) fn proposal(
    store: &Store,
    request: Token,
    person: u64,
    project: u32,
    proposer: Proposer,
    proposal: u64,
) -> root::Event {
    let key = root::Key::Core(core::Key::Core(core::CoreKey::ProposalDecision { proposer, proposal }));
    let row = match store.rows.get(&key) {
        Some(root::Record::Core(core::Record::Core(core::CoreRecord::ProposalDecision(row)))) => Some(*row),
        Some(_) | None => None,
    };
    root::Event::Core(core::Event::HistoricalProposal {
        request,
        person,
        project,
        proposer,
        proposal,
        row,
        busy: false,
    })
}

pub(crate) fn escalation(
    store: &Store,
    request: Token,
    person: u64,
    project: u32,
    task: u64,
    revision: u64,
) -> root::Event {
    let key = root::Key::Core(core::Key::Core(core::CoreKey::EscalationDecision { task, revision }));
    let row = match store.rows.get(&key) {
        Some(root::Record::Core(core::Record::Core(core::CoreRecord::EscalationDecision(row)))) => Some(row.clone()),
        Some(_) | None => None,
    };
    root::Event::Core(core::Event::HistoricalEscalation { request, person, project, task, revision, row, busy: false })
}
