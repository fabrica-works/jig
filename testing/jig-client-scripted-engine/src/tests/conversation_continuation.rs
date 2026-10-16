//! Watch continuations are owned scoped replays, independent of more live rows.
use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;
fn line(number: u64, phase: Phase) -> ConversationLine {
    let mut chip = task().chip;
    chip.task = number;
    chip.phase = phase;
    ConversationLine {
        chip,
        project: 1,
        first_words: DocumentText { bytes: text(UTF8_TEXTS[0]), cut: true },
        cached_replies: 7,
        unread: None,
        delegates: 0,
        budget: OwnerBudget {
            numbers: OwnerNumbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
            left: 100,
            funder: Funder::Period { project: 1, period: 3 },
            deadline: None,
        },
        runtime: None,
        seen: 1,
        last_reply: 4,
        changed: None,
    }
}
fn source(values: &[ConversationLine]) -> Script {
    let mut setup = script();
    setup.bounds.rows = 2;
    setup.truth.sources.push(Source {
        party: 1,
        read: Read::Conversations { after: None },
        rows: Rows::Conversations(values.to_vec()),
    });
    setup
}
fn watched(engine: &mut Engine, token: u64) -> Conversations {
    engine.open(Time::ZERO, 11, Token::new(token), Subject::Conversations).unwrap();
    engine
        .poll(Time::ZERO)
        .into_iter()
        .find_map(|up| match up {
            Up::Watch { token: observed, event: WatchEvent::Delivered(Delivery::Conversations(value)) }
                if observed == Token::new(token) =>
            {
                Some(*value)
            }
            Up::Answer { .. }
            | Up::Read { .. }
            | Up::Watch { .. }
            | Up::ApplicationAnswer { .. }
            | Up::ApplicationRead { .. }
            | Up::App { .. }
            | Up::Ended { .. } => None,
        })
        .unwrap()
}
fn read(engine: &mut Engine, sign_in: u64, token: u64, what: Read) -> ReadResult {
    engine.read(Time::ZERO, sign_in, Token::new(token), what).unwrap();
    engine
        .poll(Time::ZERO)
        .into_iter()
        .find_map(|up| match up {
            Up::Read { token: observed, terminal: result } if observed == Token::new(token) => Some(result),
            Up::Answer { .. }
            | Up::Read { .. }
            | Up::Watch { .. }
            | Up::ApplicationAnswer { .. }
            | Up::ApplicationRead { .. }
            | Up::App { .. }
            | Up::Ended { .. } => None,
        })
        .unwrap()
}
fn page(engine: &mut Engine, token: u64, after: Option<Cursor>) -> ConversationPage {
    let ReadResult::Value(ReadValue::Conversations(value)) = read(engine, 11, token, Read::Conversations { after })
    else {
        panic!("owned conversation page");
    };
    *value
}
#[test]
fn watch_owner_replay_keeps_interleaved_source_order_and_every_retained_page() {
    let ended = line(13, Phase::Done);
    let eight = line(8, Phase::Idle);
    let ten = line(10, Phase::Running);
    let twelve = line(12, Phase::Done);
    let mut foreign = line(77, Phase::Done);
    foreign.project = 2;
    let mut engine =
        Engine::new(source(&[ended.clone(), eight.clone(), foreign, ten.clone(), twelve.clone()]), 1).unwrap();
    let watch = watched(&mut engine, 700);
    assert_eq!(watch.rows.as_slice(), &[eight.clone(), ten.clone()]);
    assert!(!watch.more);
    let continuation = watch.after;
    let first = page(&mut engine, 701, Some(continuation.clone()));
    assert_eq!(first.rows.as_slice(), &[ended.clone(), eight.clone()]);
    assert!(first.before.is_none());
    assert!(!first.cut);
    let last = page(&mut engine, 702, first.after);
    assert_eq!(last.rows.as_slice(), &[ten, twelve]);
    assert!(last.after.is_none());
    assert!(last.before.is_some());
    let replay = page(&mut engine, 703, Some(continuation));
    assert_eq!(replay.rows.as_slice(), &[ended.clone(), eight.clone()]);
    let original = page(&mut engine, 704, None);
    assert_eq!(original.rows, replay.rows);
    engine.taken(Token::new(700));
    let fresh = watched(&mut engine, 705);
    let replay = page(&mut engine, 706, Some(fresh.after));
    assert_eq!(replay.rows.as_slice(), &[ended, eight]);
}
#[test]
fn owner_watch_continuations_keep_original_party_query_and_sign_in_admission() {
    let mut engine = Engine::new(source(&[line(8, Phase::Idle), line(13, Phase::Done)]), 1).unwrap();
    let after = watched(&mut engine, 710).after;
    assert_eq!(
        read(&mut engine, 22, 711, Read::Conversations { after: Some(after.clone()) }),
        ReadResult::None(ReadNone::Gone)
    );
    assert_eq!(
        read(&mut engine, 999, 712, Read::Conversations { after: Some(after.clone()) }),
        ReadResult::None(ReadNone::SignedOut)
    );
    assert_eq!(
        read(&mut engine, 11, 713, Read::Goals { project: 1, after: Some(after.clone()) }),
        ReadResult::None(ReadNone::Gone)
    );
    assert_eq!(page(&mut engine, 714, Some(after)).rows.as_slice(), &[line(8, Phase::Idle), line(13, Phase::Done)]);
}
#[test]
fn empty_and_more_live_watch_windows_both_supply_the_owned_replay() {
    for values in [vec![], vec![line(8, Phase::Idle), line(10, Phase::Idle), line(11, Phase::Idle)]] {
        let mut engine = Engine::new(source(&values), 1).unwrap();
        let watched = watched(&mut engine, 720);
        assert_eq!(watched.more, values.len() > 2);
        assert_eq!(watched.rows.len(), u32::try_from(values.len().min(2)).unwrap());
        let received = page(&mut engine, 721, Some(watched.after));
        assert_eq!(received.rows.as_slice(), &values[..values.len().min(2)]);
        assert_eq!(received.after.is_some(), values.len() > 2);
    }
}
#[test]
fn whole_watch_envelope_prices_cursor_bytes_and_refuses_text_or_reserved_row_overflow() {
    let mut bounds = Bounds { text: 32, rows: 2, document_bytes: u64::MAX, ..Bounds::default() };
    for bytes in [b"".as_slice(), [0xff; 32].as_slice()] {
        let value = Delivery::Conversations(Box::new(Conversations {
            rows: list(vec![]),
            more: false,
            after: Cursor::new(Box::from(bytes), 1024).expect("bounded opaque cursor"),
        }));
        let kept = value.clone();
        let heap = u64::try_from(size_of::<Conversations>()).unwrap() + u64::try_from(bytes.len()).unwrap();
        assert_eq!(value.heap(&bounds), Some(heap));
        bounds.document_bytes = u64::try_from(size_of::<Delivery>()).unwrap() + heap;
        assert!(payload::fits(&value, &bounds));
        bounds.document_bytes -= 1;
        assert!(!payload::fits(&value, &bounds));
        assert_eq!(value, kept);
    }
    bounds.document_bytes = u64::MAX;
    let mut value = Conversations {
        rows: list(vec![]),
        more: false,
        after: Cursor::new(Box::from(UTF8_TEXTS[1]), 1024).expect("bounded opaque cursor"),
    };
    assert!(!payload::fits(&value, &bounds));
    value.after = Cursor::new(Box::from(b"opaque".as_slice()), 1024).expect("bounded opaque cursor");
    value.rows = List::with_capacity(3);
    assert!(!payload::fits(&value, &bounds));
}

const UTF8_TEXTS: [&[u8]; 2] = [b"~\0!", &[b'~'; 33]];
