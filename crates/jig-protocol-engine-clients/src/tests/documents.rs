use super::fixtures;
use crate::translate;
use jig_protocol_client_wire::v5 as w;
#[test]
fn a_frame_with_two_projects_becomes_its_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::owner::frame(&fixtures::frame_two_projects(), &limits),
        Ok(jig_test_client_records::frame_two_projects_record())
    );
}
#[test]
fn a_task_with_a_hold_becomes_its_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::task::document(&fixtures::task_with_hold(), &limits),
        Ok(jig_test_client_records::task_with_hold_record())
    );
}
#[test]
fn a_chip_in_each_phase_becomes_its_record() {
    let limits = fixtures::wire_limits();
    let phases = fixtures::phases();
    let records = jig_test_client_records::owner_chip_records();
    let source = fixtures::chip();
    for ((phase, _compact), record) in phases.into_iter().zip(records) {
        let mut chip = source.clone();
        chip.phase = phase;
        assert_eq!(translate::chips::chip(&chip, &limits), Ok(record));
    }
}
#[test]
fn a_frame_past_the_wire_s_bound_is_refused_naming_it() {
    let frame = fixtures::frame_two_projects();
    let limits = fixtures::wire_limits();
    let rows = [
        (w::Limits { frame_projects: 1, ..limits }, w::Path::FrameProjects),
        (w::Limits { frame_charters: 0, ..limits }, w::Path::FrameCharters),
        (w::Limits { document_text_bytes: 1, ..limits }, w::Path::DocumentTextBytes),
    ];
    for (limits, path) in rows {
        assert_eq!(
            translate::owner::frame(&frame, &limits),
            Err(w::Problem { path, reason: skein_codec::Reason::Bound })
        );
    }
}

#[test]
fn a_non_utf8_document_text_fails_translation() {
    let limits = fixtures::wire_limits();
    for bytes in [&[0xc3_u8][..], &[255_u8, b'/'][..]] {
        let value = jig_domain_core::documents::Text { bytes: bytes.into(), cut: true };
        assert_eq!(
            translate::owner::text(&value, &limits),
            Err(w::Problem { path: w::Path::DocumentTextBytes, reason: skein_codec::Reason::Utf8 })
        );
    }
}
