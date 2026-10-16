use super::fixtures;
use crate::translate::pages;
use jig_client_protocol_wire::v8 as w;
use jig_domain_core::reads;

fn activity(read: &reads::Page, limits: &w::Limits) -> Result<w::ReadValue, w::Problem> {
    pages::page(&reads::Read::Activity { task: 7, before: None }, read, limits)
}
#[test]
fn an_activity_page_carries_each_turn_s_bytes() {
    let limits = fixtures::wire_limits();
    let source = fixtures::activity_page();
    assert_eq!(
        activity(&source, &limits),
        Ok(w::ReadValue::Activity(jig_client_test_records::carried_activity_page()))
    );
    for (label, limits, path) in [
        ("turn bytes", w::Limits { turn_carried_bytes: 2, ..limits }, w::Path::TurnCarriedBytes),
        ("page pieces", w::Limits { activity_page_rows: 0, ..limits }, w::Path::ActivityPageRows),
        ("opaque cursor", w::Limits { cursor_value: 1, ..limits }, w::Path::CursorValue),
    ] {
        assert_eq!(activity(&source, &limits), Err(w::Problem { path, reason: skein_codec::Reason::Bound }), "{label}");
    }
}
#[test]
fn a_turn_past_the_page_crosses_as_its_size() {
    let limits = fixtures::wire_limits();
    let source = fixtures::oversized_turn_page();
    assert_eq!(
        activity(&source, &limits),
        Ok(w::ReadValue::Activity(jig_client_test_records::oversized_activity_page()))
    );
}
#[test]
fn a_history_page_keeps_its_cut_flag() {
    let limits = fixtures::wire_limits();
    let mut source = fixtures::history_page();
    let literal = jig_client_test_records::mixed_history_page();
    for (label, cut) in [("cut", true), ("whole", false)] {
        source.cut = cut;
        let mut expected = literal.clone().into_parts();
        expected.cut = cut;
        assert_eq!(
            pages::page(&reads::Read::History { task: 7, subtree: false, from: None }, &source, &limits),
            Ok(w::ReadValue::History(w::HistoryPage::new(&limits, expected).expect(label))),
            "{label}"
        );
    }
}
#[test]
fn an_inbox_page_keeps_its_two_groups_apart() {
    let limits = fixtures::wire_limits();
    let needs = fixtures::inbox_page(false);
    let updates = fixtures::inbox_page(true);
    let records = jig_client_test_records::selected_inbox_pages();
    for (label, group, source, record) in [
        ("needs", reads::Group::Needs, &needs, records[0].clone()),
        ("updates", reads::Group::Updates, &updates, records[1].clone()),
    ] {
        let read = reads::Read::Inbox { group, filter: reads::Filter { project: None, goal: None }, from: None };
        assert_eq!(pages::page(&read, source, &limits), Ok(w::ReadValue::OwnerInbox(record)), "{label}");
    }
}
#[test]
fn a_notes_page_becomes_its_record() {
    let limits = fixtures::wire_limits();
    let source = fixtures::notes_page();
    let read = reads::Read::Notes { project: 13, from: None };
    assert_eq!(
        pages::page(&read, &source, &limits),
        Ok(w::ReadValue::Notes(jig_client_test_records::scoped_notes_page()))
    );
}
