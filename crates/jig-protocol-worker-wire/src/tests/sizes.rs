use super::fixtures::{self, SIZES_LARGE, SIZES_ONE_SLOT};
use crate::Kind;
use crate::sizes::{Cap, Sizes, WorkerCap, engine_cap, largest_body, limits, worker_cap};

#[test]
fn the_largest_turn_kept_body_is_twenty_bytes() {
    // Task u64 + attempt u64 + number u32.
    assert_eq!(largest_body(&SIZES_ONE_SLOT, Kind::TurnKept), Some(20));
}

#[test]
fn the_engine_cap_for_one_slot_is_the_hand_summed_bytes() {
    // Assignment346, message64, relayed58, kept20, ack16, cancel16, grants2*77;
    // eight frame headers add64:354+72+66+28+24+24+170=738.
    assert_eq!(engine_cap(&SIZES_ONE_SLOT), Some(Cap { bytes: 738, frames: 8 }));
}

#[test]
fn the_worker_cap_for_one_slot_is_the_hand_summed_bytes() {
    // Hello125 + turn71 + answer136 + call85 + rejected28 + exhausted28;
    // six headers48 + fact reserve16 =537. Reserve16 fits no25-byte fact.
    assert_eq!(worker_cap(&SIZES_ONE_SLOT), Some(WorkerCap { cap: Cap { bytes: 537, frames: 6 }, facts: 16 }));
}

#[test]
fn the_worker_cap_reserves_the_facts_share_last() {
    let sizes = Sizes { facts: 75, ..SIZES_ONE_SLOT };
    // Mandatory frames521 bytes + reserve75 =596; three shortest facts25 each.
    assert_eq!(worker_cap(&sizes), Some(WorkerCap { cap: Cap { bytes: 596, frames: 9 }, facts: 75 }));
}

#[test]
fn the_largest_turn_body_matches_a_measured_fullest_turn() {
    let turn = fixtures::fullest_turn();
    // Fixed28 + read9 + window1 + refusal list13 + body20 =71.
    assert_eq!(turn.measure(), 71);
    assert_eq!(largest_body(&SIZES_ONE_SLOT, Kind::Turn), Some(71));
}

#[test]
fn the_largest_assignment_body_matches_a_measured_fullest_assignment() {
    let assignment = fixtures::fullest_assignment();
    // Fixed32 + workspace29 + save1 + charter20 + transcript20 + calls66
    // + messages52 + grants126 =346.
    assert_eq!(assignment.measure(), 346);
    assert_eq!(largest_body(&SIZES_ONE_SLOT, Kind::Assignment), Some(346));
}

#[test]
fn sizes_past_a_ceiling_give_no_codec_limits() {
    for (field, sizes) in [
        ("charter", Sizes { charter: 67_108_865, ..SIZES_ONE_SLOT }),
        ("transcript", Sizes { transcript: 8_388_609, ..SIZES_ONE_SLOT }),
        ("turn", Sizes { turn: 8_388_609, ..SIZES_ONE_SLOT }),
        ("message", Sizes { message: 1_048_577, ..SIZES_ONE_SLOT }),
        ("sender", Sizes { sender: 1025, ..SIZES_ONE_SLOT }),
        ("call name", Sizes { call_name: 65, ..SIZES_ONE_SLOT }),
        ("tool name", Sizes { tool_name: 65, ..SIZES_ONE_SLOT }),
        ("call input", Sizes { call_input: 1_048_577, ..SIZES_ONE_SLOT }),
        ("answer", Sizes { answer: 1_048_577, ..SIZES_ONE_SLOT }),
        ("result", Sizes { result: 67_108_865, ..SIZES_ONE_SLOT }),
        ("detail", Sizes { detail: 4097, ..SIZES_ONE_SLOT }),
        ("grant", Sizes { grant: 8193, ..SIZES_ONE_SLOT }),
        ("account id", Sizes { account_id: 257, ..SIZES_ONE_SLOT }),
        ("name", Sizes { name: 257, ..SIZES_ONE_SLOT }),
        ("secret", Sizes { secret: 1025, ..SIZES_ONE_SLOT }),
        ("model name", Sizes { model_name: 129, ..SIZES_ONE_SLOT }),
        ("items", Sizes { items: 1_048_577, ..SIZES_ONE_SLOT }),
        ("left", Sizes { left: 65_537, ..SIZES_ONE_SLOT }),
        ("saved", Sizes { saved: 65_537, ..SIZES_ONE_SLOT }),
        ("evidence", Sizes { evidence: 1_048_577, ..SIZES_ONE_SLOT }),
        ("resource", Sizes { resource: 1025, ..SIZES_ONE_SLOT }),
        ("slots", Sizes { slots: 65_537, ..SIZES_ONE_SLOT }),
        ("inbox", Sizes { inbox: 1025, ..SIZES_ONE_SLOT }),
        ("answered", Sizes { answered: 1025, ..SIZES_ONE_SLOT }),
        ("models", Sizes { models: 65, ..SIZES_ONE_SLOT }),
        ("workstreams", Sizes { workstreams: 65_537, ..SIZES_ONE_SLOT }),
        ("accounts", Sizes { accounts: 64, ..SIZES_ONE_SLOT }),
        ("credentials", Sizes { credentials: 64, ..SIZES_ONE_SLOT }),
    ] {
        assert_eq!(limits(&sizes), None, "{field}");
    }
}

#[test]
fn a_cap_past_a_u32_is_none() {
    assert!(limits(&SIZES_LARGE).is_some());
    assert_eq!(engine_cap(&SIZES_LARGE), None);
    assert_eq!(worker_cap(&SIZES_LARGE), None);
}

#[test]
fn a_grant_count_sum_that_overflows_has_no_codec_limits() {
    let sizes = Sizes { accounts: u32::MAX, credentials: 1, ..SIZES_ONE_SLOT };
    assert_eq!(limits(&sizes), None);
}

#[test]
fn the_facts_reserve_that_overflows_the_cap_is_refused() {
    let sizes = Sizes { facts: u32::MAX, ..SIZES_ONE_SLOT };
    assert_eq!(worker_cap(&sizes), None);
}
