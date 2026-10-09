//! A scripted document owner and watchers on skein's world harness.
//! Subjects are plain counters. Document sizes and watcher pacing are seeded;
//! the referee sees only owner inputs and watcher outputs (`domain/engine.md`, 11).
use crate::referee::{Document, Referee};
use jig_core_views::{self as views, Call, Domain, End, Event, Follow, Limits, Made, Request, TurnName, Window};
use skein_lib::{Env, List, Queue, Rng, Time, Token, Wall};
use skein_world::domain::Trace;
use std::collections::BTreeMap;
pub const LIMITS: Limits =
    Limits { runs: 3, watchers: 4, watchers_per_party: 4, subjects_per_change: 4, text_bytes: 32, facts: 8 };
#[derive(Debug)]
pub struct World {
    domain: Domain,
    pub referee: Referee,
    pub now: Time,
    pub limits: Limits,
    subjects: BTreeMap<Token, u64>,
    source: BTreeMap<(Token, TurnName), u32>,
    trace: Trace,
    rng: Rng,
}
impl Default for World {
    fn default() -> Self {
        Self::new(LIMITS)
    }
}
impl World {
    #[must_use]
    pub fn new(limits: Limits) -> Self {
        Self::seeded(limits, 1)
    }
    #[must_use]
    pub fn seeded(limits: Limits, seed: u64) -> Self {
        Self {
            domain: Domain::new(&limits),
            referee: Referee::new(limits),
            now: Time::ZERO,
            limits,
            subjects: BTreeMap::new(),
            source: BTreeMap::new(),
            trace: Trace::default(),
            rng: Rng::new(seed),
        }
    }
    pub fn step(&mut self, event: Event) -> Vec<Request> {
        let mut requests = self.step_once(event);
        let mut made = Vec::new();
        for request in &requests {
            if let Request::Make { watcher, subject, window } = request {
                let outcome = if let Some(generation) = self.subjects.get(subject) {
                    let size = if window.rows == 0 { 0 } else { self.rng.below(u64::from(window.rows)) + 1 };
                    let document = Document {
                        generation: *generation,
                        bytes: vec![
                            u8::try_from(generation % 256).expect("byte counter");
                            usize::try_from(size).expect("window size")
                        ],
                    };
                    self.trace.log(self.now, format!("owner {watcher:?} {document:?}"));
                    self.referee.made(*watcher, document);
                    Made::Ready
                } else {
                    Made::Gone { why: End::Gone }
                };
                made.extend(self.step_once(Event::Made { watcher: *watcher, outcome }));
            }
        }
        requests.extend(made);
        requests
    }
    fn step_once(&mut self, event: Event) -> Vec<Request> {
        self.referee.before(&event);
        self.trace.log(self.now, format!("in {event:?}"));
        let env = Env { now: self.now, wall: Wall::EPOCH, limits: self.limits };
        let mut out = Queue::with_capacity(views::max_out(&self.limits));
        views::step(&mut self.domain, &env, event, &mut out);
        let mut requests = Vec::new();
        while let Some(request) = out.pop() {
            self.trace.log(self.now, format!("out {request:?}"));
            requests.push(request);
        }
        self.referee.saw(&requests);
        requests
    }
    pub fn watch(&mut self, watcher: u64, subject: u64) -> Vec<Request> {
        self.watch_from(watcher, subject, None)
    }
    pub fn watch_from(&mut self, watcher: u64, subject: u64, follow: Option<Follow>) -> Vec<Request> {
        self.subjects.entry(Token::new(subject)).or_default();
        self.step(Event::Watch {
            watcher: Token::new(watcher),
            party: Token::new(1),
            subject: Token::new(subject),
            window: Window { rows: 10, after: None },
            follow,
        })
    }
    pub fn changed(&mut self, subject: u64) -> Vec<Request> {
        let token = Token::new(subject);
        let generation = self.subjects.entry(token).or_default();
        *generation += 1;
        self.referee.changed(token, *generation);
        let mut subjects = List::with_capacity(1);
        subjects.push(token).expect("one subject");
        self.step(Event::Changed { subjects })
    }
    pub fn delivered(&mut self, watcher: u64) -> Vec<Request> {
        if self.referee.in_flight(Token::new(watcher)) {
            self.step(Event::Delivered { watcher: Token::new(watcher) })
        } else {
            Vec::new()
        }
    }
    pub fn stream_delivered(&mut self, watcher: u64) -> Vec<Request> {
        if self.referee.stream_in_flight(Token::new(watcher)) {
            self.step(Event::StreamDelivered { watcher: Token::new(watcher) })
        } else {
            Vec::new()
        }
    }
    pub fn text(&mut self, run: u64, turn: TurnName, text: &[u8], call: Option<Call>) -> Vec<Request> {
        let offset = self.source.entry((Token::new(run), turn)).or_default();
        let from = *offset;
        *offset += u32::try_from(text.len()).expect("bounded script");
        self.step(Event::Text { run: Token::new(run), turn, offset: from, text: text.into(), call })
    }
    pub fn settle(&mut self) {
        for _ in 0..3 {
            for watcher in self.referee.open_names() {
                self.delivered(watcher.raw());
                self.stream_delivered(watcher.raw());
            }
        }
        self.referee.assert_caught_up();
    }
    pub fn restart(&mut self) -> Vec<(Token, End)> {
        let prior_ends = self.referee.ended().len();
        for subject in self.referee.open_subjects() {
            self.step(Event::Gone { subject, why: End::Stopping });
        }
        self.settle();
        assert_eq!(self.referee.live(), 0, "the parent was told every watch ended");
        let ended = self.referee.ended().get(prior_ends..).expect("restart endings").to_vec();
        self.domain = Domain::new(&self.limits);
        self.subjects.clear();
        self.source.clear();
        // The parent keeps its old observations; subjects are new counters.
        self.referee = Referee::new(self.limits);
        ended
    }
    pub fn reclaim(&mut self) {
        self.domain.reclaim();
    }
    #[must_use]
    pub fn log(&self) -> &[String] {
        self.trace.lines()
    }
}
