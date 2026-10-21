use super::revisions::{Harness, LIMITS, task, to};
use crate::{Event, HoldKind, Holding, Kind, Name, Party, Refusal, Taken};

const HOLDER: u64 = 1;
const REQUESTER: u64 = 2;
const WAITER: u64 = 3;
const OTHER: u64 = 4;
const CONNECTOR: u16 = 1;
const KIND: u16 = 1;

fn holding() -> Holding {
    Holding::Write { resource: Name { connector: CONNECTOR, path: Box::new([Box::new([1])]) }, kind: KIND }
}

#[test]
fn a_holders_added_dependency_cannot_circle_through_a_delegates_queue() {
    let mut test = Harness::new();
    test.send(Event::Kinds {
        connector: CONNECTOR,
        kinds: Box::new([Kind { connector: CONNECTOR, kind: KIND, hold: HoldKind::Exclusive { taken: Taken::Waits } }]),
    });
    let mut holder = task(HOLDER, &[]);
    holder.holdings = Box::new([holding()]);
    test.make([holder, task(REQUESTER, &[])].into());
    let mut waiter = task(WAITER, &[]);
    waiter.holdings = Box::new([holding()]);
    test.send(Event::Make { reply_to: to(), creator: Party::Task(REQUESTER), batch: Box::new([waiter]) });
    let problem =
        crate::waits::check_added(&test.domain, &crate::compound::Staged::new(0), &LIMITS, HOLDER, &[REQUESTER])
            .expect_err("circle");
    assert_eq!(problem.why, Refusal::Circle { cut: false });
    assert_eq!(problem.blocked_by.as_deref(), Some([HOLDER, REQUESTER, WAITER].as_slice()));
}

#[test]
fn a_pool_only_propagates_stuckness_at_its_slot_threshold() {
    let mut test = Harness::new();
    test.send(Event::Kinds {
        connector: CONNECTOR,
        kinds: Box::new([Kind { connector: CONNECTOR, kind: KIND, hold: HoldKind::Pooled { taken: Taken::Waits } }]),
    });
    let Holding::Write { resource: pool, .. } = holding() else { unreachable!() };
    test.send(Event::Slots { pool: pool.clone(), slots: 2 });
    let pooled = Holding::Slot { pool, kind: KIND };
    let mut holder = task(HOLDER, &[]);
    holder.holdings = Box::new([pooled.clone()]);
    let mut other = task(OTHER, &[]);
    other.holdings = Box::new([pooled.clone()]);
    test.make([holder, task(REQUESTER, &[]), other].into());
    let mut waiter = task(WAITER, &[]);
    waiter.holdings = Box::new([pooled]);
    test.send(Event::Make { reply_to: to(), creator: Party::Task(REQUESTER), batch: Box::new([waiter]) });
    assert_eq!(
        crate::waits::check_added(&test.domain, &crate::compound::Staged::new(0), &LIMITS, HOLDER, &[REQUESTER]),
        Ok(())
    );
    let other = &mut crate::domain::task_mut(&mut test.domain, OTHER).expect("other holder").record;
    other.dependencies = Box::new([HOLDER]);
    other.waiting_on = Box::new([HOLDER]);
    let mut out = skein_lib::Queue::with_capacity(crate::max_out(&LIMITS));
    crate::domain::publish(&mut test.domain, &test.env, OTHER, &mut out);
    let problem =
        crate::waits::check_added(&test.domain, &crate::compound::Staged::new(0), &LIMITS, HOLDER, &[REQUESTER])
            .expect_err("both slots stuck");
    assert_eq!(problem.blocked_by.as_deref(), Some([HOLDER, REQUESTER, WAITER, OTHER].as_slice()));
}
