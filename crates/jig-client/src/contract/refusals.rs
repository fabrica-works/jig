//! Durable refusals with facts, never phrases; client/domain/contract.md, 4.
//! These are final answers; busy and signed out are envelopes below this vocabulary.
use super::{Ending, Executor, Grant, Revision};
use skein_lib::Wall;
/// The authority the engine found lacking.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Lacking {
    /// The budget needed and left.
    Budget { needed: u64, left: u64 },
    /// The grant missing.
    Grant(Grant),
    /// The executor kind missing.
    TaskKind(Executor),
    /// Delegation beyond depth.
    Depth { needed: u32, allowed: u32 },
    /// A deadline beyond authority.
    Deadline { asked: Wall, latest: Wall },
    /// The tasks needed and left.
    Tasks { needed: u32, left: u32 },
}

/// The admission limit reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Limit {
    /// A project live task limit.
    LiveTasks { project: u32, limit: u32 },
    /// A task inbox limit.
    Inbox { task: u64, limit: u32 },
    /// The words byte limit.
    Words { limit: u32 },
    /// The client field limit.
    Fields,
    /// The client record limit.
    Records,
    /// The client request limit.
    Requests,
    /// The client read limit.
    Reads,
    /// The watch limit.
    Watches,
    /// The client working set limit.
    Objects,
}

/// Which independently decidable facts changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Moved {
    /// The hold or its reason.
    Hold,
    /// The task specification.
    Spec,
    /// The person task question or options.
    PersonTask,
    /// The note.
    Note,
    /// The recipient at which the object waits.
    Waiting,
}

/// A durable request refusal and all facts needed to show it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Refusal {
    /// The project role does not allow the act.
    Role { project: u32, role: u16 },
    /// Authority lacked a named permission or amount.
    Authority { lacking: Lacking },
    /// An admission limit was reached.
    Limit { which: Limit },
    /// The decided thing changed.
    Moved { now: Revision, what: Moved },
    /// The object ended.
    Ended { how: Ending },
    /// No visible object with that name.
    Unknown,
    /// The deployment does not offer this act.
    NotOffered,
    /// The key names another ask.
    KeyConflict,
}
