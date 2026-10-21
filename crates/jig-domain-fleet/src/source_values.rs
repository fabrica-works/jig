//! Source facts at the fleet boundary (engine.md, 8).
//! Like hello bounds, these belong to the fleet; roots translate the link.
use crate::SourceName;
use alloc::boxed::Box;

/// The program's bounded, printable payment method, without login contents.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LoginMethod {
    length: u8,
    bytes: [u8; 32],
}
impl LoginMethod {
    #[must_use]
    pub fn new(value: &[u8]) -> Option<Self> {
        if value.is_empty() || value.len() > 32 {
            return None;
        }
        let mut bytes = [0; 32];
        for (index, byte) in value.iter().enumerate() {
            if !(0x21..=0x7e).contains(byte) {
                return None;
            }
            *bytes.get_mut(index)? = *byte;
        }
        Some(Self { length: u8::try_from(value.len()).ok()?, bytes })
    }
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.get(..usize::from(self.length)).expect("validated method")
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SourceFailure {
    Missing,
    Deadline,
    Unreadable,
    Exited,
    Unconfigured,
    Unknown,
    Closed,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LoginState {
    Usable,
    NotSignedIn,
    SignedInOtherwise(LoginMethod),
    Failed { reason: SourceFailure, detail: Box<[u8]> },
}
impl LoginState {
    /// A failed check does not prove absence, except when the program is unconfigured.
    #[must_use]
    pub fn kept(&self) -> bool {
        match self {
            Self::NotSignedIn | Self::Failed { reason: SourceFailure::Unconfigured, .. } => false,
            Self::Usable | Self::SignedInOtherwise(_) | Self::Failed { .. } => true,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Login {
    pub name: SourceName,
    pub state: LoginState,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SignOutResult {
    SignedOut,
    Failed { reason: SourceFailure, detail: Box<[u8]> },
}

/// Terminal status for one configured member, independent of its current channel.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProgramResult {
    SignedOut,
    NotInContact,
    Lost,
    Failed,
}

/// Whole configured worker names exposed in program facts and outcomes.
pub const WORKER_NAME: u32 = 4_096;
