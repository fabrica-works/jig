//! The agent capability that refused a hello (engine.md, section 8).
use crate::ui::phrases::HoleKind;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentPhrase {
    Model,
    Window,
    Output,
    Turns,
    Spend,
    Time,
    Waiting,
    Checks,
    Message,
    Inbox,
    Unacknowledged,
    ModelPosition,
}
impl AgentPhrase {
    pub(super) const fn number(self) -> u16 {
        match self {
            Self::Model => 9440,
            Self::Window => 9441,
            Self::Output => 9442,
            Self::Turns => 9443,
            Self::Spend => 9444,
            Self::Time => 9445,
            Self::Waiting => 9446,
            Self::Checks => 9447,
            Self::Message => 9448,
            Self::Inbox => 9449,
            Self::Unacknowledged => 9450,
            Self::ModelPosition => 9451,
        }
    }
    pub(super) const fn hole(self) -> Option<HoleKind> {
        match self {
            Self::Model | Self::Window | Self::Output | Self::ModelPosition => Some(HoleKind::Number),
            Self::Turns
            | Self::Spend
            | Self::Time
            | Self::Waiting
            | Self::Checks
            | Self::Message
            | Self::Inbox
            | Self::Unacknowledged => None,
        }
    }
    pub(super) const fn label(self) -> &'static [u8] {
        match self {
            Self::Model => b"Agent model missing for charter ",
            Self::Window => b"Agent window too small for charter ",
            Self::Output => b"Agent output too small for charter ",
            Self::Turns => b"Agent turns too few",
            Self::Spend => b"Agent spend too small",
            Self::Time => b"Agent time too short",
            Self::Waiting => b"Agent waiting too short",
            Self::Checks => b"Agent checks exceed the change reserve",
            Self::Message => b"Agent message bound too small",
            Self::Inbox => b"Agent inbox too small",
            Self::Unacknowledged => b"Agent unacknowledged turns too few",
            Self::ModelPosition => b"Model position ",
        }
    }
}
