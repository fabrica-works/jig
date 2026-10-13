use crate::{fixtures, net::Net};
use jig_protocol_worker_link::{Component, Event, Limits, MAX_OUT_DOWN, Request};
use skein_fake_channel::ScriptedPeer;
use skein_lib::{Duration, Queue, Rng, Time, Token};
use std::collections::VecDeque;

pub struct World {
    component: Component,
    limits: Limits,
    now: Time,
    out: Queue<Request>,
    pending: VecDeque<Option<ScriptedPeer>>,
    net: Option<Net>,
    connection: Token,
    next: u64,
    dials: Vec<Time>,
    root: Vec<Request>,
    facts: Vec<skein_channel::Closed>,
    rng: Rng,
    silence_next: bool,
    other_version_next: bool,
    refused: Vec<(u16, Box<[u8]>)>,
}
impl World {
    #[must_use]
    pub fn new(limits: &Limits, seed: u64) -> Self {
        Self {
            component: Component::new(limits, fixtures::credential(), seed).expect("world limits"),
            limits: *limits,
            now: Time::ZERO,
            out: Queue::with_capacity(MAX_OUT_DOWN),
            pending: VecDeque::new(),
            net: None,
            connection: Token::new(0),
            next: 1,
            dials: Vec::new(),
            root: Vec::new(),
            facts: Vec::new(),
            rng: Rng::new(seed),
            silence_next: false,
            other_version_next: false,
            refused: Vec::new(),
        }
    }
    pub fn engine(&mut self, peer: ScriptedPeer) {
        self.pending.push_back(Some(peer));
    }
    pub fn refuse_dial(&mut self) {
        self.pending.push_back(None);
    }
    pub fn silence_next_engine(&mut self) {
        self.silence_next = true;
    }
    pub fn change_next_engine_version(&mut self) {
        self.other_version_next = true;
    }
    pub fn down(&mut self, event: Event) {
        self.component.down(self.now, event, &mut self.out);
        self.run();
    }
    pub fn lose_channel(&mut self) {
        self.net = None;
        self.down(Event::Gone { connection: self.connection });
    }
    pub fn advance(&mut self, span: Duration) {
        let end = self.now.saturating_add(span);
        while let Some(at) = self.component.next_deadline() {
            if at > end {
                break;
            }
            self.now = self.now.max(at);
            self.component.fire(self.now, &mut self.out);
            self.run();
        }
        self.now = end;
    }
    pub fn run(&mut self) {
        for _ in 0..10000 {
            if let Some(at) = self.component.next_deadline()
                && at <= self.now
            {
                self.component.fire(self.now, &mut self.out);
            }
            if let Some(request) = self.out.pop() {
                self.route(request);
                continue;
            }
            if let Some(net) = &mut self.net
                && let Some(event) = net.tick(&mut self.rng)
            {
                self.component.down(self.now, Event::Lower { connection: self.connection, event }, &mut self.out);
                continue;
            }
            return;
        }
        panic!("a world iteration settles within its bound");
    }
    fn route(&mut self, request: Request) {
        match request {
            Request::Dial => {
                self.dials.push(self.now);
                if let Some(peer) = self.pending.pop_front() {
                    if let Some(peer) = peer {
                        let mut net = Net::new(peer, self.limits.chunk);
                        net.read_room = !std::mem::take(&mut self.silence_next);
                        net.other_version = std::mem::take(&mut self.other_version_next);
                        self.net = Some(net);
                        self.connection = Token::new(self.next);
                        self.next += 1;
                        self.component.down(self.now, Event::Dialled { connection: self.connection }, &mut self.out);
                    } else {
                        self.component.down(self.now, Event::Unreachable, &mut self.out);
                    }
                }
            }
            Request::Lower { connection, lower } => {
                assert_eq!(connection, self.connection);
                if let Some(net) = &mut self.net {
                    net.lower(lower, &mut self.rng);
                }
            }
            Request::Close { connection } | Request::Abort { connection } => {
                assert_eq!(connection, self.connection);
                if let Some(net) = self.net.take()
                    && let Some(refused) = net.refused
                {
                    self.refused.push(refused);
                }
                self.component.down(self.now, Event::Gone { connection }, &mut self.out);
            }
            Request::Fact { why } => self.facts.push(why),
        }
    }
    #[must_use]
    pub fn dials(&self) -> &[Time] {
        &self.dials
    }
    #[must_use]
    pub fn root(&self) -> &[Request] {
        &self.root
    }
    #[must_use]
    pub fn refusals(&self) -> &[(u16, Box<[u8]>)] {
        &self.refused
    }
    #[must_use]
    pub fn facts(&self) -> &[skein_channel::Closed] {
        &self.facts
    }
    #[must_use]
    pub fn credential(&self) -> &[u8] {
        self.net.as_ref().expect("channel open").peer.credential()
    }
}
