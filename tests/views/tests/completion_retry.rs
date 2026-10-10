use jig_core_views::{End, Event, Follow, Request, TurnName};
use jig_views_world::World;
use skein_lib::Token;

fn name(retry: u32) -> TurnName {
    TurnName { attempt: 1, turn: 1, retry }
}
fn open(world: &mut World, watcher: u64) -> Vec<Request> {
    world.watch_from(watcher, 7, Some(Follow { run: Token::new(7), turn: None, offset: 0 }))
}

#[test]
fn an_empty_retry_replaces_unwatched_text_at_the_world_stream_boundary() {
    let mut world = World::default();
    assert!(world.text(7, name(0), b"old prefix", None).is_empty());
    assert!(world.text(7, name(1), b"", None).is_empty());
    let opened = open(&mut world, 1);
    assert!(opened.iter().any(|request| matches!(request, Request::Stream {
        turn, from: 0, to: 0, text, cut: false, ..
    } if *turn == name(1) && text.is_empty())));
    world.settle();
    assert!(world.text(7, name(1), b"new prefix", None).iter().any(|request| matches!(request, Request::Stream {
        turn, from: 0, to: 10, text, cut: false, ..
    } if *turn == name(1) && text.as_ref() == b"new prefix")));
    world.settle();
}

#[test]
fn a_worlds_sole_watcher_reopens_from_an_unwatched_retry_without_a_replay_source() {
    let mut world = World::default();
    open(&mut world, 1);
    world.text(7, name(0), b"previous", None);
    world.settle();
    assert_eq!(
        world.step(Event::Unwatch { watcher: Token::new(1) }),
        vec![Request::Ended { watcher: Token::new(1), end: End::Unwatched }]
    );
    assert!(world.text(7, name(1), b"retained", None).is_empty());
    let reopened = open(&mut world, 2);
    assert!(reopened.iter().any(|request| matches!(request, Request::Stream {
        turn, from: 0, to: 8, text, cut: false, ..
    } if *turn == name(1) && text.as_ref() == b"retained")));
    world.settle();
}
