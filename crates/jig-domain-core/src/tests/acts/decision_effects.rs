//! Decision consequences are delivered facts, computed once by the engine.
use super::*;
use crate::acts::{Effect, Impact, ReleaseEffect};
const ROOT: u64 = 1;
const RUNNING: u64 = 2;
const HELD: u64 = 3;
const DECIDED: u64 = 4;
const PROPOSAL: u64 = 10;
const PROPOSE: u64 = 103;

fn task_offer(core: &Core, limits: &Limits, task: u64, act: Act) -> crate::acts::Offered {
    let Document::Task(value) = make(core, limits, 1, &Subject::Task { task }, &window(), Wall::EPOCH).unwrap() else {
        panic!("task document")
    };
    offered(&value.acts, act).clone()
}
fn proposal(core: &Core, task: u64) -> tasks::Proposal {
    tasks::Proposal {
        number: PROPOSAL,
        proposer: task,
        project: 1,
        action: tasks::ProposalAction::Batch(Box::new([new_row(core, 5, tasks::Funder::Task(task), None)])),
        reason: Box::new([]),
        as_holder: false,
        state: tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Person(1), since: Wall::EPOCH },
    }
}
#[test]
fn a_cancel_offer_counts_the_tree_it_would_end() {
    let (core, limits) = cancellation_fixture();
    let offer = task_offer(&core, &limits, ROOT, Act::Cancel);
    assert_eq!(
        offer.effect,
        Some(Effect::Cancel(Impact {
            tasks: PhaseCounts { starting: 1, running: 1, held: 1, ..PhaseCounts::ZERO },
            live_runs: 2,
            proposals: 1,
        }))
    );
}
#[test]
fn a_release_offer_on_an_uncertain_effect_says_uncertain() {
    let (mut core, limits) = make_tree();
    send_task(&mut core, &limits, tasks::Event::Hold { task: HELD, why: tasks::Hold::Uncertain { entry: 1 } });
    assert_eq!(task_offer(&core, &limits, HELD, Act::Release).effect, Some(Effect::Release(ReleaseEffect::Uncertain)));
}
#[test]
fn a_move_offer_names_the_reservation_it_takes() {
    let (core, limits) = make_tree();
    assert_eq!(task_offer(&core, &limits, RUNNING, Act::Move).effect, Some(Effect::Move { reservation: 10 }));
}
#[test]
fn a_party_s_own_proposal_offers_withdraw_to_its_proposer_alone() {
    let (mut core, limits) = make_tree();
    let goal = new_row(&core, 5, tasks::Funder::Pool { project: 1, person: 1, period: 1 }, Some(4));
    send_task(
        &mut core,
        &limits,
        tasks::Event::ProposePerson {
            reply_to: ReplyTo::new(Token::new(PROPOSE)),
            proposal: tasks::PersonProposal {
                number: PROPOSAL,
                proposer: 1,
                project: 1,
                action: tasks::PersonAction::Batch(Box::new([goal])),
                reason: Box::new([]),
                state: tasks::PersonProposalState::Pending { since: Wall::EPOCH },
            },
        },
    );
    let subject = Subject::Proposal { proposer: Proposer::Party(1), number: PROPOSAL };
    let Document::Proposal(own) = make(&core, &limits, 1, &subject, &window(), Wall::EPOCH).unwrap() else {
        panic!("proposal")
    };
    assert_eq!(offered(&own.acts, Act::Withdraw).admission, Admission::Allowed);
    assert_eq!(
        offers::admit(&core, &limits, 2, Act::Withdraw, &subject, Wall::EPOCH),
        Admission::Refused(people::Refusal::NotOffered)
    );
}

fn decide_one_proposal(core: &mut Core, limits: &Limits) {
    let proposal = proposal(core, DECIDED);
    send_task(core, limits, tasks::Event::Propose { reply_to: ReplyTo::new(Token::new(PROPOSE)), proposal });
    send_task(
        core,
        limits,
        tasks::Event::DecideProposal {
            reply_to: ReplyTo::new(Token::new(PROPOSE)),
            proposer: DECIDED,
            proposal: PROPOSAL,
            message: Some(PROPOSAL),
            by: tasks::Party::Person(1),
            decision: tasks::ProposalDecision::Reject { reason: Box::new([]) },
        },
    );
}

fn offered(rows: &List<crate::acts::Offered>, act: Act) -> &crate::acts::Offered {
    for row in rows {
        if row.act == act {
            return row;
        }
    }
    panic!("offered act {act:?}")
}

fn cancellation_fixture() -> (Core, Limits) {
    let (mut core, limits) = make_tree();
    decide_one_proposal(&mut core, &limits);
    let revision = core.tasks.escalation(HELD).unwrap().escalation.revision();
    send_task(
        &mut core,
        &limits,
        tasks::Event::RoutedEscalation { task: HELD, revision, holder: tasks::EscalationHolder::Person(1), entry: 1 },
    );
    let pending = proposal(&core, ROOT);
    send_task(
        &mut core,
        &limits,
        tasks::Event::Propose { reply_to: ReplyTo::new(Token::new(PROPOSE)), proposal: pending },
    );
    let mut rows = Box::new([
        core.tasks.task(ROOT).unwrap().clone(),
        core.tasks.task(RUNNING).unwrap().clone(),
        core.tasks.task(HELD).unwrap().clone(),
        core.tasks.task(DECIDED).unwrap().clone(),
    ]);
    rows[0].phase = tasks::Phase::Active(tasks::Active::Running { attempt: ROOT });
    rows[0].attempt = ROOT;
    rows[0].run_since = Some(Wall::EPOCH);
    rows[1].phase = tasks::Phase::Active(tasks::Active::Running { attempt: RUNNING });
    rows[1].attempt = RUNNING;
    rows[1].run_since = Some(Wall::EPOCH);
    assert!(!rows[3].proposal_rows.is_empty());
    assert!(rows[1].holdings.is_empty());
    fixtures::restore(&mut core, &limits, rows);
    (core, limits)
}
