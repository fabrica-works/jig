//! Count only this binary, so ordinary store step tests use their usual allocator.
#[path = "support/frame_reads.rs"]
mod frame_reads;
use frame_reads::{SETS, closed, decode, open, turn};
use jig_protocol_store::{KeptFamily, PatchSet, PatchSets, worst_case, worst_case_with_patches};
use jig_test_patches_jig::frames;
use skein_heap::{Counting, Meter};

#[global_allocator]
static HEAP: Counting = Counting;

// A body needs the held family's name, even when that family keeps no text.
const BODY_ONLY: PatchSets = PatchSets {
    sets: &[
        SETS.sets[0],
        PatchSet { families: &[KeptFamily { texts: &[], ..SETS.sets[1].families[0] }], patches: &[], ..SETS.sets[1] },
    ],
};
const TURN_ONLY: PatchSets = PatchSets { sets: &[SETS.sets[1]] };

#[test]
fn each_serving_machine_and_decode_stay_within_their_priced_heap() {
    let limits = jig_store_world::patches::limits();
    let none = worst_case(&limits).expect("serving without frame machines");
    for (label, sets, body) in [("body and turn", SETS, 5), ("body", BODY_ONLY, 5), ("turn", TURN_ONLY, 6)] {
        // At these limits serving dominates patching, so this difference is
        // exactly the frames' Patcher heaps and their boxed objects.
        let bound =
            worst_case_with_patches(&limits, &sets).expect("priced frames").checked_sub(none).expect("frame price");
        let raw = frames::frame(body, &turn(1, b"hello", b"r", 0));
        let (mut component, env) = closed(sets, &limits);
        let meter = Meter::new();
        meter.start();
        open(&mut component, &env);
        let result = decode(&mut component, &env, &raw, false);
        let measured = meter.end();
        let peak = meter.check(measured, bound, &"serving frames open and decode");
        assert!(result.is_some());
        eprintln!("{label} measured peak {peak} against {bound}");
    }
}
