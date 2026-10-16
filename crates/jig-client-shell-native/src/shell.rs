//! Connections on sockets, with private durable storage (protocol/client.md, 12).
use crate::{Conn, Connections, Limits, files::Files};
use jig_client_protocol_http::{Mapping, Unlinked};
use jig_client_protocol_link::boundary as link;
use skein_io::{
    self as io,
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{
    Env, Queue, Secret, Time, Token, Wall,
    stream::{Down, Fault, Up},
};
use std::collections::{BTreeMap, VecDeque};
pub struct Settings {
    pub engine: io::Addr,
    pub directory: Box<[u8]>,
    pub limits: Limits,
    pub io: io::Limits,
}
struct Socket {
    entity: Option<Token>,
    connected: bool,
    below: VecDeque<Down>,
}
pub struct Shell {
    pub(super) connections: Connections,
    pub(super) files: Files,
    io: io::Io,
    env: Env<io::Limits>,
    engine: io::Addr,
    sockets: BTreeMap<Conn, Socket>,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
    raw: Queue<Submit>,
    told: Queue<io::Event>,
    below: Queue<(Conn, Down)>,
    events: Queue<link::Event>,
    held: VecDeque<link::Request>,
    closing: bool,
    max: u32,
    body_bytes: u32,
}
impl Shell {
    #[must_use]
    pub fn new(settings: Settings) -> Self {
        assert!(
            settings.io.is_usable()
                && settings.io.intake >= settings.limits.http.head.max(settings.limits.http.read)
                && settings.io.output >= settings.limits.http.request.max(settings.limits.http.send),
            "transport fits io"
        );
        let max = settings.limits.exchanges.checked_add(1).expect("connections");
        let room = max.checked_mul(64).expect("queue bound");
        Self {
            connections: Connections::new(&settings.limits),
            files: Files::new(settings.directory, settings.limits.body_bytes),
            io: io::Io::new(&settings.io),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: settings.io },
            engine: settings.engine,
            sockets: BTreeMap::new(),
            complete: Queue::with_capacity(room),
            submit: Queue::with_capacity(room),
            raw: Queue::with_capacity(room),
            told: Queue::with_capacity(room),
            below: Queue::with_capacity(room),
            events: Queue::with_capacity(room),
            held: VecDeque::new(),
            closing: false,
            max,
            body_bytes: settings.limits.body_bytes,
        }
    }
    /// Adopt the caller's directory descriptor and effective user, before startup.
    pub fn adopt(&mut self, root: Fd, user: u32, seed: u64) {
        self.files.adopt(root, user, seed);
    }
    /// Read records once, after the private token and small store have loaded.
    pub fn loaded(&mut self) -> Option<(Option<Secret>, Vec<link::StoreRead>)> {
        let records = self.files.loaded()?;
        Some((self.files.token(), records))
    }
    pub fn events(&mut self) -> &mut Queue<link::Event> {
        &mut self.events
    }
    /// Keep a credential in the private file; `kept` reports its durability.
    pub fn keep(&mut self, credential: Option<Secret>) {
        self.files.credential(credential.map_or_else(|| Box::new([]) as Box<[u8]>, |value| value.expose().into()));
    }
    pub fn kept(&mut self) -> Option<bool> {
        self.files.credential_written.take()
    }
    /// # Errors
    /// Refuses a printed link whose address differs from the configured engine.
    pub fn sign_in(&mut self, printed: &[u8]) -> Result<Secret, Unlinked> {
        let parsed = jig_client_protocol_http::printed(&Mapping::new(b"/jig").expect("default mapping"), printed)?;
        let host = std::str::from_utf8(&parsed.host).map_err(|_| Unlinked::Address)?;
        let addr = host.parse().map_err(|_| Unlinked::Address)?;
        if self.engine != io::Addr::Ip(addr) {
            return Err(Unlinked::Address);
        }
        self.connections.printed(printed)
    }
    /// Route one link output; call iterate after the whole step's outputs.
    pub fn down(&mut self, request: link::Request) {
        match request {
            link::Request::Write { key, record } => self.files.edit(key, Some(record)),
            link::Request::Remove { key } => self.files.edit(key, None),
            request @ (link::Request::Exchange { .. } | link::Request::OpenStream { .. }) => {
                assert!(self.held.len() < usize::try_from(self.max * 64).expect("held bound"), "step output bound");
                self.held.push_back(request);
            }
            request @ link::Request::CloseStream => {
                self.connections.down(self.env.now, request, &mut self.below);
            }
            link::Request::Kept { .. }
            | link::Request::Dropped { .. }
            | link::Request::Answer { .. }
            | link::Request::Signing { .. }
            | link::Request::ServiceCredential { .. }
            | link::Request::Watch { .. }
            | link::Request::FamilyDelivery { .. }
            | link::Request::Ended { .. }
            | link::Request::Hello { .. }
            | link::Request::Read { .. }
            | link::Request::Fact { .. } => {}
        }
    }
    pub fn close(&mut self) {
        if !self.closing {
            self.connections.close_all(self.env.now, &mut self.below);
        }
        self.closing = true;
        self.files.close();
    }
    fn request(&mut self, request: io::Request) {
        io::down(&mut self.io, &self.env, request, &mut self.raw);
    }
    fn socket_event(&mut self, event: io::Event) {
        match event {
            io::Event::Connecting { owner, socket } => {
                self.sockets
                    .get_mut(&Conn(u32::try_from(owner.raw()).expect("connection name")))
                    .expect("connecting socket")
                    .entity = Some(socket);
            }
            io::Event::Connected { owner } => {
                self.sockets
                    .get_mut(&Conn(u32::try_from(owner.raw()).expect("connection name")))
                    .expect("connected socket")
                    .connected = true;
            }
            io::Event::Stream { owner, up } => self.connections.up(
                self.env.now,
                Conn(u32::try_from(owner.raw()).expect("connection name")),
                up,
                &mut self.events,
                &mut self.below,
            ),
            io::Event::Failed { owner, .. } => self.connections.up(
                self.env.now,
                Conn(u32::try_from(owner.raw()).expect("connection name")),
                Up::Failed(Fault::Reset),
                &mut self.events,
                &mut self.below,
            ),
            io::Event::Closed { owner } => {
                self.sockets.remove(&Conn(u32::try_from(owner.raw()).expect("connection name")));
            }
            io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Output { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. } => panic!("client owns sockets only"),
        }
    }
    fn pump(&mut self) {
        while let Some((conn, down)) = self.below.pop() {
            if !self.sockets.contains_key(&conn) {
                if matches!(down, Down::Finish) {
                    continue;
                }
                assert!(self.sockets.len() < usize::try_from(self.max).expect("socket bound"), "bounded sockets");
                self.sockets.insert(conn, Socket { entity: None, connected: false, below: VecDeque::new() });
                self.request(io::Request::Connect { owner: Token::new(u64::from(conn.0)), addr: self.engine });
            }
            self.sockets.get_mut(&conn).expect("socket exists").below.push_back(down);
        }
        let names: Vec<_> = self.sockets.keys().copied().collect();
        for conn in names {
            while self.raw.room() >= io::MAX_OUT_DOWN.submissions && self.io.takes() {
                let socket = self.sockets.get_mut(&conn).expect("socket name");
                if !socket.connected {
                    break;
                }
                let Some(down) = socket.below.pop_front() else { break };
                let stream = socket.entity.expect("connected socket");
                self.request(match down {
                    Down::Finish => io::Request::Close { entity: stream },
                    down @ (Down::Demand { .. } | Down::Send(_)) => io::Request::Stream { stream, down },
                });
            }
        }
    }
    pub fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        while self.io.is_ready()
            && self.told.room() >= io::MAX_OUT_RESUME.events
            && self.raw.room() >= io::MAX_OUT_RESUME.submissions
        {
            io::resume(&mut self.io, &self.env, &mut self.told, &mut self.raw);
        }
        while !self.io.is_ready()
            && self.told.room() >= io::MAX_OUT_UP.events
            && self.raw.room() >= io::MAX_OUT_UP.submissions
        {
            let Some(complete) = self.complete.pop() else { break };
            if Files::owns(complete.op) {
                self.files.complete(complete);
            } else {
                io::up(&mut self.io, &self.env, complete, &mut self.told, &mut self.raw);
            }
        }
        while self.io.is_due(now)
            && self.told.room() >= io::MAX_OUT_FIRE.events
            && self.raw.room() >= io::MAX_OUT_FIRE.submissions
        {
            io::fire(&mut self.io, &self.env, &mut self.told, &mut self.raw);
        }
        while let Some(event) = self.told.pop() {
            self.socket_event(event);
        }
        self.files.progress(now);
        if let Some(ok) = self.files.written.take() {
            self.events.push(if ok { link::Event::Written } else { link::Event::WriteFailed });
            if !ok {
                while let Some(request) = self.held.pop_front() {
                    if let link::Request::Exchange { exchange, .. } = request {
                        self.events.push(link::Event::Lost { exchange });
                    } else {
                        assert!(matches!(request, link::Request::OpenStream { .. }), "only exchanges and streams held");
                        self.events.push(link::Event::StreamGone);
                    }
                }
            }
        }
        if self.files.ready() && !self.closing {
            while let Some(request) = self.held.pop_front() {
                self.connections.down(now, request, &mut self.below);
            }
        }
        self.connections.take(now, &mut self.events, &mut self.below);
        self.pump();
        if self.closing {
            self.held.clear();
            let entities: Vec<_> = self.sockets.values().filter_map(|s| s.entity).collect();
            for entity in entities {
                if self.io.takes() && self.raw.room() >= io::MAX_OUT_DOWN.submissions {
                    self.request(io::Request::Abort { entity });
                }
            }
        }
        self.files.publish(&mut self.submit);
        while self.submit.room() > 0 {
            let Some(submit) = self.raw.pop() else { break };
            self.submit.push(submit);
        }
        self.io.reclaim();
    }
    pub fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    pub fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submit
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.io.is_ready()
            || self.io.is_due(now)
            || !self.complete.is_empty()
            || !self.raw.is_empty()
            || self.files.pending()
            || (!self.closing && self.files.ready() && !self.held.is_empty())
            || self.sockets.values().any(|s| s.connected && !s.below.is_empty())
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        [self.io.next_deadline(), self.files.deadline(), self.connections.next_deadline()].into_iter().flatten().min()
    }
    #[must_use]
    pub fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.closing
            && self.io.is_empty()
            && self.files.is_empty()
            && self.sockets.is_empty()
            && self.complete.is_empty()
            && self.submit.is_empty()
            && self.raw.is_empty()
    }
    #[must_use]
    pub fn operations(&self) -> u32 {
        io::operations(&self.env.limits).expect("io operations") + 2
    }
    #[must_use]
    pub fn worst_case(&self) -> u64 {
        io::worst_case(&self.env.limits).expect("io memory")
            + self.connections.stream_memory()
            + u64::from(self.max)
                * 256
                * (u64::from(self.env.limits.intake)
                    + u64::from(self.env.limits.output)
                    + u64::from(self.body_bytes)
                    + 1024)
            + u64::from(self.max) * u64::from(self.env.limits.intake)
    }
}
