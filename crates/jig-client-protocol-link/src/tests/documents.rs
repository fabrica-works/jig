use super::*;
use crate::translate::{Undecodable, documents, live};
#[test]
fn a_me_record_becomes_the_contract_s_me() {
    assert_eq!(documents::frame(fixtures::frame_record(), &fixtures::limits()), Ok(fixtures::frame()));
}
#[test]
fn a_task_one_row_past_the_client_s_limit_is_refused() {
    assert_eq!(documents::task(fixtures::task_with_lineage(3), &fixtures::limits()), Err(Undecodable::TooLarge));
}
#[test]
fn a_run_text_keeps_its_turn_retry_and_offsets() {
    assert_eq!(live::run_text(fixtures::text_record(), &fixtures::limits()), Ok(fixtures::run_text_expected()));
}

#[test]
fn a_chip_in_each_phase_becomes_the_contract_s_chip() {
    for (record, expected) in fixtures::phases() {
        assert_eq!(documents::chip(record, &fixtures::limits()), Ok(expected));
    }
}

fn a_task_keeps_every_supplied_field() {
    assert_eq!(documents::task(fixtures::task_record(), &fixtures::limits()), Ok(fixtures::task()));
}

fn a_tree_keeps_every_supplied_field() {
    assert_eq!(documents::tree(fixtures::tree_record(), &fixtures::limits()), Ok(fixtures::tree()));
}

fn a_conversations_keeps_every_supplied_field() {
    assert_eq!(
        documents::conversations(fixtures::conversations_record(), &fixtures::limits()),
        Ok(fixtures::conversations())
    );
}

fn a_inbox_keeps_every_supplied_field() {
    assert_eq!(documents::inbox(fixtures::inbox_record(), &fixtures::limits()), Ok(fixtures::inbox()));
}

fn a_goals_keeps_every_supplied_field() {
    assert_eq!(documents::goals(fixtures::goals_record(), &fixtures::limits()), Ok(fixtures::goals()));
}

fn a_tasks_keeps_every_supplied_field() {
    assert_eq!(documents::tasks(fixtures::tasks_record(), &fixtures::limits()), Ok(fixtures::tasks()));
}

fn a_deployment_keeps_every_supplied_field() {
    assert_eq!(documents::deployment(fixtures::deployment_record(), &fixtures::limits()), Ok(fixtures::deployment()));
}

fn a_owner_deployment_keeps_every_supplied_field() {
    assert_eq!(
        documents::owner_deployment(fixtures::owner_deployment_record(), &fixtures::limits()),
        Ok(fixtures::owner_deployment())
    );
}

fn a_owner_project_keeps_every_supplied_field() {
    assert_eq!(
        documents::owner_project(fixtures::owner_project_record(), &fixtures::limits()),
        Ok(fixtures::owner_project())
    );
}

fn a_run_state_keeps_every_supplied_field() {
    assert_eq!(live::run_state(fixtures::run_state_record(), &fixtures::limits()), Ok(fixtures::run_state()));
}

fn a_current_call_keeps_every_supplied_field() {
    assert_eq!(live::current_call(fixtures::current_call_record(), &fixtures::limits()), Ok(fixtures::current_call()));
}
fn a_task_piece_with_a_partial_header_is_malformed() {
    assert_eq!(
        documents::task(fixtures::task_with_piece(&[1, 0, 0]), &fixtures::limits()),
        Err(Undecodable::Malformed)
    );
}
fn a_task_piece_past_its_end_is_malformed() {
    assert_eq!(
        documents::task(fixtures::task_with_piece(&[5, 0, 0, 0, 1]), &fixtures::limits()),
        Err(Undecodable::Malformed)
    );
}
#[test]
fn a_document_at_its_measured_limit_is_admitted() {
    let record = fixtures::frame_record();
    let limits = fixtures::limits_for_document(u64::from(record.measure()));
    assert_eq!(documents::frame(record, &limits), Ok(fixtures::frame()));
}
#[test]
fn a_document_one_byte_past_its_measured_limit_is_refused() {
    let record = fixtures::frame_record();
    let limits = fixtures::limits_for_document(u64::from(record.measure()).checked_sub(1).unwrap());
    assert_eq!(documents::frame(record, &limits), Err(Undecodable::TooLarge));
}
#[test]
fn invalid_receiving_limits_are_refused() {
    let mut limits = fixtures::limits();
    limits.wire.document_text_bytes = u32::MAX;
    assert_eq!(documents::frame(fixtures::frame_record(), &limits), Err(Undecodable::TooLarge));
}

