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
        llm: None,
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

const OTHER_CHARTER: u32 = 2;
const FIRST_MODEL: u8 = 0;
const OTHER_MODEL: u8 = 1;
const LOW: u8 = 0;
const HIGH: u8 = 1;
/// One past this fixture's two efforts.
const UNKNOWN_EFFORT: u8 = 2;
/// One past this fixture's two configured models.
const UNKNOWN_MODEL: u8 = 2;

fn choice(main: tasks::ModelEffort, subagents: &[tasks::ModelEffort]) -> tasks::LlmChoice {
    tasks::LlmChoice { account: None, main, subagents: subagents.into() }
}

fn template_settings() -> Settings {
    let mut settings = config(41).settings;
    let first = scripted_model();
    let mut other = scripted_model();
    other.name = Box::from(&b"other"[..]);
    other.efforts = Box::new([]);
    let mut both = settings.charters.first().expect("fixture charter").clone();
    both.policy.model = first.clone();
    both.policy.alternatives = Box::new([other.clone()]);
    let mut narrow = both.clone();
    narrow.number = OTHER_CHARTER;
    narrow.policy.model = other.clone();
    narrow.policy.alternatives = Box::new([]);
    settings.models = Box::new([first, other]);
    settings.charters = Box::new([both, narrow]);
    settings.templates = Box::new([Template {
        executor: people::TemplateExecutor::Charter(PROJECT),
        llm: Some(choice(
            tasks::ModelEffort { model: FIRST_MODEL, effort: Some(HIGH) },
            &[tasks::ModelEffort { model: OTHER_MODEL, effort: None }],
        )),
        ..template()
    }]);
    settings
}

fn executor_override(number: u32) -> people::TemplateOverride {
    people::TemplateOverride {
        template: PROJECT,
        executor: Some(people::TemplateExecutor::Charter(number)),
        budget: None,
        deadline: None,
        priority: None,
    }
}

fn choice_fits(main: tasks::ModelEffort, subagents: &[tasks::ModelEffort], executor: u32) -> bool {
    let mut settings = template_settings();
    settings.templates.first_mut().expect("template").llm = Some(choice(main, subagents));
    crate::templates::valid(&settings, &[executor_override(executor)])
}

const FIRST_CHOICE: tasks::ModelEffort = tasks::ModelEffort { model: FIRST_MODEL, effort: Some(LOW) };
const OTHER_CHOICE: tasks::ModelEffort = tasks::ModelEffort { model: OTHER_MODEL, effort: None };
const NO_SUBAGENTS: &[tasks::ModelEffort] = &[];

#[test]
fn compatible_merged_template_choices_are_accepted() {
    for (name, main, subagents, executor) in [
        ("first effort", FIRST_CHOICE, [OTHER_CHOICE].as_slice(), PROJECT),
        ("last effort", tasks::ModelEffort { effort: Some(HIGH), ..FIRST_CHOICE }, NO_SUBAGENTS, PROJECT),
        ("other charter", OTHER_CHOICE, NO_SUBAGENTS, OTHER_CHARTER),
    ] {
        assert!(choice_fits(main, subagents, executor), "{name}");
    }
}

