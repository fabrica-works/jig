//! The client's unchanged people and referee, over link and shell machines.
use crate::{engine::Engine, fixtures, net::Net};
use jig_client_world::{ClientId, Front, FrontFault, Test, harness::Observed};
use jig_protocol_client_link::{self as link, Link};
use jig_shell_native::{Conn, Connections, Signing};
use jig_test_domain_client::{self as client, Event, JigOut, Request};
use skein_lib::{
    Queue, Time, Token, Wall,
    stream::{Down, Up},
};
use std::collections::BTreeMap;
struct Side {
    link: Link,
    connections: Connections,
    out: Queue<link::Request>,
    up: Queue<link::Event>,
    below: Queue<(Conn, Down)>,
    pending: Vec<Request>,
    callbacks: Vec<Event>,
    authenticated: bool,
    signing_out: Option<Token>,
}
impl Side {
    fn new() -> Self {
        let limits = fixtures::link_limits();
        Self {
            link: Link::new(&limits).expect("bounded link"),
            connections: Connections::new(&fixtures::connections_limits()),
            out: Queue::with_capacity(256),
            up: Queue::with_capacity(256),
            below: Queue::with_capacity(256),
            pending: Vec::new(),
            callbacks: Vec::new(),
            authenticated: false,
            signing_out: None,
        }
    }
    fn event(&mut self, now: Time, event: link::Event) {
        self.link.step(now, event, &mut self.out);
    }
    fn wire(&mut self, now: Time, request: Request) {
        if !self.authenticated {
            self.pending.push(request);
            return;
        }
        let event = match request {
            Request::Send { attempt, key, ask: client::Ask::Jig(ask) } => {
                link::Event::Send { attempt, key, ask: link::Ask::Jig(ask) }
            }
            Request::Jig(JigOut::Read { read, what }) => link::Event::Read { read, what: link::Readable::Jig(what) },
            Request::Jig(JigOut::Open { watch, subject }) => {
                link::Event::Open { watch, subject: link::Watchable::Jig(subject) }
            }
            Request::Jig(JigOut::Continue { watch, subject }) => link::Event::Continue { watch, subject },
            Request::Jig(JigOut::Close { watch }) => link::Event::Close { watch },
            Request::Send { attempt, ask: client::Ask::First(_), .. } => {
                self.callbacks.push(Event::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(client::Answered::First(client::first::Answer::Gone)),
                });
                return;
            }
            Request::First(client::first::Out::Read { read, .. }) => {
                self.callbacks.push(Event::First(client::first::Event::Read {
                    read,
                    result: client::first::ReadResult::Unknown,
                }));
                return;
            }
            Request::First(client::first::Out::Open { watch, .. }) => {
                self.callbacks
                    .push(Event::First(client::first::Event::Ended { watch, end: client::first::WatchEnd::Unknown }));
                return;
            }
            Request::First(client::first::Out::Close { watch }) => {
                self.callbacks
                    .push(Event::First(client::first::Event::Ended { watch, end: client::first::WatchEnd::Closed }));
                return;
            }
            Request::First(client::first::Out::Fact { .. })
            | Request::Jig(JigOut::Answered { .. } | JigOut::Unknown { .. })
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Show { .. }
            | Request::SignIn { .. }
            | Request::SignOut
            | Request::Fact { .. } => return,
        };
        self.event(now, event);
    }
}
pub struct Protocol {
    engine: Engine,
    clients: BTreeMap<ClientId, Side>,
    net: Net,
    callbacks: Vec<(ClientId, Event)>,
    now: Time,
}
impl Protocol {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let mut value = Self {
            engine: Engine::new(seed),
            clients: BTreeMap::new(),
            net: Net::new(),
            callbacks: Vec::new(),
            now: Time::ZERO,
        };
        value.advance(Time::ZERO, fixtures::wall(Time::ZERO));
        value
    }
    #[must_use]
    pub fn chats_made(&self) -> usize {
        self.engine.chats_made()
    }
    fn advance(&mut self, now: Time, wall: Wall) {
        self.now = now;
        self.engine.clock(now, wall);
        let mut quiet = 0;
        for _ in 0..100_000 {
            let mut active = self.engine.poll();
            while let Some(request) = self.engine.out.pop() {
                active = true;
                if let Some(request) = self.engine.request(request)
                    && let Some(event) = self.net.io(request)
                {
                    self.engine.event(jig_protocol_engine_clients::Event::Io(event));
                }
            }
            for (client, side) in &mut self.clients {
                side.link.fire(now, &mut side.out);
                while let Some(event) = side.up.pop() {
                    active = true;
                    side.event(now, event);
                }
                while let Some(request) = side.out.pop() {
                    active = true;
                    match callback(request) {
                        Ok(event) => self.callbacks.push((*client, event)),
                        Err(request) => side.connections.down(now, request, &mut side.below),
                    }
                }
                self.callbacks.extend(side.callbacks.drain(..).map(|event| (*client, event)));
                side.connections.take(now, &mut side.up, &mut side.below);
                while let Some(signing) = side.connections.signing() {
                    active = true;
                    match signing {
                        Signing::SignedIn { .. } => {
                            side.authenticated = true;
                            side.connections.down(now, link::Request::OpenStream, &mut side.below);
                            for request in std::mem::take(&mut side.pending) {
                                side.wire(now, request);
                            }
                        }
                        Signing::SignedOut { .. } => {
                            side.authenticated = false;
                            side.signing_out = None;
                            self.callbacks.push((*client, Event::SignedOut));
                        }
                        Signing::Again { exchange } | Signing::Lost { exchange } | Signing::Refused { exchange } => {
                            if side.signing_out == Some(exchange) {
                                side.signing_out = None;
                                self.callbacks.push((*client, Event::Jig(jig_domain_client::Event::SignOutLost)));
                            }
                        }
                    }
                }
                while let Some((conn, down)) = side.below.pop() {
                    active = true;
                    if let Some(event) = self.net.down(*client, conn, down) {
                        self.engine.event(jig_protocol_engine_clients::Event::Io(event));
                    }
                }
            }
            let (events, clients, changed) = self.net.advance();
            active |= changed;
            for event in events {
                self.engine.event(jig_protocol_engine_clients::Event::Io(event));
            }
            for (client, conn, up) in clients {
                self.up(client, conn, now, up);
            }
            quiet = if active { 0 } else { quiet + 1 };
            if quiet == 32 {
                return;
            }
        }
        panic!("byte protocol did not quiesce");
    }
    fn up(&mut self, client: ClientId, conn: Conn, now: Time, up: Up) {
        let side = self.clients.get_mut(&client).expect("connection client");
        side.connections.up(now, conn, up, &mut side.up, &mut side.below);
    }
}
impl Front<Test> for Protocol {
    fn bind(&mut self, client: ClientId, party: u64) {
        assert_eq!(party, fixtures::FIRST_OWNER, "printed link is the configured first owner");
        self.clients.insert(client, Side::new());
        self.callbacks.push((client, Event::SignedOut));
    }
    fn wire(&mut self, client: ClientId, now: Time, _wall: Wall, request: Request) {
        self.clients.get_mut(&client).expect("bound client").wire(now, request);
    }
    fn poll(&mut self, now: Time, wall: Wall) -> Vec<(ClientId, Event)> {
        self.advance(now, wall);
        std::mem::take(&mut self.callbacks)
    }
    fn next_due(&self) -> Option<Time> {
        self.clients
            .values()
            .filter_map(|side| side.link.next_deadline())
            .chain(self.engine.component.next_deadline())
            .chain(self.engine.driver.world.next_due())
            .min()
    }
    fn observed(&mut self) -> Vec<Observed> {
        self.engine.observed()
    }
    fn fault(&mut self, _now: Time, _fault: FrontFault) {
        panic!("faults join in 33.5");
    }
    fn reload(&mut self, _client: ClientId, _now: Time) {
        panic!("reload joins in 33.5");
    }
    fn received(&mut self, _client: ClientId, _event: &Event) {}
    fn sign_in(&mut self, client: ClientId, now: Time, _wall: Wall) {
        let side = self.clients.get_mut(&client).expect("bound client");
        side.connections.sign_in(&self.engine.link).expect("engine's actual printed link");
        side.connections.take(now, &mut side.up, &mut side.below);
    }
    fn sign_out(&mut self, client: ClientId, now: Time, _wall: Wall) {
        let side = self.clients.get_mut(&client).expect("bound client");
        side.signing_out = Some(side.connections.sign_out());
        side.connections.take(now, &mut side.up, &mut side.below);
    }
}
fn callback(request: link::Request) -> Result<Event, link::Request> {
    Ok(match request {
        link::Request::Answer { attempt, envelope, answer } => Event::Answer {
            attempt,
            envelope,
            answer: answer.map(|answer| match answer {
                link::Answered::Jig(answer) => client::Answered::Jig(answer),
                link::Answered::Family(_) => panic!("family records join in 33.2"),
            }),
        },
        link::Request::Watch { watch, event } => Event::Jig(jig_domain_client::Event::Watch { watch, event }),
        link::Request::Ended { watch, end } => Event::Jig(jig_domain_client::Event::Ended { watch, end }),
        link::Request::Hello { family: None, versions } => Event::Jig(jig_domain_client::Event::Hello { versions }),
        link::Request::Read { read, result: link::Readout::Jig(result) } => {
            Event::Jig(jig_domain_client::Event::Read { read, result })
        }
        transport @ (link::Request::Exchange { .. }
        | link::Request::Abandon { .. }
        | link::Request::OpenStream
        | link::Request::CloseStream) => return Err(transport),
        link::Request::Write { .. }
        | link::Request::Remove { .. }
        | link::Request::Kept { .. }
        | link::Request::Dropped { .. }
        | link::Request::FamilyDelivery { .. }
        | link::Request::Hello { family: Some(_), .. }
        | link::Request::Read { result: link::Readout::Family { .. }, .. }
        | link::Request::Fact { .. }
        | link::Request::ServiceCredential { .. } => return Err(request),
    })
}
