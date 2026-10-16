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
