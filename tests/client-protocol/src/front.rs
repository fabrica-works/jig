//! The client's unchanged people and referee, over link and shell machines.
use crate::{engine::Engine, family, fixtures, net::Net};
use jig_client_protocol_link::{self as link, Link};
use jig_client_shell_native::{Conn, Connections};
use jig_client_test_domain::{self as client, Event, JigOut, Request};
use jig_client_world::{ClientId, Front, FrontFault, Test, harness::Observed};
use link::Signing;
use skein_lib::{
    Queue, Time, Token, Wall,
    stream::{Down, Up},
};
use std::collections::BTreeMap;
mod faults;
pub(crate) mod persistence;
pub(super) struct Side {
    pub(super) link: Link,
    connections: Connections,
    pub(super) out: Queue<link::Request>,
    up: Queue<link::Event>,
    below: Queue<(Conn, Down)>,
    pending: Vec<Request>,
    callbacks: Vec<Event>,
    pub(super) authenticated: bool,
    signing_out: bool,
    signing_link: Option<Box<[u8]>>,
    signing: Vec<Signing>,
    next: u64,
    routes: BTreeMap<Token, (bool, Token)>,
    watched: BTreeMap<(bool, Token), Token>,
    subjects: BTreeMap<Token, client::family::Subject>,
}
impl Side {
    pub(super) fn new() -> Self {
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
            signing_out: false,
            signing_link: None,
            signing: Vec::new(),
            next: 0,
            routes: BTreeMap::new(),
            watched: BTreeMap::new(),
            subjects: BTreeMap::new(),
        }
    }
    pub(super) fn event(&mut self, now: Time, event: link::Event) {
        self.link.step(now, event, &mut self.out);
    }
    fn route(&mut self, family: bool, original: Token) -> Token {
        self.next = self.next.checked_add(1).expect("world route names fit");
        let token = Token::new(self.next);
        self.routes.insert(token, (family, original));
        token
    }
    pub(super) fn callback(&mut self, request: link::Request) -> Result<Event, link::Request> {
        Ok(match request {
            link::Request::Answer { attempt, envelope, answer, .. } => Event::Answer {
                attempt,
                envelope,
                answer: answer.and_then(|answer| match answer {
                    jig_client_domain::Answered::Jig(answer) => Some(client::Answered::Jig(answer)),
                    jig_client_domain::Answered::Family(record) => family::answer(&record).map(client::Answered::First),
                }),
            },
            link::Request::NotKept { attempt } => {
                Event::Answer { attempt, envelope: skein_lib::RequestEnvelope::Final, answer: None }
            }
            link::Request::Watch { watch, event } => {
                let (is_family, original) = *self.routes.get(&watch).expect("live watch route");
                if is_family {
                    let event = match event {
                        jig_client_domain::contract::WatchEvent::Opened => client::first::WatchEvent::Opened,
                        jig_client_domain::contract::WatchEvent::Alive => client::first::WatchEvent::Alive,
                        jig_client_domain::contract::WatchEvent::Delivered(_) => panic!("jig delivery to family route"),
                    };
                    Event::First(client::first::Event::Watch { watch: original, event })
                } else {
                    Event::Jig(jig_domain_client::Event::Watch { watch: original, event })
                }
            }
            link::Request::FamilyDelivery { watch, record } => {
                let (is_family, original) = *self.routes.get(&watch).expect("live family watch");
                assert!(is_family);
                let subject = *self.subjects.get(&watch).expect("original scoped subject");
                let value = family::document(subject, &record.bytes).expect("admitted exact document");
                Event::First(client::first::Event::Watch {
                    watch: original,
                    event: client::first::WatchEvent::Delivered(value),
                })
            }
            link::Request::Ended { watch, end } => {
                let (is_family, original) = self.routes.remove(&watch).expect("ended watch route");
                self.watched.remove(&(is_family, original));
                self.subjects.remove(&watch);
                if is_family {
                    Event::First(client::first::Event::Ended { watch: original, end: family::end(end) })
                } else {
                    Event::Jig(jig_domain_client::Event::Ended { watch: original, end })
                }
            }
            link::Request::Read { read, result } => {
                let (is_family, original) = self.routes.remove(&read).expect("original read route");
                match result {
                    jig_client_domain::Readout::Jig(result) => {
                        if is_family {
                            self.subjects.remove(&read);
                            Event::First(client::first::Event::Read {
                                read: original,
                                result: family::read_none(&result),
                            })
                        } else {
                            Event::Jig(jig_domain_client::Event::Read { read: original, result })
                        }
                    }
                    jig_client_domain::Readout::Family { family: 1, record } => {
                        assert!(is_family);
                        let subject = self.subjects.remove(&read).expect("original read subject");
                        let result = record
                            .map_or(client::first::ReadResult::Unknown, |record| family::read_result(subject, &record));
                        Event::First(client::first::Event::Read { read: original, result })
                    }
                    jig_client_domain::Readout::Family { .. } => panic!("unknown family read"),
                    jig_client_domain::Readout::NotServed { .. } => {
                        unreachable!("the link does not judge domain reads")
                    }
                    jig_client_domain::Readout::Limit => unreachable!("the link does not limit domain reads"),
                }
            }
            link::Request::Hello { family: None, versions } => {
                Event::Jig(jig_domain_client::Event::Hello { served: versions.is_some_and(link::serves) })
            }
            transport @ (link::Request::Exchange { .. }
            | link::Request::OpenStream { .. }
            | link::Request::CloseStream) => return Err(transport),
            ignored @ (link::Request::Signing { .. }
            | link::Request::Save { .. }
            | link::Request::Erase { .. }
            | link::Request::Kept { .. }
            | link::Request::Dropped
            | link::Request::Hello { family: Some(_), .. }
            | link::Request::Fact { .. }
            | link::Request::ServiceCredential { .. }) => return Err(ignored),
        })
    }
    pub(super) fn wire(&mut self, now: Time, request: Request) {
        if !self.authenticated {
            self.pending.push(request);
            return;
        }
        let event = match request {
            Request::Send { attempt, key, ask: client::Ask::Jig(ask) } => {
                link::Event::Send { attempt, key, ask: jig_client_domain::Ask::Jig(ask) }
            }
            Request::Jig(JigOut::Read { read, what }) => {
                let read = self.route(false, read);
                link::Event::Read { read, what: jig_client_domain::Readable::Jig(what) }
            }
            Request::Jig(JigOut::Open { watch, subject }) => {
                let routed = self.route(false, watch);
                self.watched.insert((false, watch), routed);
                link::Event::Open { watch: routed, subject: jig_client_domain::Watchable::Jig(subject) }
            }
            Request::Jig(JigOut::Continue { watch, subject }) => {
                let watch = *self.watched.get(&(false, watch)).expect("live jig watch");
                link::Event::Continue { watch, subject }
            }
            Request::Jig(JigOut::Close { watch }) => {
                let Some(watch) = self.watched.remove(&(false, watch)) else { return };
                link::Event::Close { watch }
            }
            Request::Send { attempt, key, ask: client::Ask::First(ask) } => {
                let Ok(ask) = family::request(&ask) else {
                    self.callbacks.push(Event::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Lost,
                        answer: None,
                    });
                    return;
                };
                link::Event::Send { attempt, key, ask: jig_client_domain::Ask::Family(ask) }
            }
            Request::First(client::first::Out::Read { read, subject }) => {
                let Ok(what) = family::read(subject) else {
                    self.callbacks.push(Event::First(client::first::Event::Read {
                        read,
                        result: client::first::ReadResult::Unsupported,
                    }));
                    return;
                };
                let read = self.route(true, read);
                self.subjects.insert(read, subject);
                link::Event::Read { read, what: jig_client_domain::Readable::Family(what) }
            }
            Request::First(client::first::Out::Open { watch, subject }) => {
                let Ok(record) = family::subject(subject) else {
                    self.callbacks.push(Event::First(client::first::Event::Ended {
                        watch,
                        end: client::first::WatchEnd::Unsupported,
                    }));
                    return;
                };
                let routed = self.route(true, watch);
                self.watched.insert((true, watch), routed);
                self.subjects.insert(routed, subject);
                link::Event::Open { watch: routed, subject: jig_client_domain::Watchable::Family(record) }
            }
            Request::First(client::first::Out::Close { watch }) => {
                let Some(watch) = self.watched.remove(&(true, watch)) else { return };
                link::Event::Close { watch }
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
    last_client: Option<ClientId>,
    release: Option<Time>,
    restart: Option<Time>,
    versions: bool,
    stores: BTreeMap<ClientId, persistence::Bytes>,
    deliveries: BTreeMap<ClientId, u64>,
    answers: BTreeMap<ClientId, u64>,
}
impl Protocol {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self::bounded(seed, &fixtures::owner_limits())
    }
    #[must_use]
    pub fn bounded(seed: u64, limits: &jig_test_domain::Limits) -> Self {
        let mut value = Self {
            engine: Engine::bounded(seed, limits),
            clients: BTreeMap::new(),
            net: Net::new(seed),
            callbacks: Vec::new(),
            now: Time::ZERO,
            last_client: None,
            release: None,
            restart: None,
            versions: false,
            stores: BTreeMap::new(),
            deliveries: BTreeMap::new(),
            answers: BTreeMap::new(),
        };
        value.advance(Time::ZERO, fixtures::wall(Time::ZERO));
        value
    }
    #[must_use]
    pub fn chats_made(&self) -> usize {
        self.engine.chats_made()
    }
    pub fn adopt_family_resource(&mut self, resource: jig_test_connector::Path) {
        self.engine.driver.world.send(jig_test_domain::Event::Connector {
            number: 1,
            event: jig_test_connector::Event::Adopt {
                project: 1,
                resource: resource.clone(),
                role: jig_test_connector::ResourceRole::Owned,
            },
        });
        self.engine.driver.world.send(jig_test_domain::Event::Connector {
            number: 1,
            event: jig_test_connector::Event::System(jig_test_connector::SystemEvent::Fact {
                resource,
                fact: jig_test_connector::Fact { state: Some(0), observed: fixtures::wall(Time::ZERO), pending: false },
                origin: jig_test_connector::Origin::Other,
            }),
        });
    }
    #[must_use]
    pub fn application_objects(&self) -> Vec<(u64, jig_test_connector::ApplicationObject)> {
        self.engine
            .driver
            .world
            .store
            .rows
            .values()
            .filter_map(|record| {
                if let jig_test_domain::Record::Connector {
                    number: 1,
                    record: jig_test_connector::Record::ApplicationObject { object, now },
                } = record
                {
                    Some((*object, *now))
                } else {
                    None
                }
            })
            .collect()
    }
    fn advance(&mut self, now: Time, wall: Wall) {
        self.now = now;
        self.engine.clock(now, wall);
        self.release_faults(now, wall);
        self.net.clock(now);
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
                    if let link::Request::Signing { signing } = request {
                        side.signing.push(signing);
                        continue;
                    }
                    match side.callback(request) {
                        Ok(event) => {
                            self.engine.rules.domain(&event).expect("no secret in a client domain event");
                            count(*client, &event, &mut self.deliveries, &mut self.answers);
                            self.callbacks.push((*client, event));
                        }
                        Err(request) => side.connections.down(now, request, &mut side.below),
                    }
                }
                self.callbacks.extend(side.callbacks.drain(..).map(|event| (*client, event)));
                side.connections.take(now, &mut side.up, &mut side.below);
                for signing in std::mem::take(&mut side.signing) {
                    active = true;
                    match signing {
                        Signing::SignedIn { .. } => {
                            side.authenticated = true;
                            side.signing_link = None;
                            if let Some(secret) = side.link.credential() {
                                self.engine.rules.secret(secret.expose());
                            }
                            for request in std::mem::take(&mut side.pending) {
                                side.wire(now, request);
                            }
                        }
                        Signing::SignedOut => {
                            side.authenticated = false;
                            side.connections.down(now, link::Request::CloseStream, &mut side.below);
                            side.event(now, link::Event::StreamGone);
                            side.signing_out = false;
                            self.callbacks.push((*client, Event::SignedOut));
                        }
                        Signing::Refused if !side.signing_out => {
                            if side.signing_link.as_ref().is_some_and(|old| old != &self.engine.link) {
                                side.signing_link = Some(self.engine.link.clone());
                                let secret = side.connections.printed(&self.engine.link).expect("new printed link");
                                side.event(now, link::Event::SignIn { with: link::With::Link(secret) });
                                side.connections.take(now, &mut side.up, &mut side.below);
                            }
                        }
                        Signing::Again | Signing::Lost | Signing::Refused => {
                            if side.signing_out {
                                side.signing_out = false;
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
            let (events, clients, changed) = self.net.advance(&mut self.engine.rules);
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
        self.last_client = Some(client);
        self.clients.get_mut(&client).expect("bound client").wire(now, request);
    }
    fn poll(&mut self, now: Time, wall: Wall) -> Vec<(ClientId, Event)> {
        self.advance(now, wall);
        std::mem::take(&mut self.callbacks)
    }
    fn next_due(&self) -> Option<Time> {
        self.clients
            .values()
            .flat_map(|side| [side.link.next_deadline(), side.connections.next_deadline()])
            .flatten()
            .chain(self.engine.component.next_deadline())
            .chain(self.engine.driver.world.next_due())
            .chain(self.release)
            .chain(self.restart)
            .chain(self.net.next_due())
            .min()
    }
    fn observed(&mut self) -> Vec<Observed> {
        self.engine.observed()
    }
    fn fault(&mut self, now: Time, fault: FrontFault) {
        self.apply_fault(now, fault);
    }
    fn reload(&mut self, client: ClientId, now: Time) {
        self.reload_side(client, now);
    }
    fn stored(&mut self, client: ClientId, records: &[client::store::Record]) {
        self.stores.insert(client, persistence::encode(records));
    }
    fn restored(&mut self, client: ClientId, _records: Vec<client::store::Record>) -> Vec<client::store::Record> {
        persistence::decode(self.stores.get(&client).expect("kept small store"))
    }
    fn received(&mut self, _client: ClientId, _event: &Event) {}
    fn sign_in(&mut self, client: ClientId, now: Time, _wall: Wall) {
        self.last_client = Some(client);
        let side = self.clients.get_mut(&client).expect("bound client");
        side.signing_link = Some(self.engine.link.clone());
        let secret = side.connections.printed(&self.engine.link).expect("engine's printed link");
        side.event(now, link::Event::SignIn { with: link::With::Link(secret) });
        side.connections.take(now, &mut side.up, &mut side.below);
    }
    fn sign_out(&mut self, client: ClientId, now: Time, _wall: Wall) {
        let side = self.clients.get_mut(&client).expect("bound client");
        side.signing_out = true;
        side.event(now, link::Event::SignOut);
        side.connections.take(now, &mut side.up, &mut side.below);
    }
}

fn count(
    client: ClientId,
    event: &Event,
    deliveries: &mut BTreeMap<ClientId, u64>,
    answers: &mut BTreeMap<ClientId, u64>,
) {
    if matches!(
        event,
        Event::Jig(jig_domain_client::Event::Watch {
            event: jig_client_domain::contract::WatchEvent::Delivered(_),
            ..
        })
    ) {
        *deliveries.entry(client).or_default() += 1;
    }
    if matches!(event, Event::Answer { envelope: skein_lib::RequestEnvelope::Final, .. }) {
        *answers.entry(client).or_default() += 1;
    }
}
