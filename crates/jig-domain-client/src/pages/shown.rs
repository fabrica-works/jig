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

pub(crate) fn owner(domain: &Domain) -> crate::watches::Owner {
    match domain.page {
        super::Page::Elsewhere => crate::watches::Owner::Sections { generation: domain.generation },
        super::Page::Starting
        | super::Page::Work { .. }
        | super::Page::SigningIn
        | super::Page::Missing
        | super::Page::OutOfDate
        | super::Page::SomeoneElse => crate::watches::Owner::Page { generation: domain.generation },
    }
}

pub(crate) fn show(
    domain: &mut Domain,
    env: &skein_lib::Env<crate::Limits>,
    sections: skein_lib::List<Section>,
    out: &mut skein_lib::Queue<crate::Request>,
) {
    if domain.page != super::Page::Elsewhere || domain.confirmed.is_none() || domain.link != crate::link::Link::Live {
        return;
    }
    if sections.len() > super::MOST_SECTIONS {
        super::routing::refuse(domain, crate::contract::Limit::Objects);
    }
    let sections = sections
        .as_slice()
        .get(..usize::try_from(sections.len().min(super::MOST_SECTIONS)).expect("bounded section count"))
        .expect("bounded prefix");
    let previous = domain.sections.clone();
    domain.sections.clear();
    for section in sections {
        let fits = match *section {
            Section::Task { task, .. } => domain.shown_task().is_none() || domain.shown_task() == Some(task),
            Section::Project { project, section } => {
                let covered = match section {
                    ProjectSection::Resources
                    | ProjectSection::Policy
                    | ProjectSection::Roles
                    | ProjectSection::Spend
                    | ProjectSection::Recurring => {
                        sections.contains(&Section::Project { project, section: ProjectSection::Settings })
                    }
                    ProjectSection::Overview
                    | ProjectSection::Goals
                    | ProjectSection::Tasks
                    | ProjectSection::Notes
                    | ProjectSection::Settings => false,
                };
                !covered
                    && (domain.shown_project().is_none() || domain.shown_project() == Some(project))
                    && (section != ProjectSection::Overview
                        || (domain.shown_inbox().is_none() || domain.shown_inbox() == Some(Some(project))))
            }
            Section::Inbox { project, .. } => domain.shown_inbox().is_none() || domain.shown_inbox() == Some(project),
            Section::Conversations { project } => {
                domain.shown_conversations().is_none() || domain.shown_conversations() == Some(project)
            }
            Section::Deployment => true,
        };
        if fits && !domain.sections.as_slice().contains(section) {
            domain.sections.push(*section).expect("section count checked");
        }
    }

    let owner = owner(domain);
    for (id, ()) in &domain.watch_ids {
        let watch = domain.watches.get(*id).expect("registered watch exists");
        if watch.owner == owner && !needed(domain, &watch.subject) {
            let watch = domain.watches.get_mut(*id).expect("registered watch exists");
            crate::domain::close_watch(watch, *id, &mut domain.timers, out);
        }
    }
    for index in 0..domain.sections.len() {
        let section = *domain.sections.get(index).expect("section index exists");
        if !previous.as_slice().contains(&section) {
            super::routing::open_section(domain, env, section, owner, out);
        }
    }
    if previous != domain.sections {
        super::task::document(domain, env, out);
        domain.changed();
    }
}

pub(crate) fn needed(domain: &Domain, subject: &crate::contract::Subject) -> bool {
    use crate::contract::Subject;
    match subject {
        Subject::Frame => false,
        Subject::Task { task } => domain.shown_task() == Some(*task),
        Subject::Tree { task, .. } => domain.shows(Section::Task { task: *task, section: TaskSection::Plan }),
        Subject::Run { task, .. } => domain.shows(Section::Task { task: *task, section: TaskSection::Activity }),
        Subject::Chips { .. } => {
            domain.conversation.task != 0
                && domain.shows(Section::Task { task: domain.conversation.task, section: TaskSection::Activity })
        }
        Subject::Inbox { .. } => domain.shown_inbox().is_some(),
        Subject::Conversations => domain.shown_conversations().is_some(),
        Subject::Goals { project } => {
            domain.shows(Section::Project { project: *project, section: ProjectSection::Overview })
                || domain.shows(Section::Project { project: *project, section: ProjectSection::Goals })
        }
        Subject::Tasks { project, .. } => {
            domain.shows(Section::Project { project: *project, section: ProjectSection::Tasks })
        }
        Subject::Deployment => domain.shows(Section::Deployment),
    }
}
