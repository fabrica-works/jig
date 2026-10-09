use jig_core_views::{End, Request};
use jig_views_world::Referee;
use skein_lib::Token;
#[test]
fn a_second_delivery_in_flight_fails_the_referee() {
    assert!(
        std::panic::catch_unwind(|| {
            let mut referee = Referee::default();
            referee.saw(&[
                Request::Watching { watcher: Token::new(1) },
                Request::Deliver { watcher: Token::new(1) },
                Request::Deliver { watcher: Token::new(1) },
            ]);
        })
        .is_err()
    );
}
#[test]
fn ending_before_the_delivery_terminal_fails_the_referee() {
    assert!(
        std::panic::catch_unwind(|| {
            let mut referee = Referee::default();
            referee.saw(&[
                Request::Watching { watcher: Token::new(1) },
                Request::Deliver { watcher: Token::new(1) },
                Request::Ended { watcher: Token::new(1), end: End::Gone },
            ]);
        })
        .is_err()
    );
}
