//! A stream's name and silence deadline (protocol/client.md, 7).
use crate::Undecodable;
use jig_client_protocol_wire::v11 as w;
use skein_lib::{Duration, Time};
#[derive(Debug)]
pub(super) struct Silence {
    pub(super) name: Option<[u8; 16]>,
    last_read: Time,
    heartbeat: Duration,
}
impl Silence {
    pub(super) fn new(heartbeat: Duration, now: Time) -> Self {
        Self { name: None, last_read: now, heartbeat }
    }
    pub(super) fn silence(&self) -> Time {
        self.last_read.saturating_add(self.heartbeat.saturating_mul(2)).saturating_add(Duration::from_nanos(1))
    }
    pub(super) fn hello(&mut self, hello: &w::Hello) -> Result<(), Undecodable> {
        if self.name.is_some() || hello.heartbeat() == Duration::ZERO {
            return Err(Undecodable::Malformed);
        }
        self.name = Some(*hello.stream());
        self.heartbeat = hello.heartbeat();
        Ok(())
    }
    pub(super) fn progress(&mut self, now: Time) {
        self.last_read = now;
    }
}
