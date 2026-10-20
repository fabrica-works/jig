//! Cases of store codec round trips and refusals (testing-strategy.md, 8).
use super::*;

#[test]
fn people_rows_read_back_as_written() {
    table([
        ("a_person_reads_back_with_its_identity", a_person_reads_back_with_its_identity),
        ("a_projects_policy_reads_back", a_projects_policy_reads_back),
        ("a_policy_larger_than_one_amendment_reads_back", a_policy_larger_than_one_amendment_reads_back),
        ("a_read_conversation_position_reads_back", a_read_conversation_position_reads_back),
        ("a_sign_in_reads_back_with_its_wall_expiry", a_sign_in_reads_back_with_its_wall_expiry),
        ("every_role_reads_back_in_a_roster", every_role_reads_back_in_a_roster),
        ("a_read_results_position_reads_back", a_read_results_position_reads_back),
    ]);
}

#[test]
fn call_rows_read_back_as_written() {
    table([
        ("a_call_record_of_each_part_reads_back", a_call_record_of_each_part_reads_back),
        ("a_refused_delegation_reads_back_with_its_problem", a_refused_delegation_reads_back_with_its_problem),
        (
            "a_refused_llm_choice_reads_back_with_its_model_and_fault",
            a_refused_llm_choice_reads_back_with_its_model_and_fault,
        ),
        ("a_recalled_notes_page_reads_back", a_recalled_notes_page_reads_back),
        ("a_settled_call_reads_back", a_settled_call_reads_back),
    ]);
}

#[test]
fn component_rows_read_back_as_written() {
    table([
        (
            "the_format_row_reads_back_with_its_versions_and_deployment",
            the_format_row_reads_back_with_its_versions_and_deployment,
        ),
        ("a_transcript_place_reads_back_with_its_window", a_transcript_place_reads_back_with_its_window),
        ("an_attempt_place_reads_back_with_its_first_turn", an_attempt_place_reads_back_with_its_first_turn),
        (
            "the_installation_reads_back_and_another_version_does_not",
            the_installation_reads_back_and_another_version_does_not,
        ),
    ]);
}

#[test]
fn notes_rows_keep_their_fields() {
    table([
        ("a_party_authored_note_reads_back_in_each_scope", a_party_authored_note_reads_back_in_each_scope),
        ("notes_rows_read_back_as_saved", notes_rows_read_back_as_saved),
    ]);
}

#[test]
fn finished_terminals_read_back_as_written() {
    table([
        (
            "a_finished_terminal_reads_back_with_its_followup_batch",
            a_finished_terminal_reads_back_with_its_followup_batch,
        ),
        (
            "a_finished_terminal_reads_back_with_its_followup_proposal",
            a_finished_terminal_reads_back_with_its_followup_proposal,
        ),
    ]);
}

#[test]
fn carried_values_read_back_as_written() {
    table([
        ("a_carried_phase_reads_back_as_its_own_family", a_carried_phase_reads_back_as_its_own_family),
        ("a_carried_change_reads_back_as_its_own_family", a_carried_change_reads_back_as_its_own_family),
    ]);
}

#[test]
fn task_values_read_back_as_written() {
    table([
        ("a_pool_slot_report_reads_back", a_pool_slot_report_reads_back),
        ("a_specification_with_every_parameter_kind_reads_back", a_specification_with_every_parameter_kind_reads_back),
        ("each_party_kind_reads_back", each_party_kind_reads_back),
        ("each_executor_kind_reads_back", each_executor_kind_reads_back),
        ("each_failure_class_reads_back", each_failure_class_reads_back),
        ("each_activation_cause_reads_back", each_activation_cause_reads_back),
        ("each_result_contract_kind_reads_back", each_result_contract_kind_reads_back),
        ("each_connector_part_kind_reads_back", each_connector_part_kind_reads_back),
        (
            "a_specification_at_its_aggregate_byte_bound_reads_back",
            a_specification_at_its_aggregate_byte_bound_reads_back,
        ),
        (
            "a_writer_resource_at_its_aggregate_byte_bound_reads_back",
            a_writer_resource_at_its_aggregate_byte_bound_reads_back,
        ),
        (
            "verdict_rules_at_their_aggregate_byte_bound_read_back",
            verdict_rules_at_their_aggregate_byte_bound_read_back,
        ),
        ("task_ledgers_read_back", task_ledgers_read_back),
        ("writer_slots_read_back", writer_slots_read_back),
        ("authorities_read_back", authorities_read_back),
    ]);
}

#[test]
fn encoding_refuses_aggregate_bounds() {
    table([
        ("a_notes_pattern_past_its_aggregate_limit_is_refused", a_notes_pattern_past_its_aggregate_limit_is_refused),
        (
            "a_specification_past_its_aggregate_byte_bound_is_refused",
            a_specification_past_its_aggregate_byte_bound_is_refused,
        ),
        (
            "grants_and_note_resources_past_their_shared_byte_bound_are_refused",
            grants_and_note_resources_past_their_shared_byte_bound_are_refused,
        ),
        (
            "a_writer_resource_past_its_aggregate_byte_bound_is_refused",
            a_writer_resource_past_its_aggregate_byte_bound_is_refused,
        ),
        (
            "verdict_rules_past_their_aggregate_byte_bound_are_refused",
            verdict_rules_past_their_aggregate_byte_bound_are_refused,
        ),
    ]);
}

#[test]
fn decoding_refuses_aggregate_bounds() {
    table([
        (
            "a_decoded_pattern_past_its_aggregate_limit_is_refused",
            a_decoded_pattern_past_its_aggregate_limit_is_refused,
        ),
        (
            "a_decoded_specification_past_its_aggregate_byte_bound_is_refused",
            a_decoded_specification_past_its_aggregate_byte_bound_is_refused,
        ),
        (
            "decoded_grants_and_note_resources_past_their_shared_byte_bound_are_refused",
            decoded_grants_and_note_resources_past_their_shared_byte_bound_are_refused,
        ),
        (
            "a_decoded_writer_resource_past_its_aggregate_byte_bound_is_refused",
            a_decoded_writer_resource_past_its_aggregate_byte_bound_is_refused,
        ),
        (
            "decoded_verdict_rules_past_their_aggregate_byte_bound_are_refused",
            decoded_verdict_rules_past_their_aggregate_byte_bound_are_refused,
        ),
    ]);
}

#[test]
fn family_limits_past_their_ceilings_are_refused() {
    table([
        ("notes_limits_past_the_ceilings_are_refused", notes_limits_past_the_ceilings_are_refused),
        ("tasks_limits_past_the_ceilings_are_refused", tasks_limits_past_the_ceilings_are_refused),
        ("people_limits_past_their_ceilings_are_refused", people_limits_past_their_ceilings_are_refused),
        (
            "core_limits_past_the_settled_answer_ceiling_are_refused",
            core_limits_past_the_settled_answer_ceiling_are_refused,
        ),
    ]);
}

#[test]
fn another_row_version_does_not_decode() {
    table([
        ("a_notes_row_of_another_version_does_not_decode", a_notes_row_of_another_version_does_not_decode),
        ("a_tasks_row_of_another_version_does_not_decode", a_tasks_row_of_another_version_does_not_decode),
        ("a_people_row_of_another_version_does_not_decode", a_people_row_of_another_version_does_not_decode),
    ]);
}
