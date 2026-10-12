//! Scheduler observations are pure over timers restored through public entrances.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "entry-point fixtures are ordinary Rust under programming-model.md section 10.2"
)]
use super::revisions::{Harness, LIMITS, task};
use crate::{
    Active, Domain, Escalation, EscalationHolder, Event, Grace, Hold, HoldKind, Holding, Key, Kind, MessageKind, Name,
    Party, Phase, Proposal, ProposalAction, ProposalHolder, ProposalState, Request, RetryNotice, Stored, Subscription,
    SubscriptionKind, Taken, WakeRule, Was, Word,
};
use skein_lib::{Duration, Time, Wall};

fn deadlines(first: usize) -> Harness {
    let mut h = Harness::new();
    h.make((1..=7).map(|n| task(n, &[])).collect());
    h.env.now = Time::from_nanos(100);
    h.env.wall = Wall::from_nanos(1000);
    let spans: Vec<_> = (0..7).map(|i| Duration::from_nanos(if i == first { 10 } else { 50 })).collect();
    h.env.limits.proposal_stall = spans[4];
    h.env.limits.escalation_stall = spans[5];
    h.env.limits.hold_wait = spans[6];
    let name = Name { connector: 7, path: Box::new([Box::from(&b"wait"[..])]) };
    for (i, span) in spans.into_iter().enumerate() {
        let number = u64::try_from(i).unwrap().checked_add(1).unwrap();
        let mut row = h.domain.task(number).unwrap().clone();
        let at = Wall::from_nanos(h.env.wall.as_nanos().checked_add(span.as_nanos()).unwrap());
        match i {
            0 => {
                row.phase = Phase::Active(Active::BackingOff { until: at });
                row.holdings = Box::new([Holding::Write { resource: name.clone(), kind: 1 }]);
                row.refusals = 1;
                row.retry_notice = Some(RetryNotice::Refused { count: 1 });
            }
            1 => row.chat_close = Some(Grace { by: 9, until: at }),
            2 => {
                row.subscriptions =
                    Box::new([Subscription { number: 1, kind: SubscriptionKind::Timer { at, period: None } }]);
            }
            3 => {
                row.phase = Phase::Active(Active::Idle);
                row.wake.words = WakeRule::Batch { count: 2, age: span };
                row.last_message = 1;
                row.inbox = Box::new([Word {
                    number: 1,
                    from: Party::Task(1),
                    kind: MessageKind::Words,
                    words: Box::new([1]),
                    at: h.env.wall,
                    hits: 1,
                    eligible: false,
                }]);
            }
            4 => {
                row.revision = 1;
                row.proposal = Some(Box::new(Proposal {
                    number: 1,
                    proposer: number,
                    project: 1,
                    action: ProposalAction::Release { task: 1 },
                    reason: Box::new([]),
                    as_holder: false,
                    state: ProposalState::Pending { holder: ProposalHolder::Person(9), since: h.env.wall },
                }));
            }
            5 => {
                row.phase = Phase::Held { was: Was::Active(Active::Due), why: Hold::Stopped, source: None };
                row.revisions.hold = 1;
                row.escalation = Escalation::Waiting {
                    revision: 1,
                    holder: EscalationHolder::Person(9),
                    entry: 1,
                    since: h.env.wall,
                };
            }
            6 => {
                row.phase = Phase::Waiting;
                row.holds_taken = false;
                row.hold_wait_since = Some(h.env.wall);
                row.holdings = Box::new([Holding::Write { resource: name.clone(), kind: 1 }]);
            }
            _ => unreachable!(),
        }
        h.stored.insert(Key::Live(number), Stored::Live(Box::new(row)));
    }
    restore(h)
}

fn restore(mut h: Harness) -> Harness {
    let rows: Vec<_> = h
        .stored
        .values()
        .filter(|r| match r {
            Stored::Live(_) | Stored::Ledger(_) => true,
            _ => false,
        })
        .cloned()
        .collect();
    h.domain = Domain::new(&LIMITS, 91, Box::new([1]));
    h.send(Event::Kinds {
        connector: 7,
        kinds: Box::new([Kind { connector: 7, kind: 1, hold: HoldKind::Exclusive { taken: Taken::Waits } }]),
    });
    for row in rows {
        assert!(!h.send(Event::Restore { record: row }).iter().any(|r| match r {
            Request::RestoreRefused { .. } => true,
            _ => false,
        }));
    }
    let out = h.send(Event::Restored);
    assert!(
        !out.iter().any(|r| match r {
            Request::RestoreRefused { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
    assert!(h.domain.ready());
    h
}

#[test]
fn an_empty_task_scheduler_has_no_deadline_before_or_after_restore() {
    let h = Harness::new();
    assert_eq!(Domain::new(&LIMITS, 91, Box::new([1])).next_deadline(), None);
    assert_eq!(h.domain.next_deadline(), None);
}

#[test]
fn each_of_seven_task_timer_owners_can_supply_the_pure_earliest_deadline() {
    for first in 0..7 {
        let mut h = deadlines(first);
        let before: Vec<_> = (1..=7).map(|n| h.domain.task(n).unwrap().clone()).collect();
        assert_eq!(h.domain.next_deadline(), Some(Time::from_nanos(110)), "timer set {first}");
        h.env.now = Time::from_nanos(110);
        for _ in 0_u32..3 {
            assert_eq!(h.domain.next_deadline(), Some(h.env.now), "observing a due timer does not expire it");
        }
        assert_eq!((1..=7).map(|n| h.domain.task(n).unwrap().clone()).collect::<Vec<_>>(), before);
        assert!(h.domain.next_deadline().unwrap() < Time::from_nanos(150));
    }
}
