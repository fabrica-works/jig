//! Operations admitted in order and committed together; contract.md, 3.4.
use super::{Amendment, Revision, Text};
use alloc::boxed::Box;
use skein_lib::Duration;
/// Each operation names its compound's project.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    Message { project: u32, task: u64, words: Text },
    Amend { project: u32, task: u64, revision: Option<Revision>, change: Box<Amendment>, reason: Text },
    Cancel { project: u32, task: u64, reason: Text, grace: Option<Duration> },
    Release { project: u32, task: u64, revision: Revision, again: Option<bool> },
    Stop { project: u32, task: u64 },
    Move { project: u32, task: u64, reason: Text },
}
/// Why the request is malformed; its refusal names the place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Malformed {
    ProjectMismatch,
}
