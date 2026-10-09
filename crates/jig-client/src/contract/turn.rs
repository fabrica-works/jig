//! Typed, bounded turns, decoded below the domain; client/domain/client.md, 10.
//! No transcript library or wire bytes enter the domain. Calls retain the tool and
//! input as written; the outcome is a summary and optional jig object references.
use super::{ObjectKey, Part, Text};
use skein_lib::List;
/// The client view of one committed model turn.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Turn {
    pub messages: List<Message>,
    pub used: Usage,
    pub spent: u64,
    pub read_through: Option<u64>,
    /// Some messages, blocks or text did not fit the receiving limits.
    pub cut: bool,
}

/// What a turn consumed, in integer units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Usage {
    pub input: u64,
    pub output: u64,
    pub cached: u64,
}

/// A speaker and its blocks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Message {
    pub speaker: Speaker,
    pub blocks: List<Block>,
}

/// The speaker of one turn message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Speaker {
    /// The person.
    Person,
    /// The agent.
    Assistant,
    /// A tool result.
    Tool,
    /// Instructions.
    System,
}

/// A message block to show.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Block {
    /// Words as written.
    Text(Text),
    /// An uninterpreted provider block, with its dialect and byte count.
    Opaque { dialect: Text, size: u64 },
    /// A tool call.
    Call(Call),
    /// A tool result summary.
    Result { call: u64, summary: Text, failed: bool },
}

/// A tool call with its verbatim input and summary.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Call {
    pub number: u64,
    pub tool: Text,
    pub input: Text,
    pub outcome: Option<Text>,
    pub failed: bool,
    pub objects: List<ObjectKey>,
    pub parts: List<Part>,
}

/// Receiving bounds for a decoded turn; total text counts every retained field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TurnLimits {
    pub messages: u32,
    pub blocks: u32,
    pub text: u32,
}
