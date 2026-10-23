//! The format row and the sizes at start (store.md 5.3, 5.4).
use super::fixtures;
use crate::boundary::{Family, OpenRefusal};
use crate::{codec, format, sizes};
use jig_domain_core as core;
use skein_lib::{Duration, JournalLimits};

// People v11 retains the whole project authority as well as its roles.
const JOURNAL: JournalLimits = JournalLimits { commits: 2, writes: 4, held: 4, now: 4, release: 4 };
fn wanted() -> codec::component::Format {
    format::current(fixtures::APPLICATION.version, fixtures::DEPLOYMENT)
}
fn store() -> skein_kv::Limits {
    skein_kv::Limits {
        key: 1024,
        value: 16_777_216,
        ops: 16,
        commit: 134_217_728,
        queued: 2,
        queued_bytes: 268_435_456,
        budget: 16_384,
        segment: 268_435_456,
        snapshot_after: 134_217_728,
        chunk: 16_778_260,
        page: skein_kv::Page { rows: 2, bytes: 16_778_240 },
        deadline: Duration::from_nanos(1_000_000_000),
    }
}
fn check(store: &skein_kv::Limits) -> Result<sizes::Sizes, sizes::SizeRefusal> {
    sizes::check(&fixtures::CORE_LIMITS, &fixtures::APPLICATION, &JOURNAL, store, 0)
}
#[test]
fn a_format_row_of_another_tasks_version_is_refused_naming_tasks() {
    let wanted = format::current(fixtures::APPLICATION.version, fixtures::DEPLOYMENT);
    let mut found = wanted;
    found.tasks = 5;
    assert_eq!(format::check(&found, &wanted), Err(OpenRefusal::Format { family: Family::Tasks, found: 5, wanted: 9 }));
}
#[test]
fn a_format_row_of_another_deployment_is_refused() {
    let wanted = wanted();
    let found = codec::component::Format { deployment: [9; 16], ..wanted };
    assert_eq!(format::check(&found, &wanted), Err(OpenRefusal::Deployment));
}
#[test]
fn the_current_format_row_is_accepted() {
    assert_eq!(format::check(&wanted(), &wanted()), Ok(()));
}
#[test]
fn another_family_is_refused_naming_the_first_difference() {
    let wanted = wanted();
    for (family, found) in [
        (Family::Component, codec::component::Format { component: 3, core: 6, ..wanted }),
        (Family::Core, codec::component::Format { core: 5, tasks: 4, ..wanted }),
        (Family::People, codec::component::Format { people: 10, notes: 2, ..wanted }),
        (Family::Notes, codec::component::Format { notes: 2, accounts: 2, ..wanted }),
        (Family::Accounts, codec::component::Format { accounts: 2, application: 2, ..wanted }),
        (Family::Application, codec::component::Format { application: 2, deployment: [9; 16], ..wanted }),
    ] {
        assert_eq!(
            format::check(&found, &wanted),
            Err(OpenRefusal::Format {
                family,
                found: match family {
                    Family::Core => 5,
                    Family::People => 10,
                    Family::Component => 3,
                    Family::Tasks | Family::Notes | Family::Accounts | Family::Application => 2,
                },
                wanted: match family {
                    Family::Core => 6,
                    Family::Tasks => 9,
                    Family::People => 11,
                    Family::Component => 2,
                    Family::Notes | Family::Accounts | Family::Application => 1,
                }
            })
        );
    }
}
fn a_key_limit_below_the_largest_note_line_is_refused() {
    let mut limits = store();
    limits.key = 111;
    assert_eq!(check(&limits), Err(sizes::SizeRefusal::Key { needs: 112, has: 111 }));
}
fn a_value_limit_below_the_largest_task_record_is_refused() {
    let mut limits = store();
    limits.value = 1;
    assert_eq!(check(&limits), Err(sizes::SizeRefusal::Value { needs: 369_330, has: 1 }));
}
fn a_value_limit_below_the_largest_core_turn_is_refused() {
    let mut limits = store();
    limits.value = 80_948;
    assert_eq!(check(&limits), Err(sizes::SizeRefusal::Value { needs: 369_330, has: 80_948 }));
}
fn a_page_smaller_than_one_row_is_refused() {
    let mut limits = store();
    limits.page.bytes = 81_060;
    assert_eq!(check(&limits), Err(sizes::SizeRefusal::Page { needs: 369_442, has: 81_060 }));
}
#[test]
fn fixture_limits_fit_their_store() {
    assert_eq!(check(&store()), Ok(sizes::Sizes { key: 112, value: 369_330, ops: 10, commit: 17_620 }));
}
fn too_few_operations_are_refused() {
    let mut limits = store();
    limits.ops = 9;
    assert_eq!(check(&limits), Err(sizes::SizeRefusal::Ops { needs: 10, has: 9 }));
}
fn too_few_commit_bytes_are_refused() {
    let mut limits = store();
    limits.commit = 17_619;
    assert_eq!(check(&limits), Err(sizes::SizeRefusal::Commit { needs: 17_620, has: 17_619 }));
}
fn too_few_in_flight_slots_are_refused() {
    let mut limits = store();
    limits.queued = 1;
    assert_eq!(check(&limits), Err(sizes::SizeRefusal::Queue { needs: 2, has: 1 }));
}
fn a_journal_bound_past_the_queue_is_refused_at_start() {
    let journal = JournalLimits { commits: 3, ..JOURNAL };
    assert_eq!(
        sizes::check(&fixtures::CORE_LIMITS, &fixtures::APPLICATION, &journal, &store(), 0),
        Err(sizes::SizeRefusal::Queue { needs: 3, has: 2 }),
    );
}
fn too_few_queued_bytes_are_refused() {
    let mut limits = store();
    limits.queued_bytes = 35_239;
    assert_eq!(check(&limits), Err(sizes::SizeRefusal::QueuedBytes { needs: 35_240, has: 35_239 }));
}
#[test]
fn a_domain_limit_past_its_family_ceiling_is_refused_at_start() {
    let mut domain = fixtures::CORE_LIMITS;
    domain.notes.body_bytes = u32::MAX;
    assert_eq!(sizes::check(&domain, &fixtures::APPLICATION, &JOURNAL, &store(), 0), Err(sizes::SizeRefusal::Ceiling));
}
#[test]
fn an_application_key_that_cannot_hold_its_tag_is_refused() {
    let application = sizes::Application { key: u32::MAX, ..fixtures::APPLICATION };
    assert_eq!(
        sizes::check(&fixtures::CORE_LIMITS, &application, &JOURNAL, &store(), 0),
        Err(sizes::SizeRefusal::Ceiling)
    );
}
fn codec_limits() -> codec::Limits {
    codec::limits(&fixtures::CORE_LIMITS).expect("fixture bounds")
}
#[test]
fn every_childs_record_dispatches_to_its_own_codec() {
    let limits = codec_limits();
    for row in [
        core::Record::Core(core::CoreRecord::Deployment(fixtures::core_rows::deployment())),
        core::Record::Tasks(jig_domain_tasks::Stored::Live(Box::new(fixtures::task_record()))),
        core::Record::People(jig_domain_people::Stored::ReadResults { party: 7, through: 3 }),
        core::Record::Notes(jig_domain_notes::Record::Entry(fixtures::note_entry())),
    ] {
        let bytes = codec::encode(&row, &limits, &[]).expect("record fits");
        assert_eq!(codec::decode(&row.key(), &bytes, &limits, &[]), Some(row));
    }
}
#[test]
fn each_childs_corrupt_record_is_refused_by_dispatch() {
    let limits = codec_limits();
    for key in [
        core::Key::Core(core::CoreKey::Deployment),
        core::Key::Tasks(jig_domain_tasks::Key::Live(1)),
        core::Key::People(jig_domain_people::Key::Person(1)),
        core::Key::Notes(jig_domain_notes::Key::Entry { name: 1 }),
    ] {
        assert_eq!(codec::decode(&key, &[], &limits, &[]), None);
    }
}
#[test]
fn the_largest_note_line_key_reaches_its_calculated_bound() {
    let mut segments = skein_lib::List::with_capacity(fixtures::NOTES_LIMITS.pattern_bytes);
    for _ in 0..fixtures::NOTES_LIMITS.pattern_bytes {
        segments.push(Box::<[u8]>::default()).expect("segment count fits");
    }
    let key = core::Key::Notes(jig_domain_notes::Key::Line {
        scope: jig_domain_notes::Scope::Resources {
            project: u32::MAX,
            connector: u16::MAX,
            pattern: jig_domain_notes::Pattern {
                segments: segments.into_boxed(),
                last: jig_domain_notes::Last::Exact(Box::new([0; 16])),
            },
        },
        name: u64::MAX,
    });
    assert_eq!(crate::keys::encode(&key, 112).expect("largest line fits").len(), 112);
    assert_eq!(crate::keys::encode(&key, 111), None);
}
#[test]
fn larger_application_encodings_are_included_in_the_bounds() {
    let application = sizes::Application { version: 1, key: 256, value: 9_000_000 };
    assert_eq!(
        sizes::check(&fixtures::CORE_LIMITS, &application, &JOURNAL, &store(), 0),
        Ok(sizes::Sizes { key: 257, value: 9_000_000, ops: 10, commit: 19_070 }),
    );
}
#[test]
fn a_journal_whose_operation_count_overflows_is_refused() {
    let journal = JournalLimits { writes: u32::MAX, ..JOURNAL };
    assert_eq!(
        sizes::check(&fixtures::CORE_LIMITS, &fixtures::APPLICATION, &journal, &store(), 0),
        Err(sizes::SizeRefusal::Ceiling),
    );
}

