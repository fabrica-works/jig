//! A deterministic hold at the kernel boundary, with independent deadlines per side.
use skein_io::kernel::{Complete, Fd, Op};
use skein_lib::{Queue, Time};

pub(crate) struct Relay {
    pub(crate) until: Option<Time>,
    held: Queue<Complete>,
    pub(crate) channel: Option<Fd>,
    pub(crate) wait_only: bool,
}
impl Relay {
    pub(crate) fn new(capacity: u32) -> Self {
        Self { until: None, held: Queue::with_capacity(capacity), channel: None, wait_only: false }
    }
    pub(crate) fn ready(&self, now: Time) -> bool {
        !self.held.is_empty() && self.until.is_none_or(|until| now >= until)
    }
    pub(crate) fn receive(&mut self, now: Time, input: &mut Queue<Complete>) {
        if self.until.is_some_and(|until| now < until) {
            for _ in 0..input.len() {
                let complete = input.pop().expect("completion count");
                if self.wait_only && matches!(&complete.kind, Op::Wait { .. })
                    || !self.wait_only
                        && matches!(&complete.kind, Op::Recv { fd, .. } if self.channel.is_none() || Some(*fd) == self.channel)
                {
                    self.held.push(complete);
                } else {
                    input.push(complete);
                }
            }
        } else {
            self.until = None;
            while let Some(complete) = self.held.pop() {
                input.push(complete);
            }
        }
    }
}

