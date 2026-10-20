use crate::{Segment, What, tests::fixtures};
use jig_domain_people::{TemplateBudget, TemplateDeadline, TemplateExecutor};
use jig_domain_tasks::{Contract, Flags, LlmChoice, ModelEffort, Retention};
use skein_lib::Duration;
const CHAT: u32 = 1;
const GOAL: u32 = 2;

#[test]
fn the_chat_and_goal_templates_parse_with_their_defaults() {
    let engine = fixtures::engine(fixtures::ENGINE_SMALL).expect("templates");
    assert_eq!(
        engine.templates.as_ref(),
        &[
            jig_domain_core::Template {
                number: CHAT,
                name: Box::from(b"chat".as_slice()),
                executor: TemplateExecutor::Charter(CHAT),
                contract: Contract::Report { words: 512 },
                shallower: 1,
                budget: TemplateBudget::AsAllowed { cap: None },
                deadline: TemplateDeadline::None,
                tracked: None,
                flags: Flags { conversation: true, listed: false, retention: Retention::Kept },
                label: 1,
                grace: Some(Duration::from_secs(5)),
                llm: None,
            },
            jig_domain_core::Template {
                number: GOAL,
                name: Box::from(b"goal".as_slice()),
                executor: TemplateExecutor::Chosen,
                contract: Contract::Report { words: 512 },
                shallower: 1,
                budget: TemplateBudget::Given,
                deadline: TemplateDeadline::Given,
                tracked: Some(1),
                flags: Flags { conversation: false, listed: true, retention: Retention::Kept },
                label: 2,
                grace: None,
                llm: None,
            },
        ]
    );
}
#[test]
fn a_template_grace_past_the_tasks_limit_refuses_the_start() {
    fixtures::refuses(
        br#""grace_ms": 5000"#,
        br#""grace_ms": 600001"#,
        fixtures::path(&[b"templates"]).child(Segment::Index(0)).member(b"grace_ms"),
        What::Grace { template: CHAT },
    );
}
#[test]
fn a_template_numbered_twice_is_refused() {
    fixtures::refuses(
        br#""number": 2, "name": "goal""#,
        br#""number": 1, "name": "goal""#,
        fixtures::path(&[b"templates"]).child(Segment::Index(1)).member(b"number"),
        What::Unique,
    );
}
#[test]
fn a_template_naming_an_unknown_charter_is_refused() {
    fixtures::refuses(
        br#""executor": {"charter": 1}"#,
        br#""executor": {"charter": 9}"#,
        fixtures::path(&[b"templates"]).child(Segment::Index(0)).member(b"executor"),
        What::Refers,
    );
}
#[test]
fn an_approval_contract_in_a_template_is_refused() {
    fixtures::refuses(
        br#""contract": {"report": {"words": 512}}"#,
        br#""contract": {"approval": {}}"#,
        fixtures::path(&[b"templates"]).child(Segment::Index(0)).member(b"contract"),
        What::Form,
    );
}
#[test]
fn a_seeded_override_naming_no_template_refuses_the_start() {
    let document = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""templates": []"#,
        br#""templates": [{"template": 99, "executor": null, "budget": null, "deadline": null, "priority": 5}]"#,
    );
    let problems = fixtures::engine(&document).expect_err("unknown template");
    assert_eq!(problems.problems.get(0).expect("one refusal").what, What::Refers);
    assert_eq!(
        problems.problems.get(0).expect("one refusal").path,
        fixtures::path(&[b"projects"])
            .child(Segment::Index(0))
            .member(b"policy")
            .member(b"templates")
            .child(Segment::Index(0))
            .member(b"template")
    );
}
#[test]
fn a_seeded_policy_overriding_a_template_twice_refuses_the_start() {
    fixtures::refuses(
        br#""templates": []"#,
        br#""templates": [{"template": 2, "executor": null, "budget": null, "deadline": null, "priority": 5},
            {"template": 2, "executor": null, "budget": null, "deadline": null, "priority": 7}]"#,
        fixtures::path(&[b"projects"])
            .child(Segment::Index(0))
            .member(b"policy")
            .member(b"templates")
            .child(Segment::Index(1))
            .member(b"template"),
        What::Unique,
    );
}

#[test]
fn a_seeded_override_reaches_the_cores_config() {
    const PROJECT: u32 = 1;
    const GOAL: u32 = 2;
    let document = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""templates": []"#,
        br#""templates": [{"template": 2, "executor": null, "budget": null, "deadline": null, "priority": 5}]"#,
    );
    let engine = fixtures::engine(&document).expect("seeded override");
    let config = crate::core(&engine, [0; 16], 7);
    assert_eq!(
        config.template_overrides.get(&PROJECT),
        Some(&Box::from([jig_domain_people::TemplateOverride {
            template: GOAL,
            executor: None,
            budget: None,
            deadline: None,
            priority: Some(5),
        }]))
    );
}

