use jig_domain_views::{Event, Follow, Request, TurnName};
use jig_views_world::World;
use skein_lib::Token;
#[test]
fn a_second_delivery_in_flight_fails_the_referee() {
    assert!(
        std::panic::catch_unwind(|| {
            let mut world = World::default();
            world.watch(1, 7);
            world.referee.saw(&[Request::Deliver { watcher: Token::new(1) }]);
        })
        .is_err()
    );
}
#[test]
fn ending_before_the_delivery_terminal_fails_the_referee() {
    assert!(
        std::panic::catch_unwind(|| {
            let mut world = World::default();
            world.watch(1, 7);
            world.referee.saw(&[Request::Ended { watcher: Token::new(1), end: jig_domain_views::End::Gone }]);
        })
        .is_err()
    );
}
#[test]
fn a_document_without_an_owner_construction_fails_the_referee() {
    assert!(
        std::panic::catch_unwind(|| {
            let mut world = World::default();
            world.watch(1, 7);
            world.delivered(1);
            world.referee.saw(&[Request::Deliver { watcher: Token::new(1) }]);
        })
        .is_err()
    );
}
#[test]
fn a_missed_final_change_fails_the_referee() {
    assert!(
        std::panic::catch_unwind(|| {
            let mut world = World::default();
            world.watch(1, 7);
            world.changed(7);
            world.referee.assert_caught_up();
        })
        .is_err()
    );
}
#[test]
fn corrupted_provisional_bytes_fail_the_referee() {
    assert!(
        std::panic::catch_unwind(|| {
            let mut world = World::default();
            let turn = TurnName { attempt: 1, turn: 1, retry: 0 };
            world.watch_from(1, 7, Some(Follow { run: Token::new(7), turn: None, offset: 0 }));
            world.text(7, turn, b"abc", None);
            world.stream_delivered(1);
            world.referee.before(&Event::Text {
                run: Token::new(7),
                turn,
                offset: 3,
                text: b"def".as_slice().into(),
                call: None,
            });
            world.referee.saw(&[Request::Stream {
                watcher: Token::new(1),
                turn,
                from: 3,
                to: 6,
                text: b"xyz".as_slice().into(),
                cut: false,
                call: None,
            }]);
        })
        .is_err()
    );
}
