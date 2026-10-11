//! Committed proposal decisions loaded by their exact maker and number
//! (`domain/people.md`, section 5.1; `client/domain/contract.md`, section 4).

use crate::Store;
use jig_core as core;
use jig_core_people::Proposer;
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
