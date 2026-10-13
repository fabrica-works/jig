//! Worker channel ownership and routing (protocol/channel.md, sections 4 and 10.2).
use crate::{
    Event, Limits, Request, Worker,
    connection::{Connection, Phase},
    opening::Verdict,
};
use alloc::boxed::Box;
use jig_protocol_worker_wire::{self as wire, Reason};
use skein_channel::{Closed, Event as ChannelEvent, Request as ChannelRequest, Role};
use skein_lib::{Decimal, Map, Queue, Reader, Time, Token, bytes};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unusable {
    Limits,
    Workers,
}
#[derive(Debug)]
pub struct Component {
    pub(crate) limits: Limits,
    pub(crate) workers: Box<[Worker]>,
    pub(crate) payload: u32,
    pub(crate) codec: wire::v1::Limits,
    pub(crate) connections: Map<Token, Connection>,
    schema: skein_channel::Schema,
    next_channel: u64,
    next_part: u64,
    stopping: bool,
}
impl Component {
    pub fn new(limits: &Limits, workers: Box<[Worker]>, payload: u32) -> Result<Self, Unusable> {
        crate::worst_case(limits).ok_or(Unusable::Limits)?;
        let codec = wire::sizes::limits(&limits.sizes).ok_or(Unusable::Limits)?;
        let schema = crate::limits::schema(limits).ok_or(Unusable::Limits)?;
        if workers.len() > usize::try_from(limits.names).expect("u32 fits usize") {
            return Err(Unusable::Workers);
        }
        for (index, worker) in workers.iter().enumerate() {
            if worker.name.len() > usize::try_from(limits.sizes.name).expect("u32 fits usize") {
                return Err(Unusable::Workers);
            }
            for earlier in workers.iter().take(index) {
                if earlier.name == worker.name {
                    return Err(Unusable::Workers);
                }
            }
        }
        let connections = Map::with_capacity(limits.workers.checked_add(limits.opening).expect("checked limits"));
        Ok(Self {
            limits: *limits,
            workers,
            payload,
            codec,
            connections,
            schema,
            next_channel: 1,
            next_part: 1,
            stopping: false,
        })
    }
    pub fn down(&mut self, now: Time, event: Event, out: &mut Queue<Request>) {
        match event {
            Event::Accepted { connection } => self.accepted(now, connection, out),
            Event::Lower { connection, event } => {
                if let Some(mut state) = self.connections.remove(&connection) {
                    state.up(event, connection, out);
                    self.progress(now, connection, &mut state, out);
                    self.keep(connection, state);
                }
            }
            Event::Gone { connection } => {
                if let Some(mut state) = self.connections.remove(&connection) {
                    lost(&mut state, out);
                }
            }
            Event::Stop | Event::Close | Event::Abort => self.stopping = true,
            Event::Down { .. } => {}
            Event::NotAdmitted { channel, text } => {
                let mut target = None;
                for (connection, state) in &self.connections {
                    if state.channel == channel && state.heard {
                        target = Some(*connection);
                        break;
                    }
                }
                if let Some(connection) = target {
                    let mut state = self.connections.remove(&connection).expect("heard channel has its connection");
                    self.refuse(now, connection, &mut state, Reason::NotAdmitted.code(), text, out);
                    self.progress(now, connection, &mut state, out);
                    self.keep(connection, state);
                }
            }
        }
    }
    fn accepted(&mut self, now: Time, connection: Token, out: &mut Queue<Request>) {
        if self.connections.len() == self.connections.capacity() || self.connections.contains_key(&connection) {
            out.push(Request::TurnAway { connection });
            return;
        }
        let mut state = Connection::new(self.schema.clone(), &self.limits, now);
        out.push(Request::Take { connection });
        self.progress(now, connection, &mut state, out);
        self.keep(connection, state);
    }
    fn keep(&mut self, connection: Token, state: Connection) {
        let replaced = self.connections.insert(connection, state).expect("an accepted connection has a slot");
        assert!(replaced.is_none(), "one state per connection");
    }
    fn progress(&mut self, now: Time, connection: Token, state: &mut Connection, out: &mut Queue<Request>) {
        // Up produces one owner event; a decision and poll can produce Ready or Closed.
        for _ in 0_u32..3 {
            for _ in 0..skein_channel::MAX_UP {
                let Some(event) = state.above.pop() else { break };
                self.heard(now, connection, state, event, out);
            }
            state.poll(connection, out);
            if state.above.is_empty() {
                break;
            }
        }
        assert!(state.above.is_empty(), "channel decisions settle within three polls");
    }
    fn heard(
        &mut self,
        now: Time,
        connection: Token,
        state: &mut Connection,
        event: ChannelEvent,
        out: &mut Queue<Request>,
    ) {
        match event {
            ChannelEvent::Opening { credential, .. } => {
                let verdict = self.judge(&credential);
                if self.stopping {
                    self.refuse(now, connection, state, Reason::Stopping.code(), Box::new([]), out);
                    return;
                }
                let worker = match verdict {
                    Verdict::Refuse(reason) => {
                        self.refuse(now, connection, state, reason.code(), Box::new([]), out);
                        return;
                    }
                    Verdict::Accept { worker } => worker,
                    Verdict::Replace { worker, older } => {
                        if let Some(mut old) = self.connections.remove(&older) {
                            self.refuse(now, older, &mut old, Reason::Replaced.code(), Box::new([]), out);
                            old.poll(older, out);
                            for _ in 0..skein_channel::MAX_UP {
                                let Some(event) = old.above.pop() else { break };
                                match event {
                                    ChannelEvent::Closed { why } => closed(older, &mut old, why, out),
                                    ChannelEvent::Opening { .. }
                                    | ChannelEvent::Ready { .. }
                                    | ChannelEvent::Body { .. }
                                    | ChannelEvent::Ping
                                    | ChannelEvent::Unsupported { .. }
                                    | ChannelEvent::Refused { .. }
                                    | ChannelEvent::Ended
                                    | ChannelEvent::Sent { .. }
                                    | ChannelEvent::Unsent { .. }
                                    | ChannelEvent::Drained
                                    | ChannelEvent::OutputFailed => {}
                                }
                            }
                            self.keep(older, old);
                        }
                        worker
                    }
                };
                state.worker = Some(worker);
                state.channel = Token::new(self.next_channel);
                self.next_channel =
                    self.next_channel.checked_add(1).expect("channel tokens never wrap in a process life");
                state.phase = Phase::Terms;
                state.hello_deadline = Some(now.saturating_add(self.limits.hello));
                state.down(ChannelRequest::Accept { version: wire::VERSION }, connection, out);
            }
            ChannelEvent::Ready { terms, .. } => {
                if let Some(kind) = wire::terms_gap(&self.schema, Role::Responder, &terms) {
                    let text = bytes::copy_of(Decimal::of(u64::from(kind)).as_bytes());
                    self.refuse(now, connection, state, 2, text, out);
                } else {
                    state.phase = Phase::Hello;
                    state.deadline = state.hello_deadline;
                    state.down(ChannelRequest::Read, connection, out);
                }
            }
            ChannelEvent::Closed { why } => closed(connection, state, why, out),
            ChannelEvent::Body { kind, body } => self.body(now, connection, state, kind, &body, out),
            ChannelEvent::Ping => state.down(ChannelRequest::Read, connection, out),
            ChannelEvent::Ended | ChannelEvent::Refused { .. } => state.down(ChannelRequest::Close, connection, out),
            ChannelEvent::Unsupported { .. }
            | ChannelEvent::Sent { .. }
            | ChannelEvent::Unsent { .. }
            | ChannelEvent::Drained
            | ChannelEvent::OutputFailed => {}
        }
    }
    fn body(
        &mut self,
        now: Time,
        connection: Token,
        state: &mut Connection,
        kind: u16,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        match state.phase {
            Phase::Hello => {
                if kind == 0x0100
                    && let Ok(record) = wire::Hello::decode(&self.codec, &mut Reader::new(body))
                {
                    let hello = crate::translate::hello(record);
                    state.phase = Phase::Open;
                    state.heard = true;
                    state.deadline = None;
                    out.push(Request::Hello { channel: state.channel, hello });
                    state.down(ChannelRequest::Read, connection, out);
                    return;
                }
                self.refuse(now, connection, state, 3, Box::new([]), out);
            }
            Phase::Open => {
                if let Some(record) = crate::translate::decode(kind, body, &self.codec) {
                    let mut parts = crate::translate::Parts::new(self.next_part);
                    let up = crate::translate::up(record, &mut parts);
                    self.next_part = parts.next();
                    out.push(Request::Up { channel: state.channel, up, parts: parts.finish() });
                    state.down(ChannelRequest::Read, connection, out);
                } else {
                    self.refuse(now, connection, state, 3, Box::new([]), out);
                }
            }
            Phase::Opening | Phase::Terms | Phase::Closing => self.refuse(now, connection, state, 3, Box::new([]), out),
        }
    }
    fn refuse(
        &self,
        now: Time,
        connection: Token,
        state: &mut Connection,
        reason: u16,
        text: Box<[u8]>,
        out: &mut Queue<Request>,
    ) {
        lost(state, out);
        state.worker = None;
        state.phase = Phase::Closing;
        state.deadline = Some(now.saturating_add(self.limits.stall));
        state.down(ChannelRequest::Refuse { reason, text }, connection, out);
    }
    pub fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        let mut due = None;
        for (token, state) in &self.connections {
            if let Some(deadline) = state.deadline
                && deadline <= now
            {
                due = Some(*token);
                break;
            }
        }
        if let Some(connection) = due {
            let mut state = self.connections.remove(&connection).expect("due connection exists");
            lost(&mut state, out);
            state.deadline = None;
            state.worker = None;
            state.phase = Phase::Closing;
            state.down(ChannelRequest::Close, connection, out);
            self.progress(now, connection, &mut state, out);
            self.keep(connection, state);
        }
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut next = None;
        for (_, state) in &self.connections {
            if let Some(deadline) = state.deadline {
                next = Some(match next {
                    Some(before) => deadline.min(before),
                    None => deadline,
                });
            }
        }
        next
    }
    pub fn reclaim(&mut self) {}
}

fn closed(connection: Token, state: &mut Connection, why: Closed, out: &mut Queue<Request>) {
    if state.terminal {
        return;
    }
    lost(state, out);
    state.terminal = true;
    state.worker = None;
    state.deadline = None;
    state.phase = Phase::Closing;
    match why {
        Closed::RefusedHere(_) => out.push(Request::Close { connection }),
        Closed::Owner | Closed::RefusedPeer(_) | Closed::Framing | Closed::Stream | Closed::Truncated => {
            out.push(Request::Abort { connection });
        }
    }
}

fn lost(state: &mut Connection, out: &mut Queue<Request>) {
    if state.heard {
        state.heard = false;
        out.push(Request::Lost { channel: state.channel });
    }
}
