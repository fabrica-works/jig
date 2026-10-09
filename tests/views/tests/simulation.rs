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
    assert_eq!(world.referee.deliveries(), 2);
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

#[test]
fn a_slow_watcher_sees_fewer_deliveries_and_never_a_gap() {
    let mut world = World::default();
    world.watch(1, 7);
    world.watch(2, 7);
    world.delivered(1);
    world.delivered(2);
    let mut fast = 1;
    let mut slow = 1;
    for _ in 0..10 {
        let changed = world.changed(7);
        fast += changed
            .iter()
            .filter(|request| matches!(request, Request::Deliver { watcher } if *watcher == Token::new(1)))
            .count();
        slow += changed
            .iter()
            .filter(|request| matches!(request, Request::Deliver { watcher } if *watcher == Token::new(2)))
            .count();
        world.delivered(1);
    }
    slow += world.delivered(2).iter().filter(|request| matches!(request, Request::Deliver { .. })).count();
    assert!(fast > slow);
    assert_eq!(slow, 3);
    world.settle();
}
#[test]
fn a_run_s_text_is_read_from_each_watcher_s_offset() {
    use jig_core_views::{Follow, TurnName};
    let mut world = World::default();
    let turn = TurnName { attempt: 1, turn: 1 };
    world.watch_from(1, 7, Some(Follow { run: Token::new(7), turn: Some(turn), offset: 0 }));
    world.watch_from(2, 7, Some(Follow { run: Token::new(7), turn: Some(turn), offset: 2 }));
    let requests = world.text(7, turn, b"abcdef", None);
    assert!(requests.iter().any(|request| matches!(request, Request::Stream { watcher, from: 2, text, .. } if *watcher == Token::new(2) && text.as_ref() == b"cdef")));
    world.stream_delivered(2);
    world.text(7, turn, b"ghi", None);
    world.stream_delivered(1);
    world.settle();
}
#[test]
fn a_new_turn_s_text_never_resets_an_earlier_one() {
    use jig_core_views::{Follow, TurnName};
    let mut world = World::default();
    world.watch_from(1, 7, Some(Follow { run: Token::new(7), turn: None, offset: 0 }));
    world.text(7, TurnName { attempt: 1, turn: 1 }, b"earlier", None);
    assert!(world.text(7, TurnName { attempt: 1, turn: 2 }, b"later", None).is_empty());
    let output = world.stream_delivered(1);
    assert!(
        matches!(output.as_slice(), [Request::Stream { turn, from: 0, to: 5, .. }] if *turn == TurnName { attempt: 1, turn: 2 })
    );
    assert!(world.text(7, TurnName { attempt: 1, turn: 1 }, b"stale", None).is_empty());
    world.settle();
}
#[test]
fn bounded_stream_text_says_which_prefix_was_cut() {
    use jig_core_views::{Follow, TurnName};
    let mut world = World::default();
    let turn = TurnName { attempt: 1, turn: 1 };
    world.watch_from(1, 7, Some(Follow { run: Token::new(7), turn: None, offset: 0 }));
    world.text(7, turn, b"first", None);
    for _ in 0..4 {
        world.text(7, turn, b"abcdefghijklmnop", None);
    }
    let output = world.stream_delivered(1);
    assert!(
        matches!(output.as_slice(), [Request::Stream { from: 37, to: 69, text, cut: true, .. }] if text.len() == 32)
    );
    world.settle();
}
#[test]
fn replay_and_dropped_facts_change_no_delivery() {
    fn run(seed: u64) -> Vec<String> {
        let mut world = World::seeded(jig_views_world::LIMITS, seed);
        world.watch(1, 7);
        world.changed(7);
        world.settle();
        world.log().to_vec()
    }
    assert_eq!(run(19), run(19));
    let mut quiet = jig_views_world::LIMITS;
    quiet.facts = 0;
    let mut a = World::new(jig_views_world::LIMITS);
    let mut b = World::new(quiet);
    assert_eq!(a.watch(1, 7), b.watch(1, 7));
    assert_eq!(a.changed(7), b.changed(7));
    assert_eq!(a.delivered(1), b.delivered(1));
}
