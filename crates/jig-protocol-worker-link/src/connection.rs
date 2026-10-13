//! Each phase owns its channel (protocol/channel.md, 10.3).
use skein_channel::Machine;
use skein_lib::{Time, Token};
#[derive(Debug)]
pub(crate) struct Connection {
    pub(crate) token: Token,
    pub(crate) machine: Machine,
}
#[derive(Debug)]
pub(crate) enum Phase {
    Idle { at: Time },
    Connecting,
    Opening { connection: Connection, until: Time },
    Open { connection: Connection, proved: Option<Time>, hello: bool },
    Closing { connection: Connection, retry: Time, until: Option<Time> },
}
impl Phase {
    pub(crate) fn connection(&mut self) -> Option<&mut Connection> {
        match self {
            Self::Opening { connection, .. } | Self::Open { connection, .. } | Self::Closing { connection, .. } => {
                Some(connection)
            }
            Self::Idle { .. } | Self::Connecting => None,
        }
    }
    pub(crate) fn token(&self) -> Option<Token> {
        match self {
            Self::Opening { connection, .. } | Self::Open { connection, .. } | Self::Closing { connection, .. } => {
                Some(connection.token)
            }
            Self::Idle { .. } | Self::Connecting => None,
        }
    }
}
