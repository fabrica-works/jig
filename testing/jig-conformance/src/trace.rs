//! Outside evidence retained without formatting (`domain/testing.md`, 5–6).
use crate::referee::Observed;
use std::ops::Deref;

/// One concrete loop boundary or independent peer/store observation.
#[derive(Debug)]
pub enum Entry {
    PeerInput,
    Committed(u64),
    Failed(u64),
    RestoredRow,
    Submitted { number: u64, writes: usize },
    Delivery,
    Stopped,
    Observed(Observed),
}

/// Diagnostic history; formatting is requested by a failing assertion only.
#[derive(Default, Debug)]
pub struct Trace(Vec<(u64, Entry)>);

impl Trace {
    pub fn push(&mut self, now: u64, entry: Entry) {
        self.0.push((now, entry));
    }
    #[must_use]
    pub fn join(&self, separator: &str) -> String {
        self.0.iter().map(|(now, entry)| format!("{now}: {entry:?}")).collect::<Vec<_>>().join(separator)
    }
}

impl Deref for Trace {
    type Target = [(u64, Entry)];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
