//! Full golden rows decode within their heap worst cases (skein's codec.md, 6).
use jig_protocol_store::{component_v1, core_v3, notes_v1, people_v2, tasks_v3};
use skein_heap::{Counting, Meter};
use skein_lib::Reader;

#[global_allocator]
static HEAP: Counting = Counting;

fn check<T>(family: &str, record: &str, bound: u64, decode: impl FnOnce(&[u8]) -> T) {
    let version = match family {
        "core" | "tasks" => 3,
        "people" => 2,
        _ => 1,
    };
    let golden = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../crates/jig-protocol-store/golden/{family}/v{version}/record_{record}_full.bin")),
    )
    .expect("full golden row");
    let meter = Meter::new();
    meter.start();
    let decoded = decode(&golden);
    let measured = meter.end();
    assert!(measured.peak() <= bound, "{family}/{record} decoding peaks within the generated bound");
    drop(decoded);
    assert_eq!(meter.held(), 0, "the decoded row's heap is released");
}

#[test]
fn a_full_notes_row_decodes_within_its_heap_worst_case() {
    let bound = notes_v1::Row::worst_case_heap(&notes_v1::CEILINGS).expect("notes heap bound");
    check("notes", "row", bound, |golden| {
        notes_v1::Row::decode(&notes_v1::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn a_full_tasks_row_decodes_within_its_heap_worst_case() {
    let bound = tasks_v3::Row::worst_case_heap(&tasks_v3::CEILINGS).expect("tasks heap bound");
    check("tasks", "row", bound, |golden| {
        tasks_v3::Row::decode(&tasks_v3::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn a_full_core_row_decodes_within_its_heap_worst_case() {
    let bound = core_v3::Row::worst_case_heap(&core_v3::CEILINGS).expect("core heap bound");
    check("core", "row", bound, |golden| {
        core_v3::Row::decode(&core_v3::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn a_full_people_row_decodes_within_its_heap_worst_case() {
    let bound = people_v2::Row::worst_case_heap(&people_v2::CEILINGS).expect("people heap bound");
    check("people", "row", bound, |golden| {
        people_v2::Row::decode(&people_v2::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn each_full_component_row_decodes_within_its_heap_worst_case() {
    check(
        "component",
        "format",
        component_v1::Format::worst_case_heap(&component_v1::CEILINGS).expect("format bound"),
        |golden| component_v1::Format::decode(&component_v1::CEILINGS, &mut Reader::new(golden)).expect("full format"),
    );
    check(
        "component",
        "transcript",
        component_v1::Transcript::worst_case_heap(&component_v1::CEILINGS).expect("transcript bound"),
        |golden| {
            component_v1::Transcript::decode(&component_v1::CEILINGS, &mut Reader::new(golden))
                .expect("full transcript")
        },
    );
    check(
        "component",
        "attempt",
        component_v1::Attempt::worst_case_heap(&component_v1::CEILINGS).expect("attempt bound"),
        |golden| {
            component_v1::Attempt::decode(&component_v1::CEILINGS, &mut Reader::new(golden)).expect("full attempt")
        },
    );
}
