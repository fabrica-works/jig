//! Fields retain text and count domain writes; client/domain/client.md, 9.
//! Jig 21 adds draft and confirmation policies. Person edits never advance writes.
use crate::contract::Text;
/// Why a person is typing into a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Purpose {
    /// The first words of a chat.
    NewChat,
    /// Words for a task.
    Composer { task: u64 },
    /// The open confirmation reason.
    Reason,
    /// Amendment words.
    Amendment,
    /// Amended instructions.
    Instructions,
    /// A new goal specification.
    NewGoal,
}

/// Text and the count of domain writes that the view observes.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Field {
    pub text: Text,
    pub writes: u64,
}
