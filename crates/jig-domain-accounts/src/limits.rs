use skein_lib::{Deadlines, Duration, List, Map, Queue};

use crate::Fact;
use crate::domain::{Account, Granted};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub accounts: u32,
    /// Accounts one holder keeps, the deployment or a party; at most `accounts` (accounts.md, 3.6).
    pub holder_accounts: u32,
    /// Sources one account keeps, in its holder's order.
    pub sources: u32,
    /// Sign-ins under way at once; skein's driver admits as many redirect listeners.
    pub sign_ins: u32,
    pub credentials: u32,
    pub backoff_base: Duration,
    pub backoff_max: Duration,
    pub retry: Duration,
    pub spent_attention: Duration,
    pub facts: u32,
}

/// Accounts hold fixed-sized names and times; every secret is below them. Each bound is at least
/// one, and a holder's accounts are within the deployment's.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.accounts == 0
        || limits.accounts.checked_add(2).is_none()
        || limits.holder_accounts == 0
        || limits.holder_accounts > limits.accounts
        || limits.sources == 0
        || limits.sign_ins == 0
        || limits.retry == Duration::ZERO
        || limits.backoff_base == Duration::ZERO
        || limits.backoff_max < limits.backoff_base
    {
        return None;
    }
    Map::<u32, Account>::worst_case(limits.accounts)?
        .checked_add(Map::<u32, Granted>::worst_case(limits.credentials)?)?
        .checked_add(Deadlines::<u32>::worst_case(limits.credentials)?)?
        .checked_add(List::<u32>::worst_case(limits.accounts)?)?
        .checked_add(Deadlines::<u32>::worst_case(limits.accounts)?)?
        .checked_add(Queue::<Fact>::worst_case(limits.facts)?)
}
