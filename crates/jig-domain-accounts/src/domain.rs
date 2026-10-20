use crate::{Event, Fact, Failure, FromComponent, Grant, GrantTable, Limits, Request, State, ToComponent, Turned};
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
    hold: Option<Hold>,
    unauthorized: Option<u64>,
    usable: bool,
    expiring: bool,
    hold_attention: bool,
}
/// What runs' agents noticed, over whatever the component says (domain/accounts.md, 3.5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Hold {
    Limited { until: Option<Time> },
    Refused,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Granted {
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
    credentials: Map<u32, Granted>,
    credential_alarms: Deadlines<u32>,
    alarms: Deadlines<u32>,
    facts: Queue<Fact>,
    lost: u64,
    now: Time,
    phase: Phase,
}
impl Domain {
    #[must_use]
    pub fn new(limits: &Limits, accounts: Box<[u32]>) -> Domain {
        assert!(crate::worst_case(limits).is_some(), "account limits are valid");
        let mut domain = Domain {
            accounts: Map::with_capacity(limits.accounts),
            credentials: Map::with_capacity(limits.credentials),
            credential_alarms: Deadlines::with_capacity(limits.credentials),
            alarms: Deadlines::with_capacity(limits.accounts),
            facts: Queue::with_capacity(limits.facts),
            lost: 0,
            now: Time::ZERO,
            phase: Phase::New,
        };
        for number in &accounts {
            let entry = Account {
                state: State::Asking,
                grant: None,
                retry: None,
                backoff: limits.backoff_base,
                hold: None,
                unauthorized: None,
                usable: false,
                expiring: false,
                hold_attention: false,
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
        Some(Grant {
            table: GrantTable::Accounts,
            account,
            generation: grant.generation,
            valid: grant.expires.saturating_since(now),
        })
    }
    #[must_use]
    pub fn credential_usable(&self, number: u32) -> bool {
        match self.credentials.get(&number) {
            Some(grant) => grant.expires > self.now,
            None => false,
        }
    }
    #[must_use]
    pub fn credential_limit(&self) -> u32 {
        self.credentials.capacity()
    }
    #[must_use]
    pub fn credential_grant(&self, number: u32, now: Time) -> Option<Grant> {
        let grant = self.credentials.get(&number)?;
        if grant.expires <= now {
            return None;
        }
        Some(Grant {
            table: GrantTable::Application,
            account: number,
            generation: grant.generation,
            valid: grant.expires.saturating_since(now),
        })
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        match self.alarms.next() {
            Some(a) => match self.credential_alarms.next() {
                Some(b) => Some(a.min(b)),
                None => Some(a),
            },
            None => self.credential_alarms.next(),
        }
    }
    #[must_use]
    pub fn is_due(&self, now: Time) -> bool {
        match self.next_deadline() {
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
    domain.now = env.now;
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
        Event::Credential { number, generation, valid } => {
            match domain.phase {
                Phase::Closing | Phase::Closed => return,
                Phase::New | Phase::Wanted => {}
            }
            let expires = env.now.saturating_add(valid);
            if domain.credentials.insert(number, Granted { generation, expires }).is_err() {
                return;
            }
            domain.credential_alarms.arm(number, expires).expect("credential alarm room");
            out.push(Request::Granted {
                grant: Grant { table: GrantTable::Application, account: number, generation, valid },
            });
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
        Event::Unauthorized { account, generation } => {
            if domain.phase == Phase::Wanted {
                unauthorized(domain, env, account, generation, out);
            }
        }
        Event::Limited { account, reset } => {
            if domain.phase == Phase::Wanted {
                limited(domain, env, account, reset);
            }
        }
        Event::Refused { account, code: _, reason: _ } => {
            if domain.phase == Phase::Wanted {
                refused(domain, env, account);
            }
        }
        Event::RunFailed { account, turned } => {
            if domain.phase != Phase::Wanted {
                return;
            }
            match turned {
                Turned::Limited { reset } => limited(domain, env, account, reset),
                Turned::Refused => refused(domain, env, account),
            }
        }
        Event::Resume { account } => {
            if domain.phase == Phase::Wanted {
                resume(domain, env, account);
            }
        }
    }
}
/// The first unauthorized of a generation asks the component to renew it; a second fails it.
fn unauthorized(domain: &mut Domain, env: &Env<Limits>, account: u32, generation: u64, out: &mut Queue<Request>) {
    let Some(entry) = domain.accounts.get_mut(&account) else { return };
    if let Some(grant) = entry.grant
        && grant.generation > generation
    {
        return;
    }
    if entry.unauthorized != Some(generation) {
        entry.unauthorized = Some(generation);
        out.push(Request::Component(ToComponent::Rejected { account, generation }));
        return;
    }
    entry.grant = None;
    entry.expiring = false;
    entry.retry = None;
    entry.state = State::NeedsSignIn;
    changed(domain, env, account);
}
/// Limited until the later of its resets; a limit with none holds until resumed.
#[expect(clippy::manual_map, reason = "the strict subset uses exhaustive matches without closures")]
fn limited(domain: &mut Domain, env: &Env<Limits>, account: u32, reset: Option<Time>) {
    let Some(entry) = domain.accounts.get_mut(&account) else { return };
    let until = match entry.hold {
        Some(Hold::Refused) => return,
        Some(Hold::Limited { until: None }) => None,
        Some(Hold::Limited { until: Some(old) }) => match reset {
            Some(new) => Some(old.max(new)),
            None => None,
        },
        None => reset,
    };
    entry.hold = Some(Hold::Limited { until });
    entry.hold_attention |= match until {
        Some(until) => until.saturating_since(env.now) >= env.limits.spent_attention,
        None => true,
    };
    changed(domain, env, account);
}
fn refused(domain: &mut Domain, env: &Env<Limits>, account: u32) {
    let Some(entry) = domain.accounts.get_mut(&account) else { return };
    entry.hold = Some(Hold::Refused);
    entry.hold_attention = true;
    changed(domain, env, account);
}
fn resume(domain: &mut Domain, env: &Env<Limits>, account: u32) {
    let Some(entry) = domain.accounts.get_mut(&account) else { return };
    match entry.hold {
        Some(Hold::Limited { .. }) => {
            entry.hold = None;
            entry.hold_attention = false;
        }
        Some(Hold::Refused) | None => return,
    }
    changed(domain, env, account);
}
fn close(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    match domain.phase {
        Phase::Closing | Phase::Closed => return,
        Phase::New | Phase::Wanted => domain.phase = Phase::Closing,
    }
    domain.credentials = Map::with_capacity(env.limits.credentials);
    domain.credential_alarms = Deadlines::with_capacity(env.limits.credentials);
    let mut numbers = List::with_capacity(env.limits.accounts);
    for (number, _) in &domain.accounts {
        numbers.push(*number).expect("configured account count");
    }
    for number in &numbers {
        let entry = domain.accounts.get_mut(number).expect("configured account");
        entry.state = State::Closing;
        entry.grant = None;
        entry.retry = None;
        entry.hold = None;
        entry.unauthorized = None;
        entry.expiring = false;
        entry.hold_attention = false;
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
            out.push(Request::Granted { grant: Grant { table: GrantTable::Accounts, account, generation, valid } });
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
            State::NeedsSignIn
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
    domain.now = env.now;
    if let Some(number) = domain.credential_alarms.expire(env.now) {
        domain.credentials.remove(&number).expect("credential alarm names a grant");
        return;
    }
    let Some(account) = domain.alarms.expire(env.now) else { return };
    let entry = domain.accounts.get_mut(&account).expect("alarm names a configured account");
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
    let held = match entry.hold {
        Some(Hold::Limited { until }) => State::Limited { until },
        Some(Hold::Refused) => State::Refused,
        None => entry.state,
    };
    let state = match entry.state {
        State::Asking | State::Granted | State::Later { .. } => held,
        State::Limited { .. }
        | State::Refused
        | State::NeedsSignIn
        | State::Misconfigured
        | State::Unloaded { .. }
        | State::Cancelled
        | State::Closing => entry.state,
    };
    let attention = match state {
        State::NeedsSignIn | State::Misconfigured | State::Unloaded { .. } => true,
        State::Asking
        | State::Granted
        | State::Later { .. }
        | State::Limited { .. }
        | State::Refused
        | State::Cancelled
        | State::Closing => false,
    };
    Fact {
        account,
        state,
        generation: match entry.grant {
            Some(grant) => Some(grant.generation),
            None => None,
        },
        usable: entry.usable,
        expiring: entry.expiring,
        spent_until: match entry.hold {
            Some(Hold::Limited { until }) => until,
            Some(Hold::Refused) | None => None,
        },
        attention: attention || entry.expiring || entry.hold_attention,
    }
}
#[expect(clippy::manual_map, reason = "the strict subset uses exhaustive matches without closures")]
fn changed(domain: &mut Domain, env: &Env<Limits>, account: u32) {
    let entry = domain.accounts.get_mut(&account).expect("event keeps its account");
    if let Some(Hold::Limited { until: Some(until) }) = entry.hold
        && until <= env.now
    {
        entry.hold = None;
        entry.hold_attention = false;
    }
    entry.usable = match entry.grant {
        Some(grant) => grant.expires > env.now && entry.hold.is_none(),
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
    let reset = match entry.hold {
        Some(Hold::Limited { until }) => until,
        Some(Hold::Refused) | None => None,
    };
    for at in [expiry, reset].into_iter().flatten() {
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
