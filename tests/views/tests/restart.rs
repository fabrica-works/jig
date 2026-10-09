use jig_core_views::{End, Follow, TurnName};
use jig_views_world::World;
use skein_lib::Token;
#[test]
fn a_restart_drops_every_watch_and_tells_the_parent_every_token_ended() {
    let mut world = World::default();
    for watcher in 1..=4 {
        world.watch_from(watcher, 7, Some(Follow { run: Token::new(7), turn: None, offset: 0 }));
    }
    world.text(7, TurnName { attempt: 1, turn: 1 }, b"old", None);
    world.changed(7);
    let ended = world.restart();
    assert_eq!(ended.len(), 4);
    assert!(ended.iter().all(|(_, end)| *end == End::Stopping));
    assert_eq!(world.referee.live(), 0);
    assert_eq!(world.watch_from(1, 7, Some(Follow { run: Token::new(7), turn: None, offset: 0 })).len(), 3);
    assert_eq!(world.text(7, TurnName { attempt: 2, turn: 1 }, b"new", None).len(), 1);
    world.settle();
}
