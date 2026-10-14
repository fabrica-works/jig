use super::fixtures;
use crate::translate;
use jig_protocol_client_wire::v7 as w;
fn a_frame_with_two_projects_becomes_its_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::owner::frame(&fixtures::frame_two_projects(), &limits),
        Ok(jig_test_client_records::frame_two_projects_record())
    );
}
fn a_task_with_a_hold_becomes_its_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::task::document(&fixtures::task_with_hold(), &limits),
        Ok(jig_test_client_records::task_with_hold_record())
    );
}
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

fn a_run_writing_with_a_call_becomes_its_run_state() {
    let limits = fixtures::wire_limits();
    let source = fixtures::run_writing();
    assert_eq!(translate::live::run(&source, &limits), Ok(jig_test_client_records::run_writing_record()));
}
fn a_run_text_keeps_its_turn_retry_and_offsets() {
    let limits = fixtures::wire_limits();
    assert_eq!(translate::live::writing(&fixtures::writing(), &limits), Ok(jig_test_client_records::writing_record()));
}
fn an_inbox_with_both_groups_becomes_its_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::inbox::document(&fixtures::inbox_document(), &limits),
        Ok(jig_test_client_records::inbox_both_groups_record())
    );
}
fn a_deployment_document_becomes_its_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::deployment::document(&fixtures::deployment_document(), &limits),
        Ok(jig_test_client_records::deployment_rows_record())
    );
}
fn conversations_keep_their_rows_more_and_opaque_cursor() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::live::conversations(
            &fixtures::conversations(),
            jig_test_client_records::live_cursor_record(),
            &limits
        ),
        Ok(jig_test_client_records::conversations_rows_record())
    );
}
fn goals_keep_all_ten_phase_counts_and_budget() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::live::goals(7, &fixtures::goals(), &limits),
        Ok(jig_test_client_records::goals_rows_record())
    );
}
fn tasks_keep_the_filter_requester_goal_and_changed_wall() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::live::tasks(&fixtures::tasks_document(), &limits),
        Ok(jig_test_client_records::tasks_rows_record())
    );
}
fn a_tree_keeps_its_rows_folds_dependencies_and_opaque_cursor() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::live::tree(&fixtures::tree_document(), Some(jig_test_client_records::live_cursor_record()), &limits),
        Ok(jig_test_client_records::tree_rows_record())
    );
}
#[test]
fn live_nested_pieces_past_the_wire_bounds_are_refused_naming_them() {
    let source = fixtures::deployment_document();
    let limits = fixtures::wire_limits();
    let rows = [
        ("host runs", w::Limits { owner_host_runs: 0, ..limits }, w::Path::OwnerHostRuns),
        ("deployment hosts", w::Limits { owner_deployment_hosts: 0, ..limits }, w::Path::OwnerDeploymentHosts),
        ("deployment attempts", w::Limits { owner_deployment_attempts: 0, ..limits }, w::Path::OwnerDeploymentAttempts),
        ("account runs", w::Limits { owner_account_runs: 0, ..limits }, w::Path::OwnerAccountRuns),
        ("account waiting", w::Limits { owner_account_waiting: 0, ..limits }, w::Path::OwnerAccountWaiting),
        ("deployment accounts", w::Limits { owner_deployment_accounts: 0, ..limits }, w::Path::OwnerDeploymentAccounts),
    ];
    for (label, limits, path) in rows {
        assert_eq!(
            translate::deployment::document(&source, &limits),
            Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
            "{label}"
        );
    }
}

#[test]
fn documents_become_their_literal_records() {
    table([
        ("a_frame_with_two_projects_becomes_its_record", a_frame_with_two_projects_becomes_its_record),
        ("a_task_with_a_hold_becomes_its_record", a_task_with_a_hold_becomes_its_record),
        ("a_chip_in_each_phase_becomes_its_record", a_chip_in_each_phase_becomes_its_record),
        ("a_run_writing_with_a_call_becomes_its_run_state", a_run_writing_with_a_call_becomes_its_run_state),
        ("a_run_text_keeps_its_turn_retry_and_offsets", a_run_text_keeps_its_turn_retry_and_offsets),
        ("an_inbox_with_both_groups_becomes_its_record", an_inbox_with_both_groups_becomes_its_record),
        ("a_deployment_document_becomes_its_record", a_deployment_document_becomes_its_record),
        ("conversations_keep_their_rows_more_and_opaque_cursor", conversations_keep_their_rows_more_and_opaque_cursor),
        ("goals_keep_all_ten_phase_counts_and_budget", goals_keep_all_ten_phase_counts_and_budget),
        (
            "tasks_keep_the_filter_requester_goal_and_changed_wall",
            tasks_keep_the_filter_requester_goal_and_changed_wall,
        ),
        (
            "a_tree_keeps_its_rows_folds_dependencies_and_opaque_cursor",
            a_tree_keeps_its_rows_folds_dependencies_and_opaque_cursor,
        ),
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
