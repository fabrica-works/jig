//! An engine at the client's typed protocol boundary, without an engine domain.
//! Keeps parties, sign-ins, documents, keyed decisions and numbered commits;
//! knows no platform, application types or transport encoding.
//! See client/domain/testing.md, 4 and 6, and contract.md, 3–5.
//!
//! | State | Input | Next | Emits |
//! |---|---|---|---|
//! | unseen key | send | decided, awaiting durability | nothing |
//! | awaiting durability | poll at due time | durable, retained | final answer |
//! | retained key | same ask | unchanged | the same final answer |
//! | retained key | different ask | unchanged | key conflict |
//! The story supplies wall times separately from the scheduling clock.
#![forbid(unsafe_code)]
mod admission;
mod application;
mod faults;
pub mod offered;
pub mod owner_wake;
mod payload;
pub use admission::{admit_activity_line, admit_answer, admit_delivery, admit_frame, admit_read_value};
pub mod policy_edit;
pub use application::{
    Application, ApplicationCommit, ApplicationScope, ApplicationValue, Decision, Never, NoApplication,
};
pub use faults::Fault;
mod reads;
mod requests;
mod truth;
mod watches;
use jig_client::contract::{Answer, Ask, ReadResult, WatchEnd, WatchEvent};
pub use reads::{Rows, Source};
use skein_lib::{Duration, Time, Token, Wall};
use std::collections::{BTreeMap, BTreeSet};
pub use truth::{Change, Rule, Script, Truth};

/// Payload and scheduling bounds of the scripted contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub text: u32,
    pub rows: u32,
    pub document_bytes: u64,
    pub commit_delay: Duration,
    pub heartbeat: Duration,
    pub watches: u32,
    pub watches_per_party: u32,
}
impl Default for Bounds {
    fn default() -> Self {
        Self {
            text: 4096,
            rows: 32,
            document_bytes: 65536,
            commit_delay: Duration::from_millis(10),
            heartbeat: Duration::from_secs(1),
            watches: 32,
            watches_per_party: 16,
        }
    }
}

/// Why a scripted typed value cannot enter the protocol boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    /// Nested bytes, text or list capacity exceeded the configured payload bound.
    Payload,
    /// A document is filed under a different object identity.
    Identity,
    /// A scripted outcome contradicts its offered admission or changes truth on a refusal.
    Script,
    /// Story work was attempted while the restarted engine is offline.
    Offline,
}

/// The envelope a world's root passes to its request table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Envelope {
    /// A durable final answer accompanies the envelope.
    Final,
    /// Nothing was made; retry under the same key.
    Again,
    /// The connection lost its answer; the key remains safe to retry.
    Lost,
    /// The sign-in ended; park requests until the party signs in again.
    SignedOut,
}

/// One event delivered to the world's client root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Up<A: Application = NoApplication> {
    /// One keyed attempt ended, with a final answer only for a final envelope.
    Answer {
        attempt: Token,
        envelope: Envelope,
        answer: Option<Answer>,
    },
    Read {
        token: Token,
        terminal: ReadResult,
    },
    Watch {
        token: Token,
        event: WatchEvent,
    },
    ApplicationAnswer {
        attempt: Token,
        envelope: Envelope,
        answer: Option<Result<A::Answer, jig_client::contract::Refusal>>,
    },
    ApplicationRead {
        token: Token,
        terminal: Result<A::Document, jig_client::contract::ReadNone>,
    },
    App {
        watch: Token,
        document: A::Document,
    },
    Ended {
        token: Token,
        why: WatchEnd,
    },
}

/// One numbered decision, including its payload and durability time for a referee.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Committed<A: Application = NoApplication> {
    pub number: u64,
    pub made: Time,
    pub at: Wall,
    pub durable: Option<Time>,
    pub party: Option<u64>,
    pub key: Option<[u8; 16]>,
    pub ask: Option<Ask>,
    pub answer: Option<Answer>,
    pub changes: Vec<Change>,
    pub application: Option<ApplicationCommit<A>>,
}
#[derive(Debug)]
struct Pending<A: Application> {
    commit: Committed<A>,
    due: Time,
    truth: Truth,
    application: A,
}
#[derive(Debug)]
struct Keyed<A: Application> {
    ask: RequestIdentity,
    answer: Retained<A>,
    commit: u64,
    retained_until: Option<Time>,
}
#[derive(Debug, PartialEq, Eq)]
enum RequestIdentity {
    Jig(Box<Ask>),
    Application { owner: u16, project: u32, digest: [u8; 32] },
}
#[derive(Debug, Clone)]
enum Retained<A: Application> {
    Jig(Answer),
    Application(Result<A::Answer, jig_client::contract::Refusal>),
}
#[derive(Debug)]
struct Due<A: Application> {
    at: Time,
    commit: u64,
    up: Up<A>,
}

