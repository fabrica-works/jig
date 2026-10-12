//! Account failure details, translated from the component (engine.md, 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountPhrase {
    Later,
    Unloaded,
    Cancelled,
    Directory,
    File,
    Unreadable,
    Refused,
    Failed,
    Stalled,
    TooLarge,
}
impl AccountPhrase {
    pub(super) const fn number(self) -> u16 {
        match self {
            Self::Later => 9400,
            Self::Unloaded => 9401,
            Self::Cancelled => 9402,
            Self::Directory => 9403,
            Self::File => 9404,
            Self::Unreadable => 9405,
            Self::Refused => 9406,
            Self::Failed => 9407,
            Self::Stalled => 9408,
            Self::TooLarge => 9409,
        }
    }
    pub(super) const fn label(self) -> &'static [u8] {
        match self {
            Self::Later => b"Account State Later",
            Self::Unloaded => b"Account State Unloaded",
            Self::Cancelled => b"Account State Cancelled",
            Self::Directory => b"Record directory",
            Self::File => b"Record file",
            Self::Unreadable => b"Record unreadable",
            Self::Refused => b"Record refused",
            Self::Failed => b"Record failed",
            Self::Stalled => b"Record stalled",
            Self::TooLarge => b"Record too large",
        }
    }
}
