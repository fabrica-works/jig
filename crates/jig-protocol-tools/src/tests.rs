use crate::{Decoded, Problem, Why};
use alloc::boxed::Box;
use jig_domain_core as core;
mod fixtures;

#[test]
fn a_cancel_call_decodes_to_its_named_action() {
    match crate::decode(b"cancel", fixtures::CANCEL, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::Control {
            target: 3,
            control: jig_domain_tasks::Control::Cancel { .. },
        }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}

#[test]
fn a_release_call_decodes_to_its_named_action() {
    match crate::decode(b"release", fixtures::RELEASE, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::Control {
            target: 3,
            control: jig_domain_tasks::Control::Release { .. },
        }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}

#[test]
fn a_introduce_call_decodes_to_its_named_action() {
    match crate::decode(b"introduce", fixtures::INTRODUCE, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::Introduce { left: 3, right: 4 }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}

#[test]
fn a_decide_call_decodes_to_its_named_action() {
    match crate::decode(b"decide", fixtures::DECIDE, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::DecideProposal { proposer: 3, proposal: 2, .. }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}

#[test]
fn a_withdraw_call_decodes_to_its_named_action() {
    match crate::decode(b"withdraw", fixtures::WITHDRAW, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::WithdrawProposal { proposal: 2 }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}

#[test]
fn a_subscribe_call_decodes_to_its_named_action() {
    match crate::decode(b"subscribe", fixtures::SUBSCRIBE, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::Subscribe { .. }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}

#[test]
fn a_unsubscribe_call_decodes_to_its_named_action() {
    match crate::decode(b"unsubscribe", fixtures::UNSUBSCRIBE, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::Unsubscribe { subscription: 1 }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}

#[test]
fn a_amend_call_decodes_to_its_named_action() {
    match crate::decode(b"amend", fixtures::AMEND, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::Amend { target: 3, .. }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}

#[test]
fn a_decide_escalation_call_decodes_to_its_named_action() {
    match crate::decode(b"decide_escalation", fixtures::DECIDE_ESCALATION, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::DecideEscalation { task: 3, revision: 1, .. }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}

#[test]
fn a_propose_call_decodes_to_its_named_action() {
    match crate::decode(b"propose", fixtures::PROPOSE, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::ProposeBatch { .. }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}

#[test]
fn a_note_call_decodes_to_its_named_action() {
    match crate::decode(b"note", fixtures::NOTE, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::Note { .. }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}

#[test]
fn a_recall_call_decodes_to_its_named_action() {
    match crate::decode(b"recall", fixtures::RECALL, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::Recall { .. }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}

#[test]
fn a_message_call_decodes_to_its_named_action() {
    match crate::decode(b"message", fixtures::MESSAGE, &fixtures::LIMITS) {
        Decoded::Action(core::NamedAction::Message { target, words, kind }) => {
            assert_eq!(target, 3);
            assert_eq!(words.as_ref(), b"is the build green?");
            assert_eq!(kind, jig_domain_tasks::MessageKind::Words);
        }
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("message decoded as {other:?}")
        }
    }
}
#[test]
fn a_delegate_call_decodes_its_batch_in_order() {
    match crate::decode(b"delegate", fixtures::DELEGATE, &fixtures::LIMITS) {
        Decoded::Delegate(batch) => {
            assert_eq!(batch.len(), 2);
            assert_eq!(batch[0].spec.words.as_ref(), b"check");
            assert_eq!(batch[1].spec.words.as_ref(), b"report");
        }
        other @ (Decoded::Action(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("delegate decoded as {other:?}")
        }
    }
}
#[test]
fn a_call_missing_a_required_field_names_it() {
    match crate::decode(b"message", br#"{"form":"words","words":"is the build green?"}"#, &fixtures::LIMITS) {
        Decoded::Problem(Problem { field: b"target", why: Why::Missing }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}
#[test]
fn a_call_with_the_wrong_field_type_names_it() {
    match crate::decode(b"message", br#"{"target":"three","form":"words","words":"hi"}"#, &fixtures::LIMITS) {
        Decoded::Problem(Problem { field: b"target", why: Why::Type }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}
#[test]
fn a_call_that_is_not_json_is_malformed() {
    match crate::decode(b"message", b"not JSON", &fixtures::LIMITS) {
        Decoded::Problem(Problem { why: Why::Malformed, .. }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}
#[test]
fn an_input_past_its_bound_is_too_large() {
    let limits = crate::Limits { input_bytes: 1, ..fixtures::LIMITS };
    match crate::decode(b"message", fixtures::MESSAGE, &limits) {
        Decoded::Problem(Problem { why: Why::TooLarge, .. }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}
#[test]
fn another_applications_tool_is_not_mine() {
    match crate::decode(b"another_tool", b"not JSON", &fixtures::LIMITS) {
        Decoded::NotMine => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_)) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}
#[test]
fn a_batch_past_its_bound_names_the_batch() {
    let limits = crate::Limits { batch: 1, ..fixtures::LIMITS };
    match crate::decode(b"delegate", fixtures::DELEGATE, &limits) {
        Decoded::Problem(Problem { field: b"batch", why: Why::TooLarge }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}
#[test]
fn a_delegated_part_renders_its_task_numbers() {
    assert_eq!(
        crate::render(&core::CallPart::Delegated(Box::new([3, 12]))),
        crate::Rendered { error: false, text: b"Delegated tasks: 3 12".as_slice().into() }
    );
}
#[test]
fn a_denied_part_renders_as_an_error_naming_what_it_lacked() {
    let part = core::CallPart::ToolDenied {
        answer: jig_domain_authority::Answer::Refuse,
        findings: Box::new([jig_domain_authority::Finding::Tool]),
    };
    assert_eq!(
        crate::render(&part),
        crate::Rendered { error: true, text: b"Refused; missing tool authority".as_slice().into() }
    );
}
#[test]
fn an_unproposable_part_renders_as_an_error_saying_no_proposal_could_be_accepted() {
    let part = core::CallPart::ProposalRefused(jig_domain_tasks::Problem {
        task: Some(1),
        why: jig_domain_tasks::Refusal::Unproposable,
        blocked_by: None,
        capacity: None,
    });
    assert_eq!(
        crate::render(&part),
        crate::Rendered { error: true, text: b"Refused: no proposal could be accepted".as_slice().into() }
    );
}
#[test]
fn a_problem_renders_the_tools_name_and_missing_field() {
    assert_eq!(
        crate::problem(b"message", &Problem { field: b"target", why: Why::Missing }),
        crate::Rendered { error: true, text: b"message: the field target is missing".as_slice().into() }
    );
}
#[test]
fn a_call_with_duplicate_fields_is_malformed() {
    match crate::decode(b"release", br#"{"target":3,"target":4}"#, &fixtures::LIMITS) {
        Decoded::Problem(Problem { why: Why::Malformed, .. }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}
fn schema(tool: &[u8]) {
    let limits = crate::Limits { input_bytes: 65536, ..fixtures::LIMITS };
    let mut declared = None;
    for declaration in crate::declarations() {
        if declaration.name == tool {
            declared = Some(declaration);
        }
    }
    let declaration = declared.expect("the declared tool");
    let schema = crate::json::parse(declaration.schema, &limits).expect("a schema is JSON");
    assert!(!declaration.description.is_empty());
    let mut expected = None;
    for (name, fields) in fixtures::FIELDS {
        if *name == declaration.name {
            expected = Some(*fields);
        }
    }
    for field in expected.expect("every tool has decoder fields") {
        assert!(schema_names(&schema, field), "the schema names {field:?}");
    }
}

#[test]
fn the_delegate_schema_parses_and_names_its_decoders_fields() {
    schema(b"delegate");
}

#[test]
fn the_message_schema_parses_and_names_its_decoders_fields() {
    schema(b"message");
}

#[test]
fn the_cancel_schema_parses_and_names_its_decoders_fields() {
    schema(b"cancel");
}

#[test]
fn the_release_schema_parses_and_names_its_decoders_fields() {
    schema(b"release");
}

#[test]
fn the_introduce_schema_parses_and_names_its_decoders_fields() {
    schema(b"introduce");
}

#[test]
fn the_decide_schema_parses_and_names_its_decoders_fields() {
    schema(b"decide");
}

#[test]
fn the_withdraw_schema_parses_and_names_its_decoders_fields() {
    schema(b"withdraw");
}

#[test]
fn the_subscribe_schema_parses_and_names_its_decoders_fields() {
    schema(b"subscribe");
}

#[test]
fn the_unsubscribe_schema_parses_and_names_its_decoders_fields() {
    schema(b"unsubscribe");
}

#[test]
fn the_amend_schema_parses_and_names_its_decoders_fields() {
    schema(b"amend");
}

#[test]
fn the_decide_escalation_schema_parses_and_names_its_decoders_fields() {
    schema(b"decide_escalation");
}

#[test]
fn the_propose_schema_parses_and_names_its_decoders_fields() {
    schema(b"propose");
}

#[test]
fn the_note_schema_parses_and_names_its_decoders_fields() {
    schema(b"note");
}

#[test]
fn the_recall_schema_parses_and_names_its_decoders_fields() {
    schema(b"recall");
}

fn schema_names(schema: &crate::json::Value, field: &[u8]) -> bool {
    if let Some(properties) = schema.field(b"properties").expect("schema object") {
        return properties.field(field).expect("properties object").is_some();
    }
    let choices = schema.required(b"oneOf").expect("a union schema").array().expect("schema alternatives");
    for choice in choices {
        let properties = choice.required(b"properties").expect("alternative properties");
        if properties.field(field).expect("properties object").is_some() {
            return true;
        }
    }
    false
}
#[test]
fn words_past_their_bound_name_the_field() {
    let limits = crate::Limits { words: 8, ..fixtures::LIMITS };
    match crate::decode(b"message", fixtures::MESSAGE, &limits) {
        Decoded::Problem(Problem { field: b"words", why: Why::TooLarge }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}
#[test]
fn nesting_past_its_bound_is_too_large() {
    let limits = crate::Limits { depth: 1, ..fixtures::LIMITS };
    match crate::decode(b"delegate", fixtures::DELEGATE, &limits) {
        Decoded::Problem(Problem { why: Why::TooLarge, .. }) => {}
        other @ (Decoded::Action(_) | Decoded::Delegate(_) | Decoded::Problem(_) | Decoded::NotMine) => {
            panic!("unexpected decoded input {other:?}")
        }
    }
}
