//! Each source's record, kept in the owner's secrets' store (protocol/llm.md,
//! 5.1, 5.2 and 5.4; store.md, 9).
//!
//! The driver starts from the records the store holds; each renewed record
//! goes to the store stating the digest of the one it replaces, and the
//! driver hears `Kept` only once the store answers that it is durable. A
//! handed-in record (llm.md 5.4) is kept the same way before it is handed
//! in, so no token is lent before its record is kept (llm.md 8).
use crate::handed_in::Unfit;
use crate::table::ACCOUNT_ID_BYTES;
use crate::translate::{self, Candidate, Up};
use jig_domain_accounts as accounts;
use skein_lib::{Map, Secret, Time, sha256};
use skein_oauth as oauth;
use skein_oauth_accounts as driver;

/// What the store is asked for one source's record: keep `value` in place
/// of the record whose digest is `replaces`, none when it holds none.
#[derive(PartialEq, Eq, Debug)]
pub struct Keep {
    pub account: u32,
    pub value: Secret,
    pub replaces: Option<[u8; 32]>,
}

/// Where a record goes next: to the driver, or first to the store.
#[derive(PartialEq, Eq)]
#[expect(missing_debug_implementations, reason = "the driver's requests carry records' tokens")]
pub enum Next {
    Driver(driver::Request),
    Store(Keep),
}

#[derive(Debug)]
enum Waiting {
    /// The driver's candidate; the store's answer goes back as its `Kept`.
    Candidate { generation: u64, digest: [u8; 32] },
    /// A handed-in record, handed in once the store has kept it.
    HandedIn { record: Candidate, digest: [u8; 32] },
}

#[derive(Debug)]
struct Slot {
    /// The digest of what the store holds for this source, read or not.
    held: Option<[u8; 32]>,
    waiting: Option<Waiting>,
    /// The store holds a record that could not be read (llm.md 5.4).
    unread: bool,
}

/// The records of a deployment's sources, by account number.
#[derive(Debug)]
pub struct Records {
    slots: Map<u32, Slot>,
}

impl Records {
    #[must_use]
    pub fn new(accounts: u32) -> Self {
        Self { slots: Map::with_capacity(accounts) }
    }

    /// Reserves a named LLM account source's record place.
    pub fn admit(&mut self, account: u32) -> Result<(), Unfit> {
        if !self.slots.contains_key(&account) {
            self.slots.insert(account, Slot { held: None, waiting: None, unread: false }).or(Err(Unfit))?;
        }
        Ok(())
    }
    /// Frees the record place once the LLM account source has been removed.
    pub fn remove(&mut self, account: u32) {
        let _removed = self.slots.remove(&account);
    }
    /// The record the store holds for `account`, read at start: the record,
    /// when it is the account's and fits the driver's limits; otherwise
    /// none, and the source fails as unloaded until a new record replaces
    /// it (llm.md 5.4).
    pub fn load(&mut self, account: u32, value: &[u8], limits: &oauth::Limits) -> Option<oauth::SavedToken> {
        self.admit(account).ok()?;
        let slot = self.slots.get_mut(&account)?;
        slot.held = Some(sha256::digest(value));
        let record = match oauth::decode_record(value, limits) {
            Ok(record) if fits(&record, account, limits) => Some(record),
            Ok(_) | Err(_) => None,
        };
        slot.unread = record.is_none();
        record
    }

    /// A handed-in record at start (owner's decision D2): handed in now when
    /// the store already holds it, else kept in place of what it holds and
    /// handed in once kept.
    pub fn hand_in(&mut self, account: u32, record: oauth::SavedToken, limits: &oauth::Limits) -> Result<Next, Unfit> {
        let value = oauth::encode_record(&record, limits).or(Err(Unfit))?;
        let digest = sha256::digest(&value);
        self.admit(account)?;
        let slot = self.slots.get_mut(&account).ok_or(Unfit)?;
        if slot.held == Some(digest) {
            return Ok(Next::Driver(driver::Request::HandIn { account, record }));
        }
        slot.waiting = Some(Waiting::HandedIn { record: Candidate(record), digest });
        Ok(Next::Store(Keep { account, value: Secret::new(value), replaces: slot.held }))
    }

