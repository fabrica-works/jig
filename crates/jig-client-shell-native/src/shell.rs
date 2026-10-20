//! Connections on sockets, with private durable storage (protocol/client.md, 12).
use crate::{Conn, Connections, Limits, files::Files};
use jig_client as client;
use jig_client_protocol_http::{Mapping, Unlinked};
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
    started: bool,
    closed: bool,
    below: VecDeque<Down>,
}
/// The shared socket loop for native clients (protocol/client.md, 12).
/// `owner` reserves the high 32 bits of socket tokens; zero uses bare connection names.
pub struct Transport {
    pub(super) connections: Connections,
    engine: io::Addr,
    sockets: BTreeMap<Conn, Socket>,
    below: Queue<(Conn, Down)>,
    owner: u64,
    closing: bool,
    max: u32,
    body_bytes: u32,
}
pub struct Shell {
    pub(super) transport: Transport,
    max: u32,
    pub(super) files: Files,
    io: io::Io,
    env: Env<io::Limits>,
    engine: io::Addr,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
    raw: Queue<Submit>,
    told: Queue<io::Event>,
    requests: Queue<io::Request>,
    events: Queue<client::Event>,
    held: VecDeque<client::Request>,
    closing: bool,
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
        let max = settings.io.sockets;
        let room = max.checked_mul(64).expect("queue bound");
        Self {
            transport: Transport::new(settings.engine, &settings.limits, max, 0),
            files: Files::new(settings.directory, settings.limits.body_bytes),
            io: io::Io::new(&settings.io),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: settings.io },
            engine: settings.engine,
            complete: Queue::with_capacity(room),
            submit: Queue::with_capacity(room),
            raw: Queue::with_capacity(room),
            told: Queue::with_capacity(room),
            requests: Queue::with_capacity(room),
            events: Queue::with_capacity(room),
            held: VecDeque::new(),
            max,
            closing: false,
        }
    }
    /// Adopt the caller's directory descriptor and effective user, before startup.
    pub fn adopt(&mut self, root: Fd, user: u32, seed: u64) {
        self.files.adopt(root, user, seed);
    }
    /// Read records once, after the private token and small store have loaded.
    pub fn loaded(&mut self) -> Option<(Option<Secret>, Vec<client::Record>)> {
        let records = self.files.loaded()?;
        Some((self.files.token(), records))
    }
    pub fn events(&mut self) -> &mut Queue<client::Event> {
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
        self.transport.connections.printed(printed)
    }
    /// Route one link output; call iterate after the whole step's outputs.
    pub fn down(&mut self, request: client::Request) {
        match request {
            client::Request::Save { records } => {
                for record in records.into_boxed() {
                    self.files.edit(record.key, Some(record.bytes));
                }
            }
            client::Request::Erase { records } => {
                for key in records.into_boxed() {
                    self.files.edit(key, None);
                }
            }
            request @ (client::Request::Exchange { .. } | client::Request::OpenStream { .. }) => {
                assert!(self.held.len() < usize::try_from(self.max * 64).expect("held bound"), "step output bound");
                self.held.push_back(request);
            }
            request @ client::Request::CloseStream => {
                self.transport.down(self.env.now, request);
            }
            client::Request::Told { .. } | client::Request::ServiceCredential { .. } => {}
            client::Request::Down { .. } | client::Request::SignOut => panic!("linked hosts emit wire work"),
        }
    }
    pub fn close(&mut self) {
        self.transport.close(self.env.now);
        self.closing = true;
        self.files.close();
    }
    fn request(&mut self, request: io::Request) {
        io::down(&mut self.io, &self.env, request, &mut self.raw);
    }
}
impl Transport {
    /// `sockets` is the capacity reserved for this transport on the caller's io.
    #[must_use]
    pub fn new(engine: io::Addr, limits: &Limits, sockets: u32, owner: u64) -> Self {
        assert!(sockets > limits.exchanges, "io fits exchanges and the stream");
        assert_eq!(owner & u64::from(u32::MAX), 0, "owner tag leaves connection bits free");
        Self {
            connections: Connections::new(limits, &engine.socket().expect("HTTP engine has an IP address").to_string()),
            engine,
            sockets: BTreeMap::new(),
            below: Queue::with_capacity(sockets.checked_mul(64).expect("queue bound")),
            owner,
            closing: false,
            max: sockets,
            body_bytes: limits.body_bytes,
        }
    }
    pub fn down(&mut self, now: Time, request: client::Request) {
        if !self.closing {
            self.connections.queue(now, request, &mut self.below);
        }
    }
    #[must_use]
    pub fn owns(&self, owner: Token) -> bool {
        owner.raw() & !u64::from(u32::MAX) == self.owner
    }
    fn conn(&self, owner: Token) -> Conn {
        assert!(self.owns(owner), "transport socket owner");
        Conn(u32::try_from(owner.raw() & u64::from(u32::MAX)).expect("connection name"))
    }
    pub fn close(&mut self, now: Time) {
        if !self.closing {
            self.connections.close_all(now, &mut self.below);
        }
        self.closing = true;
    }
    /// Feed only this transport's socket events, before `iterate`.
    pub fn event(&mut self, now: Time, event: io::Event) {
        match event {
            io::Event::Connecting { owner, socket } => {
                let conn = self.conn(owner);
                self.sockets.get_mut(&conn).expect("connecting socket").entity = Some(socket);
            }
            io::Event::Connected { owner } => {
                let conn = self.conn(owner);
                self.sockets.get_mut(&conn).expect("connected socket").connected = true;
            }
            io::Event::Stream { .. } | io::Event::Failed { .. } if self.closing => {}
            io::Event::Stream { owner, up } => self.connections.receive(now, self.conn(owner), up, &mut self.below),
            io::Event::Failed { owner, .. } => {
                self.connections.receive(now, self.conn(owner), Up::Failed(Fault::Reset), &mut self.below);
            }
            io::Event::Closed { owner } => {
                let conn = self.conn(owner);
                self.sockets.get_mut(&conn).expect("closed socket").closed = true;
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
    /// Call once per host iteration, after io events and before io's reclaim.
    pub fn iterate(&mut self, now: Time, events: &mut Queue<client::Event>, requests: &mut Queue<io::Request>) {
        if self.closing {
            while self.below.pop().is_some() {}
            for socket in self.sockets.values_mut() {
                socket.below.clear();
                if requests.room() > 0
                    && let Some(entity) = socket.entity.take()
                {
                    requests.push(io::Request::Abort { entity });
                }
            }
        } else {
            // Closed sockets still occupy io's slab until the host reclaims it.
            let free = self.max - u32::try_from(self.sockets.len()).expect("socket bound");
            self.connections.take_with_room(now, free, events, &mut self.below);
            self.pump(requests);
        }
        self.sockets.retain(|_, socket| !socket.closed && (!self.closing || socket.started));
    }
    fn pump(&mut self, requests: &mut Queue<io::Request>) {
        while let Some((conn, down)) = self.below.pop() {
            if !self.sockets.contains_key(&conn) {
                if matches!(down, Down::Finish) {
                    continue;
                }
                assert!(self.sockets.len() < usize::try_from(self.max).expect("socket bound"), "bounded sockets");
                self.sockets.insert(
                    conn,
                    Socket { entity: None, connected: false, started: false, closed: false, below: VecDeque::new() },
                );
            }
            self.sockets.get_mut(&conn).expect("socket exists").below.push_back(down);
        }
        let names: Vec<_> = self.sockets.keys().copied().collect();
        for conn in names {
            let socket = self.sockets.get_mut(&conn).expect("socket name");
            if !socket.started && requests.room() > 0 {
                socket.started = true;
                requests.push(io::Request::Connect {
                    owner: Token::new(self.owner | u64::from(conn.0)),
                    addr: self.engine,
                });
            }

            while requests.room() > 0 {
                let socket = self.sockets.get_mut(&conn).expect("socket name");
                if !socket.connected {
                    break;
                }
                let Some(down) = socket.below.pop_front() else { break };
                let stream = socket.entity.expect("connected socket");
                requests.push(match down {
                    Down::Finish => io::Request::Close { entity: stream },
                    down @ (Down::Demand { .. } | Down::Send(_)) => io::Request::Stream { stream, down },
                });
            }
        }
    }
}
impl Shell {
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
            self.transport.event(now, event);
        }
        self.files.progress(now);
        if let Some(ok) = self.files.written.take() {
            self.events.push(if ok { client::Event::Stored } else { client::Event::NotStored });
            if !ok {
                while let Some(request) = self.held.pop_front() {
                    if let client::Request::Exchange { exchange, .. } = request {
                        self.events.push(client::Event::Lost { exchange });
                    } else {
                        assert!(
                            matches!(request, client::Request::OpenStream { .. }),
                            "only exchanges and streams held"
                        );
                        self.events.push(client::Event::StreamGone);
                    }
                }
            }
        }
        if self.files.ready() && !self.closing {
            while let Some(request) = self.held.pop_front() {
                self.transport.down(now, request);
            }
        }
        self.transport.iterate(now, &mut self.events, &mut self.requests);
        if self.closing {
            self.held.clear();
        }
        while self.io.takes() && self.raw.room() >= io::MAX_OUT_DOWN.submissions {
            let Some(request) = self.requests.pop() else { break };
            self.request(request);
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
            || !self.requests.is_empty()
            || self.transport.work_pending(now)
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        [self.io.next_deadline(), self.files.deadline(), self.transport.next_deadline()].into_iter().flatten().min()
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
            && self.transport.is_empty()
            && self.requests.is_empty()
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
            + self.transport.worst_case(&self.env.limits)
            + Queue::<io::Request>::worst_case(self.requests.capacity()).expect("io request queue")
    }
}
impl Transport {
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        if self.closing {
            self.sockets.values().any(|s| s.entity.is_some())
        } else {
            (self.sockets.len() < usize::try_from(self.max).expect("socket bound") && self.connections.waiting())
                || !self.below.is_empty()
                || self.sockets.values().any(|s| !s.started || s.connected && !s.below.is_empty())
                || self.next_deadline().is_some_and(|until| now >= until)
        }
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.connections.next_deadline()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sockets.is_empty() && self.below.is_empty() && !self.connections.waiting()
    }
    #[must_use]
    pub fn worst_case(&self, io: &io::Limits) -> u64 {
        self.connections.stream_memory()
            + u64::from(self.max)
                * 256
                * (u64::from(io.intake) + u64::from(io.output) + u64::from(self.body_bytes) + 1024)
            + u64::from(self.max) * u64::from(io.intake)
    }
}
