//! Each phase owns its channel (protocol/channel.md, 10.3).
use crate::Limits;
use skein_channel::{Machine, WriteWait};
use skein_lib::{Time, Token};
#[derive(Debug)]
pub(crate) struct Connection {
    pub(crate) token: Token,
    pub(crate) machine: Machine,
    pub(crate) ping: Option<Time>,
    pub(crate) silence: Option<Time>,
    pub(crate) stall: Option<Time>,
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

impl Connection {
    pub(crate) fn queued(&self) -> bool {
        match self.machine.waiting().write {
            WriteWait::Frames => true,
            WriteWait::Nothing => false,
        }
    }
    pub(crate) fn live(&mut self, now: Time, limits: &Limits) {
        self.ping = Some(now.saturating_add(limits.ping));
        self.silence = Some(now.saturating_add(limits.silence));
    }
    pub(crate) fn quiet(&mut self) {
        self.ping = None;
        self.silence = None;
    }
    pub(crate) fn next(&self) -> Option<Time> {
        let ping = if self.queued() { None } else { self.ping };
        earlier(earlier(ping, self.silence), self.stall)
    }
}
pub(crate) fn earlier(a: Option<Time>, b: Option<Time>) -> Option<Time> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}
