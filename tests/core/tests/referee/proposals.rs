use super::*;

const PROPOSAL: u64 = 40;
const NEXT_PROPOSAL: u64 = 41;

fn authority_catches_a_proposal_its_proposer_did_not_ask_for() {
    let mut fake = Fake::ready(Recovery::Keyed);
    let mut rows = snapshot();
    rows.party_proposals.insert(PROPOSAL, PROPOSER);
    fake.caught(Observed::Durable { number: 2, snapshot: rows }, Promise::Authority);
}

fn authority_catches_a_proposal_after_its_ask_was_refused() {
    let mut fake = Fake::ready(Recovery::Keyed);
    fake.send(Observed::ProposeAsked { proposer: Source::Person(PROPOSER) }).expect("explicit ask");
    fake.send(Observed::ProposeRefused { proposer: Source::Person(PROPOSER) }).expect("ask refused");
    let mut rows = snapshot();
    rows.party_proposals.insert(PROPOSAL, PROPOSER);
    fake.caught(Observed::Durable { number: 2, snapshot: rows }, Promise::Authority);
}

fn a_proposal_asked_by_its_party_is_valid() {
    let mut fake = Fake::ready(Recovery::Keyed);
    fake.send(Observed::ProposeAsked { proposer: Source::Person(PROPOSER) }).expect("explicit ask");
    let mut rows = snapshot();
    rows.party_proposals.insert(PROPOSAL, PROPOSER);
    fake.commit(2, rows.clone()).expect("party's proposal");
    fake.send(Observed::Restart).expect("restart retains evidence");
    fake.commit(3, rows.clone()).expect("a proposal is charged once");
    rows.party_proposals.insert(NEXT_PROPOSAL, PROPOSER);
    fake.caught(Observed::Durable { number: 4, snapshot: rows }, Promise::Authority);
}

fn a_proposal_asked_by_its_run_is_valid() {
    let mut fake = Fake::ready(Recovery::Keyed);
    fake.send(Observed::ProposeAsked { proposer: Source::Task(PROPOSER) }).expect("run's ask");
    fake.send(Observed::Restart).expect("ask survives restart");
    let mut rows = snapshot();
    rows.tasks.get_mut(&PROPOSER).expect("task").proposal = Some((PROPOSAL, Source::Person(PROPOSER)));
    fake.commit(2, rows).expect("run's proposal");
}

fn a_procedures_proposal_needs_no_observed_ask() {
    let mut fake = Fake::ready(Recovery::Keyed);
    let mut rows = snapshot();
    let task = rows.tasks.get_mut(&PROPOSER).expect("procedure");
    task.executor = (1, 1);
    task.proposal = Some((PROPOSAL, Source::Person(PROPOSER)));
    fake.commit(2, rows).expect("a procedure proposes in its step");
}

pub(super) fn proposals_referee_cases() {
    authority_catches_a_proposal_its_proposer_did_not_ask_for();
    authority_catches_a_proposal_after_its_ask_was_refused();
    a_proposal_asked_by_its_party_is_valid();
    a_proposal_asked_by_its_run_is_valid();
    a_procedures_proposal_needs_no_observed_ask();
}
