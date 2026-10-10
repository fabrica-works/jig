//! Level-triggered subjects and one-terminal watches; client/domain/contract.md, 7.
//! Deliveries replace the previous document. Run text alone is offset based and
//! provisional, and never resets text belonging to an earlier turn.
use super::{Chip, Conversations, Deployment, Frame, Goals, Inbox, LiveCall, Task, Text, Tree, TurnName};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
/// A live subject followed by one unkeyed watch.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Subject {
    /// The frame, always open.
    Frame,
    /// A task.
    Task { task: u64 },
    /// A bounded live subtree.
    Tree { task: u64, window: u32 },
    /// A bounded set of task chips.
    Chips { tasks: List<u64> },
    /// Party inbox across projects.
    Inbox { window: u32 },
    /// Project live goals.
    Goals { project: u32 },
    /// Filtered live tasks in one project.
    Tasks { project: u32, filter: super::TaskFilter, window: u32 },
    /// Party live chats.
    Conversations,
    /// A task run text from the client offset.
    Run { task: u64, turn: Option<TurnName>, offset: u32 },
    /// Deployment capacity.
    Deployment,
}

/// A whole current watch value, except the run text stream.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Delivery {
    /// The frame.
    Frame(Box<Frame>),
    /// A task.
    Task(Box<Task>),
    /// A subtree.
    Tree(Box<Tree>),
    /// The named chips.
    Chips(List<Chip>),
    /// The inbox.
    Inbox(Box<Inbox>),
    /// The live goals.
    Goals(Box<Goals>),
    /// Filtered live project tasks.
    Tasks(Box<super::Tasks>),
    /// The live chats.
    Conversations(Box<Conversations>),
    /// Provisional run text.
    Run(Box<RunText>),
    /// Deployment capacity.
    Deployment(Box<Deployment>),
}

/// Provisional text named by turn, with explicit offset and truncation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RunText {
    pub attempt: u64,
    pub began: Option<Wall>,
    pub turn: TurnName,
    pub from: u32,
    pub through: u32,
    pub text: Text,
    pub cut: bool,
    pub call: Option<LiveCall>,
}

/// A nonterminal watch event.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WatchEvent {
    /// Admitted, awaiting a delivery.
    Opened,
    /// The subject current value.
    Delivered(Delivery),
    /// The heartbeat.
    Alive,
}

/// The one terminal of an open watch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WatchEnd {
    /// Closed by the client.
    Closed,
    /// Its connection failed.
    Dropped,
    /// No visible subject.
    Unknown,
    /// The party may no longer see it.
    NotAllowed,
    /// The engine watch limit is full.
    Busy,
    /// The subject ended and left live state.
    SubjectEnded,
    /// The sign in ended.
    SignedOut,
    /// The engine is stopping.
    Stopping,
}
