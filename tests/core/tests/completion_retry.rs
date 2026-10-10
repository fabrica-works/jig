use jig_core as core;
use jig_core_views as views;
use jig_core_world::watches::{open, world};
use jig_test_domain as root;
use skein_lib::Token;

fn report(world: &mut jig_core_world::effects::World, task: u64, attempt: u64, retry: u32, text: &[u8]) {
    world.send(root::Event::Core(core::Event::RunText {
        run: Token::new(task),
        attempt: Token::new(attempt),
        turn: 1,
        retry,
        offset: 0,
        text: text.into(),
        call: None,
    }));
}

#[test]
fn actual_engine_assignment_keeps_an_unwatched_retry_until_its_watch_opens() {
    let mut world = world(799, true);
    let assignment = world.assigned.as_ref().expect("actual live assignment");
    let (task, attempt) = (assignment.task, assignment.attempt);
    let durable = world.store.applied;
    report(&mut world, task, attempt, 0, b"old");
    report(&mut world, task, attempt, 1, b"");
    open(&mut world, 5, core::documents::Subject::Run { task });
    let party = world.watches.as_ref().expect("independent consumer");
    let stream = party.streams.iter().find(|stream| stream.watcher == 5).expect("mandatory empty retry");
    assert_eq!(
        (stream.turn, stream.from, stream.to, stream.text.as_ref()),
        (views::TurnName { attempt, turn: 1, retry: 1 }, 0, 0, b"".as_slice())
    );
    assert_eq!(world.store.applied, durable);
    party.finish();
}

#[test]
fn an_engine_watch_reopens_from_the_retained_retry_after_its_sole_watcher_leaves() {
    let mut world = world(800, true);
    let assignment = world.assigned.as_ref().expect("actual live assignment");
    let (task, attempt) = (assignment.task, assignment.attempt);
    open(&mut world, 5, core::documents::Subject::Run { task });
    let durable = world.store.applied;
    report(&mut world, task, attempt, 0, b"earlier");
    world.send(root::Event::Core(core::Event::StreamDelivered { watcher: Token::new(5) }));
    world.send(root::Event::Core(core::Event::Unwatch { watcher: Token::new(5) }));
    report(&mut world, task, attempt, 1, b"new prefix");
    open(&mut world, 6, core::documents::Subject::Run { task });
    let party = world.watches.as_ref().expect("independent consumer");
    assert!(party.endings.iter().any(|(watcher, end)| *watcher == 5 && *end == views::End::Unwatched));
    let stream = party.streams.iter().find(|stream| stream.watcher == 6).expect("reopened current retry");
    assert_eq!(
        (stream.turn, stream.from, stream.to, stream.text.as_ref(), stream.cut),
        (views::TurnName { attempt, turn: 1, retry: 1 }, 0, 10, b"new prefix".as_slice(), false)
    );
    assert_eq!(world.store.applied, durable, "provisional lifecycle is independent of journal commits");
    party.finish();
}
