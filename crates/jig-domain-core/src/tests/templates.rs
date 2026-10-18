use super::*;
const OWNER: u64 = 1;
const PROJECT: u32 = 1;
const UNKNOWN: u32 = 99;

#[test]
fn a_policy_change_naming_an_unknown_template_is_refused() {
    let (mut core, _) = make_tree();
    let change = people::PolicyChange::Templates {
        overrides: Box::new([people::TemplateOverride {
            template: UNKNOWN,
            executor: None,
            budget: None,
            deadline: None,
            priority: Some(5),
        }]),
    };
    assert_eq!(core.change_policy(&limits(), OWNER, PROJECT, change), Err(people::Refusal::Unknown));
}

#[test]
fn a_policy_change_past_the_templates_limit_is_refused() {
    let (mut core, _) = make_tree();
    let mut overrides = List::with_capacity(9);
    for _ in 0_u32..9 {
        overrides
            .push(people::TemplateOverride {
                template: UNKNOWN,
                executor: None,
                budget: None,
                deadline: None,
                priority: Some(5),
            })
            .expect("nine overrides");
    }
    let overrides = overrides.into_boxed();
    assert_eq!(
        core.change_policy(&limits(), OWNER, PROJECT, people::PolicyChange::Templates { overrides }),
        Err(people::Refusal::Limit { which: people::LimitKind::Policy })
    );
}

#[test]
fn a_policy_change_overriding_a_template_twice_is_refused_as_a_repeated_identity() {
    let (mut core, _) = make_tree();
    core.settings.templates = Box::new([template()]);
    let first = people::TemplateOverride {
        template: template().number,
        executor: None,
        budget: None,
        deadline: None,
        priority: Some(5),
    };
    let kept = people::PolicyChange::Templates { overrides: Box::new([first.clone()]) };
    core.change_policy(&limits(), OWNER, PROJECT, kept).expect("one override per template");
    let twice = people::PolicyChange::Templates {
        overrides: Box::new([first.clone(), people::TemplateOverride { priority: Some(7), ..first.clone() }]),
    };
    assert_eq!(
        core.change_policy(&limits(), OWNER, PROJECT, twice),
        Err(people::Refusal::Limit { which: people::LimitKind::Duplicate })
    );
    assert_eq!(core.template_overrides.get(&PROJECT), Some(&Box::from([first])));
}

fn template() -> Template {
    Template {
        number: PROJECT,
        name: Box::from(&b"work"[..]),
        executor: people::TemplateExecutor::Chosen,
        contract: tasks::Contract::Report { words: 32 },
        shallower: 1,
        budget: people::TemplateBudget::Given,
        deadline: people::TemplateDeadline::Given,
        tracked: Some(1),
        flags: tasks::Flags { conversation: false, listed: true, retention: tasks::Retention::Kept },
        label: 2,
        grace: None,
    }
}

#[test]
fn an_override_replaces_only_the_fields_it_names() {
    let empty =
        people::TemplateOverride { template: PROJECT, executor: None, budget: None, deadline: None, priority: None };
    for (name, overrides, expected) in [
        ("none", empty.clone(), template()),
        (
            "executor",
            people::TemplateOverride { executor: Some(people::TemplateExecutor::Charter(PROJECT)), ..empty.clone() },
            Template { executor: people::TemplateExecutor::Charter(PROJECT), ..template() },
        ),
        (
            "budget",
            people::TemplateOverride {
                budget: Some(people::TemplateBudget::AsAllowed { cap: Some(20) }),
                ..empty.clone()
            },
            Template { budget: people::TemplateBudget::AsAllowed { cap: Some(20) }, ..template() },
        ),
        (
            "deadline",
            people::TemplateOverride {
                deadline: Some(people::TemplateDeadline::Span(Duration::from_secs(2))),
                ..empty.clone()
            },
            Template { deadline: people::TemplateDeadline::Span(Duration::from_secs(2)), ..template() },
        ),
        (
            "priority",
            people::TemplateOverride { priority: Some(5), ..empty },
            Template { tracked: Some(5), ..template() },
        ),
    ] {
        assert_eq!(crate::templates::merged(&[template()], &[overrides]).as_ref(), &[expected], "{name}");
    }
}

#[test]
fn an_override_naming_no_configured_template_serves_nothing() {
    let overrides =
        people::TemplateOverride { template: UNKNOWN, executor: None, budget: None, deadline: None, priority: Some(5) };
    assert_eq!(crate::templates::merged(&[template()], &[overrides]).as_ref(), &[template()]);
}

#[test]
fn a_priority_override_leaves_an_untracked_template_untracked() {
    let untracked = Template { tracked: None, ..template() };
    let overrides =
        people::TemplateOverride { template: PROJECT, executor: None, budget: None, deadline: None, priority: Some(5) };
    assert_eq!(crate::templates::merged(core::slice::from_ref(&untracked), &[overrides]).as_ref(), &[untracked]);
}