#[test]
fn a_run_text_without_optional_values_keeps_their_absence() {
    assert_eq!(live::run_text(fixtures::run_text_record(), &fixtures::limits()), Ok(fixtures::run_text()));
}
#[test]
fn a_task_at_its_row_bound_is_admitted() {
    assert_eq!(
        documents::task(fixtures::task_at_row_bound(), &fixtures::limits()),
        Ok(fixtures::task_with_two_chips())
    );
}
#[test]
fn a_run_text_one_byte_past_its_text_limit_is_refused() {
    assert_eq!(
        live::run_text(fixtures::run_text_one_past_text_limit(), &fixtures::limits()),
        Err(Undecodable::TooLarge)
    );
}
#[test]
fn an_account_later_keeps_its_optional_wall() {
    for until in [None, Some(0), Some(u64::MAX)] {
        assert_eq!(
            documents::owner_deployment(fixtures::owner_with_account(until), &fixtures::limits()),
            Ok(fixtures::owner_account_expected(until))
        );
    }
}

#[test]
fn a_conversations_cursor_keeps_bytes_beyond_the_text_bound() {
    assert_eq!(
        documents::conversations(fixtures::conversations_with_cursor(), &fixtures::limits()),
        Ok(fixtures::conversations_cursor_expected())
    );
}
#[test]
fn a_raw_live_call_keeps_its_provider_retry_and_start() {
    assert_eq!(live::live_call(fixtures::raw_live_call(), &fixtures::limits()), Ok(fixtures::raw_live_call_expected()));
}
#[test]
fn a_decoded_live_call_keeps_its_committed_call() {
    assert_eq!(
        live::live_call(fixtures::decoded_live_call(), &fixtures::limits()),
        Ok(fixtures::decoded_live_call_expected())
    );
}

#[test]
fn malformed_task_pieces_are_refused() {
    table([
        ("a_task_piece_with_a_partial_header_is_malformed", a_task_piece_with_a_partial_header_is_malformed),
        ("a_task_piece_past_its_end_is_malformed", a_task_piece_past_its_end_is_malformed),
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

#[test]
fn documents_keep_every_supplied_field() {
    table([
        ("a_task_keeps_every_supplied_field", a_task_keeps_every_supplied_field),
        ("a_tree_keeps_every_supplied_field", a_tree_keeps_every_supplied_field),
        ("a_conversations_keeps_every_supplied_field", a_conversations_keeps_every_supplied_field),
        ("a_inbox_keeps_every_supplied_field", a_inbox_keeps_every_supplied_field),
        ("a_goals_keeps_every_supplied_field", a_goals_keeps_every_supplied_field),
        ("a_tasks_keeps_every_supplied_field", a_tasks_keeps_every_supplied_field),
        ("a_deployment_keeps_every_supplied_field", a_deployment_keeps_every_supplied_field),
        ("a_owner_deployment_keeps_every_supplied_field", a_owner_deployment_keeps_every_supplied_field),
        ("a_owner_project_keeps_every_supplied_field", a_owner_project_keeps_every_supplied_field),
        ("a_run_state_keeps_every_supplied_field", a_run_state_keeps_every_supplied_field),
        ("a_current_call_keeps_every_supplied_field", a_current_call_keeps_every_supplied_field),
    ]);
}

#[test]
fn a_resource_path_at_its_segment_bound_is_decoded_past_the_row_bound() {
    let mut expected = fixtures::task();
    let c::TaskContractDocument::Approval { part, .. } = &mut expected.result_contract else { panic!("approval") };
    let mut path = skein_lib::List::with_capacity(4);
    for _ in 0_u32..4 {
        path.push(Box::from(b"segment".as_slice())).unwrap();
    }
    part.key = c::OwnerPartKey::Path(path);
    assert_eq!(documents::task(fixtures::task_with_resource_path(4), &fixtures::limits()), Ok(expected));
}

#[test]
fn a_resource_path_one_segment_past_its_bound_is_refused() {
    assert_eq!(documents::task(fixtures::task_with_resource_path(5), &fixtures::limits()), Err(Undecodable::TooLarge));
}
