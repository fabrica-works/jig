use super::*;

#[test]
fn a_history_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::history_page(fixtures::read_wire_history_page(), &fixtures::limits()),
        Ok(fixtures::read_value_history_page())
    );
}
#[test]
fn a_inbox_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::inbox_page(fixtures::read_wire_inbox_page(), &fixtures::limits()),
        Ok(fixtures::read_value_inbox_page())
    );
}
#[test]
fn a_conversation_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::conversation_page(fixtures::read_wire_conversation_page(), &fixtures::limits()),
        Ok(fixtures::read_value_conversation_page())
    );
}
#[test]
fn a_goal_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::goal_page(fixtures::read_wire_goal_page(), &fixtures::limits()),
        Ok(fixtures::read_value_goal_page())
    );
}
#[test]
fn a_spend_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::spend_page(fixtures::read_wire_spend_page(), &fixtures::limits()),
        Ok(fixtures::read_value_spend_page())
    );
}
#[test]
fn a_note_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::note_page(fixtures::read_wire_note_page(), &fixtures::limits()),
        Ok(fixtures::read_value_note_page())
    );
}
#[test]
fn a_object_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::object_page(fixtures::read_wire_object_page(), &fixtures::limits()),
        Ok(fixtures::read_value_object_page())
    );
}
#[test]
fn a_owner_ended_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::owner_ended_page(fixtures::read_wire_owner_ended_page(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_ended_page())
    );
}
#[test]
fn a_owner_inbox_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::owner_inbox_page(fixtures::read_wire_owner_inbox_page(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_inbox_page())
    );
}
#[test]
fn a_owner_spend_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::owner_spend_page(fixtures::read_wire_owner_spend_page(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_spend_page())
    );
}
#[test]
fn a_proposal_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::proposal(fixtures::read_wire_proposal(), &fixtures::limits()),
        Ok(fixtures::read_value_proposal())
    );
}
#[test]
fn a_owner_proposal_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::owner_proposal(fixtures::read_wire_owner_proposal(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_proposal())
    );
}
#[test]
fn a_escalation_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::escalation(fixtures::read_wire_escalation(), &fixtures::limits()),
        Ok(fixtures::read_value_escalation())
    );
}
#[test]
fn a_owner_escalation_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::owner_escalation(fixtures::read_wire_owner_escalation(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_escalation())
    );
}
#[test]
fn a_question_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::question(fixtures::read_wire_question(), &fixtures::limits()),
        Ok(fixtures::read_value_question())
    );
}
#[test]
fn a_person_task_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::person_task(fixtures::read_wire_person_task(), &fixtures::limits()),
        Ok(fixtures::read_value_person_task())
    );
}
#[test]
fn a_owner_person_task_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::owner_person_task(fixtures::read_wire_owner_person_task(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_person_task())
    );
}
#[test]
fn a_result_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::result(fixtures::read_wire_result(), &fixtures::limits()),
        Ok(fixtures::read_value_result())
    );
}
#[test]
fn a_project_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::project(fixtures::read_wire_project(), &fixtures::limits()),
        Ok(fixtures::read_value_project())
    );
}
#[test]
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
