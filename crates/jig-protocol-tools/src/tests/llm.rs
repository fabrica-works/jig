//! Each malformed choice is refused by parsing; authority and account eligibility are core checks.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "bounded JSON fixtures are ordinary Rust tests"
)]
use super::{Decoded, Problem, Why, core, fixtures};
use ::core::str;
use jig_domain_tasks as tasks;

fn delegate(choice: &str) -> Box<[u8]> {
    str::from_utf8(fixtures::DELEGATE)
        .unwrap()
        .replace("\"executor\":", &format!("\"llm\":{choice},\"executor\":"))
        .into_bytes()
        .into_boxed_slice()
}
fn amendment(choice: &str) -> Box<[u8]> {
    str::from_utf8(fixtures::AMEND)
        .unwrap()
        .replace("\"reason\":", &format!("\"llm\":{choice},\"reason\":"))
        .into_bytes()
        .into_boxed_slice()
}
fn chosen() -> tasks::LlmChoice {
    tasks::LlmChoice {
        account: Some(42),
        main: tasks::ModelEffort { model: 0, effort: Some(1) },
        subagents: Box::new([tasks::ModelEffort { model: 1, effort: None }]),
    }
}
const CHOICE: &str = r#"{"main":{"model":0,"effort":1},"subagents":[{"model":1}],"account":42}"#;

#[test]
fn a_delegate_parses_its_explicit_llm_choice() {
    let Decoded::Delegate(batch) = crate::decode(b"delegate", &delegate(CHOICE), &fixtures::LIMITS) else {
        panic!("delegate")
    };
    assert_eq!(batch[0].llm, Some(chosen()));
}
#[test]
fn an_amendment_parses_its_explicit_llm_choice() {
    let Decoded::Action(core::NamedAction::Amend { amendment, .. }) =
        crate::decode(b"amend", &amendment(CHOICE), &fixtures::LIMITS)
    else {
        panic!("amend")
    };
    assert_eq!(amendment.llm, Some(chosen()));
}
#[test]
fn a_choice_with_an_invalid_field_is_refused_during_parsing() {
    for (choice, field, why) in [
        (r#"{"account":42}"#, b"main".as_slice(), Why::Missing),
        (r#"{"main":{"model":256}}"#, b"model", Why::Range),
        (r#"{"main":{"model":0,"effort":256}}"#, b"effort", Why::Range),
        (r#"{"main":{"model":0},"account":4294967296}"#, b"account", Why::Range),
        (r#"{"main":{"model":0},"account":"42"}"#, b"account", Why::Type),
        (r#"{"main":{"model":0},"subagents":{}}"#, b"subagents", Why::Type),
    ] {
        for (tool, input) in [(b"delegate".as_slice(), delegate(choice)), (b"amend".as_slice(), amendment(choice))] {
            let Decoded::Problem(problem) = crate::decode(tool, &input, &fixtures::LIMITS) else {
                panic!("malformed choice")
            };
            assert_eq!(problem, Problem { field, why });
        }
    }
}
#[test]
fn subagent_choice_count_is_bounded_before_allocation() {
    for count in [tasks::MODELS, tasks::MODELS + 1] {
        let models = vec![r#"{"model":0}"#; usize::try_from(count).unwrap()].join(",");
        let input = delegate(&format!(r#"{{"main":{{"model":0}},"subagents":[{models}]}}"#));
        let limits = fixtures::LIMITS;
        if count == tasks::MODELS {
            let Decoded::Delegate(batch) = crate::decode(b"delegate", &input, &limits) else { panic!("at bound") };
            assert_eq!(batch[0].llm.as_ref().unwrap().subagents.len(), usize::try_from(count).unwrap());
        } else {
            let Decoded::Problem(problem) = crate::decode(b"delegate", &input, &limits) else { panic!("over bound") };
            assert_eq!(problem, Problem { field: b"subagents", why: Why::TooLarge });
        }
    }
}
#[test]
fn an_account_refusal_renders_its_number_with_its_reason() {
    for (why, reason) in [
        (tasks::AccountWhy::Unknown, "account not found"),
        (tasks::AccountWhy::Provider, "account belongs to another provider"),
        (tasks::AccountWhy::Uses, "account does not serve the payer"),
        (tasks::AccountWhy::Limited, "kept account is limited"),
    ] {
        let part =
            core::CallPart::DelegationRefused(tasks::Problem::new(None, tasks::Refusal::Account { account: 42, why }));
        assert_eq!(
            crate::render(&part),
            crate::Rendered {
                error: true,
                text: format!("Refused account 42: {reason}").into_bytes().into_boxed_slice()
            }
        );
    }
}

#[test]
fn an_llm_amendment_with_a_priority_still_needs_its_revision() {
    let input = br#"{"target":3,"amendment":{"tracked":{"priority":1},"llm":{"main":{"model":0}},"reason":"choose"}}"#;
    let Decoded::Problem(problem) = crate::decode(b"amend", input, &fixtures::LIMITS) else {
        panic!("revision required")
    };
    assert_eq!(problem, Problem { field: b"revision", why: Why::Missing });
}
