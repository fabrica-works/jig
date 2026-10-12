//! Task resource source facts and separately supplied references (pages.md, 5.2).
use super::{OwnerResourceLine, Part};
use alloc::boxed::Box;

/// An owner row retains its metadata; a reference makes no metadata claim.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TaskResource {
    Part(Part),
    Owner(Box<OwnerResourceLine>),
}
impl TaskResource {
    /// Only an explicitly scoped application reference can issue an application read.
    #[must_use]
    pub const fn part(&self) -> Option<&Part> {
        match self {
            Self::Part(part) => Some(part),
            Self::Owner(_) => None,
        }
    }
}
