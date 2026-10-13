//! io's stream records joined to an actual HTTP client, with scripted durability.
use crate::fixtures;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_client_wire::v5 as w;
use jig_protocol_engine_clients::{Component, Event, Kept, Limits, Request, SecretsAnswer, SecretsRequest, Settings};
use skein_http::client as http;
use skein_lib::{
    Env, Queue, ReplyTo, Time, Token, Wall, Writer,
    base64::{self, Alphabet},
    stream::{Down, Read, Up},
};
use skein_world::stream::Wire;
use std::collections::{BTreeMap, VecDeque};
#[derive(Debug)]
struct Exchange {
    owner: Option<Token>,
    client: http::Client,
    events: Queue<http::Event>,
    below: Queue<Down>,
    to_engine: Wire,
    to_client: Wire,
    sent_engine: usize,
    sent_client: usize,
    upload: Option<Box<[u8]>>,
    body: Vec<u8>,
    status: Option<u16>,
    done: bool,
}
impl Exchange {
    fn new(target: &[u8], body: Box<[u8]>, bearer: Option<&[u8]>) -> Self {
        let limits = fixtures::client_limits();
        let mut headers =
            vec![skein_http::Header { name: Box::from(b"host".as_slice()), value: Box::from(b"localhost".as_slice()) }];
        if let Some(bearer) = bearer {
            headers.push(skein_http::Header { name: Box::from(b"authorization".as_slice()), value: Box::from(bearer) });
        }
        let mut result = Self {
            owner: None,
            client: http::Client::new(&limits),
            events: Queue::with_capacity(8),
            below: Queue::with_capacity(8),
            to_engine: Wire::new(1),
            to_client: Wire::new(2),
            sent_engine: 0,
            sent_client: 0,
            upload: Some(body),
            body: Vec::new(),
            status: None,
            done: false,
        };
        let length = result.upload.as_ref().expect("upload").len();
        result.down(http::Request::Call(http::Call {
            method: skein_http::Method::Post,
            target: Box::from(target),
            headers: headers.into_boxed_slice(),
            body: http::Body::Length(u64::try_from(length).expect("short body")),
            close: false,
        }));
        result.down(http::Request::Upload(Down::Demand {
            read: Read::Nothing,
            room: u32::try_from(length).expect("short body"),
        }));
        result
    }
    fn env() -> Env<http::Limits> {
        Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fixtures::client_limits() }
    }
    fn down(&mut self, request: http::Request) {
        http::down(&mut self.client, &Self::env(), request, &mut self.events, &mut self.below);
    }
    fn advance(&mut self) -> Option<Up> {
        if let Some(event) = self.events.pop() {
            match event {
                http::Event::Upload(Up::Room) => {
                    let upload = self.upload.take().expect("upload once");
                    self.down(http::Request::Upload(Down::Send(upload)));
                    self.down(http::Request::Upload(Down::Finish));
                }
                http::Event::Response(response) => {
                    self.status = Some(response.status);
                    self.read();
                }
                http::Event::Body(Up::Bytes(bytes)) => {
                    self.body.extend_from_slice(&bytes);
                    self.read();
                }
                http::Event::Body(Up::End) | http::Event::Closed => {}
                http::Event::Done(_) => self.done = true,
                http::Event::Failed(error) => panic!("client failed: {error:?}"),
                http::Event::Upload(Up::Failed(_) | Up::End | Up::Bytes(_))
                | http::Event::Body(Up::Failed(_) | Up::Room) => panic!("unexpected body event"),
            }
        }
        if let Some(down) = self.below.pop() {
            self.to_client.take(down);
        }
        let bytes = &self.to_client.received[self.sent_client..];
        self.to_engine.write(bytes);
        self.sent_client = self.to_client.received.len();
        let bytes = &self.to_engine.received[self.sent_engine..];
        self.to_client.write(bytes);
        self.sent_engine = self.to_engine.received.len();
        if let Some(up) = self.to_client.answer() {
            http::up(&mut self.client, &Self::env(), up, &mut self.events, &mut self.below);
        }
        self.to_engine.answer()
    }
    fn read(&mut self) {
        self.down(http::Request::Body(Down::Demand { read: Read::Fill(1), room: 0 }));
    }
}
#[derive(Debug)]
pub struct Peer {
    component: Component,
    env: Env<Limits>,
    out: Queue<Request>,
    sockets: BTreeMap<Token, Exchange>,
    next_socket: u64,
    links: Vec<Box<[u8]>>,
    core: VecDeque<core::Event>,
    keep: Option<(u64, Kept)>,
    digests: BTreeMap<u64, Kept>,

