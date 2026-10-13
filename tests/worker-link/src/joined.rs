//! Both real channel components, with scripted roots and seeded stream cuts.
use jig_domain_host as host;
use jig_protocol_engine_workers as engine;
use jig_protocol_worker_link::{Component, Credential, Event, Limits, Request};
use jig_protocol_worker_wire::Part;
use jig_worker_link_world::fixtures;
use skein_channel::{Lower, LowerEvent};
use skein_lib::{Duration, Intake, Queue, Rng, Secret, Time, Token, stream};
use std::collections::{BTreeSet, VecDeque};

// Kept here so the real engine stays a dev-dependency of the world.
mod net {
    use super::{Intake, Lower, LowerEvent, Rng, Token, VecDeque, stream};
    pub struct Pipe {
        ends: [End; 2],
        pub frames: [Vec<u16>; 2],
        closed: [bool; 2],
    }
    struct End {
        intake: Intake,
        bytes: VecDeque<u8>,
        read: stream::Read,
        right: Option<Token>,
        events: VecDeque<LowerEvent>,
        ended: bool,
    }
    impl End {
        fn new(chunk: u32) -> Self {
            Self {
                intake: Intake::with_capacity(chunk),
                bytes: VecDeque::new(),
                read: stream::Read::Nothing,
                right: None,
                events: VecDeque::new(),
                ended: false,
            }
        }
        fn tick(&mut self, rng: &mut Rng) -> Option<LowerEvent> {
            if let Some(event) = self.events.pop_front() {
                return Some(event);
            }
            if let Some(right) = self.right.take() {
                return Some(LowerEvent::Write(stream::OutputUp::Settled {
                    right,
                    outcome: stream::OutputOutcome::Granted,
                }));
            }
            while !self.bytes.is_empty() && self.intake.room() > 0 {
                let count = usize::try_from(rng.between(1, 17))
                    .expect("small cut")
                    .min(self.bytes.len())
                    .min(usize::try_from(self.intake.room()).expect("small intake"));
                let cut: Vec<_> = self.bytes.drain(..count).collect();
                self.intake.append(&cut).expect("checked room");
            }
            if let Some(bytes) = self.intake.meet(self.read) {
                self.read = stream::Read::Nothing;
                return Some(LowerEvent::Read(stream::Up::Bytes(bytes)));
            }
            if self.ended && self.read != stream::Read::Nothing {
                self.read = stream::Read::Nothing;
                self.ended = false;
                return Some(LowerEvent::Read(stream::Up::End));
            }
            None
        }
    }
    impl Pipe {
        pub fn new(chunk: u32) -> Self {
            Self { ends: [End::new(chunk), End::new(chunk)], frames: [Vec::new(), Vec::new()], closed: [false; 2] }
        }
        pub fn lower(&mut self, side: usize, lower: Lower) {
            let end = &mut self.ends[side];
            match lower {
                Lower::Read(stream::Down::Demand { read, room }) => {
                    assert_eq!(room, 0);
                    end.read = read;
                }
                Lower::Read(stream::Down::Finish) => self.ends[1 - side].ended = true,
                Lower::Read(stream::Down::Send(_)) | Lower::FinishWrite => panic!("independent output"),
                Lower::Write(write) => match write {
                    stream::OutputDown::Room { right, .. } => assert!(end.right.replace(right).is_none()),
                    stream::OutputDown::Cancel { right } => {
                        assert_eq!(end.right.take(), Some(right));
                        end.events.push_back(LowerEvent::Write(stream::OutputUp::Settled {
                            right,
                            outcome: stream::OutputOutcome::Cancelled,
                        }));
                    }
                    stream::OutputDown::Release { .. } => {}
                    stream::OutputDown::Send { bytes, .. } => {
                        self.frames[side].push(skein_channel::parse_header(&bytes[..8]).expect("frame").kind);
                        self.ends[1 - side].bytes.extend(bytes);
                    }
                },
            }
        }
        pub fn tick(&mut self, side: usize, rng: &mut Rng) -> Option<LowerEvent> {
            if self.closed[side] { None } else { self.ends[side].tick(rng) }
        }
        pub fn close(&mut self, side: usize) -> bool {
            if self.closed[side] {
                return false;
            }
            self.closed[side] = true;
            self.ends[1 - side].ended = true;
            true
        }
        pub fn gone(&self) -> bool {
            self.closed.iter().all(|closed| *closed)
        }
    }
}

