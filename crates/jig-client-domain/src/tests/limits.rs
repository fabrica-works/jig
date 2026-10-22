use crate::{Limits, worst_case};

#[test]
fn turn_text_at_either_receiving_bound_is_valid() {
    let base = super::fixtures::limits();
    // Leave room between the field bound and the existing turn-byte bound.
    let text = base.text.checked_sub(1).unwrap();
    for turn_text in [text, base.turn_bytes] {
        let limits = Limits { text, turn_text, ..base };
        assert!(worst_case(&limits).is_some(), "turn_text={turn_text}");
    }
}

#[test]
fn turn_text_outside_either_receiving_bound_is_refused() {
    let base = super::fixtures::limits();
    for turn_text in [base.text.checked_sub(1).unwrap(), base.turn_bytes.checked_add(1).unwrap()] {
        let limits = Limits { turn_text, ..base };
        assert_eq!(worst_case(&limits), None, "turn_text={turn_text}");
    }
}
