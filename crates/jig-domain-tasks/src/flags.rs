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
}
impl Flags {
    /// Ordinary work: no conversation, not listed, kept.
    pub const PLAIN: Flags = Flags { conversation: false, listed: false, retention: Retention::Kept };
}
/// How long an ended task is retained (domain/people.md, section 6; protocol/store.md, section 11.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Retention {
    /// Kept to the deployment's retention horizon.
    Kept,
    /// Erased once read by its requesting party and named by no live task.
    ForgottenOnceRead,
}
