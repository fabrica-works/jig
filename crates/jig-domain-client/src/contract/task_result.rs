//! A task supplies its ending; separately supplied result documents keep their own facts.
//! See client/domain/contract.md, 5.1, and pages.md, 5.2.
use super::{OwnerChipEnding, Result};
use alloc::boxed::Box;

/// The exact facts supplied with a task, without reconstructing another document.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TaskResult {
    /// A complete separately supplied result document, including its own time and acts.
    Document(Box<Result>),
    /// The task owner's ending, which does not supply a result document's metadata.
    Ending(Box<OwnerChipEnding>),
}
