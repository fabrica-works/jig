//! A scripted parent owns subjects and makes documents whenever asked.
use crate::referee::Referee;
use jig_core_views::{self as views, Domain, Event, Limits, Made, Request, Window};
use skein_lib::{Env, List, Queue, Time, Token, Wall};
pub const LIMITS: Limits =
    Limits { runs: 3, watchers: 4, watchers_per_party: 4, subjects_per_change: 4, text_bytes: 32, facts: 8 };
#[derive(Debug)]
pub struct World {
    pub domain: Domain,
    pub referee: Referee,
    pub now: Time,
    pub limits: Limits,
    pub log: Vec<String>,
}
impl Default for World {
    fn default() -> Self {
        Self::new(LIMITS)
    }
}
impl World {
    #[must_use]
    pub fn new(limits: Limits) -> Self {
        Self { domain: Domain::new(&limits), referee: Referee::default(), now: Time::ZERO, limits, log: Vec::new() }
    }
    pub fn step(&mut self, event: Event) -> Vec<Request> {
        self.referee.before(&event);
        self.log.push(format!("in {event:?}"));
        let env = Env { now: self.now, wall: Wall::EPOCH, limits: self.limits };
        let mut out = Queue::with_capacity(views::max_out(&self.limits));
        views::step(&mut self.domain, &env, event, &mut out);
        let mut requests = Vec::new();
        while let Some(request) = out.pop() {
            requests.push(request);
        }
        self.referee.saw(&requests);
        let mut made = Vec::new();
        for request in &requests {
            if let Request::Make { watcher, .. } = request {
                views::step(&mut self.domain, &env, Event::Made { watcher: *watcher, outcome: Made::Ready }, &mut out);
                while let Some(request) = out.pop() {
                    made.push(request);
                }
            }
        }
        self.referee.saw(&made);
        requests.extend(made);
        for request in &requests {
            self.log.push(format!("out {request:?}"));
        }
        requests
    }
    pub fn watch(&mut self, watcher: u64, subject: u64) -> Vec<Request> {
        self.step(Event::Watch {
            watcher: Token::new(watcher),
            party: Token::new(1),
            subject: Token::new(subject),
            window: Window { rows: 10, after: None },
            follow: None,
        })
    }
    pub fn changed(&mut self, subject: u64) -> Vec<Request> {
        let mut subjects = List::with_capacity(1);
        subjects.push(Token::new(subject)).expect("one subject");
        self.step(Event::Changed { subjects })
    }
    pub fn delivered(&mut self, watcher: u64) -> Vec<Request> {
        if self.referee.in_flight(Token::new(watcher)) {
            self.step(Event::Delivered { watcher: Token::new(watcher) })
        } else {
            Vec::new()
        }
    }
    pub fn reclaim(&mut self) {
        self.domain.reclaim();
    }
}
