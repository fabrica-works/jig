//! io-free HTTP machines, with shell-owned authentication (protocol/client.md, 4).
use crate::{Signing, Unlinked, sign_in};
use jig_protocol_client_link::boundary as link;
use skein_http::{Header, Method, client as http};
use skein_lib::{
    Env, Queue, Secret, Time, Token, Wall,
    base64::{self, Alphabet},
    stream::{self, Down, Read, Up},
};
use std::collections::{BTreeMap, VecDeque};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Conn(pub u32);
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub exchanges: u32,
    pub body_bytes: u32,
    pub http: http::Limits,
}
#[derive(Clone, Copy, Debug)]
enum Owner {
    Link(Token),
    Stream,
    SignIn(Token),
    SignOut(Token),
}
type Pending = (Owner, &'static [u8], Box<[u8]>);
struct Exchange {
    owner: Owner,
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
    host: String,
    token: Option<Secret>,
    next: u32,
    exchanges: BTreeMap<Conn, Exchange>,
    pending: VecDeque<Pending>,
    events: VecDeque<link::Event>,
    signing: VecDeque<Signing>,
}
impl Connections {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        assert!(
            limits.exchanges > 0 && limits.body_bytes > 0 && http::worst_case(&limits.http).is_some(),
            "usable transport limits"
        );
        Self {
            limits: *limits,
            host: "localhost".to_owned(),
            token: None,
            next: 1,
            exchanges: BTreeMap::new(),
            pending: VecDeque::new(),
            events: VecDeque::new(),
            signing: VecDeque::new(),
        }
    }
    pub fn set_token(&mut self, token: Option<Box<[u8]>>) {
        self.token = token.map(Secret::new);
    }
    #[must_use]
    pub fn token(&self) -> Option<&Secret> {
        self.token.as_ref()
    }
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }
    /// # Errors
    /// Refuses any link outside the printed HTTP link's form.
    pub fn sign_in(&mut self, link: &[u8]) -> Result<Token, Unlinked> {
        let (host, body) = sign_in::parse(link)?;
        if self.host != "localhost" && self.host != host {
            return Err(Unlinked::Address);
        }
        self.host = host;
        let exchange = Token::new(u64::from(self.name().0));
        self.enqueue(Owner::SignIn(exchange), b"/jig/v1/sign-in/link", body);
        Ok(exchange)
    }
    pub fn sign_out(&mut self) -> Token {
        let exchange = Token::new(u64::from(self.name().0));
        self.enqueue(Owner::SignOut(exchange), b"/jig/v1/sign-out", Box::new([]));
        exchange
    }
    pub fn signing(&mut self) -> Option<Signing> {
        self.signing.pop_front()
    }
    fn name(&mut self) -> Conn {
        let conn = Conn(self.next);
        self.next = self.next.checked_add(1).expect("finite connection namespace");
        conn
    }
    fn enqueue(&mut self, owner: Owner, path: &'static [u8], body: Box<[u8]>) {
        if self.pending.len() + self.exchanges.values().filter(|value| !matches!(value.owner, Owner::Stream)).count()
            >= usize::try_from(self.limits.exchanges).expect("u32 fits")
        {
            self.failed(owner);
        } else {
            self.pending.push_back((owner, path, body));
        }
    }
    pub fn down(&mut self, now: Time, request: link::Request, below: &mut Queue<(Conn, stream::Down)>) {
        match request {
            link::Request::Exchange { exchange, path, body } => self.enqueue(Owner::Link(exchange), path, body),
            link::Request::OpenStream => {
                if !self.exchanges.values().any(|value| matches!(value.owner, Owner::Stream)) {
                    self.start(now, Owner::Stream, b"/jig/v1/stream", Box::new([]), below);
                }
            }
            link::Request::Abandon { exchange } => self.close(now, Some(exchange), below),
            link::Request::CloseStream => self.close(now, None, below),
            link::Request::Write { .. }
            | link::Request::Remove { .. }
            | link::Request::Kept { .. }
            | link::Request::Dropped { .. }
            | link::Request::Answer { .. }
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
        self.flush(now, below);
        while let Some(event) = self.events.pop_front() {
            to_link.push(event);
        }
    }
    fn flush(&mut self, now: Time, below: &mut Queue<(Conn, Down)>) {
        while let Some((owner, path, body)) = self.pending.pop_front() {
            self.start(now, owner, path, body, below);
        }
    }
    fn env(&self, now: Time) -> Env<http::Limits> {
        Env { now, wall: Wall::EPOCH, limits: self.limits.http }
    }
    fn start(&mut self, now: Time, owner: Owner, path: &[u8], body: Box<[u8]>, below: &mut Queue<(Conn, Down)>) {
        if body.len() > usize::try_from(self.limits.body_bytes).expect("u32 fits") {
            self.failed(owner);
            return;
        }
        let conn = self.name();
        let length = body.len();
        let mut headers = vec![Header { name: b"Host".as_slice().into(), value: self.host.as_bytes().into() }];
        if !matches!(owner, Owner::Stream) {
            headers.push(Header {
                name: b"Content-Type".as_slice().into(),
                value: b"application/octet-stream".as_slice().into(),
            });
        }
        if !matches!(owner, Owner::SignIn(_))
            && let Some(token) = &self.token
        {
            let size = base64::encoded_len(Alphabet::Url, u32::try_from(token.len()).expect("short token"))
                .expect("short bearer");
            let mut writer = skein_lib::Writer::new(usize::try_from(size).expect("u32 fits"));
            base64::encode(Alphabet::Url, token.expose(), &mut writer).expect("measured bearer");
            let mut value = b"Bearer ".to_vec();
            value.extend_from_slice(&writer.finish());
            headers.push(Header { name: b"Authorization".as_slice().into(), value: value.into_boxed_slice() });
        }
        let mut value = Exchange {
            owner,
            machine: http::Client::new(&self.limits.http),
            events: Queue::with_capacity(8),
            below: Queue::with_capacity(8),
            upload: body,
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
                method: if matches!(owner, Owner::Stream) { Method::Get } else { Method::Post },
                target: path.into(),
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
                        Owner::Link(exchange) => self.events.push_back(link::Event::Answered {
                            exchange,
                            status: value.status,
                            body: std::mem::take(&mut value.body).into_boxed_slice(),
                        }),
                        Owner::Stream => self.events.push_back(link::Event::StreamEnded),
                        Owner::SignIn(exchange) | Owner::SignOut(exchange) => self.signing.push_back(sign_in::answer(
                            exchange,
                            value.status,
                            &value.body,
                            matches!(value.owner, Owner::SignOut(_)),
                            &mut self.token,
                        )),
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
            Read::Fill(
                value
                    .remaining
                    .map_or(1, |n| u32::try_from(n.max(1).min(u64::from(env.limits.read))).expect("bounded read")),
            )
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
            Owner::Link(exchange) => self.events.push_back(link::Event::Failed { exchange }),
            Owner::Stream => self.events.push_back(link::Event::StreamEnded),
            Owner::SignIn(exchange) | Owner::SignOut(exchange) => self.signing.push_back(Signing::Lost { exchange }),
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
        self.signing.clear();
    }
    fn close(&mut self, now: Time, exchange: Option<Token>, below: &mut Queue<(Conn, Down)>) {
        let conn = self.exchanges.iter().find_map(|(conn, value)| match value.owner {
            Owner::Link(token) if Some(token) == exchange => Some(*conn),
            Owner::Stream if exchange.is_none() => Some(*conn),
            Owner::Link(_) | Owner::Stream | Owner::SignIn(_) | Owner::SignOut(_) => None,
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
