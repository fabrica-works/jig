//! Owned activation facts, distinct from best-effort run observations.
//! See client/domain/objects.md, 3, and domain/tasks.md, 5.1–5.2.
use super::{CallIdentity, Closing, FailureClass, Text};
use skein_lib::{Token, Wall};

/// Exact current owner activation stage; an attempt does not imply observed work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RunStage {
    Idle,
    Due,
    Preparing,
    Claimed { attempt: u64 },
    Running { attempt: u64 },
    BackingOff { until: Wall },
}

/// Whole stage facts with independently supplied timing and observation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActiveRun {
    pub stage: RunStage,
    pub phase_since: Wall,
    /// Latest retained attempt, including while backing off.
    pub attempt: Option<u64>,
    /// Placement/adoption receipt time; this is not an observed agent start.
    pub placement: Option<Wall>,
    pub retry: Option<RetryNotice>,
    pub observation: Option<RunObservation>,
}

/// The actual decision that paused activation; refusal spends no failure try.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RetryNotice {
    Failed { class: FailureClass, count: u32, allowance: u32 },
    Refused { count: u32 },
}

/// Fenced owner observation. Every wall is an engine receipt clock.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RunObservation {
    pub channel: Token,
    pub work: RunWork,
    pub since: Wall,
    pub reported_at: Wall,
    pub started: Option<Wall>,
}

/// Work reported by the observed owner, never inferred from placement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RunWork {
    Preparing,
    Starting,
    Active,
    Writing,
    Tool { call: CallIdentity, tool: Text },
    Waiting,
    Ending,
}

/// Ordered owner closing obligation, separate from connector wait identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClosingStage {
    Run { attempt: u64 },
    Delegates,
    Effects,
    Releases,
    Settled,
}

/// The retained prior lifecycle while the current phase is held.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HeldRuntime {
    Waiting,
    Active(ActiveRun),
    Closing(Closing),
}
