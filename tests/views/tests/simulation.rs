use jig_core_views::{End, Event, Refusal, Request};
use jig_views_world::World;
use skein_lib::Token;
#[test]
fn changes_while_a_delivery_is_in_flight_make_one_next_delivery() {
    let mut world = World::default();
    world.watch(1, 7);
    for _ in 0..10 {
        assert!(world.changed(7).is_empty());
    }
    assert_eq!(world.delivered(1).len(), 2);
    assert!(world.delivered(1).is_empty());
    assert_eq!(world.referee.deliveries, 2);
}
#[test]
fn a_watch_on_a_subject_that_ends_ends_saying_why() {
    let mut world = World::default();
    world.watch(1, 7);
    assert!(world.step(Event::Gone { subject: Token::new(7), why: End::Gone }).is_empty());
    assert_eq!(world.delivered(1), vec![Request::Ended { watcher: Token::new(1), end: End::Gone }]);
}
#[test]
fn watches_past_the_limit_are_refused_busy() {
    let mut world = World::default();
    for watcher in 0..4 {
        world.watch(watcher, 7);
    }
    assert_eq!(world.watch(4, 7), vec![Request::Refused { watcher: Token::new(4), refusal: Refusal::Busy }]);
}
#[test]
fn an_idle_watch_makes_a_changed_document_immediately() {
    let mut world = World::default();
    world.watch(1, 7);
    world.delivered(1);
    assert_eq!(world.changed(7).len(), 2);
    assert!(world.changed(8).is_empty());
}
