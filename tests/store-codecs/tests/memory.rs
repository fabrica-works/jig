//! Full golden rows decode within their heap worst cases (skein's codec.md, 6).
use jig_protocol_store::{accounts_v2, component_v2, core_v5, notes_v1, people_v10, tasks_v9};
use skein_heap::{Counting, Meter};
use skein_lib::Reader;

#[global_allocator]
static HEAP: Counting = Counting;

fn check<T>(family: &str, record: &str, bound: u64, decode: impl FnOnce(&[u8]) -> T) {
    let version = match family {
        "core" => 5,
        "tasks" => 9,
        "people" => 10,
        "component" | "accounts" => 2,
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
    let bound = tasks_v9::Row::worst_case_heap(&tasks_v9::CEILINGS).expect("tasks heap bound");
    check("tasks", "row", bound, |golden| {
        tasks_v9::Row::decode(&tasks_v9::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn a_full_core_row_decodes_within_its_heap_worst_case() {
    let bound = core_v5::Row::worst_case_heap(&core_v5::CEILINGS).expect("core heap bound");
    check("core", "row", bound, |golden| {
        core_v5::Row::decode(&core_v5::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn a_full_people_row_decodes_within_its_heap_worst_case() {
    let bound = people_v10::Row::worst_case_heap(&people_v10::CEILINGS).expect("people heap bound");
    check("people", "row", bound, |golden| {
        people_v10::Row::decode(&people_v10::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn each_full_component_row_decodes_within_its_heap_worst_case() {
    check(
        "component",
        "format",
        component_v2::Format::worst_case_heap(&component_v2::CEILINGS).expect("format bound"),
        |golden| component_v2::Format::decode(&component_v2::CEILINGS, &mut Reader::new(golden)).expect("full format"),
    );
    check(
        "component",
        "transcript",
        component_v2::Transcript::worst_case_heap(&component_v2::CEILINGS).expect("transcript bound"),
        |golden| {
            component_v2::Transcript::decode(&component_v2::CEILINGS, &mut Reader::new(golden))
                .expect("full transcript")
        },
    );
    check(
        "component",
        "attempt",
        component_v2::Attempt::worst_case_heap(&component_v2::CEILINGS).expect("attempt bound"),
        |golden| {
            component_v2::Attempt::decode(&component_v2::CEILINGS, &mut Reader::new(golden)).expect("full attempt")
        },
    );
}
#[test]
fn a_full_installation_row_decodes_within_its_heap_worst_case() {
    let bound = accounts_v2::Installation::worst_case_heap(&accounts_v2::CEILINGS).expect("installation bound");
    check("accounts", "installation", bound, |golden| {
        accounts_v2::Installation::decode(&accounts_v2::CEILINGS, &mut Reader::new(golden)).expect("full installation")
    });
}
#[test]
fn a_full_accounts_row_decodes_within_its_heap_worst_case() {
    let bound = accounts_v2::Row::worst_case_heap(&accounts_v2::CEILINGS).expect("accounts heap bound");
    check("accounts", "row", bound, |golden| {
        accounts_v2::Row::decode(&accounts_v2::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn the_longest_accounts_rows_decode_within_the_rows_heap_worst_case() {
    let ceilings = accounts_v2::CEILINGS;
    let bound = accounts_v2::Row::worst_case_heap(&ceilings).expect("accounts heap bound");
    let bytes = |length: u32| vec![0xa5; usize::try_from(length).expect("ceiling fits")].into_boxed_slice();
    let party = accounts_v2::Party::new(&ceilings, accounts_v2::PartyParts { party: u64::MAX }).expect("a party");
    let account = accounts_v2::Account::new(
        &ceilings,
        accounts_v2::AccountParts {
            place: u32::MAX,
            uses: Some({
                let mut uses = skein_lib::List::with_capacity(ceilings.account_uses_some);
                for n in 0..ceilings.account_uses_some {
                    uses.push(accounts_v2::Use::Party(
                        accounts_v2::Party::new(&ceilings, accounts_v2::PartyParts { party: u64::from(n) })
                            .expect("party"),
                    ))
                    .expect("uses fit");
                }
                uses
            }),
            number: u32::MAX,
            provider: u32::MAX,
            holder: accounts_v2::Holder::Party(party),
            subject: bytes(ceilings.account_subject),
            display: bytes(ceilings.account_display),
        },
    )
    .expect("an account at its ceilings");
    let limited =
        accounts_v2::Limited::new(&ceilings, accounts_v2::LimitedParts { reset: Some(u64::MAX) }).expect("a limit");
    let source = accounts_v2::Source::new(
        &ceilings,
        accounts_v2::SourceParts {
            account: u32::MAX,
            number: u32::MAX,
            place: u32::MAX,
            kind: accounts_v2::Kind::Program,
            name: bytes(ceilings.source_name),
            held: Some(accounts_v2::Held::Limited(limited)),
        },
    )
    .expect("a source at its ceiling");
    for value in [accounts_v2::Value::Account(account), accounts_v2::Value::Source(source)] {
        let row = accounts_v2::Row::new(&ceilings, accounts_v2::RowParts { value }).expect("a row");
        let mut writer = skein_lib::Writer::new(usize::try_from(row.measure()).expect("measured length"));
        row.encode(&mut writer).expect("measured room");
        let encoded = writer.finish();
        let meter = Meter::new();
        meter.start();
        let decoded = accounts_v2::Row::decode(&ceilings, &mut Reader::new(&encoded)).expect("the longest row");
        let measured = meter.end();
        assert!(measured.peak() <= bound, "the longest accounts row decodes within the generated bound");
        assert_eq!(decoded, row);
        drop(decoded);
        assert_eq!(meter.held(), 0, "the decoded row's heap is released");
    }
}

#[test]
fn added_dependencies_kept_lines_and_circle_payloads_fit_their_heap_bounds() {
    let limits = tasks_v9::CEILINGS;
    check(
        "tasks",
        "task_definition",
        tasks_v9::TaskDefinition::worst_case_heap(&limits).expect("definition bound"),
        |golden| {
            tasks_v9::TaskDefinition::decode(&limits, &mut Reader::new(golden)).expect("full definition with additions")
        },
    );
    check("tasks", "kept_line", tasks_v9::KeptLine::worst_case_heap(&limits).expect("kept bound"), |golden| {
        tasks_v9::KeptLine::decode(&limits, &mut Reader::new(golden)).expect("full pending list")
    });
    check("tasks", "circle", tasks_v9::Circle::worst_case_heap(&limits).expect("circle bound"), |golden| {
        tasks_v9::Circle::decode(&limits, &mut Reader::new(golden)).expect("full circle")
    });
}
