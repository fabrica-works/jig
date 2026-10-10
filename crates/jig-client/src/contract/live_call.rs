//! A supplied current call is provisional, not a committed transcript call.
//! See client/domain/contract.md, 7.3, client.md, 10, and QUESTIONS.md, 42.
use super::{Call, Text};
use skein_lib::{Token, Wall};
/// The provider's id within its actual owner and issued completion.
/// Equality alone names a call; no field is a routing or transcript number.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CallIdentity {
    pub owner: Token,
    pub sequence: u32,
    pub retry: u32,
    pub id: Text,
}
/// The owner's raw call snapshot, with aggregate loss and supplied first-seen clock.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CurrentCall {
    pub identity: CallIdentity,
    pub tool: Text,
    pub input: Text,
    pub cut: bool,
    pub began: Option<Wall>,
}
/// Preserve rich decoded sources alongside the owner's raw provisional call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LiveCall {
    Decoded(Call),
    Raw(CurrentCall),
}
