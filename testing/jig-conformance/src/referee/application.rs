//! Application promises from offered requests, fake-store transactions and replies.
use super::{Promise, Violation, require};
use std::collections::{BTreeMap, BTreeSet};

/// The authenticated party and its shared request key, observed outside the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ApplicationKey {
    pub person: u64,
    pub key: [u8; 16],
}

/// The metadata and owner-encoded terminal found together in a submitted transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationAnswer {
    pub key: ApplicationKey,
    pub connector: u16,
    pub digest: [u8; 32],
    pub token: u64,
    pub until: u64,
    /// The world encodes its connector's outcome; the referee never interprets it.
    pub outcome: Box<[u8]>,
}

#[derive(Clone, Default, Debug)]
pub(super) struct Applications {
    offered: BTreeMap<u64, (ApplicationKey, u16, [u8; 32])>,
    pending: BTreeMap<u64, Vec<ApplicationAnswer>>,
    kept: BTreeMap<ApplicationKey, ApplicationAnswer>,
    allocated: BTreeSet<u64>,
}

impl Applications {
    pub(super) fn offered(&mut self, reply: u64, key: ApplicationKey, connector: u16, digest: [u8; 32]) {
        self.offered.insert(reply, (key, connector, digest));
    }
    pub(super) fn submitted(&mut self, commit: u64, answer: ApplicationAnswer) {
        self.pending.entry(commit).or_default().push(answer);
    }
    pub(super) fn durable(&mut self, commit: u64, now: u64) -> Result<(), Violation> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let ready: Vec<_> = self.pending.range(..=commit).map(|(number, _)| *number).collect();
        for number in ready {
            for answer in self.pending.remove(&number).expect("observed transaction") {
                require(answer.token != 0, Promise::Commit, "application outcome has no durable token")?;
                require(
                    self.allocated.insert(answer.token),
                    Promise::Once,
                    "connector decided a retained token again",
                )?;
                require(
                    self.kept.get(&answer.key).is_none_or(|previous| now >= previous.until),
                    Promise::Once,
                    "connector decided the same retained application key again",
                )?;
                self.kept.insert(answer.key, answer);
            }
        }
        Ok(())
    }
    pub(super) fn answered(&mut self, reply: u64, connector: u16, outcome: &[u8]) -> Result<(), Violation> {
        let offered = self.offered.remove(&reply);
        require(offered.is_some(), Promise::Commit, "application terminal has no observed party request")?;
        let (key, owner, digest) = offered.expect("observed request");
        let answer = self.kept.get(&key);
        require(answer.is_some(), Promise::Commit, "application answer preceded its durable decision")?;
        let answer = answer.expect("durable application terminal");
        require(
            owner == connector && answer.connector == connector && answer.digest == digest,
            Promise::Commit,
            "application answer belongs to another owner or request",
        )?;
        require(answer.outcome.as_ref() == outcome, Promise::Once, "application replay changed its kept outcome")
    }
    pub(super) fn restart(&mut self) {
        self.pending.clear();
        self.offered.clear();
    }
}
