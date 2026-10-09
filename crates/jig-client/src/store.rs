//! This domain's records, separate from the root request table records.
//! Records restore, save and erase under this owner; client/domain/client.md, 11.
use crate::{address::Address, contract::Text};
/// A key owned by this domain in the small store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

use crate::{Domain, Request, contract::Limit, facts::Fact};
use skein_lib::{List, Queue};
/// Version of this domain's small-store vocabulary.
pub const STORE_VERSION: u16 = 1;
impl Record {
    /// Its key, owned solely by this domain.
    #[must_use]
    pub const fn key(&self) -> StoreKey {
        match self {
            Self::Place { .. } => StoreKey::Place,
            Self::Draft { task, .. } => StoreKey::Draft { task: *task },
        }
    }
    /// The party whose work must stay hidden from another sign-in.
    #[must_use]
    pub const fn party(&self) -> u64 {
        match self {
            Self::Place { party, .. } | Self::Draft { party, .. } => *party,
        }
    }
    /// Whether this version and words fit this owner's limits.
    #[must_use]
    pub fn valid(&self, text: u32) -> bool {
        match self {
            Self::Place { version, .. } => *version == STORE_VERSION,
            Self::Draft { version, words, .. } => {
                *version == STORE_VERSION
                    && u64::try_from(words.bytes().len()).expect("text length fits u64") <= u64::from(text)
            }
        }
    }
}
fn retain(domain: &mut Domain, record: Record) -> bool {
    for index in 0..domain.kept.len() {
        let kept = domain.kept.get_mut(index).expect("bounded record index exists");
        if kept.key() == record.key() {
            *kept = record;
            return true;
        }
    }
    domain.kept.push(record).is_ok()
}
/// Restores only this version's bounded records; no table records cross here.
pub(crate) fn restore(domain: &mut Domain, text: u32, records: List<Record>, out: &mut Queue<Request>) {
    for record in records.into_boxed() {
        if record.valid(text) && !retain(domain, record) {
            out.push(Request::Fact { fact: Fact::Refused { which: Limit::Records } });
            break;
        }
    }
}
/// Keeps a record before asking its shell owner to save it.
pub fn save(domain: &mut Domain, record: Record, out: &mut Queue<Request>) -> bool {
    if !record.valid(domain.text_limit) || domain.confirmed != Some(record.party()) {
        return false;
    }
    if !retain(domain, record.clone()) {
        out.push(Request::Fact { fact: Fact::Refused { which: Limit::Records } });
        return false;
    }
    out.push(Request::Save { record });
    true
}
/// Erases only this domain's record key; table keys cannot be expressed.
pub fn erase(domain: &mut Domain, key: StoreKey, out: &mut Queue<Request>) {
    let Some(party) = domain.confirmed else {
        return;
    };
    for record in &domain.kept {
        if record.key() == key && record.party() != party {
            return;
        }
    }
    let capacity = domain.kept.capacity();
    let old = core::mem::replace(&mut domain.kept, List::with_capacity(capacity));
    for record in old.into_boxed() {
        if record.key() != key {
            domain.kept.push(record).expect("filter preserves bounded record capacity");
        }
    }
    domain.timers.cancel(crate::domain::Timer::Save(key));
    out.push(Request::Erase { key });
}
/// Saves the selected jig place for the currently confirmed party.
pub(crate) fn place(domain: &mut Domain, out: &mut Queue<Request>) {
    if let Some(party) = domain.confirmed {
        let saved = save(domain, Record::Place { version: STORE_VERSION, party, address: domain.address }, out);
        if !saved {
            crate::pages::routing::refuse(domain, Limit::Records);
        }
    }
}

/// Keeps a policy's draft immediately and coalesces its shell saves on a
/// monotonic deadline. Jig 21 chooses when a draft uses this API.
pub fn coalesce(domain: &mut Domain, record: Record, out: &mut Queue<Request>) -> bool {
    if !record.valid(domain.text_limit) || domain.confirmed != Some(record.party()) {
        return false;
    }
    let key = record.key();
    if !retain(domain, record) {
        out.push(Request::Fact { fact: Fact::Refused { which: Limit::Records } });
        return false;
    }
    domain
        .timers
        .arm(crate::domain::Timer::Save(key), domain.now.saturating_add(domain.save_delay))
        .expect("one save deadline per record fits");
    true
}
pub(crate) fn fire_save(domain: &Domain, key: StoreKey, out: &mut Queue<Request>) {
    for record in &domain.kept {
        if record.key() == key {
            out.push(Request::Save { record: record.clone() });
            return;
        }
    }
}
