//! The reference UI's places as its own records.
use super::w;
use crate::address::{Address, ProjectSection, TaskSection};
fn project_section(value: ProjectSection) -> w::ProjectSection {
    match value {
        ProjectSection::Overview => w::ProjectSection::Overview,
        ProjectSection::Goals => w::ProjectSection::Goals,
        ProjectSection::Tasks => w::ProjectSection::Tasks,
        ProjectSection::Notes => w::ProjectSection::Notes,
        ProjectSection::Settings => w::ProjectSection::Settings,
        ProjectSection::Resources => w::ProjectSection::Resources,
        ProjectSection::Policy => w::ProjectSection::Policy,
        ProjectSection::Roles => w::ProjectSection::Roles,
        ProjectSection::Spend => w::ProjectSection::Spend,
        ProjectSection::Recurring => w::ProjectSection::Recurring,
    }
}
fn task_section(value: TaskSection) -> w::TaskSection {
    match value {
        TaskSection::Overview => w::TaskSection::Overview,
        TaskSection::Activity => w::TaskSection::Activity,
        TaskSection::Plan => w::TaskSection::Plan,
        TaskSection::History => w::TaskSection::History,
        TaskSection::Budget => w::TaskSection::Budget,
        TaskSection::Panels => w::TaskSection::Panels,
    }
}
fn restore_project_section(value: &w::ProjectSection) -> ProjectSection {
    match value {
        w::ProjectSection::Overview => ProjectSection::Overview,
        w::ProjectSection::Goals => ProjectSection::Goals,
        w::ProjectSection::Tasks => ProjectSection::Tasks,
        w::ProjectSection::Notes => ProjectSection::Notes,
        w::ProjectSection::Settings => ProjectSection::Settings,
        w::ProjectSection::Resources => ProjectSection::Resources,
        w::ProjectSection::Policy => ProjectSection::Policy,
        w::ProjectSection::Roles => ProjectSection::Roles,
        w::ProjectSection::Spend => ProjectSection::Spend,
        w::ProjectSection::Recurring => ProjectSection::Recurring,
    }
}
fn restore_task_section(value: &w::TaskSection) -> TaskSection {
    match value {
        w::TaskSection::Overview => TaskSection::Overview,
        w::TaskSection::Activity => TaskSection::Activity,
        w::TaskSection::Plan => TaskSection::Plan,
        w::TaskSection::History => TaskSection::History,
        w::TaskSection::Budget => TaskSection::Budget,
        w::TaskSection::Panels => TaskSection::Panels,
    }
}
pub(super) fn encode(value: Address, limits: &w::Limits) -> Result<w::Address, w::Problem> {
    Ok(match value {
        Address::Inbox { project } => {
            w::Address::Inbox(w::AddressInbox::new(limits, w::AddressInboxParts { project })?)
        }
        Address::Conversations { project } => {
            w::Address::Conversations(w::AddressConversations::new(limits, w::AddressConversationsParts { project })?)
        }
        Address::Task { task, section } => {
            w::Address::Task(w::AddressTask::new(limits, w::AddressTaskParts { task, section: task_section(section) })?)
        }
        Address::Project { project, section } => w::Address::Project(w::AddressProject::new(
            limits,
            w::AddressProjectParts { project, section: project_section(section) },
        )?),
        Address::Deployment => w::Address::Deployment,
    })
}
pub(super) fn decode(value: &w::Address) -> Address {
    match value {
        w::Address::Inbox(value) => Address::Inbox { project: *value.project() },
        w::Address::Conversations(value) => Address::Conversations { project: *value.project() },
        w::Address::Task(value) => Address::Task { task: value.task(), section: restore_task_section(value.section()) },
        w::Address::Project(value) => {
            Address::Project { project: value.project(), section: restore_project_section(value.section()) }
        }
        w::Address::Deployment => Address::Deployment,
    }
}
