//! Opening, loss and redial transitions (protocol/channel.md, 4 and 10.1).
use crate::{
    Event, Limits, Request,
    backoff::Backoff,
    connection::{Connection, Phase},
    limits, translate,
};
use alloc::boxed::Box;
use core::mem;
use jig_domain_host as host;
use jig_protocol_worker_wire::{self as wire, sizes};
use skein_channel::{self as channel, Closed, Machine, Role};
use skein_lib::{Decimal, List, Queue, Secret, Time, Token, Writer, bytes};

#[derive(Debug)]
pub struct Credential {
    pub name: Box<[u8]>,
    pub secret: Secret,
    pub payload: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unusable {
    Limits,
    Credential,
}
#[derive(Debug)]
pub struct Component {
    limits: Limits,
    credential: wire::Credential,
    phase: Phase,
    backoff: Backoff,
    events: Queue<channel::Event>,
    below: Queue<channel::Lower>,
    next_part: u64,
    dropped: u64,
}
impl Component {
    pub fn new(limits: &Limits, credential: Credential, seed: u64) -> Result<Self, Unusable> {
        limits::worst_case(limits).ok_or(Unusable::Limits)?;
        let bodies = sizes::limits(&limits.sizes).ok_or(Unusable::Limits)?;
        let credential = wire::Credential::new(
            &bodies,
            wire::CredentialParts { name: credential.name, secret: credential.secret, payload: credential.payload },
        );
        let Ok(credential) = credential else { return Err(Unusable::Credential) };
        let backoff = Backoff::new(limits, seed);
        Ok(Self {
            limits: *limits,
            credential,
            phase: Phase::Idle { at: Time::ZERO },
            backoff,
            events: Queue::with_capacity(channel::MAX_UP.saturating_mul(3)),
            below: Queue::with_capacity(channel::MAX_DOWN.saturating_mul(3)),
            next_part: 1,
            dropped: 0,
        })
    }
    pub fn down(&mut self, now: Time, event: Event, out: &mut Queue<Request>) {
        match event {
            Event::Dialled { connection } => self.dialled(now, connection, out),
            Event::Unreachable => match self.phase {
                Phase::Connecting => {
                    let span = self.backoff.next();
                    self.phase = Phase::Idle { at: now.saturating_add(span) };
                    out.push(Request::Fact { why: Closed::Stream });
                }
                Phase::Idle { .. } | Phase::Opening { .. } | Phase::Open { .. } | Phase::Closing { .. } => {}
            },
            Event::Lower { connection, event } => {
                if self.phase.token() == Some(connection) {
                    if let Some(link) = self.phase.connection() {
                        link.machine.up(event, &mut self.events, &mut self.below);
                    }
                    self.pump(now, out);
                }
            }
            Event::Gone { connection } => self.gone(now, connection, out),
            Event::Hello { hello } => {
                self.hello(hello);
                self.pump(now, out);
            }
            Event::Up { up, parts } => {
                self.up(up, &parts);
                self.pump(now, out);
            }
        }
    }
    pub fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        match self.phase {
            Phase::Idle { at } => {
                if now >= at {
                    self.phase = Phase::Connecting;
                    out.push(Request::Dial);
                }
            }
            Phase::Opening { until, .. } => {
                if now >= until {
                    self.lose(now, Closed::Owner, out);
                    self.pump(now, out);
                }
            }
            Phase::Closing { until, .. } => {
                if let Some(until) = until
                    && now >= until
                {
                    if let Some(connection) = self.phase.token() {
                        out.push(Request::Abort { connection });
                    }
                    match &mut self.phase {
                        Phase::Closing { until, .. } => *until = None,
                        Phase::Idle { .. } | Phase::Connecting | Phase::Opening { .. } | Phase::Open { .. } => {}
                    }
                }
            }
            Phase::Open { proved, .. } => {
                if let Some(at) = proved
                    && now >= at
                {
                    self.backoff.proved();
                    match &mut self.phase {
                        Phase::Open { proved, .. } => *proved = None,
                        Phase::Idle { .. } | Phase::Connecting | Phase::Opening { .. } | Phase::Closing { .. } => {}
                    }
                }
            }
            Phase::Connecting => {}
        }
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        match self.phase {
            Phase::Idle { at } => Some(at),
            Phase::Opening { until, .. } => Some(until),
            Phase::Closing { until, .. } => until,
            Phase::Open { proved, .. } => proved,
            Phase::Connecting => None,
        }
    }
    #[must_use]
    pub fn dropped_facts(&self) -> u64 {
        0
    }

