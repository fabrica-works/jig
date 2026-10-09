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
mod payload;
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
pub enum Up {
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
    Ended {
        token: Token,
        why: WatchEnd,
    },
}

/// One numbered decision, including its payload and durability time for a referee.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Committed {
    pub number: u64,
    pub made: Time,
    pub at: Wall,
    pub durable: Option<Time>,
    pub party: Option<u64>,
    pub key: Option<[u8; 16]>,
    pub ask: Option<Ask>,
    pub answer: Option<Answer>,
    pub changes: Vec<Change>,
}
#[derive(Debug)]
struct Pending {
    commit: Committed,
    due: Time,
    truth: Truth,
}
#[derive(Debug)]
struct Keyed {
    ask: Ask,
    answer: Answer,
    commit: u64,
    retained_until: Option<Time>,
}
#[derive(Debug)]
struct Due {
    at: Time,
    commit: u64,
    up: Up,
}

/// A deterministic scripted engine; seed is retained for reproducible extensions.
#[derive(Debug)]
pub struct Engine {
    bounds: Bounds,
    truth: Truth,
    durable_truth: Truth,
    rules: Vec<Rule>,
    keys: BTreeMap<(u64, [u8; 16]), Keyed>,
    pending: Vec<Pending>,
    commits: Vec<Committed>,
    next_commit: u64,
    due: Vec<Due>,
    seed: u64,
    read_tokens: BTreeSet<Token>,
    positions: BTreeMap<u64, reads::Position>,
    next_position: u64,
    watches: BTreeMap<Token, watches::Watch>,
    watch_tokens: BTreeSet<Token>,
    runs: BTreeMap<u64, jig_client::contract::RunText>,
}
impl Engine {
    /// Checks every decoded/program-supplied document before retaining the script.
    pub fn new(script: Script, seed: u64) -> Result<Self, Invalid> {
        script.validate()?;
        Ok(Self {
            bounds: script.bounds,
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
    pub fn commits(&self) -> &[Committed] {
        &self.commits
    }
    /// Moves the story's truth through a numbered, delayed commit.
    pub fn act(&mut self, now: Time, wall: Wall, change: Change) -> Result<u64, Invalid> {
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
        for watch in self.watches.values_mut() {
            watch.dirty = true;
        }
        let number = self.next_commit;
        self.next_commit = number.checked_add(1).expect("a scenario fits the commit number space");
        let mut due = now.saturating_add(self.bounds.commit_delay);
        if let Some(last) = self.pending.last() {
            due = due.max(last.due);
        }
        let commit = Committed { number, made: now, at: wall, durable: None, party, key, ask, answer, changes };
        self.pending.push(Pending { commit, due, truth: self.truth.clone() });
        number
    }
    /// Releases due commits before any answer, preserving commit order.
    #[must_use]
    pub fn poll(&mut self, now: Time) -> Vec<Up> {
        let count = self.pending.iter().take_while(|pending| pending.due <= now).count();
        for mut pending in self.pending.drain(..count) {
            pending.commit.durable = Some(pending.due);
            self.durable_truth = pending.truth;
            if let (Some(party), Some(key)) = (pending.commit.party, pending.commit.key) {
                let retention = self.truth.frames.get(&party).expect("keyed party has a frame").retention;
                if let Some(keyed) = self.keys.get_mut(&(party, key)) {
                    keyed.retained_until = Some(pending.due.saturating_add(retention));
                }
            }
            self.commits.push(pending.commit);
        }
        self.prepare_watches(now);
        let durable = self.commits.last().map_or(0, |commit| commit.number);
        let mut out = Vec::new();
        let mut held = Vec::new();
        for due in self.due.drain(..) {
            if due.at <= now && due.commit <= durable {
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
            .min()
    }
    fn queue(&mut self, now: Time, commit: u64, up: Up) {
        self.due.push(Due { at: now, commit, up });
    }
}
#[cfg(test)]
mod tests;
