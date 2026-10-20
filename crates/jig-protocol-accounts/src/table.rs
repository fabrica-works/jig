//! The two newest lent generations per account (protocol/llm.md, 5.3).
//! A value is a credential with its scheme (protocol/channel.md, 8).
use skein_lib::{Map, Secret, Time};

/// The withdrawn arm's provider account id bound.
pub const ACCOUNT_ID_BYTES: u32 = 256;
/// A grant's credential, put in its taker's form only when handed over.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Value {
    pub token: Secret,
    pub scheme: Scheme,
}
/// How the credential is presented. `Withdrawn` keeps a provider account id
/// beside a bearer token for jig's live test only, until JP13 removes it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Scheme {
    Bearer,
    Key,
    Withdrawn { id: Secret },
}
#[derive(Debug)]
pub struct Table {
    accounts: Map<u32, Generations>,
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
        Table { accounts: Map::with_capacity(accounts), token_bytes }
    }
    pub fn granted(&mut self, account: u32, generation: u64, value: Value, expires: Time) -> Result<(), Unfit> {
        let id = match &value.scheme {
            Scheme::Bearer | Scheme::Key => 0,
            Scheme::Withdrawn { id } => id.len(),
        };
        if value.token.len() > usize::try_from(self.token_bytes).expect("u32 fits usize")
            || id > usize::try_from(ACCOUNT_ID_BYTES).expect("u32 fits usize")
        {
            return Err(Unfit);
        }
        if !self.accounts.contains_key(&account) {
            self.accounts.insert(account, Generations { newest: None, older: None }).or(Err(Unfit))?;
        }
        let slots = self.accounts.get_mut(&account).expect("admitted account");
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
        let slots = self.accounts.get(&account)?;
        for entry in [&slots.newest, &slots.older].into_iter().flatten() {
            if entry.generation == generation && now < entry.expires {
                return Some(&entry.value);
            }
        }
        None
    }
    /// Frees a removed LLM account source's generations and its table place.
    pub fn remove(&mut self, account: u32) {
        let _removed = self.accounts.remove(&account);
    }
    pub fn expire(&mut self, now: Time) {
        let mut after = 0;
        for _ in 0..self.accounts.len() {
            let Some((&number, _)) = self.accounts.range(after..).next() else { break };
            let slots = self.accounts.get_mut(&number).expect("named account");
            expire(&mut slots.newest, now);
            expire(&mut slots.older, now);
            if slots.newest.is_none() {
                slots.newest = slots.older.take();
            }
            let Some(next) = number.checked_add(1) else { break };
            after = next;
        }
    }
    #[must_use]
    pub fn next_expiry(&self) -> Option<Time> {
        let mut earliest: Option<Time> = None;
        for (_, slots) in &self.accounts {
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
    Map::<u32, Generations>::worst_case(accounts)?.checked_add(
        u64::from(token_bytes)
            .checked_add(u64::from(ACCOUNT_ID_BYTES))?
            .checked_mul(2)?
            .checked_mul(u64::from(accounts))?,
    )
}