    /// Records dropped because their bounds or named parts do not fit.
    #[must_use]
    pub fn dropped_records(&self) -> u64 {
        self.dropped
    }

    fn dialled(&mut self, now: Time, token: Token, out: &mut Queue<Request>) {
        match self.phase {
            Phase::Connecting => {}
            Phase::Idle { .. } | Phase::Opening { .. } | Phase::Open { .. } | Phase::Closing { .. } => {
                out.push(Request::Abort { connection: token });
                return;
            }
        }
        let bodies = sizes::limits(&self.limits.sizes).expect("constructor checked sizes");
        let schema = wire::schema(&bodies).expect("constructor checked schema");
        let channel = limits::channel_limits(&self.limits).expect("constructor checked channel limits");
        let machine = Machine::new(schema, Role::Initiator, channel).expect("constructor checked channel");
        self.phase = Phase::Opening {
            connection: Connection { token, machine },
            until: now.saturating_add(self.limits.handshake),
        };
        let mut writer = Writer::new(usize::try_from(self.credential.measure()).expect("checked credential fits"));
        self.credential.encode(&mut writer).expect("measured credential fits");
        self.command(channel::Request::Open { credential: writer.finish() });
        self.pump(now, out);
    }
    fn gone(&mut self, now: Time, token: Token, out: &mut Queue<Request>) {
        if self.phase.token() != Some(token) {
            return;
        }
        let phase = mem::replace(&mut self.phase, Phase::Connecting);
        let at = match phase {
            Phase::Closing { retry, .. } => retry,
            Phase::Open { .. } => {
                out.push(Request::Lost);
                out.push(Request::Fact { why: Closed::Stream });
                let span = self.backoff.next();
                now.saturating_add(span)
            }
            Phase::Opening { .. } => {
                out.push(Request::Fact { why: Closed::Stream });
                let span = self.backoff.next();
                now.saturating_add(span)
            }
            Phase::Idle { at } => at,
            Phase::Connecting => now,
        };
        self.phase = Phase::Idle { at };
    }
    fn hello(&mut self, value: host::Hello) {
        match &mut self.phase {
            Phase::Open { hello, .. } => {
                if *hello {
                    return;
                }
                let limits = sizes::limits(&self.limits.sizes).expect("checked sizes");
                let Some(frame) = translate::hello(value, &limits) else {
                    self.dropped = self.dropped.saturating_add(1);
                    return;
                };
                *hello = true;
                self.command(channel::Request::Send { token: Token::new(0), frame });
            }
            Phase::Idle { .. } | Phase::Connecting | Phase::Opening { .. } | Phase::Closing { .. } => {}
        }
    }
    fn up(&mut self, up: host::LinkUp, parts: &[wire::Part]) {
        match self.phase {
            Phase::Open { hello: true, .. } => {
                let limits = sizes::limits(&self.limits.sizes).expect("checked sizes");
                let Some(frame) = translate::up(up, parts, &limits) else {
                    self.dropped = self.dropped.saturating_add(1);
                    return;
                };
                self.command(channel::Request::Send { token: Token::new(1), frame });
            }
            Phase::Idle { .. }
            | Phase::Connecting
            | Phase::Opening { .. }
            | Phase::Open { hello: false, .. }
            | Phase::Closing { .. } => {}
        }
    }
    fn command(&mut self, request: channel::Request) {
        if let Some(link) = self.phase.connection() {
            link.machine.down(request, &mut self.events, &mut self.below);
        }
    }
    fn pump(&mut self, now: Time, out: &mut Queue<Request>) {
        if let Some(link) = self.phase.connection() {
            link.machine.poll(&mut self.events, &mut self.below);
        }
        // Each channel operation emits at most MAX_UP, and handlers perform at most two.
        for _ in 0..channel::MAX_UP.saturating_mul(3) {
            self.drain_lower(now, out);
            let Some(event) = self.events.pop() else { break };
            self.received(now, event, out);
            if let Some(link) = self.phase.connection() {
                link.machine.poll(&mut self.events, &mut self.below);
            }
        }
        self.drain_lower(now, out);
        assert!(self.events.is_empty(), "one channel event cascade fits its bound");
    }
    fn drain_lower(&mut self, now: Time, out: &mut Queue<Request>) {
        for _ in 0..channel::MAX_DOWN.saturating_mul(3) {
            let Some(lower) = self.below.pop() else { break };
            match &lower {
                channel::Lower::Write(skein_lib::stream::OutputDown::Send { bytes, .. }) => {
                    let header = channel::parse_header(bytes.get(..8).expect("a whole output frame"))
                        .expect("checked frame header");
                    if header.kind == 0x0100 {
                        match &mut self.phase {
                            Phase::Open { proved, .. } => *proved = Some(now.saturating_add(self.limits.proved)),
                            Phase::Idle { .. } | Phase::Connecting | Phase::Opening { .. } | Phase::Closing { .. } => {}
                        }
                    }
                }
                channel::Lower::Read(_)
                | channel::Lower::Write(
                    skein_lib::stream::OutputDown::Room { .. }
                    | skein_lib::stream::OutputDown::Cancel { .. }
                    | skein_lib::stream::OutputDown::Release { .. },
                )
                | channel::Lower::FinishWrite => {}
            }
            if let Some(connection) = self.phase.token() {
                out.push(Request::Lower { connection, lower });
            }
        }
    }
    fn received(&mut self, now: Time, event: channel::Event, out: &mut Queue<Request>) {
        match event {
            channel::Event::Ready { terms, .. } => {
                let bodies = sizes::limits(&self.limits.sizes).expect("checked sizes");
                let schema = wire::schema(&bodies).expect("checked schema");
                if let Some(kind) = wire::terms_gap(&schema, Role::Initiator, &terms) {
                    self.command(channel::Request::Refuse {
                        reason: 2,
                        text: bytes::copy_of(Decimal::of(u64::from(kind)).as_bytes()),
                    });
                    let started = self.begin_loss(now, Closed::RefusedHere(2), out);
                    assert!(started, "terms are checked during opening");
                } else {
                    let phase = mem::replace(&mut self.phase, Phase::Connecting);
                    self.phase = match phase {
                        Phase::Opening { connection, .. } => Phase::Open { connection, proved: None, hello: false },
                        other @ (Phase::Idle { .. }
                        | Phase::Connecting
                        | Phase::Open { .. }
                        | Phase::Closing { .. }) => other,
                    };
                    out.push(Request::Ready);
                    self.command(channel::Request::Read);
                }
            }
            channel::Event::Refused { reason, .. } => self.lose(now, Closed::RefusedPeer(reason), out),
            channel::Event::Closed { why } => {
                let started = self.begin_loss(now, why, out);
                if started {
                    self.command(channel::Request::Close);
                }
                if let Some(connection) = self.phase.token() {
                    out.push(Request::Close { connection });
                }
            }
            channel::Event::Ended | channel::Event::OutputFailed => self.lose(now, Closed::Stream, out),
            channel::Event::Body { kind, body } => {
                let limits = sizes::limits(&self.limits.sizes).expect("checked sizes");
                let mut parts = List::with_capacity(1);
                match translate::down(kind, &body, &limits, now, &mut parts, &mut self.next_part, out) {
                    Some(down) => {
                        out.push(Request::Down { down, parts: parts.into_boxed() });
                        self.command(channel::Request::Read);
                    }
                    None => {
                        self.command(channel::Request::Refuse { reason: 3, text: Box::new([]) });
                        let started = self.begin_loss(now, Closed::RefusedHere(3), out);
                        assert!(started, "a body arrives on an open channel");
                    }
                }
            }
            channel::Event::Ping | channel::Event::Unsupported { .. } => {
                self.command(channel::Request::Read);
            }
            channel::Event::Unsent { .. } => self.lose(now, Closed::Owner, out),
            channel::Event::Opening { .. } => self.lose(now, Closed::Framing, out),
            channel::Event::Drained | channel::Event::Sent { .. } => {}
        }
    }
    fn lose(&mut self, now: Time, why: Closed, out: &mut Queue<Request>) {
        if self.begin_loss(now, why, out) {
            self.command(channel::Request::Close);
        }
    }
    fn begin_loss(&mut self, now: Time, why: Closed, out: &mut Queue<Request>) -> bool {
        let phase = mem::replace(&mut self.phase, Phase::Connecting);
        let mut started = false;
        self.phase = match phase {
            Phase::Open { connection, .. } => {
                out.push(Request::Lost);
                let span = self.backoff.next();
                out.push(Request::Fact { why });
                started = true;
                Phase::Closing {
                    connection,
                    retry: now.saturating_add(span),
                    until: Some(now.saturating_add(self.limits.handshake)),
                }
            }
            Phase::Opening { connection, .. } => {
                let span = self.backoff.next();
                out.push(Request::Fact { why });
                started = true;
                Phase::Closing {
                    connection,
                    retry: now.saturating_add(span),
                    until: Some(now.saturating_add(self.limits.handshake)),
                }
            }
            other @ (Phase::Idle { .. } | Phase::Connecting | Phase::Closing { .. }) => other,
        };
        started
    }
}
