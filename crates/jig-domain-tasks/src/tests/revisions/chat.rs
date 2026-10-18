//! Whole-close entry-point checks, using the ordinary scripted parent fixture.
use super::*;

pub(super) fn cohort() -> Harness {
    let mut test = Harness::new();
    let mut root = task(1, &[]);
    root.numbers.budget = 1000;
    root.authority.budget.spend = 1000;
    test.make(vec![root]);
    let mut first = task(2, &[]);
    first.funder = Funder::Task(1);
    let mut second = first.clone();
    second.number = 3;
    let out = test.send(Event::Make { reply_to: to(), creator: Party::Task(1), batch: Box::new([first, second]) });
    assert!(
        out.iter().any(|request| match request {
            Request::Made { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
    test
}
