//! Volatile watch pacing state (`domain/engine.md`, 11).
//! No document, task, project or history is kept or interpreted here.
use crate::facts::Facts;
use crate::watch::{self, Watcher};
use crate::{Event, Fact, Limits, Request};
use skein_lib::{Env, Id, List, Map, Queue, Slab, Time, Token};

/// A bounded set of watches, discarded on restart.
#[derive(Debug)]
pub struct Domain {
    pub(crate) watchers: Slab<Watcher>,
    pub(crate) runs: Map<Token, crate::text::Run>,
    pub(crate) names: Map<Token, Id<Watcher>>,
    pub(crate) facts: Facts,
}
impl Domain {
    /// Construct empty volatile pacing state.
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self {
            runs: Map::with_capacity(limits.runs),
            watchers: Slab::with_capacity(crate::limits::slots(limits).expect("valid watch capacities")),
            names: Map::with_capacity(limits.watchers),
            facts: Facts::new(limits.facts),
        }
    }
    /// Open watches, including retired slots awaiting reclaim.
    #[must_use]
    pub fn watchers(&self) -> u32 {
        self.watchers.len()
    }
    /// Live runs whose current writing is retained, watched or not.
    #[must_use]
    pub fn runs(&self) -> u32 {
        self.runs.len()
    }
    /// Current retained live run names, bounded by `limits.runs`, for the
    /// parent's proof fencing even when none of these runs is watched.
    #[must_use]
    pub fn retained_runs(&self) -> List<Token> {
        let mut tokens = List::with_capacity(self.runs.len());
        for (token, _) in &self.runs {
            tokens.push(*token).expect("retained run count");
        }
        tokens
    }
    /// Whether an opaque run has any admitted follower.
    #[must_use]
    pub fn follows(&self, run: Token) -> bool {
        for (_, id) in &self.names {
            if self.watchers.get(*id).expect("named watch").follow.is_some_and(|follow| follow.run == run) {
                return true;
            }
        }
        false
    }
    /// Watches have no timers.
    #[must_use]
    pub const fn next_deadline(&self) -> Option<Time> {
        None
    }
    /// Watches have no timers.
    #[must_use]
    pub const fn is_due(&self, _now: Time) -> bool {
        false
    }
    /// Drain the oldest expendable observation.
    pub fn pop_fact(&mut self) -> Option<Fact> {
        self.facts.pop()
    }
    /// Observations dropped since construction.
    #[must_use]
    pub fn facts_lost(&self) -> u64 {
        self.facts.lost()
    }
    /// Reclaim only watches whose terminal was emitted.
    pub fn reclaim(&mut self) {
        self.watchers.reclaim();
    }
}
/// One request per watcher, plus admission's two outputs.
#[must_use]
pub const fn max_out(limits: &Limits) -> u32 {
    limits.watchers.saturating_add(2)
}
/// Apply one parent event with `max_out` room reserved.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::EndWatch { watcher, end } => watch::close(domain, watcher, end, out),
        Event::Watch { watcher, party, subject, window, follow } => {
            watch::open(domain, &env.limits, watcher, party, subject, window, follow, out);
        }
        Event::Changed { subjects } => watch::changed(domain, &env.limits, &subjects, out),
        Event::Made { watcher, outcome } => watch::made(domain, watcher, outcome, out),
        Event::Delivered { watcher } => watch::delivered(domain, watcher, out),
        Event::StreamDelivered { watcher } => crate::text::delivered(domain, watcher, out),
        Event::Text { run, turn, offset, text, call } => {
            crate::text::written(domain, env, run, turn, offset, &text, call, out);
        }
        Event::Called { run, call } => crate::text::called(domain, env, run, call, out),
        Event::TurnCommitted { run, attempt, turn } => crate::text::committed(domain, run, attempt, turn),
        Event::RunEnded { run } => crate::text::ended(domain, run),
        Event::Unwatch { watcher } => watch::close(domain, watcher, crate::End::Unwatched, out),
        Event::Gone { subject, why } => watch::gone(domain, subject, why, out),
    }
}
/// There are no deadlines to fire.
pub fn fire(_domain: &mut Domain, _env: &Env<Limits>, _out: &mut Queue<Request>) {}
