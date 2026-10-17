//! Connector identities are named once, as in the test engine root.
pub const FIRST: u16 = 1;
pub const SECOND: u16 = 2;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connector {
    First,
    Second,
}
impl Connector {
    #[must_use]
    pub const fn number(self) -> u16 {
        match self {
            Self::First => FIRST,
            Self::Second => SECOND,
        }
    }
    #[must_use]
    pub fn of(number: u16) -> Option<Self> {
        match number {
            FIRST => Some(Self::First),
            SECOND => Some(Self::Second),
            _ => None,
        }
    }
}
