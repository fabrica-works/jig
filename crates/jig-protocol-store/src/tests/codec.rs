//! The notes family's translation both ways (protocol/store.md 5.1).
use super::fixtures;
use crate::codec::{component, notes};
use jig_domain_notes as domain;

#[test]
fn a_notes_entry_reads_back_as_it_was_saved() {
    let limits = notes::limits(&fixtures::NOTES_LIMITS).expect("within the schema's ceilings");
    let entry = domain::Record::Entry(fixtures::note_entry());
    let bytes = notes::encode(&entry, &limits).expect("within its limits");
    assert_eq!(notes::decode(&bytes, &limits), Ok(entry));
}
#[test]
fn a_resources_line_reads_back_with_its_pattern() {
    let limits = notes::limits(&fixtures::NOTES_LIMITS).expect("within the schema's ceilings");
    let line = domain::Record::Line(fixtures::note_line());
    let bytes = notes::encode(&line, &limits).expect("within its limits");
    assert_eq!(notes::decode(&bytes, &limits), Ok(line));
}
#[test]
fn a_notes_row_of_another_version_does_not_decode() {
    let limits = notes::limits(&fixtures::NOTES_LIMITS).expect("within the schema's ceilings");
    let entry = domain::Record::Entry(fixtures::note_entry());
    let mut bytes = notes::encode(&entry, &limits).expect("within its limits");
    *bytes.get_mut(1).expect("version prefix") = 2;
    assert_eq!(
        notes::decode(&bytes, &limits),
        Err(crate::notes_v1::Problem { path: crate::notes_v1::Path::RowVersion, reason: skein_codec::Reason::Version })
    );
}
#[test]
fn notes_limits_past_the_ceilings_are_refused() {
    for field in 0_u8..4 {
        let mut limits = fixtures::NOTES_LIMITS;
        match field {
            0 => limits.pattern_bytes = u32::MAX,
            1 => limits.description_bytes = u32::MAX,
            2 => limits.body_bytes = u32::MAX,
            _ => limits.references = u32::MAX,
        }
        assert_eq!(notes::limits(&limits), None);
    }
}
#[test]
fn the_format_row_reads_back_with_its_versions_and_deployment() {
    let row =
        component::Format { component: 1, core: 2, tasks: 3, people: 4, notes: 5, application: 6, deployment: [7; 16] };
    let bytes = row.encode().expect("fixed width fields fit");
    assert_eq!(component::Format::decode(&bytes), Ok(row));
}
#[test]
fn a_transcript_place_reads_back_with_its_window() {
    let row = component::Transcript { length: u64::MAX, window: 42 };
    assert_eq!(component::Transcript::decode(&row.encode().expect("fixed width fields fit")), Ok(row));
}
#[test]
fn an_attempt_place_reads_back_with_its_first_turn() {
    let row = component::Attempt { first: u64::MAX };
    assert_eq!(component::Attempt::decode(&row.encode().expect("fixed width fields fit")), Ok(row));
}
#[test]
fn a_notes_pattern_past_its_aggregate_limit_is_refused() {
    let limits = notes::limits(&fixtures::NOTES_LIMITS).expect("within ceilings");
    let mut line = fixtures::note_line();
    line.scope = domain::Scope::Resources {
        project: 3,
        connector: 2,
        pattern: domain::Pattern { segments: Box::new([Box::new([0; 16])]), last: domain::Last::Exact(Box::new([1])) },
    };
    assert_eq!(
        notes::encode(&domain::Record::Line(line), &limits),
        Err(crate::notes_v1::Problem {
            path: crate::notes_v1::Path::PatternSegments,
            reason: skein_codec::Reason::Bound,
        })
    );
}
#[test]
fn a_party_authored_note_reads_back_in_each_scope() {
    let limits = notes::limits(&fixtures::NOTES_LIMITS).expect("within ceilings");
    for scope in [
        domain::Scope::Deployment,
        domain::Scope::Project { project: u32::MAX },
        domain::Scope::Resources {
            project: 3,
            connector: 2,
            pattern: domain::Pattern { segments: Box::new([]), last: domain::Last::Exact(Box::new([0, 255])) },
        },
    ] {
        let mut entry = fixtures::note_entry();
        entry.scope = scope;
        entry.author = domain::Author::Party { party: u64::MAX };
        let row = domain::Record::Entry(entry);
        assert_eq!(notes::decode(&notes::encode(&row, &limits).expect("within bounds"), &limits), Ok(row));
    }
}
#[test]
fn a_decoded_pattern_past_its_aggregate_limit_is_refused() {
    let limits = notes::limits(&fixtures::NOTES_LIMITS).expect("within ceilings");
    let mut line = fixtures::note_line();
    line.scope = domain::Scope::Resources {
        project: 3,
        connector: 2,
        pattern: domain::Pattern { segments: Box::new([Box::new([0; 16])]), last: domain::Last::Exact(Box::new([1])) },
    };
    let mut wider = limits;
    wider.segment_bytes = 17;
    wider.pattern_segments = 17;
    let bytes = notes::encode(&domain::Record::Line(line), &wider).expect("within wider bound");
    assert_eq!(
        notes::decode(&bytes, &limits),
        Err(crate::notes_v1::Problem {
            path: crate::notes_v1::Path::PatternSegments,
            reason: skein_codec::Reason::Bound,
        })
    );
}
