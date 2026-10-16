//! Bounded unread result and reply references (people.md, 6).
use crate::Entry;
use skein_lib::Wall;

/// One derived unread result reference; its words remain in the ended task.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ResultRef {
    pub task: u64,
    pub project: u32,
    pub at: Wall,
    /// Root-issued order in which task results committed.
    pub position: u64,
    pub forgettable: Option<Forgettable>,
}

/// Cached fields needed to erase an unread result in its mark's decision.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Forgettable {
    pub label: u32,
    pub listed: bool,
    /// Whether all conversation turns were read when this result was cached.
    pub turns_read: bool,
    /// The last turn for a conversation, absent for other tasks.
    pub last_reply: Option<u64>,
}

/// One opaque committed reply's exact immutable activity identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ReplyRef {
    pub entry: Entry,
    pub activity: u64,
}

/// A globally ordered recent update; content remains in its owner.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Update {
    Result(ResultRef),
    Reply(Entry),
    ChatReply(ReplyRef),
}

/// A party's requested read position (domain/people.md, section 6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Seen {
    /// Results committed through this position.
    Results { through: u64 },
    /// Replies of one chat through this position.
    Chat { task: u64, through: u64 },
}
