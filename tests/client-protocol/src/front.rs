//! The client's unchanged people and referee, over link and shell machines.
use crate::{engine::Engine, family, fixtures, net::Net};
use jig_client_shell_native::{Conn, Connections};
use jig_client_test_domain::{self as client, Event, Request};
use jig_client_world::{ClientId, Front, FrontFault, Test, harness::Observed};
use skein_lib::{
    Queue, Time, Wall,
    stream::{Down, Up},
};
use std::collections::BTreeMap;
mod effects;
mod faults;
pub mod observe;
pub(crate) mod persistence;
pub(super) struct Side {
    wire: crate::native::wire::Wire,
    connections: Connections,
    up: Queue<jig_client::Event>,
    below: Queue<(Conn, Down)>,
    signing_link: Option<Box<[u8]>>,
    stream_epoch: u64,
}
impl Side {
    pub(super) fn new() -> Self {
        Self {
            wire: crate::native::wire::Wire::default(),
            connections: Connections::new(&fixtures::connections_limits(), fixtures::HOST),
            up: Queue::with_capacity(256),
            below: Queue::with_capacity(256),
            signing_link: None,
            stream_epoch: 0,
        }
    }
    pub(super) fn wire(&mut self, now: Time, request: Request) {
        if let Request::Client { request } = request {
            if matches!(&request, jig_client::Request::CloseStream) {
                self.stream_epoch += 1;
            }
            self.wire.sent(&request);
            self.connections.down(now, request, &mut self.below);
        }
    }
}
pub struct Protocol {
    engine: Engine,
    clients: BTreeMap<ClientId, Side>,
    net: Net,
    callbacks: Vec<(ClientId, Event)>,
    receipts: Vec<(ClientId, observe::Callback, u64, Vec<jig_client_world::harness::Receipt>)>,
    now: Time,
    last_client: Option<ClientId>,
    release: Option<Time>,
    restart: Option<Time>,
    versions: bool,
    stores: BTreeMap<ClientId, persistence::Bytes>,
    deliveries: BTreeMap<ClientId, u64>,
    answers: BTreeMap<ClientId, u64>,
    task_queries_read: u32,
}
impl Protocol {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self::bounded(seed, &fixtures::owner_limits())
    }
    #[must_use]
    pub fn bounded(seed: u64, limits: &jig_test_domain::Limits) -> Self {
        Self::with_engine(Engine::bounded(seed, limits))
    }
    fn with_engine(engine: Engine) -> Self {
        let seed = engine.seed;
        let mut value = Self {
            engine,
            clients: BTreeMap::new(),
            net: Net::new(seed),
            callbacks: Vec::new(),
            receipts: Vec::new(),
            now: Time::ZERO,
            last_client: None,
            release: None,
            restart: None,
            versions: false,
            stores: BTreeMap::new(),
            deliveries: BTreeMap::new(),
            answers: BTreeMap::new(),
            task_queries_read: 0,
        };
        value.advance(Time::ZERO, fixtures::wall(Time::ZERO));
        value
    }
    #[must_use]
    pub fn task_queries_read(&self) -> u32 {
        self.task_queries_read
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
                    self.engine.io_event(event);
                }
            }
            while let Some(request) = self.engine.io() {
                active = true;
                if let Some(event) = self.net.io(request) {
                    self.engine.io_event(event);
                }
            }
            for (client, side) in &mut self.clients {
                side.connections.take(now, &mut side.up, &mut side.below);
                while let Some(event) = side.up.pop() {
                    active = true;
                    if side.wire.sign_in_refused(&event)
                        && side.signing_link.as_ref().is_some_and(|old| old != &self.engine.link)
                    {
                        side.signing_link = Some(self.engine.link.clone());
                        let secret = side.connections.printed(&self.engine.link).expect("new printed link");
                        self.callbacks.push((
                            *client,
                            Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Link(secret) } },
                        ));
                    }
                    let additional = side.wire.additional_receipts(&event);
                    let facts = side.wire.received(&event);
                    let receipts: Vec<_> = facts.iter().flat_map(observe::receipts).chain(additional).collect();
                    if let Some(key) = observe::callback(&event) {
                        self.receipts.push((*client, key, side.stream_epoch, receipts));
                    }
                    for fact in facts {
                        self.engine.rules.domain(&fact).expect("no secret in decoded client facts");
                        match fact {
                            crate::native::Received::Delivery(_) => *self.deliveries.entry(*client).or_default() += 1,
                            crate::native::Received::Answer { .. } => *self.answers.entry(*client).or_default() += 1,
                            crate::native::Received::SignedIn { .. } => {
                                side.signing_link = None;
                                if let Some(secret) = side.wire.credential() {
                                    self.engine.rules.secret(secret.expose());
                                }
                            }
                            crate::native::Received::Sent { .. } | crate::native::Received::Read(_) => {}
                        }
                    }
                    self.callbacks.push((*client, Event::Client { event }));
                }
                while let Some((conn, down)) = side.below.pop() {
                    active = true;
                    if let Some(event) = self.net.down(*client, conn, down) {
                        self.engine.io_event(event);
                    }
                }
            }
            let (events, clients, changed) = self.net.advance(&mut self.engine.rules);
            active |= changed;
            for event in events {
                self.engine.io_event(event);
            }
            for (client, conn, up) in clients {
                self.up(client, conn, now, up);
            }
            quiet = if active { 0 } else { quiet + 1 };
            if quiet == 32 || (!active && self.engine.out.is_empty() && !self.net.pending()) {
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
    fn constructor(
        &self,
        _: jig_client_world::harness::ClientConstructor<Test>,
    ) -> jig_client_world::harness::ClientConstructor<Test> {
        jig_client_world::test::linked
    }
    fn bind(&mut self, client: ClientId, party: u64) {
        assert_eq!(party, fixtures::FIRST_OWNER, "printed link is the configured first owner");
        self.clients.insert(client, Side::new());
    }
    fn starting(&mut self, client: ClientId) -> Vec<Event> {
        self.clients[&client].wire.credential().map_or_else(Vec::new, |credential| {
            vec![Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Held(credential) } }]
        })
    }
    fn wire(&mut self, client: ClientId, now: Time, _wall: Wall, request: Request) {
        if let Request::Client {
            request: jig_client::Request::Exchange { message: jig_client::Message::Read(bytes), .. },
        } = &request
        {
            use jig_client_protocol_wire::v11 as w;
            let owned = w::Owned::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("read envelope");
            if *owned.owner() == w::Owner::Jig
                && matches!(
                    w::Read::decode(&w::CEILINGS, &mut skein_lib::Reader::new(owned.record())),
                    Ok(w::Read::Tasks(_))
                )
            {
                self.task_queries_read += 1;
            }
        }
        self.last_client = Some(client);
        self.clients.get_mut(&client).expect("bound client").wire(now, request);
    }
    fn poll(&mut self, now: Time, wall: Wall) -> Vec<(ClientId, Event)> {
        self.advance(now, wall);
        std::mem::take(&mut self.callbacks)
    }
    fn next_due(&self) -> Option<Time> {
        if !self.callbacks.is_empty() || self.clients.values().any(|side| !side.below.is_empty() || !side.up.is_empty())
        {
            return Some(self.now);
        }
        self.clients
            .values()
            .filter_map(|side| side.connections.next_deadline())
            .chain(self.engine.component.next_deadline())
            .chain(self.engine.server.next_deadline())
            .chain(self.engine.driver.world.next_due())
            .chain(self.release)
            .chain(self.restart)
            .chain(self.net.next_due())
            .min()
    }
    fn sent(&self, request: &Request) -> Option<jig_client_world::harness::Sent> {
        observe::sent(request)
    }
    fn receipts(&mut self, client: ClientId, event: &Event) -> Vec<jig_client_world::harness::Receipt> {
        let Event::Client { event } = event else { return Vec::new() };
        let Some(key) = observe::callback(event) else { return Vec::new() };
        self.receipts
            .iter()
            .position(|(id, held, _, _)| *id == client && *held == key)
            .map_or_else(Vec::new, |at| self.receipts.remove(at).3)
    }
    fn delivers(&mut self, client: ClientId, event: &Event) -> bool {
        let Event::Client { event: event @ (jig_client::Event::Streamed { .. } | jig_client::Event::StreamGone) } =
            event
        else {
            return true;
        };
        let key = observe::callback(event).expect("stream callback key");
        let Some(at) = self.receipts.iter().position(|(id, held, _, _)| *id == client && *held == key) else {
            return false;
        };
        if self.receipts[at].2 == self.clients[&client].stream_epoch {
            true
        } else {
            self.receipts.remove(at);
            false
        }
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
        let bytes = persistence::encode(records);
        if self.stores.get(&client) != Some(&bytes) {
            self.stores.insert(client, bytes);
            self.callbacks.push((client, Event::Client { event: jig_client::Event::Stored }));
        }
    }
    fn restored(&mut self, client: ClientId, _records: Vec<client::store::Record>) -> Vec<client::store::Record> {
        persistence::decode(self.stores.get(&client).expect("kept small store"))
    }
    fn received(&mut self, _client: ClientId, _event: &Event) {}
    fn sign_in(&mut self, client: ClientId, _now: Time, _wall: Wall) {
        self.last_client = Some(client);
        let side = self.clients.get_mut(&client).expect("bound client");
        side.signing_link = Some(self.engine.link.clone());
        let secret = side.connections.printed(&self.engine.link).expect("engine's printed link");
        self.callbacks.push((
            client,
            Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Link(secret) } },
        ));
    }
    fn sign_out(&mut self, client: ClientId, _now: Time, _wall: Wall) {
        self.callbacks.push((client, Event::Client { event: jig_client::Event::SignOut }));
    }
}