/// The small engine whose model "small" takes two efforts, beside a model "large" that takes
/// none; charter 1 allows "large" too when `alternatives` names it.
fn with_models(alternatives: &[u8], llm: &[u8]) -> Box<[u8]> {
    let efforts = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""output": 1024,"#,
        br#""output": 1024, "efforts": ["low", "high"],"#,
    );
    let large = fixtures::with(
        &efforts,
        b"\"unit\": 1000}\n}]",
        br#""unit": 1000}
}, {"name": "large", "endpoint": "fake", "window": 8192, "output": 1024,
"prices": {"input": 1, "cached": 1, "output": 1, "unit": 1000}}]"#,
    );
    let charter = fixtures::with(&large, br#""alternatives": []"#, alternatives);
    fixtures::with(&charter, br#""grace_ms": 5000, "llm": null"#, llm)
}
fn chat_llm() -> crate::Path {
    fixtures::path(&[b"templates"]).child(Segment::Index(0)).member(b"llm")
}
fn refuses_choice(alternatives: &[u8], llm: &[u8], path: crate::Path, what: What) {
    let problems = fixtures::engine(&with_models(alternatives, llm)).expect_err("refused choice");
    assert_eq!(problems.problems.get(0), Some(&crate::Problem { path, what }));
}
#[test]
fn a_templates_llm_choice_names_configured_models_and_their_efforts() {
    let document = with_models(
        br#""alternatives": ["large"]"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "high"},
            "sub_agents": [{"model": "large", "effort": null}]}"#,
    );
    let engine = fixtures::engine(&document).expect("template choice");
    let chat = engine.templates.first().expect("chat");
    assert_eq!(
        chat.llm,
        Some(LlmChoice {
            main: ModelEffort { model: 0, effort: Some(1) },
            subagents: Box::from([ModelEffort { model: 1, effort: None }]),
        })
    );
}
fn a_template_without_an_llm_member_is_refused() {
    refuses_choice(br#""alternatives": []"#, br#""grace_ms": 5000"#, chat_llm(), What::Missing);
}
fn a_choice_that_is_not_an_object_is_refused() {
    refuses_choice(br#""alternatives": []"#, br#""grace_ms": 5000, "llm": "default""#, chat_llm(), What::Kind);
}
fn a_choice_naming_an_unconfigured_model_is_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "huge", "effort": null}, "sub_agents": []}"#,
        chat_llm().member(b"main").member(b"model"),
        What::Refers,
    );
}
fn an_effort_its_model_does_not_take_is_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "medium"}, "sub_agents": []}"#,
        chat_llm().member(b"main").member(b"effort"),
        What::Refers,
    );
}
fn no_effort_for_a_model_that_declares_some_is_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": null}, "sub_agents": []}"#,
        chat_llm().member(b"main").member(b"effort"),
        What::Refers,
    );
}
fn an_effort_for_a_model_that_declares_none_is_refused() {
    refuses_choice(
        br#""alternatives": ["large"]"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "low"},
            "sub_agents": [{"model": "large", "effort": "low"}]}"#,
        chat_llm().member(b"sub_agents").child(Segment::Index(0)).member(b"effort"),
        What::Refers,
    );
}
fn an_effort_longer_than_smiths_bound_is_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small",
            "effort": "an-effort-named-past-thirty-two-bytes"}, "sub_agents": []}"#,
        chat_llm().member(b"main").member(b"effort"),
        What::Bound,
    );
}
fn a_model_outside_the_templates_charter_is_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "low"},
            "sub_agents": [{"model": "large", "effort": null}]}"#,
        chat_llm().member(b"sub_agents").child(Segment::Index(0)).member(b"model"),
        What::Refers,
    );
}
fn a_sub_agent_model_named_twice_is_refused() {
    refuses_choice(
        br#""alternatives": ["large"]"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "low"},
            "sub_agents": [{"model": "large", "effort": null}, {"model": "large", "effort": null}]}"#,
        chat_llm().member(b"sub_agents").child(Segment::Index(1)).member(b"model"),
        What::Unique,
    );
}
fn sub_agents_that_are_not_a_list_are_refused() {
    refuses_choice(
        br#""alternatives": []"#,
        br#""grace_ms": 5000, "llm": {"main": {"model": "small", "effort": "low"}, "sub_agents": {}}"#,
        chat_llm().member(b"sub_agents"),
        What::Kind,
    );
}
#[test]
fn a_templates_llm_choice_problems_name_the_invalid_fields() {
    table([
        ("a_template_without_an_llm_member_is_refused", a_template_without_an_llm_member_is_refused),
        ("a_choice_that_is_not_an_object_is_refused", a_choice_that_is_not_an_object_is_refused),
        ("a_choice_naming_an_unconfigured_model_is_refused", a_choice_naming_an_unconfigured_model_is_refused),
        ("an_effort_its_model_does_not_take_is_refused", an_effort_its_model_does_not_take_is_refused),
        ("no_effort_for_a_model_that_declares_some_is_refused", no_effort_for_a_model_that_declares_some_is_refused),
        ("an_effort_for_a_model_that_declares_none_is_refused", an_effort_for_a_model_that_declares_none_is_refused),
        ("an_effort_longer_than_smiths_bound_is_refused", an_effort_longer_than_smiths_bound_is_refused),
        ("a_model_outside_the_templates_charter_is_refused", a_model_outside_the_templates_charter_is_refused),
        ("a_sub_agent_model_named_twice_is_refused", a_sub_agent_model_named_twice_is_refused),
        ("sub_agents_that_are_not_a_list_are_refused", sub_agents_that_are_not_a_list_are_refused),
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