/// An independent channel peer, hosted as a process over the same simulated io.
/// The application protocol's opening failures are supplied as bytes.
#[expect(clippy::struct_excessive_bools, reason = "independent stream demand, opening fault and liveness observations")]
pub(crate) struct Peer {
    io: skein_io::Io,
    env: skein_lib::Env<skein_io::Limits>,
    completions: Queue<Complete>,
    submissions: Queue<skein_io::kernel::Submit>,
    told: Queue<skein_io::Event>,
    asked: Queue<skein_io::Request>,
    scripted: skein_fake_channel::ScriptedPeer,
    socket: Option<skein_lib::Token>,
    started: bool,
    closing: bool,
    waiting: bool,
    bytes: std::collections::VecDeque<Box<[u8]>>,
    input: Vec<u8>,
    pub(crate) refused: Option<u16>,
    pub(crate) ended: bool,
    garbage: bool,
    mode: Option<bool>,
    heartbeat: Option<Time>,
    pub(crate) heartbeats: u32,
    pub(crate) ready: bool,
    pub(crate) start_at: Time,
    pub(crate) wire: crate::processes::Wire,
    refusal: Option<jig_protocol_worker_wire::MessageRefusal>,
    recorded: Option<crate::world::Recorded>,
    frame_reads: bool,
    limits: skein_channel::Limits,
}
impl Peer {
    pub(crate) fn opening(case: crate::stories::Opening) -> Self {
        Self::configured(Some(case), None)
    }
    pub(crate) fn silent(stalled: bool) -> Self {
        Self::configured(None, Some(stalled))
    }
    pub(crate) fn refusing(reason: jig_protocol_worker_wire::MessageRefusal, recorded: crate::world::Recorded) -> Self {
        let mut peer = Self::silent(false);
        peer.refusal = Some(reason);
        peer.recorded = Some(recorded);
        peer.frame_reads = true;
        peer
    }
    fn configured(case: Option<crate::stories::Opening>, mode: Option<bool>) -> Self {
        use crate::stories::Opening;
        use jig_protocol_worker_wire as wire;
        use skein_lib::{Duration, Secret, Writer};
        let (mut schema, limits) = crate::fixtures::peer_bounds();
        let chosen = if matches!(case, Some(Opening::Version)) { 4 } else { wire::VERSION };
        if matches!(case, Some(Opening::Family)) {
            schema.magic = *b"else";
        }
        if matches!(case, Some(Opening::Version)) {
            schema.versions.get_mut(0).expect("one version").version = chosen;
        }
        if matches!(case, Some(Opening::Terms)) {
            let kinds = &mut schema.versions.get_mut(0).expect("one version").kinds;
            for index in 0..kinds.len() {
                let kind = kinds.get_mut(index).expect("kind slot");
                if kind.kind == 0x0180 {
                    kind.largest = 1;
                }
            }
        }
        let credential = wire::Credential::new(
            &wire::CEILINGS,
            wire::CredentialParts {
                name: b"worker-a".as_slice().into(),
                secret: Secret::new(if matches!(case, Some(Opening::Secret)) {
                    b"wrong-secret".as_slice().into()
                } else {
                    b"worker-secret".as_slice().into()
                }),
                payload: if matches!(case, Some(Opening::Application)) { 2 } else { 1 },
            },
        )
        .expect("peer credential");
        let mut writer = Writer::new(credential.measure() as usize);
        credential.encode(&mut writer).expect("credential");
        let mut peer = skein_fake_channel::ScriptedPeer::new(
            schema,
            skein_channel::Role::Initiator,
            limits,
            chosen,
            writer.finish(),
        )
        .expect("independent channel peer");
        if mode.is_some() {
            peer.play(crate::fixtures::peer_hello());
        }
        let io_limits = skein_io::Limits {
            sockets: 2,
            intake: if mode.is_some() { 256 } else { 16_384 },
            output: 262_144,
            close_timeout: Duration::from_millis(50),
            ..smith_agent_process_world::limits().io
        };
        Self {
            io: skein_io::Io::new(&io_limits),
            env: skein_lib::Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: io_limits },
            completions: Queue::with_capacity(128),
            submissions: Queue::with_capacity(128),
            told: Queue::with_capacity(128),
            asked: Queue::with_capacity(128),
            scripted: peer,
            socket: None,
            started: false,
            closing: false,
            waiting: false,
            bytes: std::collections::VecDeque::new(),
            input: Vec::new(),
            refused: None,
            ended: false,
            garbage: matches!(case, Some(Opening::Garbage)),
            mode,
            heartbeat: None,
            heartbeats: 0,
            ready: false,
            start_at: Time::from_nanos(1_000_000),
            wire: crate::processes::Wire::default(),
            refusal: None,
            recorded: None,
            frame_reads: false,
            limits,
        }
    }
    pub(crate) fn shutdown(&mut self) {
        if !self.closing
            && let Some(entity) = self.socket
        {
            self.asked.push(skein_io::Request::Abort { entity });
            self.closing = true;
        }
    }
    fn stream(&mut self, down: skein_lib::stream::Down) {
        self.asked.push(skein_io::Request::Stream { stream: self.socket.expect("connected peer"), down });
    }
    fn demand(&mut self) {
        use skein_lib::stream::{Down, Read};
        if self.closing || self.waiting {
            return;
        }
        if self.bytes.is_empty()
            && let Some(mut bytes) = self.scripted.pop_output()
        {
            if self.garbage {
                bytes[0] = 0xff;
                self.garbage = false;
            }
            self.bytes.push_back(bytes);
        }
        let room = self.bytes.front().map_or(0, |bytes| u32::try_from(bytes.len()).expect("bounded frame"));
        self.waiting = true;
        self.stream(Down::Demand {
            read: if self.ready && self.mode == Some(true) {
                Read::Nothing
            } else if self.frame_reads {
                Read::Fill(self.frame_remaining())
            } else {
                Read::Fill(1)
            },
            room,
        });
    }
    fn frame_remaining(&self) -> u32 {
        let end = if self.input.len() < 8 {
            8
        } else {
            8 + skein_channel::parse_header(&self.input[..8]).expect("service frame").body_len as usize
        };
        u32::try_from(end - self.input.len()).expect("bounded frame").min(self.env.limits.intake)
    }
    fn bytes(&mut self, bytes: &[u8]) {
        self.input.extend_from_slice(bytes);
        while self.input.len() >= 8 {
            let header = skein_channel::parse_header(&self.input[..8]).expect("service frame");
            let end = 8 + header.body_len as usize;
            if self.input.len() < end {
                return;
            }
            if header.kind == 3 {
                let control =
                    skein_channel::decode_control(header.kind, &self.input[8..end], &self.limits).expect("refusal");
                let skein_channel::Control::Refuse { reason, .. } = control else { panic!("refusal kind") };
                self.refused = Some(reason);
            }
            if header.kind == 0x0188 && self.refusal.is_some() {
                use jig_protocol_worker_wire as wire;
                let sources = wire::Sources::decode(&wire::CEILINGS, &mut skein_lib::Reader::new(&self.input[8..end]))
                    .expect("sources fixture");
                assert!(sources.sources().is_empty(), "recorded key-run fixture");
                let logins =
                    wire::Logins::new(&wire::CEILINGS, wire::LoginsParts { logins: skein_lib::List::with_capacity(0) })
                        .expect("empty inventory");
                let mut body = skein_lib::Writer::new(4);
                logins.encode(&mut body).expect("empty inventory bytes");
                self.play(0x0108, &body.finish());
            }
            if header.kind == 0x0181 && self.refusal.is_some() {
                let message = jig_protocol_worker_wire::Inbound::decode(
                    &jig_protocol_worker_wire::CEILINGS,
                    &mut skein_lib::Reader::new(&self.input[8..end]),
                )
                .expect("refused inbound");
                self.report_refusal(message.task(), message.attempt(), message.name());
            }
            self.scripted.feed(&self.input[..end]).expect("independent peer decodes service");
            self.input.drain(..end);
            if self.scripted.is_ready() && self.mode.is_some() && !self.ready {
                self.ready = true;
                if self.mode == Some(true) {
                    self.heartbeat = Some(self.env.now);
                }
            }
        }
    }
    fn report_refusal(&mut self, task: u64, attempt: u64, name: u64) {
        use jig_protocol_worker_wire as wire;
        let recorded = self.recorded.take().expect("one recorded native response");
        let mut turn = recorded.turn.into_parts();
        turn.task = task;
        turn.attempt = attempt;
        turn.refused = skein_lib::List::with_capacity(1);
        turn.refused
            .push(
                wire::Refused::new(
                    &wire::CEILINGS,
                    wire::RefusedParts { name, reason: self.refusal.take().expect("one refusal") },
                )
                .expect("typed refusal"),
            )
            .expect("one refused message");
        let turn = wire::Turn::new(&wire::CEILINGS, turn).expect("recorded refused turn");
        let mut body = skein_lib::Writer::new(turn.measure() as usize);
        turn.encode(&mut body).expect("refusal with native turn");
        self.play(0x0101, &body.finish());
        let mut answer = recorded.answer.into_parts();
        answer.task = task;
        answer.attempt = attempt;
        let answer = wire::Answer::new(&wire::CEILINGS, answer).expect("recorded native result");
        let mut body = skein_lib::Writer::new(answer.measure() as usize);
        answer.encode(&mut body).expect("native result");
        self.play(0x0103, &body.finish());
    }
    fn play(&mut self, kind: u16, body: &[u8]) {
        let mut frame =
            skein_channel::frame_writer(kind, u32::try_from(body.len()).expect("frame size")).expect("frame");
        frame.put(body).expect("frame body");
        self.scripted.play(frame.finish().expect("worker frame"));
    }
    fn event(&mut self, event: skein_io::Event) {
        use skein_io::Event;
        use skein_lib::stream::{Down, Up};
        match event {
            Event::Connecting { socket, .. } => self.socket = Some(socket),
            Event::Connected { .. } => self.demand(),
            Event::Stream { up: Up::Bytes(bytes), .. } => {
                self.waiting = false;
                self.bytes(&bytes);
                self.demand();
            }
            Event::Stream { up: Up::Room, .. } => {
                self.waiting = false;
                let bytes = self.bytes.pop_front().expect("demand requested room");
                self.stream(Down::Send(bytes));
                self.demand();
            }
            Event::Stream { up: Up::End | Up::Failed(_), .. } | Event::Failed { .. } => {
                self.ended = true;
                self.shutdown();
            }
            Event::Closed { .. } => {
                self.socket = None;
                self.closing = true;
            }
            Event::Listening { .. }
            | Event::Accepted { .. }
            | Event::Output { .. }
            | Event::Spawned { .. }
            | Event::Exited { .. }
            | Event::Usage { .. }
            | Event::Window { .. }
            | Event::Resized
            | Event::Shutdown { .. } => panic!("channel peer event {event:?}"),
        }
    }
}
impl skein_shell::Host for Peer {
    fn iterate(&mut self, now: Time, wall: skein_lib::Wall) {
        use skein_io as io;
        self.env.now = now;
        self.env.wall = wall;
        if !self.started && now >= self.start_at {
            self.started = true;
            self.asked.push(io::Request::Connect {
                owner: skein_lib::Token::new(1),
                addr: skein_io::Addr::Ip("127.0.0.1:7200".parse::<std::net::SocketAddr>().expect("loopback")),
            });
        }
        if self.heartbeat.is_some_and(|at| now >= at) && !self.closing {
            self.scripted.play(
                skein_channel::control_frame(&skein_channel::Control::Ping, &self.limits).expect("peer heartbeat"),
            );
            self.heartbeats += 1;
            self.heartbeat = Some(now.saturating_add(skein_lib::Duration::from_secs(1)));
            self.demand();
        }
        while self.io.is_ready() {
            io::resume(&mut self.io, &self.env, &mut self.told, &mut self.submissions);
        }
        while let Some(complete) = self.completions.pop() {
            self.wire.complete(&complete, false, now);
            io::up(&mut self.io, &self.env, complete, &mut self.told, &mut self.submissions);
            while self.io.is_ready() {
                io::resume(&mut self.io, &self.env, &mut self.told, &mut self.submissions);
            }
        }
        while self.io.is_due(now) {
            io::fire(&mut self.io, &self.env, &mut self.told, &mut self.submissions);
        }
        while let Some(event) = self.told.pop() {
            self.event(event);
        }
        while let Some(request) = self.asked.pop() {
            io::down(&mut self.io, &self.env, request, &mut self.submissions);
        }
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<skein_io::kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        (!self.started && now >= self.start_at)
            || !self.asked.is_empty()
            || !self.told.is_empty()
            || !self.completions.is_empty()
            || self.io.is_ready()
            || self.io.is_due(now)
            || self.heartbeat.is_some_and(|at| now >= at)
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.started {
            [self.io.next_deadline(), self.heartbeat.filter(|_| !self.closing)].into_iter().flatten().min()
        } else {
            Some(self.start_at)
        }
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.closing && self.io.is_empty() && self.completions.is_empty() && self.submissions.is_empty()
    }
    fn worst_case(&self) -> u64 {
        4_194_304
    }
    fn operations(&self) -> u32 {
        skein_io::operations(&self.env.limits).expect("peer io")
    }
}
