//! The two newest lent generations per account (protocol/llm.md, 5.3).
//! Routing supplies the token and provider account id as separate secrets.
use skein_lib::{List, Secret, Time};

/// The provider account id bound; empty when the provider names none.
pub const ACCOUNT_ID_BYTES: u32 = 256;
/// A grant's two secret parts, put in its taker's form only when handed over.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Value {
    pub token: Secret,
    pub account_id: Secret,
}
#[derive(Debug)]
pub struct Table {
    accounts: List<Generations>,
    token_bytes: u32,
}
#[derive(Debug)]
struct Generations {
    newest: Option<Entry>,
    older: Option<Entry>,
}
#[derive(Debug)]
struct Entry {
    generation: u64,
    value: Value,
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
        Table { accounts: slots, token_bytes }
    }
    pub fn granted(&mut self, account: u32, generation: u64, value: Value, expires: Time) -> Result<(), Unfit> {
        if value.token.len() > usize::try_from(self.token_bytes).expect("u32 fits usize")
            || value.account_id.len() > usize::try_from(ACCOUNT_ID_BYTES).expect("u32 fits usize")
        {
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
    pub fn value(&self, account: u32, generation: u64, now: Time) -> Option<&Value> {
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
/// The fixed slots and both bounded parts of two generations per account.
#[must_use]
pub fn worst_case(accounts: u32, token_bytes: u32) -> Option<u64> {
    if accounts == 0 || token_bytes == 0 {
        return None;
    }
    List::<Generations>::worst_case(accounts)?.checked_add(
        u64::from(token_bytes)
            .checked_add(u64::from(ACCOUNT_ID_BYTES))?
            .checked_mul(2)?
            .checked_mul(u64::from(accounts))?,
    )
}
