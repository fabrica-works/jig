//! Durable indexes for parties' reads (protocol/store.md, section 7.4).
use skein_lib::Wall;

/// A number ordered newest first by its stored complement.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Newest(u64);
impl Newest {
    #[must_use]
    pub const fn of(value: u64) -> Self {
        Self(!value)
    }
    #[must_use]
    pub const fn value(self) -> u64 {
        !self.0
    }
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

/// A result sorts before a reply of the same task and time.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum UpdateObject {
    Result,
    Reply { message: Newest, activity: Newest },
}

/// One person's result or reply, saved with the record it names.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Update {
    pub party: u64,
    pub at: Wall,
    pub task: u64,
    pub object: UpdateObject,
}
impl Update {
    #[must_use]
    pub const fn key(self) -> Key {
        Key::Update {
            party: self.party,
            at: Newest::of(self.at.as_nanos()),
            task: Newest::of(self.task),
            object: self.object,
        }
    }
}

/// An ended chat kept in both conversation orders until it is forgotten.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Conversation {
    pub party: u64,
    pub task: u64,
}

/// Logical store `key` for task state or financial history, with no byte encoding performed
/// in this child. (domain/tasks.md, section 2).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Key {
    /// Logical mutable live-task row.
    Live(u64),
    /// Working-set pointer retained while a live task names the ending.
    Stub(u64),
    /// Logical finite period/pool row.
    Ledger(crate::Funder),
    /// One occupied writer slot for a held resource.
    Writer(u64),
    /// Last known capacity for a connector pool.
    Pool(u64),
    /// A person-origin proposal.
    PersonProposal(u64),
    /// Logical historical ended-task row.
    Ended(u64),
    /// A past funding period.
    ClosedLedger(crate::Funder),
    /// A decided person proposal.
    DecidedProposal(u64),
    /// Immutable plan change outside the live arena.
    History {
        task: u64,
        revision: u64,
    },
    /// One immutable lifecycle milestone in a task's history.
    Milestone {
        task: u64,
        position: u64,
    },
    /// One immutable line in the ordered task activity range.
    Activity {
        task: u64,
        position: u64,
    },
    /// One party's update in newest-first order.
    Update {
        party: u64,
        at: Newest,
        task: Newest,
        object: UpdateObject,
    },
    ConversationNewest {
        party: u64,
        task: Newest,
    },
    ConversationOldest {
        party: u64,
        task: u64,
    },
}

impl Key {
    /// Whether a restart loads this family (protocol/store.md, 7.2).
    #[must_use]
    pub const fn live(&self) -> bool {
        match self {
            Self::Live(_)
            | Self::Stub(_)
            | Self::Ledger(_)
            | Self::Writer(_)
            | Self::Pool(_)
            | Self::PersonProposal(_) => true,
            Self::Ended(_)
            | Self::ClosedLedger(_)
            | Self::DecidedProposal(_)
            | Self::History { .. }
            | Self::Milestone { .. }
            | Self::Activity { .. }
            | Self::Update { .. }
            | Self::ConversationNewest { .. }
            | Self::ConversationOldest { .. } => false,
        }
    }
}
