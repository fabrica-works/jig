//! This domain's records, separate from the root request table records.
//! Restore and save transitions land in jig 20.4; client/domain/client.md, 11.
use crate::{address::Address, contract::Text};
/// A key owned by this domain in the small store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StoreKey {
    /// The previous jig address.
    Place,
    /// A draft for a task.
    Draft { task: u64 },
}

/// A versioned record naming the party that owns kept work.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Record {
    /// Where the party was.
    Place { version: u16, party: u64, address: Address },
    /// Words not yet sent.
    Draft { version: u16, party: u64, task: u64, words: Text },
}
