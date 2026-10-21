//! A task's immutable identity and retention (domain/tasks.md, sections 3 and 7).
/// The flags a task is made with; amendments never change them.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Flags {
    /// Turns are entries for its requesting party; only agent tasks have turns.
    pub conversation: bool,
    /// Ended tasks are indexed by project and label.
    pub listed: bool,
    /// How long the ended task is kept.
    pub retention: Retention,
    /// Which authority may end this task, fixed at creation.
    pub ended_by: EndedBy,
}
impl Flags {
    /// Ordinary work: no conversation, not listed, kept.
    pub const PLAIN: Flags =
        Flags { ended_by: EndedBy::Result, conversation: false, listed: false, retention: Retention::Kept };
}
/// How long an ended task is retained (domain/people.md, section 6; protocol/store.md, section 11.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Retention {
    /// Kept to the deployment's retention horizon.
    Kept,
    /// Erased once read by its requesting party and named by no live task.
    ForgottenOnceRead,
}

/// The immutable authority that may end a task (domain/tasks.md, 5.7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EndedBy {
    Result,
    Requester,
}

/// Requester authority uses the existing requester, never a new cancel right.
pub(crate) fn valid(flags: Flags, executor: crate::Executor, requester: crate::Party) -> bool {
    if flags.ended_by == EndedBy::Result {
        return true;
    }
    let agent = match executor {
        crate::Executor::Agent { .. } => true,
        crate::Executor::Person(_) | crate::Executor::Procedure { .. } => false,
    };
    let person = match requester {
        crate::Party::Person(number) => number != 0,
        crate::Party::Task(_) | crate::Party::Deployment { .. } => false,
    };
    flags.conversation && agent && person
}
