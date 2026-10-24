//! A decision can commit before the turn carrying its entry's first read fence
//! (domain/engine.md, 7.2; domain/hosts.md, section 9).
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "unit-test parent scripts are ordinary Rust under programming-model.md section 10.2"
)]
use super::revisions::{Harness, amendment, task, to};
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
    let _first_turn = turn(&mut h, FIRST_TURN, None, 0, &[]);
    h
}

fn turn(
    h: &mut Harness,
    number: u32,
    read: Option<u64>,
    cumulative: u64,
    offers: &[crate::Word],
) -> Result<Accepted, Refusal> {
    let out = h.send(Event::Turn {
        reply_to: to(),
        task: HOLDER,
        attempt: ATTEMPT,
        turn: number,
        read,
        refused: Box::new([]),
        offered: offers.last().map(|word| word.number),
        outstanding: offers.iter().map(|word| word.number).collect(),
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
        let admitted = turn(&mut h, READING_TURN, Some(read), TURN_SPEND, &offered);
        let holder = h.domain.task(HOLDER).expect("holder live");
        results.push((name, decided, admitted, holder.turn, holder.run_spent));
        expected.push((name, Some(outcome), Ok(Accepted::New), READING_TURN, TURN_SPEND));
    }
    assert_eq!(results, expected);
}

#[test]
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
        let admitted = turn(&mut h, READING_TURN, Some(read), TURN_SPEND, &offered);
        let holder = h.domain.task(HOLDER).expect("holder live");
        results.push((name, decided, admitted, holder.turn, holder.run_spent));
        expected.push((name, Some(outcome), Ok(Accepted::New), READING_TURN, TURN_SPEND));
    }
    assert_eq!(results, expected);
}

fn word(number: u64, kind: crate::MessageKind) -> crate::Word {
    crate::Word {
        number,
        from: Party::Person(REQUESTER),
        kind,
        words: Box::from(b"message".as_slice()),
        at: skein_lib::Wall::EPOCH,
        hits: 1,
        eligible: false,
    }
}

fn amend_words(h: &mut Harness, words: &[u8]) {
    let mut change = amendment();
    let mut spec = h.domain.task(HOLDER).unwrap().spec.clone();
    spec.words = words.into();
    change.spec = Some(spec);
    h.amend(HOLDER, change);
}

fn replaced_word() -> (Harness, Vec<crate::Word>) {
    // Two offers fill the run inbox, with the delegate's result room separate.
    const OFFER_BOUND: u32 = 2;
    let mut h = running_holder();
    h.env.limits.run_inbox = OFFER_BOUND;
    h.send(Event::Message { reply_to: to(), project: 1, task: HOLDER, word: word(ENTRY, crate::MessageKind::Words) });
    amend_words(&mut h, b"first amendment");
    let offers = h.domain.task(HOLDER).unwrap().inbox.clone();
    amend_words(&mut h, b"replacing amendment");
    (h, offers.to_vec())
}

#[test]
fn a_replaced_offered_word_at_the_inbox_bound_remains_readable() {
    let (mut h, offers) = replaced_word();
    assert_eq!(
        h.domain.task(HOLDER).unwrap().inbox.iter().map(|word| word.number).collect::<Vec<_>>(),
        [ENTRY, NEXT_ENTRY + 1]
    );
    assert_eq!(turn(&mut h, READING_TURN, Some(NEXT_ENTRY), TURN_SPEND, &offers), Ok(Accepted::New));
}

#[test]
fn holes_and_consumed_words_cannot_advance_or_charge() {
    let (mut h, offers) = replaced_word();
    let before = h.domain.task(HOLDER).unwrap().clone();
    assert_eq!(turn(&mut h, READING_TURN, Some(ENTRY - 1), TURN_SPEND, &offers), Err(Refusal::Read));
    assert_eq!(h.domain.task(HOLDER).unwrap(), &before);
    turn(&mut h, READING_TURN, Some(NEXT_ENTRY), TURN_SPEND, &offers).expect("reads the replaced offer");
    let before = h.domain.task(HOLDER).unwrap().clone();
    // The parent retires the evidence with that commit. Keep the high-water
    // authentic while testing admission without the consumed exact offer.
    let out = h.send(Event::Turn {
        reply_to: to(),
        task: HOLDER,
        attempt: ATTEMPT,
        turn: READING_TURN + 1,
        read: Some(NEXT_ENTRY),
        refused: Box::new([]),
        offered: Some(NEXT_ENTRY),
        outstanding: Box::new([]),
        cumulative: TURN_SPEND + 1,
    });
    assert!(out.iter().any(|request| match request {
        Request::Refused { problem, .. } => problem.why == Refusal::Read,
        _ => false,
    }));
    assert_eq!(h.domain.task(HOLDER).unwrap(), &before);
}

#[test]
fn an_old_passed_proposal_decided_without_an_offer_does_not_supply_a_read_fence() {
    let mut h = running_holder();
    h.send(Event::Hold { task: CHILD, why: Hold::Stopped });
    let old = ENTRY - 1;
    h.send(Event::Propose {
        reply_to: to(),
        proposal: Proposal {
            number: old,
            proposer: CHILD,
            project: 1,
            action: ProposalAction::Release { task: CHILD },
            reason: Box::from(b"release".as_slice()),
            as_holder: false,
            state: ProposalState::Pending { holder: ProposalHolder::Person(REQUESTER), since: h.env.wall },
        },
    });
    h.send(Event::DecideProposal {
        reply_to: to(),
        proposer: CHILD,
        proposal: old,
        message: None,
        by: Party::Person(REQUESTER),
        decision: ProposalDecision::Pass { holder: ProposalHolder::Task(HOLDER) },
    });
    assert!(crate::proposals::waiting_for(&h.domain, HOLDER).iter().any(|word| word.number == old));
    h.send(Event::DecideProposal {
        reply_to: to(),
        proposer: CHILD,
        proposal: old,
        message: None,
        by: Party::Task(HOLDER),
        decision: ProposalDecision::Pass { holder: ProposalHolder::Person(REQUESTER) },
    });
    h.send(Event::Message { reply_to: to(), project: 1, task: HOLDER, word: word(ENTRY, crate::MessageKind::Words) });
    let offers = h.domain.task(HOLDER).expect("holder live").inbox.clone();
    assert_eq!(turn(&mut h, READING_TURN, Some(old), TURN_SPEND, &offers), Err(Refusal::Read));
}

#[test]
fn wrong_attempt_and_oversized_offer_evidence_cannot_advance_or_charge() {
    let mut h = running_holder();
    let before = h.domain.task(HOLDER).unwrap().clone();
    for (attempt, offers, expected) in [
        (ATTEMPT + 1, vec![ENTRY], Refusal::Attempt),
        (ATTEMPT, (ENTRY..=ENTRY + u64::from(h.env.limits.run_inbox)).collect(), Refusal::Read),
    ] {
        let out = h.send(Event::Turn {
            reply_to: to(),
            task: HOLDER,
            attempt,
            turn: READING_TURN,
            read: Some(ENTRY),
            refused: Box::new([]),
            offered: offers.last().copied(),
            outstanding: offers.into(),
            cumulative: TURN_SPEND,
        });
        assert!(out.iter().any(|request| match request {
            Request::Refused { problem, .. } => problem.why == expected,
            _ => false,
        }));
        assert_eq!(h.domain.task(HOLDER).unwrap(), &before);
    }
}
