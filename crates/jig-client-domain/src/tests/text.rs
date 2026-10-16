use super::fixtures::{
    Client, FIRST_RETRY, FIRST_TURN, INITIAL_TASK_WATCH, NEXT_RETRY, NEXT_TURN, RUN, RUN_ATTEMPT, SUBJECT_TASK,
    run_subject,
};
use crate::{
    Down, Told,
    contract::{ProvisionalTurnName, Subject, WatchEnd},
};
use skein_lib::{Duration, Time};

const FIRST_NAME: ProvisionalTurnName =
    ProvisionalTurnName { attempt: RUN_ATTEMPT, number: FIRST_TURN, retry: FIRST_RETRY };
const NEXT_NAME: ProvisionalTurnName = ProvisionalTurnName { number: NEXT_TURN, ..FIRST_NAME };
const RETRIED: ProvisionalTurnName = ProvisionalTurnName { retry: NEXT_RETRY, ..FIRST_NAME };
const STREAMED: &[Told] = &[Told::Changed { handle: RUN }, Told::Text { handle: RUN }];
#[derive(Debug)]
struct Cut {
    case: &'static str,
    first: &'static [u8],
    next: &'static [u8],
    expected: &'static [u8],
    from: u32,
}
const CUTS: &[Cut] = &[
    Cut { case: "ASCII tail", first: b"ab", next: b"cdef", expected: b"cdef", from: 2 },
    Cut { case: "UTF-8 tail", first: "aé".as_bytes(), next: "界".as_bytes(), expected: "界".as_bytes(), from: 3 },
];

#[test]
fn a_run_s_text_is_held_past_its_offset_and_told() {
    let mut c = Client::running();
    c.run(FIRST_TURN, FIRST_RETRY, 0, b"ab");
    assert_eq!(c.told().as_ref(), STREAMED);
    c.run(FIRST_TURN, FIRST_RETRY, 1, b"bc");
    let text = c.domain.text(RUN).unwrap();
    assert_eq!(text.text.bytes(), b"abc");
    assert_eq!((text.from, text.through), (0, 3));
    assert!(!text.cut);
    assert_eq!(c.told().as_ref(), STREAMED);
    c.run(FIRST_TURN, FIRST_RETRY, 5, b"x");
    assert_eq!(c.domain.text(RUN).unwrap().text.bytes(), b"abc");
    assert!(c.told().is_empty());
}

#[test]
fn a_later_turn_s_text_never_resets_an_earlier_one() {
    let mut c = Client::running();
    c.run(FIRST_TURN, FIRST_RETRY, 0, b"ab");
    let text = c.domain.text(RUN).unwrap();
    assert_eq!((text.turn, text.text.bytes()), (FIRST_NAME, b"ab".as_slice()));
    assert_eq!(c.told().as_ref(), STREAMED);
    c.run(NEXT_TURN, FIRST_RETRY, 0, b"c");
    let text = c.domain.text(RUN).unwrap();
    assert_eq!((text.turn, text.text.bytes(), text.from), (NEXT_NAME, b"c".as_slice(), 0));
    assert_eq!(c.told().as_ref(), STREAMED);
    c.run(FIRST_TURN, FIRST_RETRY, 2, b"d");
    let text = c.domain.text(RUN).unwrap();
    assert_eq!((text.turn, text.text.bytes(), text.from), (NEXT_NAME, b"c".as_slice(), 0));
    assert!(c.told().is_empty());
}

#[test]
fn a_later_retry_replaces_the_turn_s_earlier_text() {
    let mut c = Client::running();
    c.run(FIRST_TURN, FIRST_RETRY, 0, b"ab");
    c.told();
    c.run(FIRST_TURN, NEXT_RETRY, 3, b"new");
    let text = c.domain.text(RUN).unwrap();
    assert_eq!((text.turn, text.text.bytes(), text.from), (RETRIED, b"new".as_slice(), 3));
    assert!(text.cut);
    assert_eq!(c.told().as_ref(), STREAMED);
    c.run(FIRST_TURN, FIRST_RETRY, 2, b"d");
    let text = c.domain.text(RUN).unwrap();
    assert_eq!((text.turn, text.text.bytes(), text.from), (RETRIED, b"new".as_slice(), 3));
    assert!(c.told().is_empty());
}

#[test]
fn a_dropped_run_watch_continues_from_the_offset_held() {
    let mut c = Client::running();
    c.run(FIRST_TURN, FIRST_RETRY, 0, b"abc");
    c.told();
    c.end(c.name(run_subject()), WatchEnd::Dropped);
    c.told();
    c.at(Time::ZERO.saturating_add(Duration::from_millis(80)));
    let downs = c.downs();
    let [Down::Continue { watch, subject }] = downs.as_ref() else {
        panic!("continued run");
    };
    assert_ne!(*watch, INITIAL_TASK_WATCH);
    assert_eq!(*subject, Subject::Run { task: SUBJECT_TASK, turn: Some(FIRST_NAME), offset: 3 });
    c.run(FIRST_TURN, FIRST_RETRY, 3, b"d");
    assert_eq!(c.domain.text(RUN).unwrap().text.bytes(), b"abcd");
    assert_eq!(c.told().as_ref(), STREAMED);
}

#[test]
fn provisional_text_from_before_a_restart_is_dropped_at_the_next_delivery() {
    let mut c = Client::running();
    c.run(FIRST_TURN, FIRST_RETRY, 0, b"old text");
    c.told();
    c.end(c.name(run_subject()), WatchEnd::Stopping);
    c.told();
    assert_eq!(c.domain.text(RUN).unwrap().text.bytes(), b"old text");
    c.at(Time::ZERO.saturating_add(Duration::from_millis(80)));
    c.downs();
    c.run(FIRST_TURN, FIRST_RETRY, 0, b"new");
    let text = c.domain.text(RUN).unwrap();
    assert_eq!((text.text.bytes(), text.from, text.through, text.cut), (b"new".as_slice(), 0, 3, false));
    assert_eq!(c.told().as_ref(), STREAMED);
}

#[test]
fn text_past_the_turn_s_bound_is_cut_and_marked() {
    for &Cut { case, first, next, expected, from } in CUTS {
        let mut c = Client::running();
        c.env.limits.turn_bytes = 4;
        c.run(FIRST_TURN, FIRST_RETRY, 0, first);
        c.run(FIRST_TURN, FIRST_RETRY, u32::try_from(first.len()).unwrap(), next);
        let text = c.domain.text(RUN).unwrap();
        assert_eq!(text.text.bytes(), expected, "{case}");
        assert_eq!(text.from, from, "{case}");
        assert!(text.cut, "{case}");
    }
}
