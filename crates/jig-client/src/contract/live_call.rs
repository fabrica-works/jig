//! A supplied current call is provisional, not a committed transcript call.
//! See client/domain/contract.md, 7.3, and client.md, 10.
use super::{Call, Text};
/// The owner's raw call snapshot, with its aggregate loss fact.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CurrentCall {
    pub number: u64,
    pub tool: Text,
    pub input: Text,
    pub cut: bool,
}
/// Preserve rich decoded sources alongside the owner's raw provisional call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LiveCall {
    Decoded(Call),
    Raw(CurrentCall),
}
impl LiveCall {
    /// The supplied call identity, without interpreting tool or input bytes.
    #[must_use]
    pub const fn number(&self) -> u64 {
        match self {
            Self::Decoded(call) => call.number,
            Self::Raw(call) => call.number,
        }
    }
}
