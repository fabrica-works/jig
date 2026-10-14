use super::*;

fn a_history_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::history_page(fixtures::read_wire_history_page(), &fixtures::limits()),
        Ok(fixtures::read_value_history_page())
    );
}
fn a_inbox_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::inbox_page(fixtures::read_wire_inbox_page(), &fixtures::limits()),
        Ok(fixtures::read_value_inbox_page())
    );
}
fn a_conversation_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::conversation_page(fixtures::read_wire_conversation_page(), &fixtures::limits()),
        Ok(fixtures::read_value_conversation_page())
    );
}
fn a_goal_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::goal_page(fixtures::read_wire_goal_page(), &fixtures::limits()),
        Ok(fixtures::read_value_goal_page())
    );
}
fn a_spend_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::spend_page(fixtures::read_wire_spend_page(), &fixtures::limits()),
        Ok(fixtures::read_value_spend_page())
    );
}
fn a_note_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::note_page(fixtures::read_wire_note_page(), &fixtures::limits()),
        Ok(fixtures::read_value_note_page())
    );
}
fn a_object_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::object_page(fixtures::read_wire_object_page(), &fixtures::limits()),
        Ok(fixtures::read_value_object_page())
    );
}
fn a_owner_ended_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::owner_ended_page(fixtures::read_wire_owner_ended_page(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_ended_page())
    );
}
fn a_owner_inbox_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::owner_inbox_page(fixtures::read_wire_owner_inbox_page(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_inbox_page())
    );
}
fn a_owner_spend_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::owner_spend_page(fixtures::read_wire_owner_spend_page(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_spend_page())
    );
}
fn a_proposal_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::proposal(fixtures::read_wire_proposal(), &fixtures::limits()),
        Ok(fixtures::read_value_proposal())
    );
}
fn a_owner_proposal_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::owner_proposal(fixtures::read_wire_owner_proposal(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_proposal())
    );
}
fn a_escalation_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::escalation(fixtures::read_wire_escalation(), &fixtures::limits()),
        Ok(fixtures::read_value_escalation())
    );
}
fn a_owner_escalation_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::owner_escalation(fixtures::read_wire_owner_escalation(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_escalation())
    );
}
fn a_question_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::question(fixtures::read_wire_question(), &fixtures::limits()),
        Ok(fixtures::read_value_question())
    );
}
fn a_person_task_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::person_task(fixtures::read_wire_person_task(), &fixtures::limits()),
        Ok(fixtures::read_value_person_task())
    );
}
fn a_owner_person_task_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::owner_person_task(fixtures::read_wire_owner_person_task(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_person_task())
    );
}
fn a_result_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::result(fixtures::read_wire_result(), &fixtures::limits()),
        Ok(fixtures::read_value_result())
    );
}
fn a_project_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::project(fixtures::read_wire_project(), &fixtures::limits()),
        Ok(fixtures::read_value_project())
    );
}
fn a_owner_historical_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::owner_historical(fixtures::read_wire_owner_historical(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_historical())
    );
}

#[test]
fn a_page_keeps_its_two_cursors_as_bytes() {
    assert_eq!(
        translate::pages::note_page(fixtures::note_page_with_cursors(), &fixtures::limits()),
        Ok(fixtures::expected_note_page_with_cursors())
    );
}
#[test]
fn a_historical_task_keeps_its_ending() {
    assert_eq!(
        translate::objects::owner_historical(fixtures::historical_task_record(), &fixtures::limits()),
        Ok(fixtures::expected_historical_task())
    );
}
#[test]
fn a_page_admits_each_generated_row_at_its_limit() {
    assert_eq!(
        translate::pages::object_page(fixtures::object_page_piece(fixtures::result_rows(2)), &fixtures::limits()),
        Ok(fixtures::expected_object_page(2))
    );
}
#[test]
fn a_page_one_row_past_its_limit_is_refused() {
    assert_eq!(
        translate::pages::object_page(fixtures::object_page_piece(fixtures::result_rows(3)), &fixtures::limits()),
        Err(Undecodable::TooLarge)
    );
}
#[test]
fn a_page_with_an_incomplete_piece_header_is_refused() {
    assert_eq!(
        translate::pages::object_page(fixtures::object_page_piece(Box::from([1, 0, 0])), &fixtures::limits()),
        Err(Undecodable::Malformed)
    );
}
#[test]
fn a_page_with_trailing_row_bytes_is_refused() {
    assert_eq!(
        translate::pages::object_page(
            fixtures::object_page_piece(fixtures::trailing_result_row()),
            &fixtures::limits()
        ),
        Err(Undecodable::Malformed)
    );
}
#[test]
fn a_page_one_byte_past_the_document_limit_is_refused() {
    let record = fixtures::object_page_piece(fixtures::result_rows(1));
    let mut client = fixtures::limits().client;
    client.document_bytes = u64::from(record.measure()).checked_sub(1).unwrap();
    let limits = Limits::new(client, 4, fixtures::limits().deadline).unwrap();
    assert_eq!(translate::pages::object_page(record, &limits), Err(Undecodable::TooLarge));
}
#[test]
fn a_page_with_mutated_original_wire_limits_is_refused() {
    let mut limits = fixtures::limits();
    limits.wire.cursor_value = 1;
    assert_eq!(translate::pages::note_page(fixtures::read_wire_note_page(), &limits), Err(Undecodable::TooLarge));
}

