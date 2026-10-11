//! Level-triggered subjects and one-terminal watches; client/domain/contract.md, 7.
//! Deliveries replace the previous document. Run text alone is offset based and
//! provisional, and never resets text belonging to an earlier turn.
use super::{Chip, Conversations, CurrentCall, Deployment, Frame, Goals, Inbox, LiveCall, Task, Text, Tree, TurnName};
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
    Run { task: u64, turn: Option<ProvisionalTurnName>, offset: u32 },
    /// Deployment capacity.
    Deployment,
}

/// A whole current watch value, except the run text stream.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Delivery {
    /// Whole authentic project owner document.
    OwnerProject(Box<super::OwnerProject>),
    /// The frame.
    Frame(Box<Frame>),
    /// A task.
    Task(Box<Task>),
    /// Whole supplied pending owner proposal.
    OwnerProposal(Box<super::OwnerProposal>),
    /// A subtree.
    Tree(Box<Tree>),
    /// Whole subtree owner facts.
    OwnerTree(Box<super::OwnerTree>),
    /// The named chips.
    Chips(List<Chip>),
    /// The inbox.
    Inbox(Box<Inbox>),
    /// Whole exact owner Inbox, preserving both allocated groups.
    OwnerInbox(Box<super::OwnerInbox>),
    /// The live goals.
    Goals(Box<Goals>),
    /// Filtered live project tasks.
    Tasks(Box<super::Tasks>),
    /// The live chats.
    Conversations(Box<Conversations>),
    /// Provisional run text.
    Run(Box<RunText>),
    /// Whole current run, including a call independent of provisional writing.
    RunState(Box<RunState>),
    /// Deployment capacity.
    Deployment(Box<Deployment>),
    /// Whole deployment-owned facts and diagnostics.
    OwnerDeployment(Box<super::OwnerDeployment>),
}

/// Current owner run state; absent writing does not assert a call ended.
/// The committed position is owner progress, not activity already read by the client.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RunState {
    pub task: u64,
    pub attempt: Option<u64>,
    pub committed_turn: u32,
    pub writing: Option<RunText>,
    pub call: Option<CurrentCall>,
}

/// A provisional retry's actual attempt and activation turn, independent of a committed turn name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProvisionalTurnName {
    pub attempt: u64,
    pub number: u32,
    pub retry: u32,
}
impl ProvisionalTurnName {
    /// The attempt/turn projection used by activity fences; it does not attest a commit.
    #[must_use]
    pub const fn turn(self) -> TurnName {
        TurnName { attempt: self.attempt, number: self.number }
    }
}

/// Provisional text named by retry, with explicit offset and truncation.
/// A cut delivery replaces retained text from its supplied origin.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RunText {
    pub attempt: u64,
    pub began: Option<Wall>,
    pub turn: ProvisionalTurnName,
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