#[test]
fn merged_template_choices_outside_the_effective_charter_are_refused() {
    for (name, main, subagents, executor) in [
        ("missing main effort", tasks::ModelEffort { effort: None, ..FIRST_CHOICE }, NO_SUBAGENTS, PROJECT),
        ("unknown main model", tasks::ModelEffort { model: UNKNOWN_MODEL, effort: None }, NO_SUBAGENTS, PROJECT),
        (
            "main effort past end",
            tasks::ModelEffort { effort: Some(UNKNOWN_EFFORT), ..FIRST_CHOICE },
            NO_SUBAGENTS,
            PROJECT,
        ),
        ("main outside charter", FIRST_CHOICE, NO_SUBAGENTS, OTHER_CHARTER),
        ("sub-agent outside charter", OTHER_CHOICE, &[FIRST_CHOICE], OTHER_CHARTER),
        ("sub-agent missing effort", FIRST_CHOICE, &[tasks::ModelEffort { effort: None, ..FIRST_CHOICE }], PROJECT),
        (
            "sub-agent effort past end",
            FIRST_CHOICE,
            &[tasks::ModelEffort { effort: Some(UNKNOWN_EFFORT), ..FIRST_CHOICE }],
            PROJECT,
        ),
        (
            "sub-agent takes no effort",
            FIRST_CHOICE,
            &[tasks::ModelEffort { effort: Some(LOW), ..OTHER_CHOICE }],
            PROJECT,
        ),
        ("unknown executor", FIRST_CHOICE, NO_SUBAGENTS, UNKNOWN),
    ] {
        assert!(!choice_fits(main, subagents, executor), "{name}");
    }
}

fn template_core() -> (Core, Limits) {
    let (mut core, limits) = make_tree();
    core.settings = template_settings();
    (core, limits)
}

fn template_policy(core: &Core) -> people::PolicyValue {
    let templates = match core.template_overrides.get(&PROJECT) {
        Some(overrides) => overrides.as_ref(),
        None => &[],
    };
    policy::snapshot(
        core.authority.policy(PROJECT).expect("policy"),
        core.permission_roles.get(&PROJECT).expect("permissions"),
        templates,
    )
    .expect("snapshot")
}

#[test]
fn an_incompatible_template_edit_leaves_the_entire_policy_unchanged() {
    let (mut core, limits) = template_core();
    let before = template_policy(&core);
    let change = people::PolicyChange::Templates { overrides: Box::new([executor_override(OTHER_CHARTER)]) };
    assert_eq!(
        core.change_policy(&limits, OWNER, PROJECT, change),
        Err(people::Refusal::Authority { proposable: None, lacking: people::Lack::Shape }),
    );
    assert_eq!(template_policy(&core), before);
}

#[test]
fn invalid_restored_template_overrides_leave_the_entire_policy_unchanged() {
    let (mut core, limits) = template_core();
    let before = template_policy(&core);
    for (name, overrides) in [
        ("incompatible", Box::<[people::TemplateOverride]>::from([executor_override(OTHER_CHARTER)])),
        ("unknown charter", Box::from([executor_override(UNKNOWN)])),
        ("duplicate template", Box::from([executor_override(PROJECT), executor_override(PROJECT)])),
    ] {
        let mut restored = before.clone();
        restored.period_spend = 1; // A second field must not slip through a refused restore.
        restored.templates = overrides;
        assert!(!core.restore_policy(&limits, PROJECT, restored), "{name}");
        assert_eq!(template_policy(&core), before, "{name}");
    }
}

#[test]
fn a_valid_template_override_changes_and_restores_the_policy() {
    let (mut core, limits) = template_core();
    let overrides: Box<[people::TemplateOverride]> = Box::new([executor_override(PROJECT)]);
    let value = core
        .change_policy(&limits, OWNER, PROJECT, people::PolicyChange::Templates { overrides: overrides.clone() })
        .expect("compatible override");
    assert!(core.restore_policy(&limits, PROJECT, value));
    assert_eq!(core.template_overrides.get(&PROJECT), Some(&overrides));
}

#[test]
#[should_panic(expected = "merged template choices fit their effective executors")]
fn startup_rejects_an_incompatible_merged_template() {
    let mut config = config(41);
    config.settings = template_settings();
    config.template_overrides.insert(PROJECT, Box::new([executor_override(OTHER_CHARTER)])).expect("project room");
    let mut limits = limits();
    limits.tasks.charters = OTHER_CHARTER;
    for charter in &config.settings.charters {
        let bytes = policy_owned_bytes(&charter.policy).expect("bounded fixture policy");
        limits.run_bytes = limits.run_bytes.max(u32::try_from(bytes).expect("fixture policy size"));
    }
    drop(Core::new(config, &limits));
}
