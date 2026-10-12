use crate::{Event, Fact, Failure, FromComponent, Grant, Limits, Request, State, ToComponent};
use alloc::boxed::Box;
use skein_lib::{Deadlines, Duration, Env, List, Map, Queue, Time};

/// Want or release every configured account, with the component's close.
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    limits.accounts.checked_add(2).expect("account output bound")
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub(crate) struct Account {
    state: State,
    grant: Option<Granted>,
    retry: Option<Time>,
    backoff: Duration,
    spent: Option<Time>,
    usable: bool,
    expiring: bool,
    spent_attention: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Granted {
    generation: u64,
    expires: Time,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Phase {
    New,
    Wanted,
    Closing,
    Closed,
}
#[derive(Debug)]
pub struct Domain {
    accounts: Map<u32, Account>,
    alarms: Deadlines<u32>,
    facts: Queue<Fact>,
    lost: u64,
    phase: Phase,
}
impl Domain {
    #[must_use]
    pub fn new(limits: &Limits, accounts: Box<[u32]>) -> Domain {
        assert!(crate::worst_case(limits).is_some(), "account limits are valid");
        let mut domain = Domain {
            accounts: Map::with_capacity(limits.accounts),
            alarms: Deadlines::with_capacity(limits.accounts),
            facts: Queue::with_capacity(limits.facts),
            lost: 0,
            phase: Phase::New,
        };
        for number in &accounts {
            let entry = Account {
                state: State::Asking,
                grant: None,
                retry: None,
                backoff: limits.backoff_base,
                spent: None,
                usable: false,
                expiring: false,
                spent_attention: false,
            };
            assert!(domain.accounts.insert(*number, entry) == Ok(None), "configured accounts fit and are distinct");
        }
        domain
    }
    #[must_use]
    pub fn usable(&self, account: u32) -> bool {
        match self.accounts.get(&account) {
            Some(entry) => entry.usable,
            None => false,
        }
    }
    #[must_use]
    pub fn grant(&self, account: u32, now: Time) -> Option<Grant> {
        let entry = self.accounts.get(&account)?;
        let grant = entry.grant?;
        if !entry.usable || grant.expires <= now {
            return None;
        }
        Some(Grant { account, generation: grant.generation, valid: grant.expires.saturating_since(now) })
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.alarms.next()
    }
    #[must_use]
    pub fn is_due(&self, now: Time) -> bool {
        match self.alarms.next() {
            Some(at) => at <= now,
            None => false,
        }
    }
    #[must_use]
    pub fn waiting(&self) -> bool {
        self.phase == Phase::Closing
    }
    #[must_use]
    pub fn account_views(&self) -> List<Fact> {
        let mut result = List::with_capacity(self.accounts.capacity());
        for (number, entry) in &self.accounts {
            result.push(fact(*number, entry)).expect("one view per account");
        }
        result
    }
    #[must_use]
    pub fn accounts(&self) -> u32 {
        self.accounts.len()
    }
    pub fn pop_fact(&mut self) -> Option<Fact> {
        self.facts.pop()
    }
    #[must_use]
    pub const fn facts_lost(&self) -> u64 {
        self.lost
    }
}

pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Want => {
            if domain.phase != Phase::New {
                return;
            }
            domain.phase = Phase::Wanted;
            for (account, _) in &domain.accounts {
                out.push(Request::Component(ToComponent::Want { account: *account }));
            }
        }
        Event::Close => close(domain, env, out),
        Event::Component(FromComponent::Closed) => {
            if domain.phase != Phase::Closing {
                return;
            }
            domain.phase = Phase::Closed;
            domain.accounts = Map::with_capacity(env.limits.accounts);
            out.push(Request::Closed);
        }
        Event::Component(event) => {
            if domain.phase != Phase::Wanted {
                return;
            }
            component(domain, env, event, out);
        }
        Event::Rejected { account, generation } => {
            if domain.phase == Phase::Wanted && domain.accounts.contains_key(&account) {
                out.push(Request::Component(ToComponent::Rejected { account, generation }));
            }
        }
        Event::Exhausted { account, retry_after } => {
            if domain.phase != Phase::Wanted {
                return;
            }
            let Some(entry) = domain.accounts.get_mut(&account) else { return };
            let until = env.now.saturating_add(retry_after);
            entry.spent = Some(match entry.spent {
                Some(old) => old.max(until),
                None => until,
            });
            entry.spent_attention |= retry_after >= env.limits.spent_attention;
            changed(domain, env, account);
        }
    }
}
fn close(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    match domain.phase {
        Phase::Closing | Phase::Closed => return,
        Phase::New | Phase::Wanted => domain.phase = Phase::Closing,
    }
    let mut numbers = List::with_capacity(env.limits.accounts);
    for (number, _) in &domain.accounts {
        numbers.push(*number).expect("configured account count");
    }
    for number in &numbers {
        let entry = domain.accounts.get_mut(number).expect("configured account");
        entry.state = State::Closing;
        entry.grant = None;
        entry.retry = None;
        entry.spent = None;
        entry.expiring = false;
        entry.spent_attention = false;
        out.push(Request::Component(ToComponent::Release { account: *number }));
    }
    for number in &numbers {
        changed(domain, env, *number);
    }
    out.push(Request::Component(ToComponent::Close));
}
fn component(domain: &mut Domain, env: &Env<Limits>, event: FromComponent, out: &mut Queue<Request>) {
    let account = match event {
        FromComponent::Granted { account, generation, valid } => {
            let Some(entry) = domain.accounts.get_mut(&account) else { return };
            entry.grant = Some(Granted { generation, expires: env.now.saturating_add(valid) });
            entry.state = State::Granted;
            entry.retry = None;
            entry.expiring = false;
            entry.backoff = env.limits.backoff_base;
            out.push(Request::Granted { grant: Grant { account, generation, valid } });
            account
        }
        FromComponent::Expiring { account, generation } => {
            let Some(entry) = domain.accounts.get_mut(&account) else { return };
            if match entry.grant {
                Some(grant) => grant.generation != generation,
                None => true,
            } {
                return;
            }
            entry.expiring = true;
            account
        }
        FromComponent::Failed { account, failure } => {
            let Some(entry) = domain.accounts.get_mut(&account) else { return };
            failed(entry, env, failure);
            account
        }
        FromComponent::Closed => return,
    };
    changed(domain, env, account);
}
fn failed(entry: &mut Account, env: &Env<Limits>, failure: Failure) {
    entry.retry = None;
    entry.state = match failure {
        Failure::Later { after } => {
            let delay = match after {
                Some(delay) => delay,
                None => entry.backoff,
            };
            let until = env.now.saturating_add(delay);
            entry.retry = Some(until);
            entry.backoff = entry.backoff.saturating_mul(2).min(env.limits.backoff_max);
            State::Later { until }
        }
        Failure::SignInAgain => {
            entry.grant = None;
            entry.expiring = false;
            entry.retry = Some(env.now.saturating_add(env.limits.retry));
            State::SignInAgain
        }
        Failure::Unloaded { at, why } => {
            entry.grant = None;
            entry.expiring = false;
            entry.retry = Some(env.now.saturating_add(env.limits.retry));
            State::Unloaded { at, why }
        }
        Failure::Misconfigured => {
            entry.grant = None;
            entry.expiring = false;
            State::Misconfigured
        }
        Failure::Cancelled => {
            entry.grant = None;
            entry.expiring = false;
            State::Cancelled
        }
    };
}
/// One expired account, after component inputs so a grant arriving now wins.
pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(account) = domain.alarms.expire(env.now) else { return };
    let entry = domain.accounts.get_mut(&account).expect("alarm names a configured account");
    if match entry.spent {
        Some(until) => until <= env.now,
        None => false,
    } {
        entry.spent = None;
        entry.spent_attention = false;
    }
    if match entry.retry {
        Some(until) => until <= env.now,
        None => false,
    } {
        entry.retry = None;
        entry.state = State::Asking;
        out.push(Request::Component(ToComponent::Want { account }));
    }
    changed(domain, env, account);
}
#[expect(clippy::manual_map, reason = "the strict subset uses exhaustive matches without closures")]
fn fact(account: u32, entry: &Account) -> Fact {
    let attention = match entry.state {
        State::SignInAgain | State::Misconfigured | State::Unloaded { .. } => true,
        State::Asking | State::Granted | State::Later { .. } | State::Cancelled | State::Closing => false,
    };
    Fact {
        account,
        state: entry.state,
        generation: match entry.grant {
            Some(grant) => Some(grant.generation),
            None => None,
        },
        usable: entry.usable,
        expiring: entry.expiring,
        spent_until: entry.spent,
        attention: attention || entry.expiring || entry.spent_attention,
    }
}
#[expect(clippy::manual_map, reason = "the strict subset uses exhaustive matches without closures")]
fn changed(domain: &mut Domain, env: &Env<Limits>, account: u32) {
    let entry = domain.accounts.get_mut(&account).expect("event keeps its account");
    if let Some(until) = entry.spent
        && until <= env.now
    {
        entry.spent = None;
        entry.spent_attention = false;
    }
    entry.usable = match entry.grant {
        Some(grant) => grant.expires > env.now && entry.spent.is_none(),
        None => false,
    };
    let record = fact(account, entry);
    if domain.facts.try_push(record).is_err() {
        domain.lost = domain.lost.saturating_add(1);
    }
    let mut alarm = entry.retry;
    let expiry = match entry.grant {
        Some(grant) => Some(grant.expires),
        None => None,
    };
    for at in [expiry, entry.spent].into_iter().flatten() {
        if at > env.now {
            alarm = Some(match alarm {
                Some(old) => old.min(at),
                None => at,
            });
        }
    }
    match alarm {
        Some(at) => domain.alarms.arm(account, at).expect("one alarm per configured account"),
        None => domain.alarms.cancel(account),
    }
}
