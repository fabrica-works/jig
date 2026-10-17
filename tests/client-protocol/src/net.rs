//! Seeded byte streams, bounded by real demands; cuts fail both ends.
use crate::referee::Rules;
use jig_client_shell_native::Conn;
use jig_client_world::ClientId;
use skein_io::{Event, Request};
use skein_lib::{
    Duration, Rng, Time, Token,
    stream::{Down, Fault, Up},
};
use skein_world::stream::Wire;
use std::collections::BTreeMap;
mod records;
use records::Records;
#[derive(Default)]
struct Checked {
    request: bool,
    response: bool,
    events: usize,
}
#[derive(Default)]
struct Room {
    shell: bool,
    engine: bool,
}
struct Socket {
    client: ClientId,
    conn: Conn,
    owner: Option<Token>,
    shell: Wire,
    engine: Wire,
    sent_shell: usize,
    sent_engine: usize,
    request: Vec<u8>,
    response: Vec<u8>,
    checked: Checked,
    source_records: Records,
    crossed_records: Records,
    broken: bool,
    closed: bool,
    fail: bool,
    upload_limit: Option<usize>,
    room: Room,
}
pub struct Net {
    sockets: BTreeMap<Token, Socket>,
    names: BTreeMap<(ClientId, Conn), Token>,
    next: u64,
    listener: Token,
    rng: Rng,
    now: Time,
    slow: Option<(ClientId, Time)>,
    cut_answer: Option<usize>,
    cut_upload: Option<usize>,
    pub cuts: u64,
    offline: Option<Time>,
}
impl Net {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            sockets: BTreeMap::new(),
            names: BTreeMap::new(),
            next: 100,
            listener: Token::new(0),
            rng: Rng::new(seed),
            now: Time::ZERO,
            slow: None,
            cut_answer: None,
            cut_upload: None,
            cuts: 0,
            offline: None,
        }
    }
    /// Whether another seeded wire attempt can transfer bytes, room or EOF.
    /// Idle read demands consume no RNG draws, so skipping those attempts
    /// retains the exact fragment and room schedule of subsequent traffic.
    #[must_use]
    pub fn pending(&self) -> bool {
        self.offline.is_none()
            && self.sockets.values().any(|socket| {
                !socket.broken
                    && (socket.shell.received.len() != socket.request.len()
                        || socket.engine.received.len() != socket.response.len()
                        || socket.room.shell
                        || (!socket.closed && socket.room.engine)
                        || (socket.shell.eof && !socket.shell.ended)
                        || (!socket.closed && socket.engine.eof && !socket.engine.ended))
            })
    }
    pub fn clock(&mut self, now: Time) {
        self.now = now;
        if self.offline.is_some_and(|until| until <= now) {
            self.offline = None;
        }
        if self.slow.is_some_and(|(_, until)| until <= now) {
            self.slow = None;
        }
    }
    pub fn offline(&mut self, until: Time) {
        self.offline = Some(until);
    }
    #[must_use]
    pub fn unread(&self, client: ClientId) -> bool {
        self.slow.is_some_and(|(id, until)| id == client && until > self.now)
            && self
                .sockets
                .values()
                .any(|socket| socket.client == client && !socket.broken && !socket.closed && stream(socket))
    }
    #[must_use]
    pub fn next_due(&self) -> Option<Time> {
        self.slow.map(|(_, at)| at).into_iter().chain(self.offline).min()
    }
    pub fn slow(&mut self, client: ClientId, span: Duration) {
        self.slow = Some((client, self.now.saturating_add(span)));
    }
    pub fn cut_answer(&mut self) {
        self.cut_answer = Some(usize::try_from(self.rng.between(1, 8)).expect("short cut"));
    }
    pub fn cut_upload(&mut self) {
        self.cut_upload = Some(usize::try_from(self.rng.between(1, 8)).expect("short cut"));
    }
    pub fn cut_client(&mut self, client: ClientId) {
        for socket in self.sockets.values_mut().filter(|s| s.client == client && !s.broken) {
            socket.broken = true;
            socket.fail = true;
        }
    }
    pub fn cut_stream(&mut self, client: ClientId) {
        for socket in self.sockets.values_mut().filter(|s| s.client == client && stream(s) && !s.broken) {
            socket.broken = true;
            socket.fail = true;
        }
    }
    pub fn cut_all(&mut self) {
        for socket in self.sockets.values_mut().filter(|s| !s.broken) {
            socket.broken = true;
            socket.fail = true;
        }
    }
    pub fn retire(&mut self, client: ClientId) {
        self.names.retain(|(id, _), _| *id != client);
    }
    pub fn down(&mut self, client: ClientId, conn: Conn, down: Down) -> Option<Event> {
        let event = if self.names.contains_key(&(client, conn)) {
            None
        } else {
            let socket = Token::new(self.next);
            self.next += 1;
            self.names.insert((client, conn), socket);
            self.sockets.insert(
                socket,
                Socket {
                    client,
                    conn,
                    owner: None,
                    shell: Wire::new(self.rng.next_u64()),
                    engine: Wire::new(self.rng.next_u64()),
                    sent_shell: 0,
                    sent_engine: 0,
                    request: Vec::new(),
                    response: Vec::new(),
                    checked: Checked::default(),
                    source_records: Records::default(),
                    crossed_records: Records::default(),
                    broken: false,
                    closed: false,
                    fail: false,
                    upload_limit: None,
                    room: Room::default(),
                },
            );
            Some(Event::Accepted { owner: self.listener, socket, peer: crate::fixtures::address() })
        };
        let socket = self.sockets.get_mut(&self.names[&(client, conn)]).expect("named socket");
        if !socket.broken {
            socket.room.shell = room(&down, socket.room.shell);
            socket.shell.take(down);
        }
        event
    }
    pub fn io(&mut self, request: Request) -> Option<Event> {
        match request {
            Request::Listen { owner, addr } => {
                self.listener = owner;
                Some(Event::Listening { owner, listener: Token::new(99), addr })
            }
            Request::Bind { socket, owner } => {
                self.sockets.get_mut(&socket).expect("accepted socket").owner = Some(owner);
                None
            }
            Request::Stream { stream, down } => {
                let socket = self.sockets.get_mut(&stream).expect("bound socket");
                if !socket.broken {
                    socket.room.engine = room(&down, socket.room.engine);
                    socket.engine.take(down);
                }
                None
            }
            Request::Close { entity } | Request::Abort { entity } => {
                let socket = self.sockets.get_mut(&entity).expect("owned socket");
                socket.closed = true;
                socket.shell.eof = true;
                Some(Event::Closed { owner: socket.owner.expect("bound socket") })
            }
            Request::Reject { socket } => {
                self.sockets.get_mut(&socket).expect("accepted socket").broken = true;
                None
            }
            Request::Connect { .. }
            | Request::Spawn { .. }
            | Request::Signal { .. }
            | Request::Usage { .. }
            | Request::Window { .. }
            | Request::Output { .. } => panic!("unrelated io"),
        }
    }
    pub fn advance(&mut self, rules: &mut Rules) -> (Vec<Event>, Vec<(ClientId, Conn, Up)>, bool) {
        let mut events = Vec::new();
        let mut clients = Vec::new();
        let mut changed = false;
        for socket in self.sockets.values_mut() {
            let keyed = !socket.checked.response && socket.shell.received.starts_with(b"POST /jig/v1/request ");
            if keyed
                && socket.upload_limit.is_none()
                && let Some(limit) = self.cut_upload
                && let Some(bytes) = body(&socket.shell.received)
                && bytes.len() > limit
            {
                socket.upload_limit = Some(socket.shell.received.len() - bytes.len() + limit);
                self.cut_upload = None;
            }
            let upload_cut =
                socket.upload_limit.is_some_and(|limit| socket.sent_shell == limit && body(&socket.request).is_some());
            let answer_cut = keyed && self.cut_answer.is_some_and(|n| socket.engine.received.len() >= n);
            if !socket.broken && (upload_cut || answer_cut) {
                socket.broken = true;
                socket.fail = true;
                if upload_cut {
                    self.cut_upload = None;
                } else {
                    self.cut_answer = None;
                }
            }
            if socket.fail {
                socket.fail = false;
                self.cuts += 1;
                rules.cuts += 1;
                clients.push((socket.client, socket.conn, Up::Failed(Fault::Reset)));
                if let Some(owner) = socket.owner {
                    events.push(Event::Stream { owner, up: Up::Failed(Fault::Reset) });
                }
                changed = true;
            }
            if socket.broken || self.offline.is_some() {
                continue;
            }
            let through = socket.upload_limit.unwrap_or(socket.shell.received.len()).min(socket.shell.received.len());
            socket.engine.write(&socket.shell.received[socket.sent_shell..through]);
            socket.sent_shell = through;
            socket.shell.write(&socket.engine.received[socket.sent_engine..]);
            socket.sent_engine = socket.engine.received.len();
            socket.engine.eof = socket.shell.finished;
            let mut record_changed = false;
            let slow = self.slow.is_some_and(|(client, _)| client == socket.client && stream(socket));
            let shell_progress = socket.shell.trace.len();
            let shell_up = if slow { None } else { socket.shell.answer() };
            // A seeded Wire may buffer a fragment before it can answer a
            // complete line. That is progress even without a callback.
            changed |= socket.shell.trace.len() != shell_progress;
            if let Some(up) = shell_up {
                socket.room.shell = false;
                if let Up::Bytes(bytes) = &up {
                    socket.response.extend_from_slice(bytes);
                    record_changed = true;
                }
                if matches!(&up, Up::End) {
                    socket.broken = true;
                }
                clients.push((socket.client, socket.conn, up));
                changed = true;
            }
            let engine_progress = socket.engine.trace.len();
            if !slow
                && !socket.closed
                && let Some(owner) = socket.owner
                && let Some(up) = socket.engine.answer()
            {
                if let Up::Bytes(bytes) = &up {
                    socket.request.extend_from_slice(bytes);
                    record_changed = true;
                }
                socket.room.engine = false;
                events.push(Event::Stream { owner, up });
                changed = true;
            }
            changed |= socket.engine.trace.len() != engine_progress;
            if record_changed {
                check(socket, rules);
            }
        }
        (events, clients, changed)
    }
}
fn stream(socket: &Socket) -> bool {
    socket.shell.received.starts_with(b"GET /jig/v1/stream ")
}
fn body(http: &[u8]) -> Option<&[u8]> {
    let end = http.windows(4).position(|bytes| bytes == b"\r\n\r\n")?;
    Some(&http[end + 4..])
}
fn whole(http: &[u8]) -> Option<&[u8]> {
    let bytes = body(http)?;
    let head = std::str::from_utf8(&http[..http.len() - bytes.len()]).ok()?;
    let length = head.lines().find_map(|line| {
        line.to_ascii_lowercase().strip_prefix("content-length:").and_then(|s| s.trim().parse::<usize>().ok())
    })?;
    (bytes.len() == length).then_some(bytes)
}
fn check(socket: &mut Socket, rules: &mut Rules) {
    let Some(path) = socket.shell.received.split(|b| *b == b' ').nth(1) else { return };
    if !socket.checked.request
        && !stream(socket)
        && let Some(actual) = whole(&socket.request)
    {
        let encoded = whole(&socket.shell.received).expect("complete source");
        rules.record(path, false, encoded, actual).expect("request crossed unchanged");
        socket.checked.request = true;
    }
    if stream(socket) {
        socket.source_records.feed(&socket.engine.received);
        socket.crossed_records.feed(&socket.response);
        for (before, after) in
            socket.source_records.values.iter().zip(&socket.crossed_records.values).skip(socket.checked.events)
        {
            rules.record(path, true, before, after).expect("stream record crossed unchanged");
            socket.checked.events += 1;
        }
    } else if !socket.checked.response
        && let Some(actual) = whole(&socket.response)
    {
        let encoded = whole(&socket.engine.received).expect("complete response source");
        if actual.is_empty() && encoded.is_empty() && !socket.response.starts_with(b"HTTP/1.1 200 ") {
            socket.checked.response = true;
            return;
        }
        rules.record(path, true, encoded, actual).unwrap_or_else(|error| {
            panic!("response {path:?} lengths {}/{} tag {:?}: {error:?}", encoded.len(), actual.len(), actual.first())
        });
        socket.checked.response = true;
    }
}

fn room(down: &Down, pending: bool) -> bool {
    match down {
        Down::Demand { room, .. } => *room > 0,
        Down::Send(_) | Down::Finish => pending,
    }
}