    last_core: Option<String>,
}
impl Peer {
    #[must_use]
    pub fn started(settings: &Settings) -> Self {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fixtures::limits() };
        let mut out = Queue::with_capacity(64);
        let component = Component::new(settings.clone(), &env, &mut out).expect("component starts");
        let mut peer = Self {
            component,
            env,
            out,
            sockets: BTreeMap::new(),
            next_socket: 100,
            links: Vec::new(),
            core: VecDeque::new(),
            keep: None,
            digests: BTreeMap::new(),

            last_core: None,
        };
        peer.pump();
        peer
    }
    fn event(&mut self, event: Event) {
        self.component.down(&self.env, event, &mut self.out);
        self.pump();
    }
    fn pump(&mut self) {
        // Every packet is tiny; this bound includes the seeded wire's harmless delays.
        for _ in 0..512 {
            self.component.up(&self.env, &mut self.out);
            if let Some(request) = self.out.pop() {
                match request {
                    Request::Io(request) => self.io(request),
                    Request::Core(event) => {
                        self.last_core = Some(format!("{event:?}"));
                        self.core.push_back(*event);
                    }
                    Request::Secrets(SecretsRequest::Keep { sign_in, value }) => self.keep = Some((sign_in, value)),
                    Request::Secrets(SecretsRequest::Get { sign_in }) => {
                        self.component.down(
                            &self.env,
                            Event::Secrets(SecretsAnswer::Got { sign_in, value: self.digests.get(&sign_in).cloned() }),
                            &mut self.out,
                        );
                    }
                    Request::Secrets(SecretsRequest::Erase { sign_in }) => {
                        self.digests.remove(&sign_in);
                    }
                    Request::Link { link } => self.links.push(link),
                    Request::Application(_) | Request::Provider { .. } | Request::Closed => {}
                }
            }
            let mut ups = Vec::new();
            for exchange in self.sockets.values_mut() {
                if let Some(owner) = exchange.owner
                    && let Some(up) = exchange.advance()
                {
                    ups.push((owner, up));
                }
            }
            for (owner, up) in ups {
                self.component.down(&self.env, Event::Io(skein_io::Event::Stream { owner, up }), &mut self.out);
            }
        }
    }
    fn io(&mut self, request: skein_io::Request) {
        match request {
            skein_io::Request::Listen { owner, addr } => self.component.down(
                &self.env,
                Event::Io(skein_io::Event::Listening { owner, listener: Token::new(99), addr }),
                &mut self.out,
            ),
            skein_io::Request::Bind { socket, owner } => {
                self.sockets.get_mut(&socket).expect("accepted socket").owner = Some(owner);
            }
            skein_io::Request::Stream { stream, down } => {
                self.sockets.get_mut(&stream).expect("socket stream").to_engine.take(down);
            }
            skein_io::Request::Close { entity } | skein_io::Request::Abort { entity } => {
                if entity == Token::new(99) {
                    self.component.down(
                        &self.env,
                        Event::Io(skein_io::Event::Closed { owner: Token::new(0) }),
                        &mut self.out,
                    );
                }
                if let Some(exchange) = self.sockets.get(&entity) {
                    let owner = exchange.owner.expect("bound socket");
                    self.component.down(&self.env, Event::Io(skein_io::Event::Closed { owner }), &mut self.out);
                }
            }
            skein_io::Request::Reject { .. } => panic!("unexpected rejected socket"),
            skein_io::Request::Connect { .. }
            | skein_io::Request::Spawn { .. }
            | skein_io::Request::Signal { .. }
            | skein_io::Request::Usage { .. }
            | skein_io::Request::Window { .. }
            | skein_io::Request::Output { .. } => panic!("unexpected io request"),
        }
    }
    #[must_use]
    pub fn printed_link(&self) -> Box<[u8]> {
        self.links.last().expect("printed link").clone()
    }
    pub fn post_link(&mut self, link: &[u8]) -> Token {
        let secret = link.split(|byte| *byte == b'=').next_back().expect("link query");
        let secret = base64::decode(Alphabet::Url, secret, 32).expect("link secret");
        let record = w::LinkSignIn::new(&self.env.limits.wire, w::LinkSignInParts { secret }).expect("bounded secret");
        let mut out = Writer::new(usize::try_from(record.measure()).expect("record size"));
        record.encode(&mut out).expect("encoded link");
        self.post(b"/jig/v1/sign-in/link", out.finish(), None)
    }
    pub fn post(&mut self, target: &[u8], body: Box<[u8]>, bearer: Option<&[u8]>) -> Token {
        let socket = Token::new(self.next_socket);
        self.next_socket += 1;
        self.sockets.insert(socket, Exchange::new(target, body, bearer));
        self.event(Event::Io(skein_io::Event::Accepted {
            owner: Token::new(0),
            socket,
            peer: fixtures::settings().listener,
        }));
        socket
    }
    fn take_sign_in(&mut self) -> ReplyTo {
        let event = self.core.pop_front().expect("core sign-in");
        // The assertion first checks the variant; destructuring reads its public vocabulary.
        assert!(matches!(event, core::Event::SignIn { .. }));
        if let core::Event::SignIn { reply_to, identity } = event {
            assert_eq!(identity, fixtures::first_owner());
            reply_to
        } else {
            panic!("expected sign-in")
        }
    }
    pub fn answer_signed_in(&mut self, sign_in: u64, person: u64) {
        let to = self.take_sign_in();

        self.event(Event::Reply {
            to,
            reply: people::Reply::SignedIn { sign_in, person, expires: fixtures::expires() },
        });
    }
    pub fn refuse_sign_in(&mut self) {
        let to = self.take_sign_in();
        self.event(Event::SignInRefused { to });
    }
    pub fn answer_kept(&mut self, sign_in: u64) {
        let (number, value) = self.keep.take().expect("digest keep requested");
        assert_eq!(number, sign_in);
        self.digests.insert(number, value);
        self.event(Event::Secrets(SecretsAnswer::Kept { sign_in }));
    }
    pub fn fail_keep(&mut self) {
        self.keep = None;
        self.event(Event::Secrets(SecretsAnswer::Failed));
    }
    #[must_use]
    pub fn digest_kept(&self) -> Option<u64> {
        self.keep.as_ref().map(|(number, _)| *number)
    }
    #[must_use]
    pub fn answered(&self, exchange: Token) -> Option<&[u8]> {
        let exchange = self.sockets.get(&exchange).expect("exchange socket");
        exchange.done.then_some(&exchange.body)
    }
    #[must_use]
    pub fn sign_in_answer(&self, exchange: Token) -> w::SignedIn {
        match w::SignInAnswer::decode(
            &self.env.limits.wire,
            &mut skein_lib::Reader::new(self.answered(exchange).expect("answered")),
        )
        .expect("wire answer")
        {
            w::SignInAnswer::SignedIn(answer) => answer,
            w::SignInAnswer::Refused => panic!("sign-in refused"),
        }
    }
    pub fn answer_request(&mut self, to: ReplyTo, reply: people::Reply) {
        self.event(Event::Reply { to, reply });
    }
    pub fn take_core(&mut self) -> Option<core::Event> {
        self.core.pop_front()
    }
    #[must_use]
    pub fn core_record(&self) -> &str {
        self.last_core.as_deref().expect("core sign-in")
    }
}
