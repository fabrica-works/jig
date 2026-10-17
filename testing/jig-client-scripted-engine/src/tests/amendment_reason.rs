//! Whole request receiving and durable key equality include the supplied reason.
use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;
fn amendment(reason: &[u8]) -> Ask {
    Ask::Amend {
        project: 1,
        task: 8,
        revision: Some(Revision { value: 1 }),
        reason: text(reason),
        change: Box::new(Amendment {
            addressed: None,
            spec: None,
            instructions: Some(text(b"")),
            wake: None,
            dependencies: None,
            authority: None,
            tracked: None,
        }),
    }
}
#[test]
fn whole_amendment_request_prices_full_reason_and_rejects_text_or_document_overflow() {
    let mut bounds = Bounds { text: 32, rows: 2, document_bytes: u64::MAX, ..Bounds::default() };
    for bytes in [b"".as_slice(), UTF8_TEXTS[0]] {
        let ask = amendment(bytes);
        let before = ask.clone();
        let heap = u64::try_from(size_of::<Amendment>()).unwrap() + u64::try_from(bytes.len()).unwrap();
        assert_eq!(ask.heap(&bounds).unwrap(), heap);
        let total = u64::try_from(size_of::<Ask>()).unwrap() + heap;
        bounds.document_bytes = total;
        assert!(payload::fits(&ask, &bounds));
        bounds.document_bytes = total - 1;
        assert!(!payload::fits(&ask, &bounds));
        assert_eq!(ask, before);
        bounds.document_bytes = u64::MAX;
    }
    assert!(!payload::fits(&amendment(UTF8_TEXTS[1]), &bounds));
}
#[test]
fn durable_amendment_key_replay_keeps_reason_and_changed_reason_is_a_conflict() {
    let mut setup = script();
    let mut row = task();
    row.acts = list(vec![Offered { act: ActKind::Amend, admission: Admission::Allowed }]);
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
    let ask = amendment(UTF8_TEXTS[2]);
    let mut engine = Engine::new(setup, 1).unwrap();
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    send(&mut engine, 1, 12, 1, 2, ask.clone());
    assert!(engine.poll(Time::from_nanos(9_999_999)).is_empty());
    assert!(engine.commits().is_empty());
    let up = engine.poll(Time::from_nanos(10_000_001));
    assert_eq!(up.len(), 2);
    assert_eq!(final_answer(&up[0]), final_answer(&up[1]));
    assert_eq!(engine.commits().len(), 1);
    assert_eq!(engine.commits()[0].ask.as_ref(), Some(&ask));
    assert_eq!(engine.commits()[0].key, Some([1; 16]));
    assert_eq!(engine.commits()[0].durable, Some(Time::from_nanos(10_000_000)));
    send(&mut engine, 20_000_000, 11, 1, 3, amendment(b"different"));
    assert_eq!(
        final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]),
        &Answer::Refused { refusal: Refusal::KeyConflict, at: None }
    );
    send(&mut engine, 20_000_000, 12, 1, 4, ask.clone());
    assert_eq!(final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]), final_answer(&up[0]));
    assert_eq!(engine.commits().len(), 1);
    assert_eq!(engine.commits()[0].ask.as_ref(), Some(&ask));
}

const UTF8_TEXTS: [&[u8]; 3] = [&[b'~'; 32], &[b'~'; 33], b"~\0!"];
