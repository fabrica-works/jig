//! One engine watch per subject, held through its closing terminal.
use crate::{
    Domain, Latest, Limits, Request, State, Told, Up, Watchable, Why,
    contract::{WatchEnd, WatchEvent},
    domain::tell,
};
use skein_lib::{Duration, Env, Queue, Time, Token};
#[derive(Debug)]
pub(crate) struct Watch {
    pub subject: Watchable,
    pub name: Option<Token>,
    pub latest: Option<Latest>,
    pub state: State,
    pub closing: bool,
    pub own: bool,
    pub until: Option<Time>,
    pub span: Option<Duration>,
}
impl Domain {
    pub(crate) fn up(&mut self, env: &Env<Limits>, up: Up, out: &mut Queue<Request>) {
        let name = match &up {
            Up::Watched { watch, .. } | Up::FamilyDelivered { watch, .. } | Up::WatchEnded { watch, .. } => *watch,
        };
        let mut found = None;
        for (id, w) in &self.watches {
            if w.name == Some(name) {
                found = Some(*id);
            }
        }
        let Some(id) = found else {
            return;
        };
        match up {
            Up::Watched { event, .. } => match event {
                WatchEvent::Delivered(value) => self.deliver(id, Latest::Jig(value), out),
                WatchEvent::Opened | WatchEvent::Alive => {}
            },
            Up::FamilyDelivered { record, .. } => self.deliver(id, Latest::Family(record), out),
            Up::WatchEnded { end, .. } => self.ended(env, id, end, out),
        }
    }
    fn select_handles(&mut self, id: u64) {
        self.scratch.clear();
        for (handle, held) in &self.handles {
            if held.watch == Some(id) {
                self.scratch.push(*handle).expect("bounded handles");
            }
        }
    }
    fn deliver(&mut self, id: u64, latest: Latest, out: &mut Queue<Request>) {
        let w = self.watches.get_mut(&id).expect("delivery watch");
        if w.closing {
            return;
        }
        w.latest = Some(latest);
        w.state = State::Live;
        w.span = None;
        self.select_handles(id);
        for index in 0..self.scratch.len() {
            let handle = *self.scratch.get(index).expect("selected handle");
            let held = self.handles.get_mut(&handle).expect("selected handle exists");
            held.state = State::Live;
            held.previous = None;
            tell(out, Told::Changed { handle });
        }
        self.scratch.clear();
    }
    fn ended(&mut self, env: &Env<Limits>, id: u64, end: WatchEnd, out: &mut Queue<Request>) {
        let w = self.watches.get_mut(&id).expect("terminal watch");
        w.name = None;
        if w.closing {
            self.watches.remove(&id);
            return;
        }
        let why = match end {
            WatchEnd::Dropped | WatchEnd::Stopping => None,
            WatchEnd::Closed | WatchEnd::SubjectEnded => Some(Why::SubjectEnded),
            WatchEnd::Unknown => Some(Why::Unknown),
            WatchEnd::NotAllowed => Some(Why::NotAllowed),
            WatchEnd::Busy => Some(Why::Busy),
            WatchEnd::SignedOut => Some(Why::SignedOut),
        };
        match why {
            None => {
                let span = match w.span {
                    Some(s) => core::cmp::min(s.saturating_mul(2), env.limits.backoff.most),
                    None => env.limits.backoff.first,
                };
                w.span = Some(span);
                w.state = State::Behind;
                let nanos = span.as_nanos();
                w.until =
                    Some(env.now.saturating_add(Duration::from_nanos(self.rng.between(nanos.div_euclid(2), nanos))));
            }
            Some(_) => {
                w.state = State::Ended;
            }
        }
        self.select_handles(id);
        for index in 0..self.scratch.len() {
            let handle = *self.scratch.get(index).expect("selected handle");
            let held = self.handles.get_mut(&handle).expect("selected handle exists");
            match why {
                None => {
                    held.state = State::Behind;
                    tell(out, Told::Changed { handle });
                }
                Some(why) => {
                    held.state = State::Ended;
                    held.watch = None;
                    held.previous =
                        self.watches.get(&id).expect("ending watch").latest.clone().or(held.previous.take());
                    tell(out, Told::Ended { handle, why });
                }
            }
        }
        self.scratch.clear();
        if why.is_some() {
            self.watches.remove(&id);
        }
    }
}
