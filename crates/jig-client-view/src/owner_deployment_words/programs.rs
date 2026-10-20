//! Program login and cleanup words (accounts.md, 3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramPhrase {
    Programs,
    Awaiting,
    Received,
    Usable,
    NotSignedIn,
    Otherwise,
    Failed,
    SignedOut,
    SignOutFailed,
    Cleanup,
    Missing,
    Deadline,
    Unreadable,
    Exited,
    Unconfigured,
    Unknown,
    Closed,
    StatusUnknown,
}
impl ProgramPhrase {
    pub(super) const fn hole(self) -> Option<skein_ui::phrases::HoleKind> {
        match self {
            Self::Programs
            | Self::Awaiting
            | Self::Received
            | Self::Usable
            | Self::NotSignedIn
            | Self::Otherwise
            | Self::Failed
            | Self::SignedOut
            | Self::SignOutFailed
            | Self::Cleanup
            | Self::Missing
            | Self::Deadline
            | Self::Unreadable
            | Self::Exited
            | Self::Unconfigured
            | Self::Unknown
            | Self::Closed
            | Self::StatusUnknown => None,
        }
    }
    pub(super) const fn number(self) -> u16 {
        match self {
            Self::Programs => 9700,
            Self::Awaiting => 9701,
            Self::Received => 9702,
            Self::Usable => 9703,
            Self::NotSignedIn => 9704,
            Self::Otherwise => 9705,
            Self::Failed => 9706,
            Self::SignedOut => 9707,
            Self::SignOutFailed => 9708,
            Self::Cleanup => 9709,
            Self::Missing => 9710,
            Self::Deadline => 9711,
            Self::Unreadable => 9712,
            Self::Exited => 9713,
            Self::Unconfigured => 9714,
            Self::Unknown => 9715,
            Self::Closed => 9716,
            Self::StatusUnknown => 9717,
        }
    }
    pub(super) const fn label(self) -> &'static [u8] {
        match self {
            Self::Programs => b"Program logins",
            Self::Awaiting => b"Awaiting login inventory",
            Self::Received => b"Login inventory received",
            Self::Usable => b"Login usable",
            Self::NotSignedIn => b"Not signed in",
            Self::Otherwise => b"Signed in otherwise",
            Self::Failed => b"Login check failed",
            Self::SignedOut => b"Last sign-out confirmed",
            Self::SignOutFailed => b"Last sign-out failed",
            Self::Cleanup => b"Keep signed out",
            Self::Missing => b"Program missing",
            Self::Deadline => b"Program timed out",
            Self::Unreadable => b"Program status unreadable",
            Self::Exited => b"Program command failed",
            Self::Unconfigured => b"No program configured",
            Self::Unknown => b"Unknown program source",
            Self::Closed => b"Worker closed",
            Self::StatusUnknown => b"Login status unknown",
        }
    }
}
