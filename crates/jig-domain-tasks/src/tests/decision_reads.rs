//! A decision can commit before the turn carrying its entry's first read fence
//! (domain/engine.md, 7.2; domain/hosts.md, section 9).
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "unit-test parent scripts are ordinary Rust under programming-model.md section 10.2"
)]
use super::revisions::{Harness, task, to};
use crate::{
    Accepted, EscalationDecision, EscalationHolder, EscalationOutcome, Event, Funder, Hold, Party, Proposal,
    ProposalAction, ProposalDecision, ProposalHolder, ProposalOutcome, ProposalState, Refusal, Request,
};

const HOLDER: u64 = 1;
const CHILD: u64 = 2;
const REQUESTER: u64 = 9;
const ENTRY: u64 = 100;
const NEXT_ENTRY: u64 = 101;
const ATTEMPT: u64 = 1;
const FIRST_TURN: u32 = 1;
const READING_TURN: u32 = 2;
// The holder's fixture budget is 100; leave room for its run and its delegate.
const CHILD_BUDGET: u64 = 20;
const RUN_BUDGET: u64 = 10;
const TURN_SPEND: u64 = 1;

fn running_holder() -> Harness {
    let mut h = Harness::new();
    h.make([task(HOLDER, &[])].into());
    let mut child = task(CHILD, &[]);
    child.funder = Funder::Task(HOLDER);
    child.numbers.budget = CHILD_BUDGET;
    child.authority.budget.spend = CHILD_BUDGET;
    h.send(Event::Make { reply_to: to(), creator: Party::Task(HOLDER), batch: Box::new([child]) });
    h.send(Event::Prepare { reply_to: to(), task: HOLDER });
    h.send(Event::Claim {
        account: None,
        reply_to: to(),
        task: HOLDER,
        attempt: ATTEMPT,
        resumed: false,
        budget: RUN_BUDGET,
        writes: Box::new([]),
    });
    h.send(Event::Started { task: HOLDER, attempt: ATTEMPT });
    let _first_turn = turn(&mut h, FIRST_TURN, None, 0);
    h
}

fn turn(h: &mut Harness, number: u32, read: Option<u64>, cumulative: u64) -> Result<Accepted, Refusal> {
    let out = h.send(Event::Turn {
        reply_to: to(),
        task: HOLDER,
        attempt: ATTEMPT,
        turn: number,
        read,
        refused: Box::new([]),
        offered: read,
        cumulative,
    });
    out.iter()
        .find_map(|request| match request {
            Request::TurnAcknowledged { accepted, .. } => Some(Ok(*accepted)),
            Request::Refused { problem, .. } => Some(Err(problem.why)),
            _ => None,
        })
        .expect("one turn admission terminal")
}

#[test]
// I3 6.7: deciding drops the virtual entry before its first read fence commits.
#[ignore = "I3 6.7: decided escalation entry invalidates its holder's read fence"]
fn decided_escalation_entries_remain_readable_by_their_holder() {
    let mut results = Vec::new();
    let mut expected = Vec::new();
    for (name, entry, decision, outcome) in [
        (
            "pass",
            Some(NEXT_ENTRY),
            EscalationDecision::Pass { holder: EscalationHolder::Person(REQUESTER) },
            EscalationOutcome::Passed { revision: 1 },
        ),
        ("release", None, EscalationDecision::Release, EscalationOutcome::Released),
        (
            "reject",
            None,
            EscalationDecision::Reject { reason: Box::from(b"no".as_slice()) },
            EscalationOutcome::Rejected,
        ),
    ] {
        let mut h = running_holder();
        h.send(Event::Hold { task: CHILD, why: Hold::Stopped });
        h.route(CHILD, EscalationHolder::Task(HOLDER), ENTRY);
        let offered = crate::escalation::waiting_for(&h.domain, HOLDER);
        let read = offered.iter().find(|word| word.number == ENTRY).expect("holder offered escalation").number;
        let revision = h.domain.task(CHILD).expect("held child").revisions.hold;
        // The model reads the entry and calls decide during this turn. Tool
        // results commit before the completed turn carries its read fence.
        let out =
            h.send(Event::DecideEscalation { reply_to: to(), task: CHILD, revision, by: HOLDER, entry, decision });
        let decided = out.iter().find_map(|request| match request {
            Request::EscalationDecided { outcome, .. } => Some(*outcome),
            _ => None,
        });
        let admitted = turn(&mut h, READING_TURN, Some(read), TURN_SPEND);
        let holder = h.domain.task(HOLDER).expect("holder live");
        results.push((name, decided, admitted, holder.turn, holder.run_spent));
        expected.push((name, Some(outcome), Ok(Accepted::New), READING_TURN, TURN_SPEND));
    }
    assert_eq!(results, expected);
}

#[test]
// I3 6.7: the same lost virtual read fence affects proposal decisions.
#[ignore = "I3 6.7: decided proposal entry invalidates its holder's read fence"]
fn decided_proposal_entries_remain_readable_by_their_holder() {
    let mut results = Vec::new();
    let mut expected = Vec::new();
    for (name, message, decision, outcome) in [
        ("accept", Some(NEXT_ENTRY), ProposalDecision::Accept, ProposalOutcome::Accepted),
        (
            "reject",
            Some(NEXT_ENTRY),
            ProposalDecision::Reject { reason: Box::from(b"no".as_slice()) },
            ProposalOutcome::Rejected,
        ),
        ("pass", None, ProposalDecision::Pass { holder: ProposalHolder::Person(REQUESTER) }, ProposalOutcome::Passed),
    ] {
        let mut h = running_holder();
        h.send(Event::Hold { task: CHILD, why: Hold::Stopped });
        h.send(Event::Propose {
            reply_to: to(),
            proposal: Proposal {
                number: ENTRY,
                proposer: CHILD,
                project: 1,
                action: ProposalAction::Release { task: CHILD },
                reason: Box::from(b"please release".as_slice()),
                as_holder: false,
                state: ProposalState::Pending { holder: ProposalHolder::Task(HOLDER), since: h.env.wall },
            },
        });
        let offered = crate::proposals::waiting_for(&h.domain, HOLDER);
        let read = offered.iter().find(|word| word.number == ENTRY).expect("holder offered proposal").number;
        let out = h.send(Event::DecideProposal {
            reply_to: to(),
            proposer: CHILD,
            proposal: ENTRY,
            message,
            by: Party::Task(HOLDER),
            decision,
        });
        let decided = out.iter().find_map(|request| match request {
            Request::ProposalDecided { outcome, .. } => Some(*outcome),
            _ => None,
        });
        let admitted = turn(&mut h, READING_TURN, Some(read), TURN_SPEND);
        let holder = h.domain.task(HOLDER).expect("holder live");
        results.push((name, decided, admitted, holder.turn, holder.run_spent));
        expected.push((name, Some(outcome), Ok(Accepted::New), READING_TURN, TURN_SPEND));
    }
    assert_eq!(results, expected);
}
