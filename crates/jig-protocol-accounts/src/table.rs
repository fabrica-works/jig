//! The two newest lent generations per account (protocol/llm.md, 5.3).
//! Routing supplies smith's envelope: an account id and its access token.
use crate::ACCOUNT_ID_BYTES;
use skein_lib::{List, Secret, Time};

#[derive(Debug)]
pub struct Table {
    accounts: List<Generations>,
    value_bytes: u64,
}
#[derive(Debug)]
struct Generations {
    newest: Option<Entry>,
    older: Option<Entry>,
}
#[derive(Debug)]
struct Entry {
    generation: u64,
    value: Secret,
    expires: Time,
}
/// An account or value beyond the configured bounds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unfit;

impl Table {
    #[must_use]
    pub fn new(accounts: u32, token_bytes: u32) -> Table {
        let mut slots = List::with_capacity(accounts);
        for _account in 0..accounts {
            slots.push(Generations { newest: None, older: None }).expect("one slot per configured account");
        }
        Table { accounts: slots, value_bytes: value_bytes(token_bytes) }
    }
    pub fn granted(&mut self, account: u32, generation: u64, value: Secret, expires: Time) -> Result<(), Unfit> {
        if u64::try_from(value.len()).expect("a value's length fits u64") > self.value_bytes {
            return Err(Unfit);
        }
        let Some(slots) = self.accounts.get_mut(account) else { return Err(Unfit) };
        let entry = Entry { generation, value, expires };
        match &slots.newest {
            Some(newest) if generation < newest.generation => match &slots.older {
                Some(older) if generation < older.generation => (),
                Some(_) | None => slots.older = Some(entry),
            },
            Some(newest) if generation == newest.generation => slots.newest = Some(entry),
            Some(_) | None => {
                slots.older = slots.newest.take();
                slots.newest = Some(entry);
            }
        }
        Ok(())
    }
    #[must_use]
    pub fn value(&self, account: u32, generation: u64, now: Time) -> Option<&Secret> {
        let slots = self.accounts.get(account)?;
        for entry in [&slots.newest, &slots.older].into_iter().flatten() {
            if entry.generation == generation && now < entry.expires {
                return Some(&entry.value);
            }
        }
        None
    }
    pub fn expire(&mut self, now: Time) {
        for account in 0..self.accounts.len() {
            let slots = self.accounts.get_mut(account).expect("configured account slot");
            expire(&mut slots.newest, now);
            expire(&mut slots.older, now);
            if slots.newest.is_none() {
                slots.newest = slots.older.take();
            }
        }
    }
    #[must_use]
    pub fn next_expiry(&self) -> Option<Time> {
        let mut earliest: Option<Time> = None;
        for slots in &self.accounts {
            for entry in [&slots.newest, &slots.older].into_iter().flatten() {
                earliest = Some(match earliest {
                    Some(time) => time.min(entry.expires),
                    None => entry.expires,
                });
            }
        }
        earliest
    }
}
fn expire(slot: &mut Option<Entry>, now: Time) {
    if let Some(entry) = slot
        && now >= entry.expires
    {
        *slot = None;
    }
}
fn value_bytes(token_bytes: u32) -> u64 {
    u64::from(token_bytes)
        .checked_add(u64::from(ACCOUNT_ID_BYTES))
        .expect("u32 bounds fit u64")
        .checked_add(2)
        .expect("envelope header fits u64")
}
/// The fixed slots and both bounded envelope allocations per account.
#[must_use]
pub fn worst_case(accounts: u32, token_bytes: u32) -> Option<u64> {
    if accounts == 0 || token_bytes == 0 {
        return None;
    }
    List::<Generations>::worst_case(accounts)?
        .checked_add(value_bytes(token_bytes).checked_mul(2)?.checked_mul(u64::from(accounts))?)
}
