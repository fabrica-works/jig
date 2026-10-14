//! The subjects named by the current page's sections (client/domain/client.md, 5.1).
use crate::{Domain, address::ProjectSection, pages::Section};
impl Domain {
    /// Whether this page includes the section.
    #[must_use]
    pub fn shows(&self, section: Section) -> bool {
        self.sections().contains(&section)
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
