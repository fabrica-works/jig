//! The store's records, restored before the child starts (engine.md, 5.4). A record that keeps
//! the accounts' rules is taken; any other is counted and left, never trusted, so that a provider
//! configured again finds its accounts. Every number a record carries moves the counter past it,
//! taken or not, so that no number is given twice.
use crate::domain::{Account, Domain, Hold, Source, settle};
use crate::records::{program_name, within};
use crate::{DISPLAY, Fact, Held, Kind, Limits, SUBJECT, Stored};
use skein_lib::{Duration, Env, List, Map, Wall};

pub(crate) fn restore(domain: &mut Domain, env: &Env<Limits>, record: Stored) {
    let taken = match record {
        Stored::Next { number } => {
            domain.next = domain.next.max(number);
            domain.kept = number;
            true
        }
        Stored::Account { number, provider, holder, place, uses, subject, display } => {
            domain.past(number);
            // An account of a provider no longer configured waits for it.
            domain.providers.get(provider).is_some()
                && account(
                    domain,
                    env,
                    number,
                    Account {
                        provider,
                        holder,
                        place,
                        uses,
                        plan: false,
                        subject,
                        display,
                        sources: Map::with_capacity(0),
                    },
                )
        }
        Stored::Source { account, number, place, kind, name, held } => {
            domain.past(number);
            let mut entry = Source::new(account, kind, name, &env.limits);
            hold(&mut entry, env, held);
            source(domain, env, number, place, entry)
        }
    };
    if !taken {
        domain.unrestored = domain.unrestored.saturating_add(1);
    }
}

/// The start drops each account left with no source: none of its records says how it is used.
pub(crate) fn finish(domain: &mut Domain, env: &Env<Limits>) {
    let mut empty = List::with_capacity(env.limits.accounts);
    for (number, account) in &domain.accounts {
        if account.sources.is_empty() {
            empty.push(*number).expect("one per account");
        }
    }
    for number in &empty {
        drop(domain.accounts.remove(number).expect("a listed account"));
        domain.unrestored = domain.unrestored.saturating_add(1);
    }
}

/// A free number, a bounded subject and display name, and a subject no other account of its
/// provider holds (accounts.md, 3.1 and 3.2).
fn account(domain: &mut Domain, env: &Env<Limits>, number: u32, mut entry: Account) -> bool {
    if !free(domain, number)
        || !within(&entry.subject, SUBJECT)
        || !within(&entry.display, DISPLAY)
        || !crate::records::valid_uses(entry.uses.as_deref())
    {
        return false;
    }
    if !entry.subject.is_empty() {
        for (_, other) in &domain.accounts {
            if other.provider == entry.provider && other.subject == entry.subject {
                return false;
            }
        }
    }
    entry.sources = Map::with_capacity(env.limits.sources);
    match domain.accounts.insert(number, entry) {
        Ok(old) => old.is_none(),
        Err(_) => false,
    }
}

/// A free number, a kept account with room at a free place, one signed-in source an account, and
/// a program's name valid and unique in the deployment (accounts.md, 3.4).
fn source(domain: &mut Domain, env: &Env<Limits>, number: u32, place: u32, entry: Source) -> bool {
    if !free(domain, number) {
        return false;
    }
    let Some(account) = domain.accounts.get(&entry.account) else { return false };
    if account.sources.contains_key(&place) || account.sources.len() >= account.sources.capacity() {
        return false;
    }
    match entry.kind {
        Kind::Program => {
            if !program_name(&entry.name) {
                return false;
            }
            for (_, other) in &domain.sources {
                if other.kind == Kind::Program && other.name == entry.name {
                    return false;
                }
            }
        }
        Kind::SignIn => {
            if !entry.name.is_empty() {
                return false;
            }
            for (_, other) in &account.sources {
                if domain.sources.get(other).expect("an account's sources are kept").kind == Kind::SignIn {
                    return false;
                }
            }
        }
        Kind::Key | Kind::None => {
            if !entry.name.is_empty() {
                return false;
            }
        }
    }
    let account = entry.account;
    let plan = entry.kind == Kind::SignIn || entry.kind == Kind::Program;
    if domain.sources.insert(number, entry) != Ok(None) {
        return false;
    }
    let account = domain.accounts.get_mut(&account).expect("looked up above");
    account.plan |= plan;
    let places = &mut account.sources;
    assert!(places.insert(place, number) == Ok(None), "a free place with room, checked above");
    let _: Fact = settle(domain, env, number);
    true
}

/// A number neither an account nor a source holds; the counter never gives `u32::MAX`.
fn free(domain: &Domain, number: u32) -> bool {
    number != u32::MAX && !domain.accounts.contains_key(&number) && !domain.sources.contains_key(&number)
}

/// A kept hold means the same after a restart: a reset passed while the child was down has
/// lifted it, and a later one is projected onto this run's clock (engine.md, 5.4).
fn hold(entry: &mut Source, env: &Env<Limits>, kept: Option<Held>) {
    let (hold, attention) = match kept {
        None => return,
        Some(Held::Refused) => (Hold::Refused, true),
        Some(Held::Limited { reset: None }) => (Hold::Limited { until: None }, true),
        Some(Held::Limited { reset: Some(reset) }) => {
            let Some(left) = left(env.wall, reset) else { return };
            (Hold::Limited { until: Some(env.now.saturating_add(left)) }, left >= env.limits.spent_attention)
        }
    };
    entry.hold = Some(hold);
    entry.hold_attention = attention;
}

/// The time from now to a later wall time; none once it has passed.
fn left(now: Wall, reset: Wall) -> Option<Duration> {
    match reset.as_nanos().checked_sub(now.as_nanos()) {
        Some(0) | None => None,
        Some(nanos) => Some(Duration::from_nanos(nanos)),
    }
}
