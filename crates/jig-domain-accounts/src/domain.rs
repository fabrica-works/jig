use crate::{
    Configured, Event, Fact, Failure, FromComponent, Grant, GrantTable, Holder, Kind, Limits, Providers, Request,
    State, ToComponent, Turned,
};
use alloc::boxed::Box;
use skein_lib::{Deadlines, Duration, Env, List, Map, Queue, Time};

/// Want or release every signed-in source, at most one an account, with the component's close.
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    limits.accounts.checked_add(2).expect("account output bound")
}

/// An account (accounts.md, 3.1): its provider, its holder, the subject a signed-in source gave
/// it, the display name its holder gives it, and its sources' numbers by place, in its holder's
/// order. Its uses (4.2) join it when accounts are chosen by them.
#[derive(Hash, Debug)]
pub struct Account {
    pub(crate) provider: Option<u32>,
    pub(crate) holder: Holder,
    pub(crate) subject: Box<[u8]>,
    pub(crate) display: Box<[u8]>,
    pub(crate) sources: Map<u32, u32>,
}
impl Account {
    /// None only for a configured account whose model no configured provider serves (D1).
    #[must_use]
    pub const fn provider(&self) -> Option<u32> {
        self.provider
    }
    #[must_use]
    pub const fn holder(&self) -> Holder {
        self.holder
    }
    /// Empty when no signed-in source gave one.
    #[must_use]
    pub fn subject(&self) -> &[u8] {
        &self.subject
    }
    #[must_use]
    pub fn display(&self) -> &[u8] {
        &self.display
    }
    /// Its sources' numbers by place.
    #[must_use]
    pub const fn sources(&self) -> &Map<u32, u32> {
        &self.sources
    }
}

