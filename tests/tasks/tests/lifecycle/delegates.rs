use jig_domain_tasks::{Edge, Event, Key, Party, Refusal, Stored};
use jig_tasks_world::{LIMITS, Reply, World, task};

fn delegates() -> World {
    let mut world = World::new(185, LIMITS);
    assert_eq!(world.make(Party::Person(1), vec![task(1, &[])]), Reply::Made(vec![1]));
    world.carve_pool(0, 100);
    let mut child = task(2, &[]);
    child.funder = jig_domain_tasks::Funder::Pool { project: 1, person: 9, period: 0 };
    assert_eq!(world.make(Party::Task(1), vec![child]), Reply::Made(vec![2]));
    world
}

fn finish_delegate(world: &mut World) {
    world.claim(2, 2);
    world.finish(2);
    world.settle(2);
}

#[test]
fn a_delegate_ending_saves_its_ended_delegate_row() {
    let mut world = delegates();
    finish_delegate(&mut world);
    assert_eq!(
        world.records.get(&Key::EndedDelegate { requester: 1, task: 2 }),
        Some(&Stored::EndedDelegate(Edge { requester: 1, task: 2 }))
    );
    assert_eq!(
        world.records.get(&Key::Delegate { requester: 1, task: 2 }),
        Some(&Stored::Delegate(Edge { requester: 1, task: 2 }))
    );
    assert!(!world.records.keys().any(|key| matches!(key, Key::Delegate { task: 1, .. })));
}

#[test]
fn an_adopted_task_leaves_its_old_requesters_delegate_rows() {
    let mut world = delegates();
    let reply_to = world.to();
    let reply = reply_to.into_token().raw();
    world.send(Event::Move {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(reply)),
        task: 2,
        person: 9,
        period: 0,
        pool_budget: 100,
        period_budget: 1000,
        reason: Box::new([]),
    });
    assert_eq!(world.replies[&reply], Reply::Done);
    assert_eq!(world.record(2).requester, Party::Person(9));
    assert!(!world.records.contains_key(&Key::Delegate { requester: 1, task: 2 }));
    finish_delegate(&mut world);
    assert!(!world.records.contains_key(&Key::EndedDelegate { requester: 1, task: 2 }));
}

const MOVED_REQUESTER: u64 = 9;
const MOVED_DELEGATE: u64 = 2;
const OLD_ROOT: u64 = 1;

#[test]
fn a_moved_delegate_outlives_its_old_root_and_tears_down_cleanly() {
    let mut world = delegates();
    let reply_to = world.to();
    let call = reply_to.into_token();
    world.send(Event::Move {
        reply_to: skein_lib::ReplyTo::new(call),
        task: MOVED_DELEGATE,
        person: MOVED_REQUESTER,
        period: 0,
        pool_budget: 100,
        period_budget: 1000,
        reason: Box::new([]),
    });
    assert_eq!(world.replies.get(&call.raw()), Some(&Reply::Done));
    assert_eq!(world.record(MOVED_DELEGATE).requester, Party::Person(MOVED_REQUESTER));
    assert_eq!(world.record(MOVED_DELEGATE).root, MOVED_DELEGATE);
    assert_eq!(world.record(MOVED_DELEGATE).depth, 0);
    assert!(world.record(OLD_ROOT).delegates.is_empty());
    assert!(world.record(OLD_ROOT).references.contains(&MOVED_DELEGATE));
    world.claim(OLD_ROOT, OLD_ROOT);
    world.finish(OLD_ROOT);
    world.settle(OLD_ROOT);
    assert_eq!(world.live(), 1);
    finish_delegate(&mut world);
    world.complete_cancel();
    assert_eq!(world.live(), 0);
}

#[test]
fn a_refused_move_keeps_the_old_requesters_delegate_in_the_referee() {
    let mut world = delegates();
    let reply_to = world.to();
    let call = reply_to.into_token();
    world.send(Event::Move {
        reply_to: skein_lib::ReplyTo::new(call),
        task: MOVED_DELEGATE,
        person: 0,
        period: 0,
        pool_budget: 100,
        period_budget: 1000,
        reason: Box::new([]),
    });
    assert!(matches!(world.replies.get(&call.raw()), Some(Reply::Refused(problem)) if problem.why == Refusal::State));
    assert_eq!(world.record(MOVED_DELEGATE).requester, Party::Task(OLD_ROOT));
    assert_eq!(world.referee.expectations().descendants(OLD_ROOT), [OLD_ROOT, MOVED_DELEGATE]);
    finish_delegate(&mut world);
    world.claim(OLD_ROOT, OLD_ROOT);
    world.finish(OLD_ROOT);
    world.settle(OLD_ROOT);
    world.complete_cancel();
}

#[test]
fn a_delegate_moved_to_a_party_is_indexed_by_that_party_when_it_ends() {
    let mut world = delegates();
    let reply_to = world.to();
    world.send(Event::Move {
        reply_to,
        task: MOVED_DELEGATE,
        person: MOVED_REQUESTER,
        period: 0,
        pool_budget: 100,
        period_budget: 1000,
        reason: Box::new([]),
    });
    finish_delegate(&mut world);
    let Stored::Ended(ended) = &world.records[&Key::Ended(MOVED_DELEGATE)] else { panic!("ended delegate") };
    let key = Key::EndedByParty {
        party: MOVED_REQUESTER,
        label: 0,
        ended_at: jig_domain_tasks::Newest::of(ended.ended_at.expect("ended").as_nanos()),
        task: MOVED_DELEGATE,
    };
    assert_eq!(
        world.records.get(&key),
        Some(&Stored::EndedByParty(jig_domain_tasks::EndedByParty {
            party: MOVED_REQUESTER,
            label: 0,
            ended_at: ended.ended_at.expect("ended"),
            task: MOVED_DELEGATE
        }))
    );
    assert_eq!(world.records.keys().filter(|key| matches!(key, Key::EndedByParty { .. })).count(), 1);
}
