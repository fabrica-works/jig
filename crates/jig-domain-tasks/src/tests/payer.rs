#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "test scripts are ordinary Rust under programming-model.md section 10.2"
)]
use super::revisions::{Harness, LIMITS, task, to};
use crate::{Event, Funder, Party, Payer, PayerLink};

const ROOT: u64 = 1;
const CHILD: u64 = 2;
const HOLDER: u64 = 9;
const OTHER: u64 = 17;
const PROJECT: u32 = 1;

fn funded_tree() -> Harness {
    let mut h = Harness::new();
    h.make(vec![task(ROOT, &[])]);
    let mut child = task(CHILD, &[]);
    child.authority.budget.spend = 20;
    child.numbers.budget = 20;
    child.funder = Funder::Task(ROOT);
    h.send(Event::Make { reply_to: to(), creator: Party::Task(ROOT), batch: Box::new([child]) });
    h
}

#[test]
fn payer_follows_funding_not_requester() {
    let mut h = funded_tree();
    assert_eq!(h.domain.payer(CHILD), Some(Payer::Project(PROJECT)));
    h.send(Event::Move {
        reply_to: to(),
        task: ROOT,
        person: OTHER,
        period: 0,
        pool_budget: 100,
        period_budget: 10_000,
        reason: Box::new([]),
    });
    assert_eq!(h.domain.payer(CHILD), Some(Payer::Party(OTHER)));
    h.send(Event::OpenPeriod { reply_to: to(), project: PROJECT, period: 1, budget: 10_000 });
    h.restart();
    assert_eq!(h.domain.payer(ROOT), Some(Payer::Party(OTHER)));
    assert_eq!(h.domain.payer(CHILD), Some(Payer::Party(OTHER)));
}

#[test]
fn prospective_payers_use_staged_financial_links_first() {
    let mut h = Harness::new();
    h.make(vec![task(ROOT, &[])]);
    let pool = Funder::Pool { person: HOLDER, project: PROJECT, period: 3 };
    let links = [PayerLink { task: ROOT, funder: pool }, PayerLink { task: CHILD, funder: Funder::Task(ROOT) }];
    assert_eq!(h.domain.prospective_payer(Funder::Task(CHILD), &links), Some(Payer::Party(HOLDER)));
    assert_eq!(h.domain.payer(ROOT), Some(Payer::Project(PROJECT)));
    for funder in
        [Funder::Period { project: PROJECT, period: 4 }, Funder::Recurring { project: PROJECT, task: ROOT, period: 4 }]
    {
        assert_eq!(h.domain.prospective_payer(funder, &[]), Some(Payer::Project(PROJECT)));
    }
}

#[test]
fn invalid_prospective_chains_are_bounded_and_change_nothing() {
    let h = Harness::new();
    let before = h.stored.clone();
    let cycle =
        [PayerLink { task: ROOT, funder: Funder::Task(CHILD) }, PayerLink { task: CHILD, funder: Funder::Task(ROOT) }];
    let duplicate = [cycle[0], cycle[0]];
    let mut chain = Vec::new();
    for n in 0..u64::from(LIMITS.tasks) {
        chain.push(PayerLink { task: n, funder: Funder::Task(n.checked_add(1).unwrap()) });
    }
    assert_eq!(h.domain.prospective_payer(Funder::Task(ROOT), &cycle), None);
    assert_eq!(h.domain.prospective_payer(Funder::Task(ROOT), &duplicate), None);
    assert_eq!(h.domain.prospective_payer(Funder::Task(0), &chain), None);
    chain.last_mut().unwrap().funder = Funder::Period { project: PROJECT, period: 0 };
    assert_eq!(h.domain.prospective_payer(Funder::Task(0), &chain), Some(Payer::Project(PROJECT)));
    chain.push(PayerLink { task: 100, funder: Funder::Task(0) });
    assert_eq!(h.domain.prospective_payer(Funder::Task(0), &chain), None);
    assert_eq!(h.stored, before);
}

#[test]
fn refunding_changes_the_payer_of_financial_descendants() {
    let mut h = funded_tree();
    h.send(Event::CarvePool { reply_to: to(), project: PROJECT, person: OTHER, period: 0, budget: 100 });
    h.send(Event::Amend {
        reply_to: to(),
        by: Party::Person(OTHER),
        task: ROOT,
        message: 1,
        stop_run: false,
        funder: Some(Funder::Pool { project: PROJECT, person: OTHER, period: 0 }),
        amendment: crate::Amendment {
            tracked: None,
            addressed: None,
            instructions: None,
            release: None,
            spec: None,
            wake: None,
            dependencies: None,
            authority: None,
            llm: None,
            reason: Box::new([]),
        },
    });
    assert_eq!(h.domain.task(ROOT).unwrap().requester, Party::Person(HOLDER));
    assert_eq!(h.domain.payer(CHILD), Some(Payer::Party(OTHER)));
    h.restart();
    assert_eq!(h.domain.payer(CHILD), Some(Payer::Party(OTHER)));
}

#[test]
fn compound_payer_reads_a_move_before_the_live_financial_chain_changes() {
    let h = funded_tree();
    let before = h.stored.clone();
    let mut admission = crate::CompoundAdmission::new(&h.domain, 1);
    assert_eq!(admission.payer(CHILD), Some(Payer::Project(PROJECT)));
    admission
        .check(
            &h.env,
            crate::CompoundContext {
                person: OTHER,
                project: PROJECT,
                period: 0,
                pool_budget: 100,
                period_budget: 10_000,
            },
            crate::Operation::Move { task: ROOT, reason: Box::new([]) },
        )
        .unwrap();
    assert_eq!(admission.payer(CHILD), Some(Payer::Party(OTHER)));
    assert_eq!(h.domain.payer(CHILD), Some(Payer::Project(PROJECT)));
    assert_eq!(h.stored, before);
}