    /// The driver's renewed record, for the store; a record that cannot be
    /// encoded, or one while another is on its way, is answered `NotKept`.
    pub fn keep(&mut self, account: u32, record: oauth::SavedToken, limits: &oauth::Limits) -> Next {
        let generation = record.generation;
        let not_kept = Next::Driver(driver::Request::Kept { account, generation, keeping: driver::Keeping::NotKept });
        if self.admit(account).is_err() {
            return not_kept;
        }
        let Some(slot) = self.slots.get_mut(&account) else { return not_kept };
        if slot.waiting.is_some() {
            return not_kept;
        }
        let Ok(value) = oauth::encode_record(&record, limits) else { return not_kept };
        let digest = sha256::digest(&value);
        slot.waiting = Some(Waiting::Candidate { generation, digest });
        Next::Store(Keep { account, value: Secret::new(value), replaces: slot.held })
    }

    /// The store kept `account`'s record: the driver's answer, or the
    /// handed-in record now kept.
    pub fn kept(&mut self, account: u32) -> Option<driver::Request> {
        let slot = self.slots.get_mut(&account)?;
        let (request, digest) = match slot.waiting.take()? {
            Waiting::Candidate { generation, digest } => {
                (driver::Request::Kept { account, generation, keeping: driver::Keeping::Kept }, digest)
            }
            Waiting::HandedIn { record, digest } => (driver::Request::HandIn { account, record: record.0 }, digest),
        };
        slot.held = Some(digest);
        slot.unread = false;
        Some(request)
    }

    /// The store holds another record than the one stated: the driver's
    /// candidate is not kept; a handed-in record is not handed in, and its
    /// source has no record until the next start reads the store again.
    pub fn stale(&mut self, account: u32) -> Option<driver::Request> {
        match self.slots.get_mut(&account)?.waiting.take()? {
            Waiting::Candidate { generation, digest: _ } => {
                Some(driver::Request::Kept { account, generation, keeping: driver::Keeping::NotKept })
            }
            Waiting::HandedIn { record: _, digest: _ } => None,
        }
    }

    /// Translates the driver's event (llm.md 5.2); a source whose stored
    /// record could not be read fails as unloaded, saying where and why,
    /// where the driver, holding no record for it, fails its grant as
    /// expired.
    #[must_use]
    pub fn up(&self, event: driver::Event, now: Time, required: &[u8]) -> Up {
        if let driver::Event::Failed { account, ends: driver::Ends::Grant, failure: driver::Failure::Expired } = event
            && let Some(slot) = self.slots.get(&account)
            && slot.unread
        {
            return Up::Child(accounts::FromComponent::Failed {
                account,
                failure: accounts::Failure::Unloaded { at: accounts::Place::File, why: accounts::Unread::Unreadable },
            });
        }
        translate::up(event, now, required)
    }

    /// Whether a write is on its way to the store.
    #[must_use]
    pub fn keeping(&self) -> bool {
        for (_, slot) in &self.slots {
            if slot.waiting.is_some() {
                return true;
            }
        }
        false
    }
}

/// Whether a stored record is `account`'s and fits the driver's limits.
fn fits(record: &oauth::SavedToken, account: u32, limits: &oauth::Limits) -> bool {
    let id = match &record.metadata {
        Some(id) => id.len(),
        None => 0,
    };
    record.key == account
        && id <= usize::try_from(ACCOUNT_ID_BYTES).expect("u32 fits usize")
        && oauth::encode_record(record, limits).is_ok()
}

/// The bytes the records need: their slots, and the record each may hold
/// while the store keeps it.
#[must_use]
pub fn worst_case(accounts: u32, limits: &oauth::Limits) -> Option<u64> {
    Map::<u32, Slot>::worst_case(accounts)?
        .checked_add(u64::from(accounts).checked_mul(u64::from(limits.record_bytes))?)
}