#[test]
fn a_map_budget_past_the_commit_envelope_is_refused() {
    for (label, budget, expected) in [
        ("exact_u32_commit", 4_294_966_059, Ok(sizes::Sizes { key: 112, value: 369_330, ops: 10, commit: u32::MAX })),
        ("one_byte_past_u32", 4_294_966_060, Err(sizes::SizeRefusal::Ceiling)),
        ("u64_sum_overflows", u64::MAX, Err(sizes::SizeRefusal::Ceiling)),
    ] {
        let limits = skein_kv::Limits { budget, commit: u32::MAX, queued_bytes: u64::MAX, ..store() };
        assert_eq!(check(&limits), expected, "{label}");
    }
}

#[test]
fn store_limits_below_their_required_sizes_are_refused() {
    table([
        ("a_key_limit_below_the_largest_note_line_is_refused", a_key_limit_below_the_largest_note_line_is_refused),
        (
            "a_value_limit_below_the_largest_task_record_is_refused",
            a_value_limit_below_the_largest_task_record_is_refused,
        ),
        ("a_value_limit_below_the_largest_core_turn_is_refused", a_value_limit_below_the_largest_core_turn_is_refused),
        ("a_page_smaller_than_one_row_is_refused", a_page_smaller_than_one_row_is_refused),
        ("too_few_operations_are_refused", too_few_operations_are_refused),
        ("too_few_commit_bytes_are_refused", too_few_commit_bytes_are_refused),
        ("too_few_in_flight_slots_are_refused", too_few_in_flight_slots_are_refused),
        ("a_journal_bound_past_the_queue_is_refused_at_start", a_journal_bound_past_the_queue_is_refused_at_start),
        ("too_few_queued_bytes_are_refused", too_few_queued_bytes_are_refused),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    let mut failures = [None; N];
    let mut passed = true;
    for ((name, case), failure) in cases.into_iter().zip(&mut failures) {
        if std::panic::catch_unwind(case).is_err() {
            *failure = Some(name);
            passed = false;
        }
    }
    assert!(passed, "{failures:?}");
}
