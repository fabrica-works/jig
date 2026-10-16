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
