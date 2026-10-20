use super::revisions::{Harness, LIMITS, task, to};
use crate::{Event, Funder, Party, Payer, PayerLink, Request};

const ROOT: u64 = 1;
const CHILD: u64 = 2;
const HOLDER: u64 = 9;
const OTHER: u64 = 17;
const PROJECT: u32 = 1;

#[test]
fn payer_follows_funding_not_requester() {
    let mut h = Harness::new();
    h.make(vec![task(ROOT, &[])]);
    let mut child = task(CHILD, &[]);
    child.authority.budget.spend = 20;
    child.numbers.budget = 20;
    child.funder = Funder::Task(ROOT);
    h.send(Event::Make { reply_to: to(), creator: Party::Task(ROOT), batch: Box::new([child]) });
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
    let out = h.send(Event::Prepare { reply_to: to(), task: ROOT });
    assert!(out.iter().any(|r| matches!(r, Request::Done { .. })));
}

#[test]
fn invalid_prospective_chains_are_bounded_and_change_nothing() {
    let h = Harness::new();
    let cycle =
        [PayerLink { task: ROOT, funder: Funder::Task(CHILD) }, PayerLink { task: CHILD, funder: Funder::Task(ROOT) }];
    let duplicate = [cycle[0], cycle[0]];
    let mut chain = Vec::new();
    for n in 0..u64::from(LIMITS.tasks) {
        chain.push(PayerLink { task: n, funder: Funder::Task(n + 1) });
    }
    assert_eq!(h.domain.prospective_payer(Funder::Task(ROOT), &cycle), None);
    assert_eq!(h.domain.prospective_payer(Funder::Task(ROOT), &duplicate), None);
    assert_eq!(h.domain.prospective_payer(Funder::Task(0), &chain), None);
    chain.last_mut().unwrap().funder = Funder::Period { project: PROJECT, period: 0 };
    assert_eq!(h.domain.prospective_payer(Funder::Task(0), &chain), Some(Payer::Project(PROJECT)));
    chain.push(PayerLink { task: 100, funder: Funder::Task(0) });
    assert_eq!(h.domain.prospective_payer(Funder::Task(0), &chain), None);
    assert!(h.stored.values().all(|r| !matches!(r, crate::Stored::Live(_))));
}
