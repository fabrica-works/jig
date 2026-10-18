use crate::{Segment, What, tests::fixtures};
use jig_domain_people::{TemplateBudget, TemplateDeadline, TemplateExecutor};
use jig_domain_tasks::{Contract, Flags, Retention};
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
