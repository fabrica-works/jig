//! Offered consequences survive the component's record encoding.
use super::fixtures;
use crate::translate;
use jig_client_protocol_wire::v11 as w;
use jig_domain_core::{acts, documents::PhaseCounts};
const RELEASES: [(acts::ReleaseEffect, w::ReleaseEffect); 3] = [
    (acts::ReleaseEffect::Judged, w::ReleaseEffect::Judged),
    (acts::ReleaseEffect::RunAgain, w::ReleaseEffect::RunAgain),
    (acts::ReleaseEffect::Uncertain, w::ReleaseEffect::Uncertain),
];
#[test]
fn a_release_offer_preserves_each_supplied_consequence() {
    let limits = fixtures::wire_limits();
    for (source, expected) in RELEASES {
        let value = translate::task::effect(&acts::Effect::Release(source), &limits).unwrap();
        let w::Effect::Release(value) = value else { panic!("release effect") };
        assert_eq!(value.what(), &expected);
    }
}
#[test]
fn a_cancel_offer_preserves_its_working_census_runs_and_proposals() {
    let limits = fixtures::wire_limits();
    let effect = acts::Effect::Cancel(acts::Impact {
        tasks: PhaseCounts {
            idle: 2,
            starting: 3,
            running: 5,
            backing_off: 7,
            waiting: 11,
            held: 13,
            closing: 19,
            done: 23,
            failed: 29,
            cancelled: 31,
        },
        live_runs: 37,
        proposals: 41,
    });
    let rows = fixtures::list([acts::Offered {
        act: acts::Act::Cancel,
        admission: acts::Admission::Allowed,
        effect: Some(effect),
    }]);
    let bytes =
        translate::task::offered(&fixtures::models(), &rows, &limits, limits.task_acts, w::Path::TaskActs).unwrap();
    let bytes = skein_codec::piece::split(&mut skein_lib::Reader::new(&bytes)).unwrap().unwrap();
    let value = w::Offered::decode(&limits, &mut skein_lib::Reader::new(&bytes)).unwrap();
    assert_eq!(value.act(), &w::ActKind::Cancel);
    let Some(w::Effect::Cancel(value)) = value.effect() else { panic!("cancel effect") };
    let counts = value.tasks();
    assert_eq!((counts.ended(), counts.running(), counts.waiting(), counts.held()), (0, 17, 11, 13));
    assert_eq!((counts.closing(), counts.done(), counts.failed(), counts.cancelled()), (19, 23, 29, 31));
    assert_eq!((value.live_runs(), value.proposals()), (37, 41));
}
