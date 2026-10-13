//! The layer's scripted routing around the component.
use crate::net::Net;
use jig_protocol_engine_workers::{Component, Event, Limits, MAX_OUT_DOWN, Request, Worker};
use skein_fake_channel::ScriptedPeer;
use skein_lib::{Duration, Queue, Rng, Time, Token};
use std::collections::BTreeMap;
pub struct World {
    component: Component,
    nets: BTreeMap<Token, Net>,
    out: Queue<Request>,
    root: Vec<Request>,
    now: Time,
    rng: Rng,
    next: u64,
    chunk: u32,
}
impl World {
    #[must_use]
    pub fn new(limits: &Limits, workers: &[Worker], seed: u64) -> Self {
        Self {
            component: Component::new(limits, workers.to_vec().into_boxed_slice(), crate::fixtures::PAYLOAD)
                .expect("valid engine"),
            nets: BTreeMap::new(),
            out: Queue::with_capacity(MAX_OUT_DOWN),
            root: Vec::new(),
            now: Time::ZERO,
            rng: Rng::new(seed),
            next: 100,
            chunk: limits.chunk,
        }
    }
    pub fn connect(&mut self, peer: ScriptedPeer) -> Token {
        let connection = Token::new(self.next);
        self.next += 1;
        assert!(self.nets.insert(connection, Net::new(peer, self.chunk)).is_none());
        self.down(Event::Accepted { connection });
        connection
    }
    pub fn down(&mut self, event: Event) {
        self.component.down(self.now, event, &mut self.out);
        self.route();
    }
    fn route(&mut self) {
        while let Some(request) = self.out.pop() {
            match request {
                Request::Take { connection } => self.net(connection).taken = true,
                Request::TurnAway { connection } => self.net(connection).gone = true,
                Request::Lower { connection, lower } => {
                    self.nets.get_mut(&connection).expect("a known stream").lower(lower, &mut self.rng);
                }
                Request::Close { connection } | Request::Abort { connection } => {
                    self.net(connection).gone = true;
                    self.component.down(self.now, Event::Gone { connection }, &mut self.out);
                }
                Request::Hello { .. } | Request::Up { .. } | Request::Lost { .. } | Request::Closed => {
                    self.root.push(request);
                }
            }
        }
    }
    pub fn run(&mut self) {
        for _ in 0..10_000 {
            let mut event = None;
            for (connection, net) in &mut self.nets {
                if let Some(lower) = net.tick(&mut self.rng) {
                    event = Some(Event::Lower { connection: *connection, event: lower });
                    break;
                }
            }
            if let Some(event) = event {
                self.down(event);
            } else {
                self.component.reclaim();
                return;
            }
        }
        panic!("world must settle");
    }
    pub fn advance(&mut self, duration: Duration) {
        self.now = self.now.saturating_add(duration);
        while self.component.next_deadline().is_some_and(|at| at <= self.now) {
            self.component.fire(self.now, &mut self.out);
            self.route();
        }
        self.run();
    }
    #[must_use]
    pub fn refusal(&self, connection: Token) -> Option<(u16, Box<[u8]>)> {
        self.nets.get(&connection)?.refused.clone()
    }
    #[must_use]
    pub fn root(&self) -> &[Request] {
        &self.root
    }
    #[must_use]
    pub fn ready(&self, connection: Token) -> bool {
        self.nets.get(&connection).expect("known stream").peer.is_ready()
    }
    #[must_use]
    pub fn gone(&self, connection: Token) -> bool {
        self.nets.get(&connection).expect("known stream").gone
    }
    pub fn withhold_write_room(&mut self, connection: Token) {
        self.net(connection).write_room = false;
    }
    fn net(&mut self, connection: Token) -> &mut Net {
        self.nets.get_mut(&connection).expect("known stream")
    }
}
