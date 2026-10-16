//! Template defaults served to clients (protocol/configuration.md, 3.1).
use skein_lib::Duration;
/// Who executes work filled from a template.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TemplateExecutor {
    Charter(u32),
    Chosen,
}
/// The budget the client fills in.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TemplateBudget {
    Amount(u64),
    AsAllowed { cap: Option<u64> },
    Given,
}
/// The deadline the client fills in.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TemplateDeadline {
    None,
    Span(Duration),
    Given,
}

/// A project's override of the named template; absent fields keep its defaults.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TemplateOverride {
    pub template: u32,
    pub executor: Option<TemplateExecutor>,
    pub budget: Option<TemplateBudget>,
    pub deadline: Option<TemplateDeadline>,
    pub priority: Option<u32>,
}
