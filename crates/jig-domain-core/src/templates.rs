//! Templates are data served to clients, never used by decisions (engine.md, 11.1).
use alloc::boxed::Box;
use jig_domain_people::{TemplateBudget, TemplateDeadline, TemplateExecutor};
use jig_domain_tasks::{Contract, Flags};
use skein_lib::Duration;
/// The application's numbered defaults for one shape of work (configuration.md, 3.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Template {
    pub number: u32,
    pub name: Box<[u8]>,
    pub executor: TemplateExecutor,
    pub contract: Contract,
    pub shallower: u32,
    pub budget: TemplateBudget,
    pub deadline: TemplateDeadline,
    pub tracked: Option<u32>,
    pub flags: Flags,
    pub label: u32,
    pub grace: Option<Duration>,
    /// The LLM choice a task filled from it carries, each model configured with an effort it
    /// takes; none for its charter's default (configuration.md, 3.1; domain/tasks.md, 3).
    pub llm: Option<jig_domain_tasks::LlmChoice>,
}

/// Apply the project's named overrides in configuration order (configuration.md, 3.1).
///
/// A policy overrides a template at most once, refused at start and on a policy change, so
/// the first override naming a template is its only one.
#[must_use]
pub fn merged(templates: &[Template], overrides: &[jig_domain_people::TemplateOverride]) -> Box<[Template]> {
    let mut merged = skein_lib::List::with_capacity(u32::try_from(templates.len()).expect("configured template count"));
    for template in templates {
        let mut value = template.clone();
        for entry in overrides {
            if entry.template != template.number {
                continue;
            }
            if let Some(executor) = entry.executor {
                value.executor = executor;
            }
            if let Some(budget) = entry.budget {
                value.budget = budget;
            }
            if let Some(deadline) = entry.deadline {
                value.deadline = deadline;
            }
            if let (Some(priority), Some(_)) = (entry.priority, value.tracked) {
                value.tracked = Some(priority);
            }
            break;
        }
        merged.push(value).expect("one merged value per configured template");
    }
    merged.into_boxed()
}

/// Whether a template's retained choice fits its effective executor (configuration.md, 3.1).
/// A chosen executor still requires configured models and valid efforts; a named charter must
/// also allow every model. This checks data served in `me`, without granting task authority.
#[must_use]
pub fn valid_executor(
    template: &Template,
    executor: TemplateExecutor,
    charters: &[crate::Charter],
    models: &[crate::Model],
) -> bool {
    if template.flags.ended_by == jig_domain_tasks::EndedBy::Requester
        && (!template.flags.conversation || executor == TemplateExecutor::Chosen)
    {
        return false;
    }
    let policy = match executor {
        TemplateExecutor::Chosen => None,
        TemplateExecutor::Charter(number) => {
            let mut found = None;
            for charter in charters {
                if charter.number == number {
                    found = Some(&charter.policy);
                    break;
                }
            }
            let Some(policy) = found else { return false };
            Some(policy)
        }
    };
    let Some(choice) = &template.llm else { return true };
    if choice.subagents.len() > usize::try_from(jig_domain_tasks::MODELS).expect("model count")
        || !takes(models, choice.main)
    {
        return false;
    }
    for chosen in &choice.subagents {
        if !takes(models, *chosen) {
            return false;
        }
    }
    match policy {
        Some(policy) => crate::llm::takes(models, policy, choice),
        None => true,
    }
}

/// Validate every effective executor before publishing merged templates, including on restore.
/// An override retained after its template was dropped has no merged value to validate.
pub(crate) fn valid(settings: &crate::Settings, overrides: &[jig_domain_people::TemplateOverride]) -> bool {
    for (at, entry) in overrides.iter().enumerate() {
        for earlier in overrides.iter().take(at) {
            if earlier.template == entry.template {
                return false;
            }
        }
    }
    for template in &settings.templates {
        let mut executor = template.executor;
        for entry in overrides {
            if entry.template == template.number {
                if let Some(value) = entry.executor {
                    executor = value;
                }
                break;
            }
        }
        if !valid_executor(template, executor, &settings.charters, &settings.models) {
            return false;
        }
    }
    true
}

pub(crate) fn list_worst_case(limits: &crate::Limits) -> Option<u64> {
    let fields = u64::from(limits.tasks.result_bytes)
        .checked_mul(u64::try_from(size_of::<jig_domain_tasks::FieldRule>()).ok()?.checked_add(1)?)?;
    let choices = skein_lib::List::<jig_domain_tasks::Verdict>::worst_case(limits.tasks.contract_choices)?
        .checked_add(u64::from(limits.tasks.contract_choices).checked_mul(fields)?)?;
    let models = u64::from(jig_domain_tasks::MODELS)
        .checked_mul(u64::try_from(size_of::<jig_domain_tasks::ModelEffort>()).ok()?)?;
    skein_lib::List::<Template>::worst_case(limits.templates)?.checked_add(
        u64::from(limits.templates)
            .checked_mul(u64::from(limits.documents.text_bytes).checked_add(choices)?.checked_add(models)?)?,
    )
}

pub(crate) fn assert_configured(config: &crate::Config, limits: &crate::Limits) {
    assert!(
        config.settings.templates.len() <= usize::try_from(limits.templates).expect("template bound"),
        "core prices configured templates"
    );
    for template in &config.settings.templates {
        assert!(
            template.name.len() <= usize::try_from(limits.documents.text_bytes).expect("template name bound"),
            "core prices template names"
        );
        assert!(jig_domain_tasks::valid_contract(&limits.tasks, &template.contract), "core prices template contracts");
    }
    assert!(valid(&config.settings, &[]), "configured template choices fit their executors");
    for project in &config.seed_projects {
        let overrides = &project.policy.templates;
        assert!(valid(&config.settings, overrides), "merged template choices fit their effective executors");
        assert!(
            overrides.len() <= usize::try_from(limits.templates).expect("template bound"),
            "core prices policy overrides"
        );
    }
}

/// Whether `chosen` names a configured model and an effort it takes: one it declares, or none
/// for a model that declares none (configuration.md, 3.1).
fn takes(models: &[crate::Model], chosen: jig_domain_tasks::ModelEffort) -> bool {
    let Some(model) = models.get(usize::from(chosen.model)) else {
        return false;
    };
    match chosen.effort {
        Some(effort) => usize::from(effort) < model.efforts.len(),
        None => model.efforts.is_empty(),
    }
}
