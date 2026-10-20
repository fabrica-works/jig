//! Tiny world scripting the accounts component on the driver's terms.
use jig_domain_accounts::{
    self as accounts, Domain, Ended, Event, Fact, Holder, Kind, Limits, Offers, Provider, Providers, Request, Stored,
};
use skein_lib::{Duration, Env, Queue, Time, Wall};
pub const ANALYST: u32 = 7;
/// A holder keeps fewer accounts than the deployment, so that each bound is passed alone.
pub const LIMITS: Limits = Limits {
    accounts: 3,
    holder_accounts: 2,
    program_names: 12,
    program_hosts: 65,
    sources: 3,
    sign_ins: 1,
    credentials: 2,
    backoff_base: Duration::from_secs(2),
    backoff_max: Duration::from_secs(8),
    retry: Duration::from_secs(30),
    spent_attention: Duration::from_secs(30),
    facts: 16,
};
#[derive(Debug)]
pub struct World {
    pub domain: Domain,
    pub now: Time,
    /// The wall clock the child sees, for what the store keeps (engine.md, 5.4).
    pub wall: Wall,
}
impl Default for World {
    fn default() -> World {
        World::restoring(&[ANALYST])
    }
}
impl World {
    /// Explicit durable rows for the source-state stories.
    #[must_use]
    pub fn restoring(sources: &[u32]) -> World {
        let mut rows = Vec::new();
        for (place, number) in sources.iter().enumerate() {
            let mut row = account(1_000 + number, 1, Holder::Deployment, b"");
            if let Stored::Account { place: at, .. } = &mut row {
                *at = u32::try_from(place).expect("bounded source places");
            }
            rows.push(row);
            rows.push(source(1_000 + number, *number, 0, Kind::SignIn));
        }
        Self::with(providers(vec![provider(1, true, false)]), &rows)
    }
    #[must_use]
    pub fn with(providers: Providers, records: &[Stored]) -> World {
        let mut world = World { domain: Domain::new(&LIMITS, providers), now: Time::ZERO, wall: Wall::EPOCH };
        world.restore(records.to_vec());
        world
    }
    #[must_use]
    pub fn wanting(accounts: &[u32]) -> World {
        let mut world = Self::restoring(accounts);
        world.step(Event::Want);
        world
    }
    /// Restores each record, as the store gives them before the start.
    pub fn restore(&mut self, records: Vec<Stored>) {
        for record in records {
            assert_eq!(self.step(Event::Restore { record }), vec![]);
        }
    }
    #[must_use]
    pub fn views(&self) -> Vec<Fact> {
        self.domain.account_views().as_slice().to_vec()
    }
    pub fn step(&mut self, event: Event) -> Vec<Request> {
        let mut out = Queue::with_capacity(accounts::max_out(&LIMITS));
        accounts::step(&mut self.domain, &Env { now: self.now, wall: self.wall, limits: LIMITS }, event, &mut out);
        drain(&mut out)
    }
    /// Fires at a time, the wall clock moving with the monotonic one.
    pub fn fire_at(&mut self, seconds: u64) -> Vec<Request> {
        let now = Time::ZERO.saturating_add(Duration::from_secs(seconds));
        self.wall = Wall::from_nanos(self.wall.as_nanos().saturating_add(now.saturating_since(self.now).as_nanos()));
        self.now = now;
        let mut out = Queue::with_capacity(accounts::max_out(&LIMITS));
        accounts::fire(&mut self.domain, &Env { now: self.now, wall: self.wall, limits: LIMITS }, &mut out);
        drain(&mut out)
    }
    /// Every fact queued since the last call.
    pub fn facts(&mut self) -> Vec<Fact> {
        let mut result = Vec::new();
        while let Some(fact) = self.domain.pop_fact() {
            result.push(fact);
        }
        result
    }
    /// How each party's sign-in ended since the last call.
    pub fn ended(&mut self) -> Vec<Ended> {
        let mut result = Vec::new();
        while let Some(ended) = self.domain.pop_ended() {
            result.push(ended);
        }
        result
    }
}
/// A provider serving one model, offering a sign-in, a key, or neither: the `none` source.
#[must_use]
pub fn provider(number: u32, sign_in: bool, key: bool) -> Provider {
    Provider {
        name: b"Fake".as_slice().into(),
        signed_in_plan: true,
        manage_usage: Some(b"https://usage.example/".as_slice().into()),
        number,
        models: Box::new([Box::from(&b"model"[..])]),
        offers: Offers { sign_in, key, programs: Box::new([]) },
    }
}
#[must_use]
pub fn providers(list: Vec<Provider>) -> Providers {
    Providers { providers: list.into_boxed_slice() }
}
#[must_use]
pub fn account(number: u32, provider: u32, holder: Holder, subject: &[u8]) -> Stored {
    Stored::Account { place: 0, uses: None, number, provider, holder, subject: subject.into(), display: Box::new([]) }
}
#[must_use]
pub fn source(account: u32, number: u32, place: u32, kind: Kind) -> Stored {
    Stored::Source { account, number, place, kind, name: Box::new([]), held: None }
}
fn drain(out: &mut Queue<Request>) -> Vec<Request> {
    let mut result = Vec::new();
    while let Some(request) = out.pop() {
        result.push(request);
    }
    result
}
