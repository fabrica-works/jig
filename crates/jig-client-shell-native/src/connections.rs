//! io-free HTTP machines, with transport-owned exchange deadlines (protocol/client.md, 4).
use jig_client_protocol_http::{self as mapping, Call, Mapping, Unlinked};
use jig_client_protocol_link::boundary as link;
use skein_http::{Header, client as http};
use skein_lib::{
    Duration, Env, Queue, Secret, Time, Token, Wall,
    stream::{self, Down, Read, Up},
};
use std::collections::{BTreeMap, VecDeque};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Conn(pub u32);
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub exchanges: u32,
    pub body_bytes: u32,
    pub deadline: Duration,
    pub http: http::Limits,
}
#[derive(Clone, Copy, Debug)]
enum Owner {
    Link(Token),
    Stream,
}
type Pending = (Owner, Call);
struct Exchange {
    owner: Owner,
    until: Time,
    machine: http::Client,
    events: Queue<http::Event>,
    below: Queue<Down>,
    upload: Box<[u8]>,
    sent: usize,
    body: Vec<u8>,
    status: u16,
    remaining: Option<u64>,
    terminal: bool,
}
pub struct Connections {
    limits: Limits,
    mapping: Mapping,
    host: String,
    next: u32,
    exchanges: BTreeMap<Conn, Exchange>,
    pending: VecDeque<Pending>,
    events: VecDeque<link::Event>,
}
impl Connections {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        assert!(
            limits.exchanges > 0
                && limits.body_bytes > 0
                && limits.deadline != Duration::ZERO
                && http::worst_case(&limits.http).is_some(),
            "usable transport limits"
        );
        Self {
            limits: *limits,
            mapping: Mapping::new(b"/jig").expect("default mapping"),
            host: "localhost".to_owned(),
            next: 1,
            exchanges: BTreeMap::new(),
            pending: VecDeque::new(),
            events: VecDeque::new(),
        }
    }
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }
    /// # Errors
    /// Refuses any link outside the printed HTTP link's form.
    pub fn printed(&mut self, link: &[u8]) -> Result<Secret, Unlinked> {
        let printed = mapping::printed(&self.mapping, link)?;
        let host = String::from_utf8(printed.host.into_vec()).map_err(|_| Unlinked::Address)?;
        if self.host != "localhost" && self.host != host {
            return Err(Unlinked::Address);
        }
        self.host = host;
        Ok(printed.secret)
    }
    fn name(&mut self) -> Conn {
        let conn = Conn(self.next);
        self.next = self.next.checked_add(1).expect("finite connection namespace");
        conn
    }
    fn enqueue(&mut self, owner: Owner, call: Call) {
        if self.pending.len() + self.exchanges.values().filter(|value| !matches!(value.owner, Owner::Stream)).count()
            >= usize::try_from(self.limits.exchanges).expect("u32 fits")
        {
            self.failed(owner);
        } else {
            self.pending.push_back((owner, call));
        }
    }
    pub fn down(&mut self, now: Time, request: link::Request, below: &mut Queue<(Conn, stream::Down)>) {
        match request {
            link::Request::Exchange { exchange, message, credential } => {
                self.enqueue(Owner::Link(exchange), mapping::exchange(&self.mapping, credential.as_ref(), message));
            }
            link::Request::OpenStream { credential } => {
                if !self.exchanges.values().any(|value| matches!(value.owner, Owner::Stream)) {
                    self.start(now, Owner::Stream, mapping::stream(&self.mapping, credential.as_ref()), below);
                }
            }
            link::Request::CloseStream => self.close(now, None, below),
            link::Request::Write { .. }
            | link::Request::Remove { .. }
            | link::Request::Kept { .. }
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
        self.flush(now, below);
    }
    pub fn take(&mut self, now: Time, to_link: &mut Queue<link::Event>, below: &mut Queue<(Conn, Down)>) {
        self.expire(now, below);
        self.flush(now, below);
        while let Some(event) = self.events.pop_front() {
            to_link.push(event);
        }
    }
    fn flush(&mut self, now: Time, below: &mut Queue<(Conn, Down)>) {
        while let Some((owner, call)) = self.pending.pop_front() {
            self.start(now, owner, call, below);
        }
    }
    fn env(&self, now: Time) -> Env<http::Limits> {
        Env { now, wall: Wall::EPOCH, limits: self.limits.http }
    }
    fn start(&mut self, now: Time, owner: Owner, call: Call, below: &mut Queue<(Conn, Down)>) {
        if call.body.len() > usize::try_from(self.limits.body_bytes).expect("u32 fits") {
            self.failed(owner);
            return;
        }
        let conn = self.name();
        let length = call.body.len();
        let mut headers = vec![Header { name: b"Host".as_slice().into(), value: self.host.as_bytes().into() }];
        headers.extend(call.headers.into_boxed());
        let mut value = Exchange {
            owner,
            until: now.saturating_add(self.limits.deadline),
            machine: http::Client::new(&self.limits.http),
            events: Queue::with_capacity(8),
            below: Queue::with_capacity(8),
            upload: call.body,
            sent: 0,
            body: Vec::new(),
            status: 0,
            remaining: None,
            terminal: false,
        };
        let env = self.env(now);
        http::down(
            &mut value.machine,
            &env,
            http::Request::Call(http::Call {
                method: call.method,
                target: call.path,
                headers: headers.into_boxed_slice(),
                body: if matches!(owner, Owner::Stream) {
                    http::Body::None
                } else {
                    http::Body::Length(u64::try_from(length).expect("body fits"))
                },
                close: false,
            }),
            &mut value.events,
            &mut value.below,
        );
        if length > 0 {
            Self::upload(&env, &mut value);
        } else if !matches!(owner, Owner::Stream) {
            http::down(
                &mut value.machine,
                &env,
                http::Request::Upload(Down::Finish),
                &mut value.events,
                &mut value.below,
            );
        }
        self.exchanges.insert(conn, value);
        self.pump(now, conn, below);
    }
    fn upload(env: &Env<http::Limits>, value: &mut Exchange) {
        let room = u32::try_from(value.upload.len() - value.sent).expect("bounded upload").min(env.limits.send);
        http::down(
            &mut value.machine,
            env,
            http::Request::Upload(Down::Demand { read: Read::Nothing, room }),
            &mut value.events,
            &mut value.below,
        );
    }
    pub fn up(
        &mut self,
        now: Time,
        conn: Conn,
        up: Up,
        to_link: &mut Queue<link::Event>,
        below: &mut Queue<(Conn, Down)>,
    ) {
        self.expire(now, below);
        let env = self.env(now);
        if let Some(value) = self.exchanges.get_mut(&conn) {
            http::up(&mut value.machine, &env, up, &mut value.events, &mut value.below);
        }
        self.pump(now, conn, below);
        self.take(now, to_link, below);
    }
    fn pump(&mut self, now: Time, conn: Conn, below: &mut Queue<(Conn, Down)>) {
        let env = self.env(now);
        let Some(mut value) = self.exchanges.remove(&conn) else {
            return;
        };
        while let Some(event) = value.events.pop() {
            match event {
                http::Event::Upload(Up::Room) => Self::send_upload(&env, &mut value),
                http::Event::Response(response) => {
                    value.status = response.status;
                    value.remaining =
                        if let http::Framing::Length(length) = response.framing { Some(length) } else { None };
                    if matches!(value.owner, Owner::Stream) {
                        self.events.push_back(link::Event::StreamOpened { status: value.status });
                    }
                    Self::read(&env, &mut value);
                }
                http::Event::Body(Up::Bytes(bytes)) => {
                    if let Some(remaining) = &mut value.remaining {
                        *remaining -= u64::try_from(bytes.len()).expect("bounded read");
                    }
                    if matches!(value.owner, Owner::Stream) {
                        self.events.push_back(link::Event::StreamBytes { bytes });
                    } else if value.body.len() + bytes.len()
                        <= usize::try_from(self.limits.body_bytes).expect("u32 fits")
                    {
                        value.body.extend_from_slice(&bytes);
                    } else {
                        self.failed(value.owner);
                        value.terminal = true;
                    }
                    if !value.terminal {
                        Self::read(&env, &mut value);
                    }
                }
                http::Event::Done(_) => {
                    match value.owner {
                        Owner::Link(exchange) => {
                            if !value.terminal {
                                let heard = mapping::answered(
                                    exchange,
                                    value.status,
                                    std::mem::take(&mut value.body).into_boxed_slice(),
                                );
                                self.events.push_back(match heard {
                                    link::Heard::Reply { exchange, message } => {
                                        link::Event::Reply { exchange, message }
                                    }
                                    link::Heard::Lost { exchange } => link::Event::Lost { exchange },
                                });
                            }
                        }
                        Owner::Stream => self.events.push_back(link::Event::StreamEnded),
                    }
                    value.terminal = true;
                }
                http::Event::Failed(_) => {
                    if !value.terminal {
                        self.failed(value.owner);
                    }
                    value.terminal = true;
                }
                http::Event::Body(Up::End | Up::Failed(_))
                | http::Event::Upload(Up::Failed(_))
                | http::Event::Closed => {}
                http::Event::Body(Up::Room) | http::Event::Upload(Up::End | Up::Bytes(_)) => {
                    unreachable!("HTTP body faces follow their direction")
                }
            }
        }
        while let Some(down) = value.below.pop() {
            below.push((conn, down));
        }
        if value.terminal {
            http::down(&mut value.machine, &env, http::Request::Close, &mut value.events, &mut value.below);
            while let Some(down) = value.below.pop() {
                below.push((conn, down));
            }
            below.push((conn, Down::Finish));
        } else {
            self.exchanges.insert(conn, value);
        }
    }
    fn send_upload(env: &Env<http::Limits>, value: &mut Exchange) {
        let end = (value.sent + usize::try_from(env.limits.send).expect("u32 fits")).min(value.upload.len());
        let bytes = value.upload[value.sent..end].into();
        value.sent = end;
        http::down(
            &mut value.machine,
            env,
            http::Request::Upload(Down::Send(bytes)),
            &mut value.events,
            &mut value.below,
        );
        if end == value.upload.len() {
            http::down(
                &mut value.machine,
                env,
                http::Request::Upload(Down::Finish),
                &mut value.events,
                &mut value.below,
            );
            value.upload = Box::new([]);
        } else {
            Self::upload(env, value);
        }
    }
    fn read(env: &Env<http::Limits>, value: &mut Exchange) {
        let read = if matches!(value.owner, Owner::Stream) {
            Read::Line { max: env.limits.read.min(1024) }
        } else {
            match value.remaining {
                Some(n) => Read::Fill(u32::try_from(n.max(1).min(u64::from(env.limits.read))).expect("bounded read")),
                None => Read::Arrived { max: env.limits.read },
            }
        };
        http::down(
            &mut value.machine,
            env,
            http::Request::Body(Down::Demand { read, room: 0 }),
            &mut value.events,
            &mut value.below,
        );
    }
    fn failed(&mut self, owner: Owner) {
        match owner {
            Owner::Link(exchange) => self.events.push_back(link::Event::Lost { exchange }),
            Owner::Stream => self.events.push_back(link::Event::StreamEnded),
        }
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.exchanges.values().filter(|value| matches!(value.owner, Owner::Link(_))).map(|value| value.until).min()
    }
    fn expire(&mut self, now: Time, below: &mut Queue<(Conn, Down)>) {
        let expired: Vec<_> = self
            .exchanges
            .values()
            .filter_map(|value| match value.owner {
                Owner::Link(exchange) if now >= value.until => Some(exchange),
                Owner::Link(_) | Owner::Stream => None,
            })
            .collect();
        for exchange in expired {
            self.close(now, Some(exchange), below);
            self.failed(Owner::Link(exchange));
        }
    }
    pub(crate) fn close_all(&mut self, now: Time, below: &mut Queue<(Conn, Down)>) {
        let env = self.env(now);
        for (conn, mut value) in std::mem::take(&mut self.exchanges) {
            http::down(&mut value.machine, &env, http::Request::Close, &mut value.events, &mut value.below);
            while let Some(down) = value.below.pop() {
                below.push((conn, down));
            }
            below.push((conn, Down::Finish));
        }
        self.pending.clear();
        self.events.clear();
    }
    fn close(&mut self, now: Time, exchange: Option<Token>, below: &mut Queue<(Conn, Down)>) {
        let conn = self.exchanges.iter().find_map(|(conn, value)| match value.owner {
            Owner::Link(token) if Some(token) == exchange => Some(*conn),
            Owner::Stream if exchange.is_none() => Some(*conn),
            Owner::Link(_) | Owner::Stream => None,
        });
        if let Some(conn) = conn {
            let mut value = self.exchanges.remove(&conn).expect("live connection");
            http::down(&mut value.machine, &self.env(now), http::Request::Close, &mut value.events, &mut value.below);
            while let Some(down) = value.below.pop() {
                below.push((conn, down));
            }
            below.push((conn, Down::Finish));
        }
    }
}