pub struct Joined {
    engine: engine::Component,
    worker: Component,
    pipe: Option<net::Pipe>,
    pub engine_root: Vec<engine::Request>,
    pub worker_root: Vec<Request>,
    pub values: Vec<Request>,
    pub worker_order: Vec<&'static str>,
    pub frames: Vec<Vec<u16>>,
    engine_out: Queue<engine::Request>,
    worker_out: Queue<Request>,
    now: Time,
    rng: Rng,
    connection: Token,
    next: u64,
    channel: Option<Token>,
    chunk: u32,
    retained_turn: bool,
    acknowledge: bool,
    committed: BTreeSet<(Token, Token, u32)>,
    pub dials: Vec<Time>,
}
impl Joined {
    #[must_use]
    pub fn new(limits: &Limits, seed: u64) -> Self {
        Self::with_credential(limits, seed, fixtures::credential())
    }
    #[must_use]
    pub fn with_credential(limits: &Limits, seed: u64, credential: Credential) -> Self {
        let engine_limits = engine::Limits {
            sizes: limits.sizes,
            names: 1,
            workers: 1,
            opening: 1,
            chunk: limits.chunk,
            handshake: limits.handshake,
            hello: limits.handshake,
            ping: limits.ping,
            silence: limits.silence,
            stall: limits.stall,
        };
        let workers = Box::new([engine::Worker {
            name: fixtures::WORKER_A.into(),
            digest: skein_io::digest::digest(fixtures::SECRET_A),
        }]);
        Self {
            engine: engine::Component::new(&engine_limits, workers, fixtures::PAYLOAD).expect("engine limits"),
            worker: Component::new(limits, credential, seed).expect("worker limits"),
            pipe: None,
            engine_root: Vec::new(),
            worker_root: Vec::new(),
            values: Vec::new(),
            worker_order: Vec::new(),
            frames: Vec::new(),
            engine_out: Queue::with_capacity(engine::MAX_OUT_DOWN.max(engine::MAX_OUT_FIRE)),
            worker_out: Queue::with_capacity(jig_protocol_worker_link::MAX_OUT_DOWN),
            now: Time::ZERO,
            rng: Rng::new(seed),
            connection: Token::new(0),
            next: 1,
            channel: None,
            chunk: limits.chunk,
            retained_turn: false,
            acknowledge: true,
            committed: BTreeSet::new(),
            dials: Vec::new(),
        }
    }
    pub fn run(&mut self) {
        for _ in 0..10_000 {
            if let Some(request) = self.worker_out.pop() {
                self.route_worker(request);
                continue;
            }
            if let Some(request) = self.engine_out.pop() {
                self.route_engine(request);
                continue;
            }
            if let Some(pipe) = &mut self.pipe {
                if let Some(event) = pipe.tick(0, &mut self.rng) {
                    self.worker_event(Event::Lower { connection: self.connection, event });
                    continue;
                }
                if let Some(event) = pipe.tick(1, &mut self.rng) {
                    self.engine_event(engine::Event::Lower { connection: self.connection, event });
                    continue;
                }
            }
            if self.worker.next_deadline().is_some_and(|at| at <= self.now) {
                self.worker.fire(self.now, &mut self.worker_out);
                continue;
            }
            if self.engine.next_deadline().is_some_and(|at| at <= self.now) {
                self.engine.fire(self.now, &mut self.engine_out);
                continue;
            }
            self.engine.reclaim();
            return;
        }
        panic!("joined world settles");
    }
    fn worker_event(&mut self, event: Event) {
        self.worker.down(self.now, event, &mut self.worker_out);
    }
    fn engine_event(&mut self, event: engine::Event) {
        self.engine.down(self.now, event, &mut self.engine_out);
    }
    pub fn worker_up(&mut self, up: host::LinkUp, parts: Box<[Part]>) {
        if let host::LinkUp::Turn { run, attempt, turn } = &up {
            assert_eq!((*run, *attempt, turn.turn), (fixtures::RUN_SEVEN, fixtures::ATTEMPT_TWO, 1));
            self.retained_turn = true;
        }
        self.worker_event(Event::Up { up, parts });
    }
    pub fn engine_down(&mut self, down: host::LinkDown, parts: Box<[Part]>, values: Box<[Option<engine::GrantValue>]>) {
        self.engine_event(engine::Event::Down { channel: self.channel.expect("admitted hello"), down, parts, values });
    }
    fn route_worker(&mut self, request: Request) {
        match request {
            Request::Dial => {
                assert!(self.pipe.is_none());
                self.connection = Token::new(self.next);
                self.next += 1;
                self.dials.push(self.now);
                self.pipe = Some(net::Pipe::new(self.chunk));
                self.engine_event(engine::Event::Accepted { connection: self.connection });
                self.worker_event(Event::Dialled { connection: self.connection });
            }
            Request::Lower { connection, lower } => self.lower(0, connection, lower),
            Request::Close { connection } | Request::Abort { connection } => self.close(0, connection),
            Request::Ready => {
                self.worker_event(Event::Hello { hello: fixtures::hello() });
                if self.retained_turn {
                    self.worker_event(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
                }
                self.worker_root.push(request);
            }
            Request::Resend { run, attempt, turn } => {
                assert_eq!((run, attempt, turn), (fixtures::RUN_SEVEN, fixtures::ATTEMPT_TWO, 1));
                if self.retained_turn {
                    self.worker_event(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
                }
                self.worker_root.push(request);
            }
            Request::Down { ref down, .. } => {
                if matches!(down, host::LinkDown::TurnKept { .. } | host::LinkDown::Acknowledge { .. }) {
                    self.retained_turn = false;
                }
                self.worker_order.push("down");
                self.worker_root.push(request);
            }
            Request::Value { .. } => {
                self.worker_order.push("value");
                self.values.push(request);
            }
            Request::RelayCancelled { .. }
            | Request::Lost
            | Request::GracePassed
            | Request::Closed
            | Request::Fact { .. } => {
                self.worker_root.push(request);
            }
        }
    }
    fn route_engine(&mut self, request: engine::Request) {
        match request {
            engine::Request::Take { connection } => assert_eq!(connection, self.connection),
            engine::Request::Lower { connection, lower } => self.lower(1, connection, lower),
            engine::Request::Close { connection }
            | engine::Request::Abort { connection }
            | engine::Request::TurnAway { connection } => self.close(1, connection),
            engine::Request::Hello { channel, .. } => {
                self.channel = Some(channel);
                self.engine_root.push(request);
            }
            engine::Request::Up { channel, ref up, .. } => {
                let down = match up {
                    host::LinkUp::Turn { run, attempt, turn } => {
                        self.committed.insert((*run, *attempt, turn.turn));
                        Some(host::LinkDown::TurnKept { run: *run, attempt: *attempt, turn: turn.turn })
                    }
                    host::LinkUp::Call { run, attempt, call, .. } => Some(host::LinkDown::Relayed {
                        run: *run,
                        attempt: *attempt,
                        call: call.clone(),
                        answer: host::Relayed::Answer { error: false, text: b"answer".as_slice().into() },
                    }),
                    host::LinkUp::Answer { run, attempt, .. } => {
                        Some(host::LinkDown::Acknowledge { run: *run, attempt: *attempt })
                    }
                    host::LinkUp::Activity { .. } | host::LinkUp::Rejected { .. } | host::LinkUp::Exhausted { .. } => {
                        None
                    }
                };
                self.engine_root.push(request);
                if self.acknowledge
                    && let Some(down) = down
                {
                    self.engine_event(engine::Event::Down { channel, down, parts: Box::new([]), values: Box::new([]) });
                }
            }
            engine::Request::Lost { .. } | engine::Request::Closed => self.engine_root.push(request),
        }
    }
    fn lower(&mut self, side: usize, connection: Token, lower: Lower) {
        assert_eq!(connection, self.connection);
        if let Some(pipe) = &mut self.pipe {
            pipe.lower(side, lower);
        }
    }
    fn close(&mut self, side: usize, connection: Token) {
        assert_eq!(connection, self.connection);
        let Some(pipe) = &mut self.pipe else { return };
        let closed = pipe.close(side);
        if pipe.gone() {
            self.archive();
        }
        if closed {
            if side == 0 {
                self.worker_event(Event::Gone { connection });
            } else {
                self.engine_event(engine::Event::Gone { connection });
            }
        }
    }
    fn archive(&mut self) {
        let pipe = self.pipe.take().expect("live pipe");
        self.frames.push(pipe.frames[0].clone());
        self.channel = None;
    }
    pub fn cut(&mut self) {
        self.archive();
        self.worker_event(Event::Gone { connection: self.connection });
        self.engine_event(engine::Event::Gone { connection: self.connection });
        self.run();
    }
    pub fn advance(&mut self, duration: Duration) {
        let end = self.now.saturating_add(duration);
        loop {
            let next = [self.worker.next_deadline(), self.engine.next_deadline()].into_iter().flatten().min();
            let Some(at) = next.filter(|at| *at <= end) else { break };
            self.now = self.now.max(at);
            self.run();
        }
        self.now = end;
    }
    pub fn acknowledge(&mut self, enabled: bool) {
        self.acknowledge = enabled;
    }
    #[must_use]
    pub fn engine_turns(&self) -> Vec<u32> {
        self.committed.iter().map(|(_, _, turn)| *turn).collect()
    }
    #[must_use]
    pub fn engine_channels(&self) -> Vec<Token> {
        self.engine_root
            .iter()
            .filter_map(|request| match request {
                engine::Request::Hello { channel, .. } => Some(*channel),
                engine::Request::Take { .. }
                | engine::Request::TurnAway { .. }
                | engine::Request::Lower { .. }
                | engine::Request::Close { .. }
                | engine::Request::Abort { .. }
                | engine::Request::Up { .. }
                | engine::Request::Lost { .. }
                | engine::Request::Closed => None,
            })
            .collect()
    }
    #[must_use]
    pub fn frames(&self) -> Vec<Vec<u16>> {
        self.frames.iter().cloned().chain(self.pipe.iter().map(|pipe| pipe.frames[0].clone())).collect()
    }
    pub fn finish(&mut self) {
        self.worker_event(Event::Stop);
        self.engine_event(engine::Event::Close);
        self.run();
        assert!(self.pipe.is_none());
        assert_eq!(self.worker.next_deadline(), None);
        assert_eq!(self.engine.next_deadline(), None);
        assert!(self.worker_root.iter().any(|r| matches!(r, Request::Closed)));
        assert!(self.engine_root.iter().any(|r| matches!(r, engine::Request::Closed)));
    }
}
#[must_use]
pub fn grant_value(table: host::GrantTable) -> engine::GrantValue {
    let (token, account) = match table {
        host::GrantTable::Accounts => (fixtures::TOKEN_ONE, fixtures::ACCOUNT_ONE),
        host::GrantTable::Application => (fixtures::TOKEN_TWO, b"".as_slice()),
    };
    engine::GrantValue { value: Secret::new(token.into()), account_id: Secret::new(account.into()) }
}
#[must_use]
pub fn values() -> Box<[Option<engine::GrantValue>]> {
    Box::new([Some(grant_value(host::GrantTable::Accounts)), Some(grant_value(host::GrantTable::Application))])
}
