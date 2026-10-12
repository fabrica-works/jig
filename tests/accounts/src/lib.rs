//! Tiny world scripting the accounts component on the driver's terms.
use jig_domain_accounts::{self as accounts, Domain, Event, Fact, Limits, Request};
use skein_lib::{Duration, Env, Queue, Time, Wall};
pub const ANALYST: u32 = 7;
pub const LIMITS: Limits = Limits {
    accounts: 2,
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
}
impl Default for World {
    fn default() -> World {
        World::configured(&[ANALYST])
    }
}
impl World {
    #[must_use]
    pub fn configured(accounts: &[u32]) -> World {
        World { domain: Domain::new(&LIMITS, accounts.into()), now: Time::ZERO }
    }
    #[must_use]
    pub fn wanting(accounts: &[u32]) -> World {
        let mut world = Self::configured(accounts);
        world.step(Event::Want);
        world
    }
    #[must_use]
    pub fn views(&self) -> Vec<Fact> {
        self.domain.account_views().as_slice().to_vec()
    }
    pub fn step(&mut self, event: Event) -> Vec<Request> {
        let mut out = Queue::with_capacity(accounts::max_out(&LIMITS));
        accounts::step(&mut self.domain, &Env { now: self.now, wall: Wall::EPOCH, limits: LIMITS }, event, &mut out);
        drain(&mut out)
    }
    pub fn fire_at(&mut self, seconds: u64) -> Vec<Request> {
        self.now = Time::ZERO.saturating_add(Duration::from_secs(seconds));
        let mut out = Queue::with_capacity(accounts::max_out(&LIMITS));
        accounts::fire(&mut self.domain, &Env { now: self.now, wall: Wall::EPOCH, limits: LIMITS }, &mut out);
        drain(&mut out)
    }
}
fn drain(out: &mut Queue<Request>) -> Vec<Request> {
    let mut result = Vec::new();
    while let Some(request) = out.pop() {
        result.push(request);
    }
    result
}
