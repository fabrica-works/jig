//! Durable refusals with facts, never phrases; client/domain/contract.md, 4.
//! These are final answers; busy and signed out are envelopes below this vocabulary.
use super::{AdmissionLimit, Ending, Executor, Grant, HoldReason, Revision};
use alloc::boxed::Box;
use skein_lib::Wall;
/// The authority the engine found lacking.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Lacking {
    /// No policy role may accept this kind of proposal.
    Approver,
    /// The budget needed and left.
    Budget {
        needed: u64,
        left: u64,
    },
    /// The grant missing.
    Grant(Grant),
    /// Only the supplied operation kind is known.
    GrantKind {
        kind: u16,
    },
    /// No valid funding relation supports the request.
    Funding,
    Tool,
    Scope,
    /// A model the request names is beyond authority (domain/authority.md, sections 3 and 5).
    Models,
    Reference,
    ResourceAccess,
    ResourceUnavailable,
    Shape,
    Again {
        required: bool,
    },
    Account,
    Writer,
    Required {
        connector: u16,
        kind: u16,
    },
    Failed {
        connector: u16,
        kind: u16,
    },
    Unguarded {
        connector: u16,
        kind: u16,
    },
    /// The executor kind missing.
    TaskKind {
        executor: Option<Executor>,
    },
    /// Delegation beyond depth.
    Depth {
        needed: Option<u32>,
        allowed: u32,
    },
    /// A deadline beyond authority.
    Deadline {
        asked: Option<Wall>,
        latest: Option<Wall>,
    },
    /// The tasks needed and left.
    Tasks {
        needed: Option<u32>,
        left: u32,
    },
}

/// The admission limit reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Limit {
    /// An engine structural admission category, without an invented numerical bound.
    Admission { which: AdmissionLimit },
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

/// The current hold facts actually supplied by the owner.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CurrentHold {
    /// A complete held object's document.
    Document(Box<HoldReason>),
    /// Admission supplies only its exact category and numeric identities.
    Reason(HoldCategory),
}
/// Sparse engine hold facts; absent words, clocks and parts stay absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HoldCategory {
    Failures { class: HoldFailure },
    Stalled,
    Dependency { task: u64 },
    Stopped,
    StoppedBy { party: u64 },
    Drift,
    Effects,
    EffectFailed,
    Uncertain { entry: u64 },
    Budget,
    Deadline,
    HoldsWaited,
    ResourceUnavailable,
    ResourceAccess,
    PoolLost { pool: u64 },
    Procedure,
}
/// Exact admission failure categories, independent of detailed attempt failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HoldFailure {
    Transient,
    Permanent,
    Run,
    Agent,
    Lost,
    Invalid,
}

/// Which independently decidable facts changed.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Moved {
    /// The hold or its reason.
    Hold { reason: Option<CurrentHold> },
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
    /// A root task has no escalation holder.
    NoEscalationHolder,
    /// A malformed operation, at its place in the compound.
    Malformed { at: u32, why: super::Malformed },
    /// The project role does not allow the act.
    Role { held: Option<HeldRole> },
    /// Another person holds this role-addressed task.
    Taken { by: u64 },
    /// The object waits for someone else.
    NotWaiting,
    /// This holder cannot pass further.
    NoFurther,
    /// Authority lacked a named permission or amount; proposal eligibility is supplied when known.
    Authority { lacking: Lacking, proposable: Option<bool> },
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

/// The role the engine actually found held, when known.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeldRole {
    Owner,
    Maintainer,
    Member,
    Observer,
    Policy { role: u32 },
}