#[test]
fn a_single_read_one_text_byte_past_the_limit_is_refused() {
    assert_eq!(
        translate::objects::proposal(fixtures::proposal_with_words(&[b'a'; 33]), &fixtures::limits()),
        Err(Undecodable::TooLarge)
    );
}

fn a_activity_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::activity_page(fixtures::read_wire_activity_page(), &fixtures::limits()),
        Ok(fixtures::read_value_activity_page())
    );
}

#[test]
fn an_activity_page_decodes_each_turn() {
    assert_eq!(
        translate::pages::activity_page(fixtures::decoded_activity_record(), &fixtures::limits()),
        Ok(fixtures::decoded_activity_page())
    );
}
#[test]
fn an_undecodable_turn_crosses_up_as_such() {
    let limits = fixtures::limits();
    for (record, expected) in fixtures::unread_pages() {
        assert_eq!(translate::pages::activity_page(record, &limits), Ok(expected));
    }
}
#[test]
fn an_archived_turn_keeps_its_envelope_when_decoding_fails() {
    assert_eq!(
        translate::pages::activity_page(fixtures::archived_activity_record(), &fixtures::limits()),
        Ok(fixtures::archived_activity_page())
    );
}
#[test]
fn a_turns_receiving_cut_stays_on_its_decoded_value() {
    let limits = fixtures::turn_limits(2, 65536);
    assert_eq!(
        translate::pages::activity_page(fixtures::cut_activity_record(), &limits),
        Ok(fixtures::cut_activity_page())
    );
}
#[test]
fn a_turn_past_its_admission_bound_is_carried_as_too_large() {
    let limits = fixtures::turn_limits(32, 126);
    assert_eq!(
        translate::pages::turn_fields(fixtures::FULL_TURN, &limits),
        c::TurnRead::Undecodable { why: c::TurnProblem::TooLarge }
    );
}

#[test]
fn an_owner_turn_without_carried_bytes_crosses_up_with_its_problem() {
    let limits = fixtures::limits();
    for (name, record, expected) in fixtures::owner_turn_pages() {
        assert_eq!(translate::pages::activity_page(record, &limits), Ok(expected), "{name}");
    }
}

#[test]
fn page_and_object_records_become_their_literal_contract_values() {
    table([
        (
            "a_history_page_record_becomes_its_literal_contract_value",
            a_history_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_inbox_page_record_becomes_its_literal_contract_value",
            a_inbox_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_conversation_page_record_becomes_its_literal_contract_value",
            a_conversation_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_goal_page_record_becomes_its_literal_contract_value",
            a_goal_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_spend_page_record_becomes_its_literal_contract_value",
            a_spend_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_note_page_record_becomes_its_literal_contract_value",
            a_note_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_object_page_record_becomes_its_literal_contract_value",
            a_object_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_owner_ended_page_record_becomes_its_literal_contract_value",
            a_owner_ended_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_owner_inbox_page_record_becomes_its_literal_contract_value",
            a_owner_inbox_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_owner_spend_page_record_becomes_its_literal_contract_value",
            a_owner_spend_page_record_becomes_its_literal_contract_value,
        ),
        ("a_proposal_record_becomes_its_literal_contract_value", a_proposal_record_becomes_its_literal_contract_value),
        (
            "a_owner_proposal_record_becomes_its_literal_contract_value",
            a_owner_proposal_record_becomes_its_literal_contract_value,
        ),
        (
            "a_escalation_record_becomes_its_literal_contract_value",
            a_escalation_record_becomes_its_literal_contract_value,
        ),
        (
            "a_owner_escalation_record_becomes_its_literal_contract_value",
            a_owner_escalation_record_becomes_its_literal_contract_value,
        ),
        ("a_question_record_becomes_its_literal_contract_value", a_question_record_becomes_its_literal_contract_value),
        (
            "a_person_task_record_becomes_its_literal_contract_value",
            a_person_task_record_becomes_its_literal_contract_value,
        ),
        (
            "a_owner_person_task_record_becomes_its_literal_contract_value",
            a_owner_person_task_record_becomes_its_literal_contract_value,
        ),
        ("a_result_record_becomes_its_literal_contract_value", a_result_record_becomes_its_literal_contract_value),
        ("a_project_record_becomes_its_literal_contract_value", a_project_record_becomes_its_literal_contract_value),
        (
            "a_owner_historical_record_becomes_its_literal_contract_value",
            a_owner_historical_record_becomes_its_literal_contract_value,
        ),
        (
            "a_activity_page_record_becomes_its_literal_contract_value",
            a_activity_page_record_becomes_its_literal_contract_value,
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