/// A deterministic scripted engine; seed is retained for reproducible extensions.
#[derive(Debug)]
pub struct Engine<A: Application = NoApplication> {
    bounds: Bounds,
    versions: jig_client::contract::VersionRange,
    truth: Truth,
    durable_truth: Truth,
    rules: Vec<Rule>,
    keys: BTreeMap<(u64, [u8; 16]), Keyed<A>>,
    pending: Vec<Pending<A>>,
    commits: Vec<Committed<A>>,
    next_commit: u64,
    due: Vec<Due<A>>,
    seed: u64,
    read_tokens: BTreeSet<Token>,
    positions: BTreeMap<u64, reads::Position>,
    next_position: u64,
    watches: BTreeMap<Token, watches::Watch>,
    watch_tokens: BTreeSet<Token>,
    runs: BTreeMap<u64, watches::Run>,
    application: A,
    durable_application: A,
    application_watches: BTreeMap<Token, application::Open<A>>,
    attempts: BTreeMap<Token, u64>,
    reads: BTreeMap<Token, u64>,
    offline: Option<Time>,
    slow: Duration,
    request_fault: Option<Envelope>,
    clock: Time,
}
impl Engine<NoApplication> {
    /// Constructs the jig-only engine.
    pub fn new(script: Script, seed: u64) -> Result<Self, Invalid> {
        Self::with_application(script, seed, NoApplication)
    }
}
impl<A: Application> Engine<A> {
    /// Checks every decoded/program-supplied document before retaining the script.
    pub fn with_application(script: Script, seed: u64, application: A) -> Result<Self, Invalid> {
        script.validate()?;
        if !application.fits(ApplicationValue::State(&application), &script.bounds) {
            return Err(Invalid::Payload);
        }
        Ok(Self {
            durable_application: application.clone(),
            application,
            application_watches: BTreeMap::new(),
            attempts: BTreeMap::new(),
            reads: BTreeMap::new(),
            offline: None,
            slow: Duration::ZERO,
            request_fault: None,
            clock: Time::ZERO,
            bounds: script.bounds,
            versions: script.versions,
            durable_truth: script.truth.clone(),
            truth: script.truth,
            rules: script.rules,
            keys: BTreeMap::new(),
            pending: Vec::new(),
            commits: Vec::new(),
            next_commit: 1,
            due: Vec::new(),
            seed,
            read_tokens: BTreeSet::new(),
            positions: BTreeMap::new(),
            next_position: 1,
            watches: BTreeMap::new(),
            watch_tokens: BTreeSet::new(),
            runs: BTreeMap::new(),
        })
    }
    /// The scheduling bounds used at every typed entrance.
    #[must_use]
    pub fn bounds(&self) -> Bounds {
        self.bounds
    }
    /// Configured range for the stream hello, independent of Frame and commits.
    #[must_use]
    pub fn versions(&self) -> jig_client::contract::VersionRange {
        self.versions
    }
    /// The injected seed, for replay ledgers.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }
    /// The live truth may include decisions awaiting durability; clients cannot read it directly.
    #[must_use]
    pub fn truth(&self) -> &Truth {
        &self.truth
    }
    /// Only durable commits, in their commit order.
    #[must_use]
    pub fn commits(&self) -> &[Committed<A>] {
        &self.commits
    }
    /// Decisions made but not yet durable; restart removes this staged prefix.
    pub fn pending_commits(&self) -> impl Iterator<Item = &Committed<A>> {
        self.pending.iter().map(|pending| &pending.commit)
    }
    /// Replaces a scenario's outcome for future new keys. Retained keys stay alike.
    /// The world can script an ask after its person generates the actual words.
    pub fn script(&mut self, rule: Rule) -> Result<(), Invalid> {
        rule.validate(&self.bounds)?;
        self.rules.retain(|old| old.party != rule.party || old.ask != rule.ask);
        self.rules.push(rule);
        Ok(())
    }
    /// Moves the story's truth through a numbered, delayed commit.
    pub fn act(&mut self, now: Time, wall: Wall, change: Change) -> Result<u64, Invalid> {
        if !self.online(now) {
            return Err(Invalid::Offline);
        }
        change.validate(&self.bounds)?;
        let mut truth = self.truth.clone();
        truth.apply(&change)?;
        self.truth = truth;
        Ok(self.commit(now, wall, None, None, None, None, vec![change]))
    }
    #[expect(clippy::too_many_arguments, reason = "the ledger records all names and payloads of one atomic decision")]
    fn commit(
        &mut self,
        now: Time,
        wall: Wall,
        party: Option<u64>,
        key: Option<[u8; 16]>,
        ask: Option<Ask>,
        answer: Option<Answer>,
        changes: Vec<Change>,
    ) -> u64 {
        self.clock = self.clock.max(now);
        for watch in self.watches.values_mut() {
            if !watch.dirty {
                watch.ready = now.saturating_add(self.slow);
            }
            watch.dirty = true;
        }
        let subjects: Vec<_> = changes.iter().flat_map(|change| self.application.marks(change)).collect();
        for watch in self.application_watches.values_mut() {
            if subjects.contains(&watch.subject) {
                if !watch.dirty {
                    watch.ready = now.saturating_add(self.slow);
                }
                watch.dirty = true;
            }
        }
        let number = self.next_commit;
        self.next_commit = number.checked_add(1).expect("a scenario fits the commit number space");
        let mut due = now.saturating_add(self.bounds.commit_delay);
        if let Some(last) = self.pending.last() {
            due = due.max(last.due);
        }
        let commit = Committed {
            number,
            made: now,
            at: wall,
            durable: None,
            party,
            key,
            ask,
            answer,
            changes,
            application: None,
        };
        self.pending.push(Pending { commit, due, truth: self.truth.clone(), application: self.application.clone() });
        number
    }
    fn advance_commits(&mut self, now: Time) {
        let count = self.pending.iter().take_while(|pending| pending.due <= now).count();
        for mut pending in self.pending.drain(..count) {
            pending.commit.durable = Some(pending.due);
            self.durable_truth = pending.truth;
            self.durable_application = pending.application;
            for change in &pending.commit.changes {
                if let Change::Source(Source {
                    read: jig_client::contract::Read::Activity { task, .. },
                    rows: Rows::Activity(rows),
                    ..
                }) = change
                    && self.runs.get(task).is_some_and(|run| match run {
                        watches::Run::Text(run) => {
                            rows.iter().any(|line| line.event.committed_turn() == Some(run.turn.turn()))
                        }
                        watches::Run::State(_) => false,
                    })
                {
                    self.runs.remove(task);
                }
            }
            if let (Some(party), Some(key)) = (pending.commit.party, pending.commit.key) {
                let retention = self.durable_truth.frames.get(&party).expect("keyed party has a frame").retention;
                if let Some(keyed) = self.keys.get_mut(&(party, key)) {
                    keyed.retained_until = Some(pending.due.saturating_add(retention));
                }
            }
            self.commits.push(pending.commit);
        }
    }
    /// Releases due commits before answers and current watch values.
    #[must_use]
    pub fn poll(&mut self, now: Time) -> Vec<Up<A>> {
        if self.online(now) {
            self.offline = None;
            self.advance_commits(now);
            self.prepare_watches(now);
            self.prepare_application_watches(now);
        }
        let durable = self.commits.last().map_or(0, |commit| commit.number);
        let mut out = Vec::new();
        let mut held = Vec::new();
        for due in self.due.drain(..) {
            if due.at <= now && due.commit <= durable {
                match &due.up {
                    Up::Answer { attempt, .. } | Up::ApplicationAnswer { attempt, .. } => {
                        self.attempts.remove(attempt);
                    }
                    Up::Read { token, .. } | Up::ApplicationRead { token, .. } => {
                        self.reads.remove(token);
                    }
                    Up::Watch { .. } | Up::App { .. } | Up::Ended { .. } => {}
                }
                out.push(due.up);
            } else {
                held.push(due);
            }
        }
        self.due = held;
        out
    }
    /// The next commit or output time; a world may jump its clock to it.
    #[must_use]
    pub fn next_due(&self) -> Option<Time> {
        self.pending
            .iter()
            .map(|pending| pending.due)
            .chain(self.due.iter().map(|due| {
                let barrier = self
                    .pending
                    .iter()
                    .find(|pending| pending.commit.number == due.commit)
                    .map_or(Time::ZERO, |pending| pending.due);
                due.at.max(barrier)
            }))
            .chain(self.watch_due())
            .chain(self.application_watch_due())
            .chain(self.offline)
            .min()
            .map(|due| due.max(self.clock))
    }
    fn online(&mut self, now: Time) -> bool {
        self.clock = self.clock.max(now);
        if self.offline.is_some_and(|until| now < until) {
            false
        } else {
            self.offline = None;
            true
        }
    }
    fn queue(&mut self, now: Time, commit: u64, up: Up<A>) {
        let direct = matches!(up, Up::Watch { event: WatchEvent::Delivered(_), .. } | Up::App { .. });
        self.clock = self.clock.max(now);
        self.due.push(Due { at: if direct { now } else { now.saturating_add(self.slow) }, commit, up });
    }
}
#[cfg(test)]
mod tests;

pub mod owner_authority;
pub mod owner_budget;

pub use payload::owner_outcome::{admit_owner_outcome, owner_outcome_bytes};

pub mod owner_chip;

pub mod owner_amendment;

pub mod owner_activity;

pub mod owner_history;

pub mod owner_milestone;

pub mod owner_inbox;

pub mod owner_tree;

pub mod owner_person_task;
pub mod owner_proposal;

pub mod historical_pages;
pub mod owner_project;
