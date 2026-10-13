//! One accepted stream and its opening (protocol/channel.md, 10.3).
use crate::Limits;
use skein_channel::{Event, Lower, LowerEvent, Machine, Request, Role, Schema, StreamMode};
use skein_lib::{Queue, Time, Token};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Phase {
    Opening,
    Terms,
    Hello,
    Open,
    Closing,
}
#[derive(Debug)]
pub(crate) struct Connection {
    pub machine: Machine,
    pub phase: Phase,
    pub worker: Option<u32>,
    pub channel: Token,
    pub deadline: Option<Time>,
    pub above: Queue<Event>,
    pub below: Queue<Lower>,
    pub terminal: bool,
    pub heard: bool,
    pub hello_deadline: Option<Time>,
}
impl Connection {
    pub fn new(schema: Schema, limits: &Limits, now: Time) -> Self {
        let channel = crate::limits::channel(limits).expect("checked limits");
        let machine = Machine::with_mode(schema, Role::Responder, channel, StreamMode::One).expect("checked schema");
        Self {
            machine,
            phase: Phase::Opening,
            worker: None,
            channel: Token::new(0),
            deadline: Some(now.saturating_add(limits.handshake)),
            above: Queue::with_capacity(skein_channel::MAX_UP),
            below: Queue::with_capacity(skein_channel::MAX_DOWN),
            terminal: false,
            heard: false,
            hello_deadline: None,
        }
    }
    pub fn down(&mut self, request: Request, connection: Token, out: &mut Queue<crate::Request>) {
        self.machine.down(request, &mut self.above, &mut self.below);
        self.flush(connection, out);
    }
    pub fn up(&mut self, event: LowerEvent, connection: Token, out: &mut Queue<crate::Request>) {
        self.machine.up(event, &mut self.above, &mut self.below);
        self.flush(connection, out);
    }
    pub fn poll(&mut self, connection: Token, out: &mut Queue<crate::Request>) {
        self.machine.poll(&mut self.above, &mut self.below);
        self.flush(connection, out);
    }
    fn flush(&mut self, connection: Token, out: &mut Queue<crate::Request>) {
        for _ in 0..skein_channel::MAX_DOWN {
            let Some(lower) = self.below.pop() else { break };
            out.push(crate::Request::Lower { connection, lower });
        }
    }
}
