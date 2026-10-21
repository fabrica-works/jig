//! Facts about the independent revisions a party decides about
//! (client/domain/contract.md, section 5.2). The parent translates task state
//! into these values; people keeps them with a keyed refusal for replay.

/// The current revision of a changed decision subject.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Moved {
    /// A hold began anew or its reason changed.
    Hold { revision: u64, reason: HoldReason },
    /// A task's words, parameters, dependencies or policy changed.
    Spec { revision: u64 },
    /// A person task's question, options or shown results changed.
    Person { revision: u64 },
    /// A note changed after it was recalled.
    Note { revision: u32 },
}

/// The reason a task is held, in the party's vocabulary.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HoldReason {
    /// Its retry allowance was exhausted.
    Failures { class: FailureClass },
    /// Its executor stopped progressing.
    Stalled,
    /// An input failed or was cancelled.
    Dependency { task: u64 },
    /// An authorized stop has no named party.
    Stopped,
    /// A named party stopped the task.
    StoppedBy { party: u64 },
    /// A resource drifted away from what the task needs.
    Drift,
    /// Its effects need a decision.
    Effects,
    /// An effect failed permanently.
    EffectFailed,
    /// An effect's result is uncertain.
    Uncertain { entry: u64 },
    /// It lacks budget for its next activation.
    Budget,
    /// Its deadline has passed.
    Deadline,
    /// It waited too long for its resource holds.
    HoldsWaited,
    /// Joining a queue would close a circle of waits (domain/tasks.md, 6.7).
    Circle,
    /// A required resource is unavailable.
    ResourceUnavailable,
    /// Adoption no longer permits its workspace writes.
    ResourceAccess,
    /// Its allocation in a pool disappeared.
    PoolLost { pool: u64 },
    /// Its procedure asked for a decision.
    Procedure,
}

/// The class of failures that exhausted a task's retry allowance.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FailureClass {
    /// A temporary neighbour or read failure.
    Transient,
    /// A permanent failure awaiting a change outside the engine.
    Permanent,
    /// The executor reported failure.
    Run,
    /// The agent crashed, stalled or broke its channel.
    Agent,
    /// The host disappeared.
    Lost,
    /// The result broke its contract.
    Invalid,
}
