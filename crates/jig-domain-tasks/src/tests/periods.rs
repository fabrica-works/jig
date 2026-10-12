//! Period inputs and their durable output records, through the child entry points.
use super::revisions::{Harness, LIMITS, to};
use crate::{Domain, Event, Funder, PeriodTimes, Request, Stored, max_out, step};
use skein_lib::{Queue, Wall};

#[test]
fn supplied_period_boundaries_preserve_the_opening_wall_across_duplicates_and_restart() {
    let mut test = Harness::new();
    test.env.wall = Wall::from_nanos(1200);
    test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 1, budget: 100 });
    let source = Funder::Period { project: 1, period: 1 };
    let first = PeriodTimes { began: test.env.wall, next: None };
    assert_eq!(test.domain.funding(source).unwrap().period, Some(first));
    let next = Some(Wall::from_nanos(1100));
    test.env.wall = Wall::from_nanos(900);
    let out = test.send(Event::PeriodNext { project: 1, period: 1, next });
    let mut saved = false;
    for request in out {
        if let Request::Save { record: Stored::Ledger(row) } = request {
            assert_eq!(row.funder, source);
            assert_eq!(row.period, Some(PeriodTimes { began: first.began, next }));
            saved = true;
        }
    }
    assert!(saved);
    assert!(test.send(Event::PeriodNext { project: 1, period: 1, next }).is_empty());
    test.restart();
    assert_eq!(test.domain.funding(source).unwrap().period, Some(PeriodTimes { began: first.began, next }));
    test.send(Event::PeriodNext { project: 1, period: 1, next: None });
    assert_eq!(test.domain.funding(source).unwrap().period, Some(first));
    test.env.wall = Wall::from_nanos(800);
    test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 2, budget: 100 });
    assert!(test.send(Event::PeriodNext { project: 1, period: 1, next }).is_empty());
    assert!(test.send(Event::PeriodNext { project: 9, period: 2, next }).is_empty());
    test.restart();
    assert_eq!(
        test.domain.funding(Funder::Period { project: 1, period: 2 }).unwrap().period,
        Some(PeriodTimes { began: Wall::from_nanos(800), next: None })
    );
}

#[test]
fn cold_restore_refuses_a_period_without_its_actual_opening_fact() {
    let test = Harness::new();
    let mut row = *test.domain.funding(Funder::Period { project: 1, period: 0 }).unwrap();
    let original = row;
    for invalid in [false, true] {
        row = original;
        if invalid {
            row.funder = Funder::Pool { project: 1, period: 0, person: 9 };
            row.parent = Some(original.funder);
        } else {
            row.period = None;
        }
        let mut domain = Domain::new(&LIMITS, 91, Box::new([1]));
        let mut out = Queue::with_capacity(max_out(&LIMITS));
        step(&mut domain, &test.env, Event::Restore { record: Stored::Ledger(row) }, &mut out);
        let mut refused = false;
        while let Some(request) = out.pop() {
            if let Request::RestoreRefused { .. } = request {
                refused = true;
            }
        }
        assert!(refused);
    }
}
