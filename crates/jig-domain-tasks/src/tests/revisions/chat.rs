//! Whole-close entry-point checks, using the ordinary scripted parent fixture.
use super::*;
use crate::{Disposal, Grace, Keep, Phase};
use skein_lib::List;

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

fn close(test: &mut Harness, pool: u64, second: Keep) -> Vec<Request> {
    let mut disposals = List::with_capacity(2);
    disposals.push(Disposal { task: 2, keep: Keep::MakeMine }).unwrap();
    disposals.push(Disposal { task: 3, keep: second }).unwrap();
    test.send(Event::CloseChat {
        reply_to: to(),
        task: 1,
        person: 9,
        message: 1,
        until: Wall::from_nanos(Duration::from_secs(5).as_nanos()),
        disposals,
        period: 0,
        pool_budget: pool,
        period_budget: 10_000,
    })
}

#[test]
fn combined_pool_refusal_preserves_the_whole_close_and_its_saved_rows() {
    let mut test = cohort();
    let before = test.stored.clone();
    let root = test.domain.task(1).unwrap().clone();
    let out = close(&mut test, 150, Keep::MakeMine);
    assert!(
        out.iter().any(|request| match request {
            Request::Refused { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
    assert_eq!(test.stored, before);
    assert_eq!(test.domain.task(1), Some(&root));
    for number in [2, 3] {
        assert_eq!(test.domain.task(number).unwrap().requester, Party::Task(1));
    }
    assert!(!out.iter().any(|request| match request {
        Request::Save { .. } | Request::Done { .. } => true,
        _ => false,
    }));
}

#[test]
fn disposals_finish_request_and_restart_grace_are_one_whole_close() {
    let mut test = cohort();
    let mut grandchild = task(4, &[]);
    grandchild.funder = Funder::Task(3);
    grandchild.numbers.budget = 20;
    grandchild.authority.budget.spend = 20;
    test.send(Event::Make { reply_to: to(), creator: Party::Task(3), batch: Box::new([grandchild]) });
    let out = close(&mut test, 150, Keep::Cancel);
    assert_eq!(
        out.iter()
            .filter(|request| match request {
                Request::Done { .. } => true,
                _ => false,
            })
            .count(),
        1,
        "{out:?}"
    );
    let root = test.domain.task(1).unwrap();
    assert_eq!(
        root.grace,
        Some(Grace {
            by: Party::Person(9),
            until: Wall::from_nanos(5_000_000_000),
            reason: b"Chat finish grace expired.".as_slice().into(),
            message: 1,
        })
    );
    assert_eq!(root.last_message, 1);
    assert_eq!(test.domain.task(2).unwrap().requester, Party::Person(9));
    assert_eq!(test.domain.task(2).unwrap().funder, Funder::Pool { project: 1, person: 9, period: 0 });
    let grandchild_close = out
        .iter()
        .position(|request| match request {
            Request::Save { record: Stored::Live(row) } if row.number == 4 => match row.phase {
                Phase::Closing(_) => true,
                _ => false,
            },
            _ => false,
        })
        .unwrap();
    let child_close = out
        .iter()
        .position(|request| match request {
            Request::Save { record: Stored::Live(row) } if row.number == 3 => match row.phase {
                Phase::Closing(_) => true,
                _ => false,
            },
            _ => false,
        })
        .unwrap();
    assert!(grandchild_close < child_close, "cancellation is deepest first");
    test.send(Event::EffectsSettled { task: 4 });
    test.send(Event::Settled { task: 4 });
    test.send(Event::DelegateResult {
        task: 3,
        word: crate::Word {
            number: 2,
            from: Party::Task(4),
            kind: MessageKind::Result(crate::ResultKind::Cancelled),
            words: Box::from(&b"cancelled"[..]),
            at: test.env.wall,
            hits: 1,
            eligible: false,
        },
    });
    test.send(Event::EffectsSettled { task: 3 });
    test.send(Event::Settled { task: 3 });
    test.send(Event::DelegateResult {
        task: 1,
        word: crate::Word {
            number: 3,
            from: Party::Task(3),
            kind: MessageKind::Result(crate::ResultKind::Cancelled),
            words: Box::from(&b"cancelled"[..]),
            at: test.env.wall,
            hits: 1,
            eligible: false,
        },
    });
    assert!(test.domain.task(3).is_none());
    test.env.now = Time::from_nanos(2_000_000_000);
    test.env.wall = Wall::from_nanos(2_000_000_000);
    test.restart();
    test.env.now = Time::from_nanos(4_999_999_999);
    test.env.wall = Wall::from_nanos(4_999_999_999);
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    crate::fire(&mut test.domain, &test.env, &mut out);
    assert!(test.domain.task(1).is_some());
    test.env.now = Time::from_nanos(5_000_000_000);
    test.env.wall = Wall::from_nanos(5_000_000_000);
    crate::fire(&mut test.domain, &test.env, &mut out);
    test.send(Event::EffectsSettled { task: 1 });
    test.send(Event::Settled { task: 1 });
    let Stored::Ended(ended) = &test.stored[&Key::Ended(1)] else { panic!("expired chat saved its ending") };
    assert!(match ended.phase {
        Phase::Ended(crate::Ending::Cancelled { by: Party::Person(9), .. }) => true,
        _ => false,
    });
    assert!(test.domain.task(2).is_some(), "kept delegate survives its old chat");
}
