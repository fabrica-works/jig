//! One page at a time, with a generation and its watch freshness.
//! Reads, object sources and parts belong to the current generation. See client/domain/client.md, 5 and 8.2.
use crate::{address::Address, contract::Limit};
pub mod conversations;
pub mod deployment;
pub mod inbox;
pub mod navigation;
pub(crate) mod opening;
pub mod project;
pub(crate) mod replies;
pub(crate) mod routing;
pub(crate) mod shown;
pub mod task;

/// One section naming its subject (client/domain/client.md, 5.1; pages.md, 12.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Section {
    /// One inbox group and its project filter.
    Inbox { group: crate::contract::InboxReadGroup, project: Option<u32> },
    /// The party's chats in a project or any.
    Conversations { project: Option<u32> },
    /// One section of one task.
    Task { task: u64, section: crate::address::TaskSection },
    /// One section of one project.
    Project { project: u32, section: crate::address::ProjectSection },
    /// The deployment's capacity and configuration.
    Deployment,
}
/// The most sections a page shows: 2 + 1 + 6 + 9 + 1.
pub const MOST_SECTIONS: u32 = 19;

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
    Seen(crate::contract::Seen),
}
