use crate::{busy::Busy, tests::fixtures};
use skein_lib::{Duration, Time, Token};
const RUN: Token = Token::new(7);
const ATTEMPT: Token = Token::new(2);
#[test]
fn a_turn_kept_drops_its_busy_entry() {
    let mut busy = Busy::new(&fixtures::LIMITS, 1);
    assert!(busy.add(Time::ZERO, RUN, ATTEMPT, 1));
    busy.kept(RUN, ATTEMPT, 1);
    assert_eq!(busy.next(), None);
    assert_eq!(busy.expire(Time::ZERO.saturating_add(Duration::from_secs(1))), None);
}
#[test]
fn a_busy_turns_resend_falls_within_its_jitter() {
    let mut busy = Busy::new(&fixtures::LIMITS, 1);
    assert!(busy.add(Time::ZERO, RUN, ATTEMPT, 1));
    let at = busy.next().expect("deadline");
    let span = at.saturating_since(Time::ZERO);
    assert!(span >= Duration::from_millis(100) && span <= Duration::from_millis(200));
    assert_eq!(busy.expire(at), Some((RUN, ATTEMPT, 1)));
    assert_eq!(busy.next(), None);
}
#[test]
fn acknowledging_an_attempt_drops_only_its_busy_turns() {
    let mut limits = fixtures::LIMITS;
    limits.sizes.slots = 2;
    limits.sizes.turns = 2;
    let mut busy = Busy::new(&limits, 1);
    for turn in [1, 2] {
        assert!(busy.add(Time::ZERO, RUN, ATTEMPT, turn));
    }
    assert!(busy.add(Time::ZERO, RUN, Token::new(3), 1));
    busy.acknowledged(RUN, ATTEMPT);
    assert_eq!(busy.expire(Time::ZERO.saturating_add(Duration::from_secs(1))), Some((RUN, Token::new(3), 1)));
    assert_eq!(busy.next(), None);
}
#[test]
fn busy_turns_fill_the_table_and_loss_clears_it() {
    let mut busy = Busy::new(&fixtures::LIMITS, 1);
    assert!(busy.add(Time::ZERO, RUN, ATTEMPT, 1));
    assert!(busy.add(Time::ZERO, RUN, ATTEMPT, 1));
    assert!(!busy.add(Time::ZERO, RUN, ATTEMPT, 2));
    busy.clear();
    assert_eq!(busy.next(), None);
    assert!(busy.add(Time::ZERO, RUN, ATTEMPT, 2));
}
