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

#[test]
fn pending_read_memory_includes_its_family_owner() {
    use crate::{Handle, reads::PendingRead};
    use skein_lib::{Map, Token};
    let empty = Limits { reads: 0, ..super::fixtures::limits() };
    let full = super::fixtures::limits();
    let growth = worst_case(&full).unwrap().checked_sub(worst_case(&empty).unwrap()).unwrap();
    let priced = Map::<Handle, PendingRead>::worst_case(full.reads)
        .unwrap()
        .checked_sub(Map::<Handle, PendingRead>::worst_case(0).unwrap())
        .unwrap();
    let token_only = Map::<Handle, Token>::worst_case(full.reads)
        .unwrap()
        .checked_sub(Map::<Handle, Token>::worst_case(0).unwrap())
        .unwrap();
    assert_eq!(growth, priced);
    assert!(growth > token_only);
}

#[test]
fn read_output_overflow_is_refused() {
    assert_eq!(worst_case(&Limits { reads: u32::MAX, ..super::fixtures::limits() }), None);
}