/// A source, with its own state (accounts.md, 3.5): what the component says of a signed-in
/// source's grant, and what runs' agents noticed of any source.
#[derive(PartialEq, Eq, Hash, Debug)]
pub(crate) struct Source {
    pub(crate) account: u32,
    pub(crate) kind: Kind,
    pub(crate) name: Box<[u8]>,
    state: State,
    grant: Option<Granted>,
    retry: Option<Time>,
    backoff: Duration,
    pub(crate) hold: Option<Hold>,
    unauthorized: Option<u64>,
    usable: bool,
    expiring: bool,
    pub(crate) hold_attention: bool,
}
impl Source {
    /// A signed-in source asks the component for its grant, a key waits for it, and a program
    /// waits for its logins; a `none` source holds nothing to wait for.
    pub(crate) fn new(account: u32, kind: Kind, name: Box<[u8]>, limits: &Limits) -> Source {
        Source {
            account,
            kind,
            name,
            state: match kind {
                Kind::None => State::Granted,
                Kind::SignIn | Kind::Key | Kind::Program => State::Asking,
            },
            grant: None,
            retry: None,
            backoff: limits.backoff_base,
            hold: None,
            unauthorized: None,
            usable: false,
            expiring: false,
            hold_attention: false,
        }
    }
}
/// What runs' agents noticed, over whatever the component says (domain/accounts.md, 3.5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Hold {
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
/// What an account's runs take: the first source by place that is usable; a program's not
/// held is passed over, being usable on some host alone (T12); any other unusable source holds
/// the account until its holder acts on it (accounts.md, 3.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Choice {
    Usable(u32),
    Held(u32),
    Empty,
}
#[derive(Debug)]
pub struct Domain {
    pub(crate) accounts: Map<u32, Account>,
    pub(crate) sources: Map<u32, Source>,
    /// The number the counter gives next: accounts and sources alike, never twice.
    pub(crate) next: u32,
    /// The configuration's, sized by it and outside the child's bound, as the core's settings are.
    pub(crate) providers: Providers,
    credentials: Map<u32, Granted>,
    credential_alarms: Deadlines<u32>,
    alarms: Deadlines<u32>,
    facts: Queue<Fact>,
    lost: u64,
    pub(crate) unrestored: u64,
    now: Time,
    phase: Phase,
}
impl Domain {
    /// Each configured account is the deployment's, with one signed-in source numbered as it
    /// is, so that its grants keep their number; the counter starts past them.
    #[must_use]
    pub fn new(limits: &Limits, providers: Providers, configured: Box<[Configured]>) -> Domain {
        assert!(crate::worst_case(limits).is_some(), "account limits are valid");
        let sources = limits.accounts.checked_mul(limits.sources).expect("checked by the worst case");
        let mut domain = Domain {
            accounts: Map::with_capacity(limits.accounts),
            sources: Map::with_capacity(sources),
            next: 0,
            providers,
            credentials: Map::with_capacity(limits.credentials),
            credential_alarms: Deadlines::with_capacity(limits.credentials),
            alarms: Deadlines::with_capacity(sources),
            facts: Queue::with_capacity(limits.facts),
            lost: 0,
            unrestored: 0,
            now: Time::ZERO,
            phase: Phase::New,
        };
        for entry in &configured {
            let mut places = Map::with_capacity(limits.sources);
            assert!(places.insert(0, entry.number) == Ok(None), "a first place");
            let account = Account {
                provider: entry.provider,
                holder: Holder::Deployment,
                subject: Box::new([]),
                display: Box::new([]),
                sources: places,
            };
            let fresh = match domain.accounts.insert(entry.number, account) {
                Ok(old) => old.is_none(),
                Err(_) => false,
            };
            assert!(fresh, "configured accounts fit and are distinct");
            let source = Source::new(entry.number, Kind::SignIn, Box::new([]), limits);
            assert!(domain.sources.insert(entry.number, source) == Ok(None), "configured sources fit");
            domain.past(entry.number);
        }
        domain
    }
    /// The counter passes a number given or seen.
    pub(crate) fn past(&mut self, number: u32) {
        self.next = self.next.max(number.saturating_add(1));
    }
    /// Numbers the counter has left to give; `u32::MAX` is never given.
    pub(crate) const fn numbers_left(&self) -> u32 {
        u32::MAX.saturating_sub(self.next)
    }
    fn take(&mut self) -> u32 {
        let number = self.next;
        self.next = self.next.checked_add(1).expect("a number checked left");
        number
    }
    #[must_use]
    pub fn account(&self, number: u32) -> Option<&Account> {
        self.accounts.get(&number)
    }
    /// A source's state, by the source's number.
    #[must_use]
    pub fn source(&self, number: u32) -> Option<Fact> {
        let entry = self.sources.get(&number)?;
        Some(fact(number, entry))
    }
    /// The number the counter gives next; the store keeps it (engine.md, 5.4).
    #[must_use]
    pub const fn next_number(&self) -> u32 {
        self.next
    }
    /// Records the store gave that broke a rule, and accounts left with no source, not restored.
    #[must_use]
    pub const fn unrestored(&self) -> u64 {
        self.unrestored
    }
    fn choose(&self, account: &Account) -> Choice {
        let mut first = None;
        for (_, number) in &account.sources {
            let entry = self.sources.get(number).expect("an account's sources are kept");
            if first.is_none() {
                first = Some(*number);
            }
            if entry.usable {
                return Choice::Usable(*number);
            }
            let passed = match entry.kind {
                Kind::Program => entry.hold.is_none(),
                Kind::SignIn | Kind::Key | Kind::None => false,
            };
            if !passed {
                return Choice::Held(*number);
            }
        }
        match first {
            Some(number) => Choice::Held(number),
            None => Choice::Empty,
        }
    }
    fn chosen(&self, account: u32) -> Option<u32> {
        match self.choose(self.accounts.get(&account)?) {
            Choice::Usable(source) => Some(source),
            Choice::Held(_) | Choice::Empty => None,
        }
    }
    /// Whether an account's runs have a source to take.
    #[must_use]
    pub fn usable(&self, account: u32) -> bool {
        self.chosen(account).is_some()
    }
    /// The grant of the source an account's runs take, named by the source's number; none for a
    /// `none` source, which holds nothing to grant.
    #[must_use]
    pub fn grant(&self, account: u32, now: Time) -> Option<Grant> {
        self.source_grant(self.chosen(account)?, now)
    }
    /// A source's grant, by the source's number, while it is usable and valid.
    #[must_use]
    pub fn source_grant(&self, source: u32, now: Time) -> Option<Grant> {
        let entry = self.sources.get(&source)?;
        let grant = entry.grant?;
        if !entry.usable || grant.expires <= now {
            return None;
        }
        Some(Grant {
            table: GrantTable::Accounts,
            account: source,
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
    /// One view per account: the state of the source its runs take, or of the one holding it.
    #[must_use]
    pub fn account_views(&self) -> List<Fact> {
        let mut result = List::with_capacity(self.accounts.capacity());
        for (_, account) in &self.accounts {
            let number = match self.choose(account) {
                Choice::Usable(number) | Choice::Held(number) => number,
                Choice::Empty => continue,
            };
            let entry = self.sources.get(&number).expect("an account's sources are kept");
            result.push(fact(number, entry)).expect("one view per account");
        }
        result
    }
    /// One view per source, by account and then by place.
    #[must_use]
    pub fn source_views(&self) -> List<Fact> {
        let mut result = List::with_capacity(self.sources.capacity());
        for (_, account) in &self.accounts {
            for (_, number) in &account.sources {
                let entry = self.sources.get(number).expect("an account's sources are kept");
                result.push(fact(*number, entry)).expect("one view per source");
            }
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
        Event::Restore { record } => {
            if domain.phase == Phase::New {
                crate::restore::restore(domain, env, record);
            }
        }
        Event::Want => {
            if domain.phase != Phase::New {
                return;
            }
            domain.phase = Phase::Wanted;
            crate::restore::finish(domain, env);
            none_accounts(domain, env);
            for (number, source) in &domain.sources {
                if source.kind == Kind::SignIn {
                    out.push(Request::Component(ToComponent::Want { account: *number }));
                }
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
            domain.sources = Map::with_capacity(domain.sources.capacity());
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
/// The deployment's account of each provider offering no source, made at the first start that
/// configures it (accounts.md, 3.1), while the accounts' bound and the counter leave room.
fn none_accounts(domain: &mut Domain, env: &Env<Limits>) {
    for index in 0..domain.providers.providers.len() {
        let provider = domain.providers.providers.get(index).expect("an index below the length");
        if !provider.offers.none() {
            continue;
        }
        let provider = provider.number;
        let mut held = false;
        for (_, account) in &domain.accounts {
            if account.holder == Holder::Deployment && account.provider == Some(provider) {
                for (_, number) in &account.sources {
                    held |= domain.sources.get(number).expect("an account's sources are kept").kind == Kind::None;
                }
            }
        }
        if held || domain.accounts.len() >= domain.accounts.capacity() || domain.numbers_left() < 2 {
            continue;
        }
        let account = domain.take();
        let source = domain.take();
        let mut places = Map::with_capacity(env.limits.sources);
        assert!(places.insert(0, source) == Ok(None), "a first place");
        let entry = Account {
            provider: Some(provider),
            holder: Holder::Deployment,
            subject: Box::new([]),
            display: Box::new([]),
            sources: places,
        };
        let fresh = match domain.accounts.insert(account, entry) {
            Ok(old) => old.is_none(),
            Err(_) => false,
        };
        assert!(fresh, "a new number, with room checked");
        let entry = Source::new(account, Kind::None, Box::new([]), &env.limits);
        assert!(domain.sources.insert(source, entry) == Ok(None), "a new number, within the sources' bound");
        changed(domain, env, source);
    }
}
/// The first unauthorized of a signed-in source's generation asks the component to renew it;
/// a second fails it. A key refused so is refused; a program's login is its host's to check.
fn unauthorized(domain: &mut Domain, env: &Env<Limits>, source: u32, generation: u64, out: &mut Queue<Request>) {
    let kind = match domain.sources.get(&source) {
        Some(entry) => entry.kind,
        None => return,
    };
    match kind {
        Kind::SignIn => {}
        Kind::Key => return refused(domain, env, source),
        Kind::Program | Kind::None => return,
    }
    let entry = domain.sources.get_mut(&source).expect("looked up above");
    if let Some(grant) = entry.grant
        && grant.generation > generation
    {
        return;
    }
    if entry.unauthorized != Some(generation) {
        entry.unauthorized = Some(generation);
        out.push(Request::Component(ToComponent::Rejected { account: source, generation }));
        return;
    }
    entry.grant = None;
    entry.expiring = false;
    entry.retry = None;
    entry.state = State::NeedsSignIn;
    changed(domain, env, source);
}
/// Limited until the later of its resets; a limit with none holds until resumed.
#[expect(clippy::manual_map, reason = "the strict subset uses exhaustive matches without closures")]
fn limited(domain: &mut Domain, env: &Env<Limits>, source: u32, reset: Option<Time>) {
    let Some(entry) = domain.sources.get_mut(&source) else { return };
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
    changed(domain, env, source);
}
fn refused(domain: &mut Domain, env: &Env<Limits>, source: u32) {
    let Some(entry) = domain.sources.get_mut(&source) else { return };
    entry.hold = Some(Hold::Refused);
    entry.hold_attention = true;
    changed(domain, env, source);
}
fn resume(domain: &mut Domain, env: &Env<Limits>, source: u32) {
    let Some(entry) = domain.sources.get_mut(&source) else { return };
    match entry.hold {
        Some(Hold::Limited { .. }) => {
            entry.hold = None;
            entry.hold_attention = false;
        }
        Some(Hold::Refused) | None => return,
    }
    changed(domain, env, source);
}
fn close(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    match domain.phase {
        Phase::Closing | Phase::Closed => return,
        Phase::New | Phase::Wanted => domain.phase = Phase::Closing,
    }
    domain.credentials = Map::with_capacity(env.limits.credentials);
    domain.credential_alarms = Deadlines::with_capacity(env.limits.credentials);
    let mut numbers = List::with_capacity(domain.sources.capacity());
    for (number, _) in &domain.sources {
        numbers.push(*number).expect("one per source");
    }
    for number in &numbers {
        let entry = domain.sources.get_mut(number).expect("a listed source");
        entry.state = State::Closing;
        entry.grant = None;
        entry.retry = None;
        entry.hold = None;
        entry.unauthorized = None;
        entry.expiring = false;
        entry.hold_attention = false;
        if entry.kind == Kind::SignIn {
            out.push(Request::Component(ToComponent::Release { account: *number }));
        }
    }
    for number in &numbers {
        changed(domain, env, *number);
    }
    out.push(Request::Component(ToComponent::Close));
}
/// The component serves signed-in sources; it names each by its number.
fn component(domain: &mut Domain, env: &Env<Limits>, event: FromComponent, out: &mut Queue<Request>) {
    let source = match event {
        FromComponent::Granted { account, .. }
        | FromComponent::Expiring { account, .. }
        | FromComponent::Failed { account, .. } => account,
        FromComponent::Closed => return,
    };
    let Some(entry) = domain.sources.get_mut(&source) else { return };
    if entry.kind != Kind::SignIn {
        return;
    }
    match event {
        FromComponent::Granted { account: _, generation, valid } => {
            entry.grant = Some(Granted { generation, expires: env.now.saturating_add(valid) });
            entry.state = State::Granted;
            entry.retry = None;
            entry.expiring = false;
            entry.backoff = env.limits.backoff_base;
            out.push(Request::Granted {
                grant: Grant { table: GrantTable::Accounts, account: source, generation, valid },
            });
        }
        FromComponent::Expiring { account: _, generation } => {
            if match entry.grant {
                Some(grant) => grant.generation != generation,
                None => true,
            } {
                return;
            }
            entry.expiring = true;
        }
        FromComponent::Failed { account: _, failure } => failed(entry, env, failure),
        FromComponent::Closed => return,
    }
    changed(domain, env, source);
}
fn failed(entry: &mut Source, env: &Env<Limits>, failure: Failure) {
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
/// One expired source, after component inputs so a grant arriving now wins.
pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    domain.now = env.now;
    if let Some(number) = domain.credential_alarms.expire(env.now) {
        domain.credentials.remove(&number).expect("credential alarm names a grant");
        return;
    }
    let Some(source) = domain.alarms.expire(env.now) else { return };
    let entry = domain.sources.get_mut(&source).expect("alarm names a kept source");
    if match entry.retry {
        Some(until) => until <= env.now,
        None => false,
    } {
        entry.retry = None;
        entry.state = State::Asking;
        out.push(Request::Component(ToComponent::Want { account: source }));
    }
    changed(domain, env, source);
}
#[expect(clippy::manual_map, reason = "the strict subset uses exhaustive matches without closures")]
fn fact(number: u32, entry: &Source) -> Fact {
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
        account: entry.account,
        source: number,
        kind: entry.kind,
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
/// A source changed: it settles and its fact is queued.
fn changed(domain: &mut Domain, env: &Env<Limits>, source: u32) {
    let record = settle(domain, env, source);
    if domain.facts.try_push(record).is_err() {
        domain.lost = domain.lost.saturating_add(1);
    }
}
/// A passed limit lifts, a source's usability follows its kind and its one alarm is rearmed;
/// its fact is returned.
#[expect(clippy::manual_map, reason = "the strict subset uses exhaustive matches without closures")]
pub(crate) fn settle(domain: &mut Domain, env: &Env<Limits>, source: u32) -> Fact {
    let entry = domain.sources.get_mut(&source).expect("event keeps its source");
    if let Some(Hold::Limited { until: Some(until) }) = entry.hold
        && until <= env.now
    {
        entry.hold = None;
        entry.hold_attention = false;
    }
    entry.usable = entry.hold.is_none()
        && match entry.kind {
            Kind::SignIn | Kind::Key => match entry.grant {
                Some(grant) => grant.expires > env.now,
                None => false,
            },
            Kind::None => entry.state == State::Granted,
            Kind::Program => false,
        };
    let record = fact(source, entry);
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
        Some(at) => domain.alarms.arm(source, at).expect("one alarm per source"),
        None => domain.alarms.cancel(source),
    }
    record
}
