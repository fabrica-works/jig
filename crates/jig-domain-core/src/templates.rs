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

pub(crate) fn list_worst_case(limits: &crate::Limits) -> Option<u64> {
    let fields = u64::from(limits.tasks.result_bytes)
        .checked_mul(u64::try_from(size_of::<jig_domain_tasks::FieldRule>()).ok()?.checked_add(1)?)?;
    let choices = skein_lib::List::<jig_domain_tasks::Verdict>::worst_case(limits.tasks.contract_choices)?
        .checked_add(u64::from(limits.tasks.contract_choices).checked_mul(fields)?)?;
    skein_lib::List::<Template>::worst_case(limits.templates)?.checked_add(
        u64::from(limits.templates).checked_mul(u64::from(limits.documents.text_bytes).checked_add(choices)?)?,
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
    assert!(config.template_overrides.capacity() <= limits.authority.projects, "core prices project overrides");
    for (_, overrides) in &config.template_overrides {
        assert!(
            overrides.len() <= usize::try_from(limits.templates).expect("template bound"),
            "core prices policy overrides"
        );
    }
}
