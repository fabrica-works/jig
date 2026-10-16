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
