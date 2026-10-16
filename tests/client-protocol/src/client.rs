//! The application-held client against the real engine over native HTTP bytes.
use crate::{engine::Engine, fixtures, net::Net};
use jig_client::{self as client, Ask, Event, Handle, Key, Latest, Link, Outcome, Readable, Told, Watchable};
use jig_client_protocol_link as link;
use jig_client_shell_native::{Conn, Connections};
use jig_client_world::{ClientId, harness::Observed};
use skein_lib::{Duration, List, Queue, Time, stream::Down};
use std::collections::BTreeMap;
const CLIENT: ClientId = ClientId(0);
/// Checks revisions from the documents the application was told changed.
#[derive(Default)]
pub struct Referee {
    revisions: Vec<(Watchable, u64)>,
}
impl Referee {
    /// # Errors
    /// A subject was told with a revision older than one already delivered.
    pub fn document(&mut self, subject: Watchable, revision: u64) -> Result<(), &'static str> {
        if let Some((_, previous)) = self.revisions.iter_mut().find(|(held, _)| *held == subject) {
            if revision < *previous {
                return Err("document revision moved backwards");
            }
            *previous = revision;
        } else {
            self.revisions.push((subject, revision));
        }
        Ok(())
    }
}
/// A scripted application, one client and its host's small store.
pub struct Story {
    pub client: client::Client,
    connections: Connections,
    engine: Engine,
    net: Net,
    now: Time,
    out: Queue<client::Request>,
    up: Queue<link::Event>,
    below: Queue<(Conn, Down)>,
    told: Vec<Told>,
    store: BTreeMap<Box<[u8]>, Box<[u8]>>,
    subjects: BTreeMap<Handle, Watchable>,
    seen: Vec<Observed>,
    referee: Referee,
}
impl Story {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let limits = client::Limits { domain: fixtures::domain_limits(), exchanges: 8 };
        let mut story = Self {
            client: client::Client::new(&limits, seed).expect("bounded client"),
            connections: Connections::new(&fixtures::connections_limits()),
            engine: Engine::bounded(seed, &fixtures::owner_limits()),
            net: Net::new(seed),
            now: Time::ZERO,
            out: Queue::with_capacity(client::max_out(&limits)),
            up: Queue::with_capacity(256),
            below: Queue::with_capacity(256),
            told: Vec::new(),
            store: BTreeMap::new(),
            subjects: BTreeMap::new(),
            seen: Vec::new(),
            referee: Referee::default(),
        };
        story.event(Event::Start { kept: List::with_capacity(0) });
        story.advance();
        story
    }
    #[must_use]
    pub fn live(seed: u64) -> Self {
        let mut story = Self::new(seed);
        story.sign_in();
        assert!(story.run_until(|s| s.client.link().state == Link::Live, Duration::from_secs(1)));
        story
    }
    pub fn sign_in(&mut self) {
        let secret = self.connections.printed(&self.engine.link).expect("engine's printed link");
        self.event(Event::SignIn { with: client::With::Link(secret) });
    }
    pub fn watch(&mut self, handle: Handle, subject: Watchable) {
        self.subjects.insert(handle, subject.clone());
        self.event(Event::Watch { handle, subject });
    }
    pub fn read(&mut self, handle: Handle, read: Readable) {
        self.event(Event::Read { handle, read });
    }
    pub fn send(&mut self, key: Key, request: Ask) {
        self.event(Event::Send { key, request });
    }
    pub fn event(&mut self, event: Event) {
        self.client.step(self.now, fixtures::wall(self.now), event, &mut self.out);
    }
    pub fn cut_answer(&mut self) {
        self.net.cut_answer();
    }
    pub fn cut_stream(&mut self) {
        self.net.cut_stream(CLIENT);
    }
    #[must_use]
    pub fn told_links(&self) -> Vec<Link> {
        self.told.iter().filter_map(|t| if let Told::Link { link } = t { Some(*link) } else { None }).collect()
    }
    #[must_use]
    pub fn answered(&self, key: Key) -> Vec<Outcome> {
        self.told
            .iter()
            .filter_map(|t| {
                if let Told::Answered { key: held, outcome } = t {
                    (*held == key).then(|| outcome.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    #[must_use]
    pub fn pages(&self, handle: Handle) -> Vec<client::Readout> {
        self.told
            .iter()
            .filter_map(|t| {
                if let Told::Page { handle: held, page } = t { (*held == handle).then(|| page.clone()) } else { None }
            })
            .collect()
    }
    #[must_use]
    pub fn made(&self, party: u64) -> usize {
        self.seen.iter().filter(|o| matches!(o, Observed::Made { party: held, .. } if *held == party)).count()
    }
    #[must_use]
    pub fn deliveries_taken(&self) -> bool {
        !self.engine.offered.is_empty() && self.engine.offered == self.engine.taken
    }
    pub fn run_until(&mut self, done: impl Fn(&Self) -> bool, within: Duration) -> bool {
        let end = self.now.saturating_add(within);
        for _ in 0..100_000 {
            self.advance();
            if done(self) {
                return true;
            }
            if self.now >= end {
                return false;
            }
            let next = self
                .client
                .next_deadline()
                .into_iter()
                .chain(self.connections.next_deadline())
                .chain(self.engine.component.next_deadline())
                .chain(self.engine.driver.world.next_due())
                .chain(self.net.next_due())
                .filter(|t| *t > self.now)
                .min()
                .unwrap_or(end);
            self.now = next.min(end);
            self.client.fire(self.now, fixtures::wall(self.now), &mut self.out);
        }
        panic!("story did not settle")
    }
    fn advance(&mut self) {
        self.engine.clock(self.now, fixtures::wall(self.now));
        self.net.clock(self.now);
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
            while let Some(event) = self.up.pop() {
                active = true;
                self.event(transport(event));
            }
            while let Some(request) = self.out.pop() {
                active = true;
                match request {
                    client::Request::Told { told } => {
                        if let Told::Changed { handle } = &told
                            && let Some(subject) = self.subjects.get(handle)
                            && let Some(document) = self.client.document(*handle)
                            && let Some(Latest::Jig(client::contract::Delivery::Task(task))) = document.latest
                        {
                            self.referee.document(subject.clone(), task.spec_revision.value).expect("ordered document");
                        }
                        self.told.push(told);
                    }
                    client::Request::Save { records } => {
                        for record in records.into_boxed() {
                            self.store.insert(record.key, record.bytes);
                        }
                        self.event(Event::Stored);
                    }
                    client::Request::Erase { records } => {
                        for key in records.into_boxed() {
                            self.store.remove(&key);
                        }
                        self.event(Event::Stored);
                    }
                    client::Request::Exchange { exchange, credential, message } => self.connections.down(
                        self.now,
                        link::Request::Exchange { exchange, credential, message },
                        &mut self.below,
                    ),
                    client::Request::OpenStream { credential } => {
                        self.connections.down(self.now, link::Request::OpenStream { credential }, &mut self.below);
                    }
                    client::Request::CloseStream => {
                        self.connections.down(self.now, link::Request::CloseStream, &mut self.below);
                    }
                }
            }
            self.connections.take(self.now, &mut self.up, &mut self.below);
            while let Some((conn, down)) = self.below.pop() {
                active = true;
                if let Some(event) = self.net.down(CLIENT, conn, down) {
                    self.engine.event(jig_protocol_engine_clients::Event::Io(event));
                }
            }
            let (events, clients, changed) = self.net.advance(&mut self.engine.rules);
            active |= changed;
            for event in events {
                self.engine.event(jig_protocol_engine_clients::Event::Io(event));
            }
            for (id, conn, up) in clients {
                assert_eq!(id, CLIENT);
                self.connections.up(self.now, conn, up, &mut self.up, &mut self.below);
            }
            self.seen.extend(self.engine.observed());
            quiet = if active { 0 } else { quiet + 1 };
            if quiet == 32 {
                return;
            }
        }
        panic!("client HTTP bytes did not quiesce")
    }
}
fn transport(event: link::Event) -> Event {
    match event {
        link::Event::Reply { exchange, message } => Event::Reply { exchange, message },
        link::Event::Lost { exchange } => Event::Lost { exchange },
        link::Event::Streamed { message } => Event::Streamed { message },
        link::Event::StreamGone => Event::StreamGone,
        link::Event::SignIn { .. }
        | link::Event::SignOut
        | link::Event::Start { .. }
        | link::Event::Save { .. }
        | link::Event::Erase { .. }
        | link::Event::Stored
        | link::Event::NotStored
        | link::Event::Send { .. }
        | link::Event::Read { .. }
        | link::Event::Open { .. }
        | link::Event::Continue { .. }
        | link::Event::Close { .. } => panic!("connections only tell transport events"),
    }
}
