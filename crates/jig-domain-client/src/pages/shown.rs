//! The subjects named by the current page's sections (client/domain/client.md, 5.1).
use crate::{
    Domain,
    address::{Address, ProjectSection, TaskSection},
    objects::Body,
    pages::Section,
};
impl Domain {
    /// Whether this page includes the section.
    #[must_use]
    pub fn shows(&self, section: Section) -> bool {
        if !self.sections().contains(&section) {
            return false;
        }
        match section {
            Section::Task { task, section } => {
                if let Some(id) = self.objects.find(crate::contract::ObjectKey::Task(task)) {
                    match &self.objects.get(id).expect("named task exists").body {
                        Some(Body::Task(task)) => return super::task::offered(&super::task::offers(task), section),
                        Some(
                            Body::Historical(_)
                            | Body::Chip(_)
                            | Body::Proposal(_)
                            | Body::OwnerProposal(_)
                            | Body::Escalation(_)
                            | Body::OwnerEscalation(_)
                            | Body::Question(_)
                            | Body::PersonTask(_)
                            | Body::OwnerPersonTask(_)
                            | Body::Result(_)
                            | Body::Note { .. },
                        )
                        | None => {}
                    }
                }
            }
            Section::Inbox { .. } | Section::Conversations { .. } | Section::Project { .. } | Section::Deployment => {}
        }
        true
    }
    /// The task named by this page, if any.
    #[must_use]
    pub fn shown_task(&self) -> Option<u64> {
        for section in self.sections() {
            match section {
                Section::Task { task, .. } => return Some(*task),
                Section::Inbox { .. }
                | Section::Conversations { .. }
                | Section::Project { .. }
                | Section::Deployment => {}
            }
        }
        None
    }
    /// The project named by this page, if any.
    #[must_use]
    pub fn shown_project(&self) -> Option<u32> {
        for section in self.sections() {
            match section {
                Section::Project { project, .. } => return Some(*project),
                Section::Inbox { .. } | Section::Conversations { .. } | Section::Task { .. } | Section::Deployment => {}
            }
        }
        None
    }
    /// The inbox filter named by this page, including a project's overview.
    #[must_use]
    pub fn shown_inbox(&self) -> Option<Option<u32>> {
        for section in self.sections() {
            match section {
                Section::Inbox { project, .. } => return Some(*project),
                Section::Project { project, section: ProjectSection::Overview } => return Some(Some(*project)),
                Section::Conversations { .. }
                | Section::Task { .. }
                | Section::Project { .. }
                | Section::Deployment => {}
            }
        }
        None
    }
    #[expect(clippy::option_option, reason = "a section may name every project or one project")]
    pub(crate) fn shown_conversations(&self) -> Option<Option<u32>> {
        for section in self.sections() {
            match section {
                Section::Conversations { project } => return Some(*project),
                Section::Inbox { .. } | Section::Task { .. } | Section::Project { .. } | Section::Deployment => {}
            }
        }
        None
    }
}

/// Builds each address's union in the sections' display order, with Settings whole.
pub(crate) fn arrange(domain: &mut Domain) {
    match domain.address {
        Address::Task { task, section: own } => {
            domain.sections.clear();
            for section in [
                TaskSection::Overview,
                TaskSection::Activity,
                TaskSection::Plan,
                TaskSection::History,
                TaskSection::Budget,
                TaskSection::Panels,
            ] {
                if section == own || domain.arrangement.task.as_slice().contains(&section) {
                    domain.sections.push(Section::Task { task, section }).expect("task sections fit");
                }
            }
        }
        Address::Project { project, section: own } => {
            domain.sections.clear();
            let settings = own == ProjectSection::Settings
                || domain.arrangement.project.as_slice().contains(&ProjectSection::Settings);
            for section in [
                ProjectSection::Overview,
                ProjectSection::Goals,
                ProjectSection::Tasks,
                ProjectSection::Notes,
                ProjectSection::Settings,
                ProjectSection::Resources,
                ProjectSection::Policy,
                ProjectSection::Roles,
                ProjectSection::Spend,
                ProjectSection::Recurring,
            ] {
                let covered = match section {
                    ProjectSection::Resources
                    | ProjectSection::Policy
                    | ProjectSection::Roles
                    | ProjectSection::Spend
                    | ProjectSection::Recurring => settings,
                    ProjectSection::Overview
                    | ProjectSection::Goals
                    | ProjectSection::Tasks
                    | ProjectSection::Notes
                    | ProjectSection::Settings => false,
                };
                if !covered && (section == own || domain.arrangement.project.as_slice().contains(&section)) {
                    domain.sections.push(Section::Project { project, section }).expect("project sections fit");
                }
            }
        }
        Address::Inbox { .. } | Address::Conversations { .. } | Address::Deployment => {}
    }
}
