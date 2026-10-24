use jig_protocol_store::worst_case_with_patches;
use jig_store_world::{
    World,
    patches::{BASE, Records, SETS, limits},
    world::Step,
};
use skein_heap::{Counting, Meter, Span};
use std::cell::RefCell;

#[global_allocator]
static HEAP: Counting = Counting;

// Span removes allocations made by the simulator, disk and shell between
// calls; incoming file buffers transfer back into the store's account.
// Outgoing allocations stay charged, conservatively, even after the driver
// frees them: this check cannot underestimate the store's retained heap.
struct Measuring {
    meter: Meter,
    outside: Option<Span>,
    net: i64,
    peak: u64,
    bound: u64,
    steps: u64,
}
thread_local! {
    static MEASURING: RefCell<Option<Measuring>> = const { RefCell::new(None) };
}

fn observe(step: Step) {
    MEASURING.with_borrow_mut(|slot| match step {
        Step::Construct => {
            let bound = worst_case_with_patches(&limits(), &SETS).expect("patching bound");
            let measuring = Measuring { meter: Meter::new(), outside: None, net: 0, peak: 0, bound, steps: 0 };
            measuring.meter.start();
            *slot = Some(measuring);
        }
        Step::Begin { input } => {
            let measuring = slot.as_mut().expect("constructed");
            measuring.net += measuring.outside.take().expect("between steps").end().net;
            measuring.net -= i64::try_from(input).expect("input heap fits");
            measuring.meter.start();
        }
        Step::End => {
            let measuring = slot.as_mut().expect("constructed");
            let measured = measuring.meter.end();
            let bound = u64::try_from(i64::try_from(measuring.bound).expect("finite bound") + measuring.net)
                .expect("positive adjusted bound");
            let own = measuring.meter.check(measured, bound, &"store construction/down/up/fire");
            let own = u64::try_from(i64::try_from(own).expect("finite heap") - measuring.net).expect("store held");
            measuring.peak = measuring.peak.max(own);
            measuring.steps += 1;
            measuring.outside = Some(Span::start());
        }
    });
}

#[test]
fn a_whole_patching_open_stays_within_its_priced_peak() {
    let records = Records::story();
    let mut source = records.committed(920, BASE);
    let items: Vec<_> = source
        .contents()
        .into_iter()
        .map(|(path, bytes)| match bytes {
            Some(bytes) => skein_fake_machine::Item::file(&path, &bytes).mode(0o600),
            None => skein_fake_machine::Item::directory(&path).mode(0o700),
        })
        .collect();
    let mut peaks = Vec::new();
    for sets in [BASE, SETS] {
        let mut world = World::unopened_observed(920, limits(), Some(observe));
        world.lay(&items);
        world.patch_sets(sets);
        assert!(world.open().is_some(), "measured open: {:?}", world.refusal());
        let (peak, steps) = MEASURING.with_borrow(|slot| {
            let measuring = slot.as_ref().expect("measured");
            (measuring.peak, measuring.steps)
        });
        eprintln!("store peak {peak} bytes over {steps} steps");
        peaks.push(peak);
        MEASURING.with_borrow_mut(|slot| {
            *slot = None;
        });
    }
    assert!(peaks[1] > peaks[0], "patching peak must exceed serving: {peaks:?}");
}
