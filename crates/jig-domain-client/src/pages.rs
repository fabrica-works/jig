//! One page at a time, with a generation and its watch freshness.
//! Reads, object sources and parts belong to the current generation. See client/domain/client.md, 5 and 8.2.
use crate::address::{Address, ProjectSection, TaskSection};
use jig_client_domain::contract::Limit;
use skein_lib::List;
pub mod conversations;
pub mod deployment;
pub mod inbox;
pub mod navigation;
pub(crate) mod opening;
pub mod project;
pub(crate) mod replies;
pub(crate) mod routing;
pub(crate) mod shown;
pub use shown::SectionStatus;
pub mod task;

/// One section naming its subject (client/domain/client.md, 5.1; pages.md, 12.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Section {
    /// One inbox group and its project filter.
    Inbox { group: jig_client_domain::contract::InboxReadGroup, project: Option<u32> },
    /// The party's chats in a project or any.
    Conversations { project: Option<u32> },
    /// One section of one task.
    Task { task: u64, section: TaskSection },
    /// One section of one project.
    Project { project: u32, section: ProjectSection },
    /// The deployment's capacity and configuration.
    Deployment,
}
/// The most sections a page shows: 2 + 1 + 6 + 9 + 1.
pub const MOST_SECTIONS: u32 = 19;

/// Sections added beside each address's own (client/domain/client.md, 5.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arrangement {
    /// Additional sections of every task page.
    pub task: List<TaskSection>,
    /// Additional sections of every project page.
    pub project: List<ProjectSection>,
}
impl Arrangement {
    /// Jig's arrangement adds no sections.
    #[must_use]
    pub fn none() -> Self {
        Self { task: List::with_capacity(0), project: List::with_capacity(0) }
    }
    pub(crate) fn valid(&self) -> bool {
        if self.task.len() > 6 || self.project.len() > 10 {
            return false;
        }
        for (i, section) in self.task.as_slice().iter().enumerate() {
            if self.task.as_slice().get(..i).expect("prefix is within the list").contains(section) {
                return false;
            }
        }
        let settings = self.project.as_slice().contains(&ProjectSection::Settings);
        for (i, section) in self.project.as_slice().iter().enumerate() {
            if self.project.as_slice().get(..i).expect("prefix is within the list").contains(section) {
                return false;
            }
            match section {
                ProjectSection::Resources
                | ProjectSection::Policy
                | ProjectSection::Roles
                | ProjectSection::Spend
                | ProjectSection::Recurring => {
                    if settings {
                        return false;
                    }
                }
                ProjectSection::Overview
                | ProjectSection::Goals
                | ProjectSection::Tasks
                | ProjectSection::Notes
                | ProjectSection::Settings => {}
            }
        }
        true
    }
}

/// The visible page, including states unrelated to work.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Page {
    /// Waiting for the frame's first delivery.
    Starting,
    /// A jig page, with its generation and freshness.
    Work { address: Address, generation: u32, behind: bool, excess: u32, refused: Option<Limit> },
    /// A page owned by another child of the root.
    Elsewhere,
    /// The sign in has ended.
    SigningIn,
    /// No visible page or object exists here.
    Missing,
    /// This client is outside the engine's versions.
    OutOfDate,
    /// Kept work belongs to another party and stays hidden.
    SomeoneElse,
}

/// The page policy awaiting a durable keyed answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntentPurpose {
    /// One offered Project settings replacement.
    ProjectSettings,
    /// A new chat whose task page opens once made.
    StartChat,
    /// A goal made or proposed from the project board.
    SetGoal,
    /// One explicitly opened read position.
    Seen(jig_client_domain::contract::Seen),
}
