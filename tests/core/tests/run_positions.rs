use jig_core as core;
use jig_core_views as views;
use jig_core_world::watches::{open, world};
use jig_test_domain as root;
use skein_lib::Token;

#[test]
fn a_reconnected_run_watch_receives_only_its_unacknowledged_suffix() {
    let mut world = world(769, true);
    let assignment = world.assigned.as_ref().expect("live chat");
    let (task, attempt) = (assignment.task, assignment.attempt);
    open(&mut world, 5, core::documents::Subject::Run { task });
    let durable = world.store.applied;
    world.send(root::Event::Text { task, attempt, turn: 1, offset: 0, text: b"abcdef".as_slice().into() });
    let turn = views::TurnName { attempt, turn: 1 };
    for (watcher, recalled) in [(6, turn), (7, views::TurnName { attempt, turn: 2 })] {
        world.send(root::Event::Core(core::Event::WatchRun {
            watcher: Token::new(watcher),
            sign_in: world.sign_in.expect("signed in"),
            task,
            turn: Some(recalled),
            offset: 2,
        }));
    }
    let streams = &world.watches.as_ref().expect("independent party").streams;
    let resumed = streams.iter().find(|stream| stream.watcher == 6).expect("resumed stream");
    assert_eq!(
        (resumed.turn, resumed.from, resumed.to, resumed.text.as_ref(), resumed.cut),
        (turn, 2, 6, b"cdef".as_slice(), false)
    );
    let different = streams.iter().find(|stream| stream.watcher == 7).expect("different recalled turn");
    assert_eq!(
        (different.turn, different.from, different.to, different.text.as_ref()),
        (turn, 0, 6, b"abcdef".as_slice())
    );
    assert_eq!(world.store.applied, durable, "watch admission and provisional suffixes decide nothing");
    world.watches.as_ref().expect("independent party").finish();
}
