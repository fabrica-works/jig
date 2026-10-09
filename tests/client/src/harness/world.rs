//! Tabs retain records and history; epochs discard old deliveries. Each callback
//! writes its store, dispatches wire duties, then applies and checks patches
//! (client/domain/testing.md, 4; client/domain/view.md, 11).
use super::{App, Duty, Front, FrontFault, StoreWrite};
use crate::model::Model;
use jig_client_view::{Act, NodeId, Tree};
use jig_fake_person::{Doing, Next, Person, RandomDoing, RandomPerson, Step, TreeFace};
use skein_lib::{Duration, Env, Queue, Rng, Time, Wall};
use skein_world::domain::{Referee, Schedule, Span, Trace, Verdict};

/// One tab or device in this world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClientId(pub usize);
/// One scripted or random person.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PersonId(pub usize);
/// Deterministic latency, storage and iteration bounds.
#[derive(Clone, Debug)]
pub struct Settings {
    pub seed: u64,
    pub latency: Span,
    pub store_records: usize,
    pub iterations: u32,
    pub memory: bool,
    pub random_steps: u32,
    pub word_bytes: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            seed: 1,
            latency: Span::millis(1, 3),
            store_records: 32,
            iterations: 4096,
            memory: false,
            random_steps: 32,
            word_bytes: 32,
        }
    }
}
/// The boundary at which a run yields its evidence.
#[derive(Clone, Copy, Debug)]
pub enum Until {
    Done,
    Time(Time),
    Steps(u32),
}
/// Clock, callback and heap evidence from a completed run.
#[derive(Debug)]
pub struct Outcome {
    /// Observation-based referee result at this run boundary.
    pub verdict: Verdict,
    /// Safety checks and liveness obligations met.
    pub judged: (u64, u64),
    pub now: Time,
    pub iterations: u32,
    pub callbacks: u64,
    pub dropped: u64,
    pub trace: Vec<String>,
    pub heap_peak: u64,
    pub heap_bound: u64,
}
struct Client<A: App> {
    root: A::Root,
    view: A::View,
    model: Model,
    kept: Vec<A::Record>,
    deferred: Vec<StoreWrite<A::Record, A::Key>>,
    history: Vec<A::Address>,
    at: usize,
    party: u64,
    epoch: u64,
}
enum PersonKind<A: App> {
    Script(Person<A::Address>),
    Random { person: RandomPerson, left: u32 },
}
struct Walker<A: App> {
    client: ClientId,
    person: PersonKind<A>,
    done: bool,
    wake: Option<Time>,
}
struct Due<A: App> {
    client: ClientId,
    epoch: u64,
    event: A::Event,
}
/// One deterministic loop shared by client, joint and application worlds.
pub struct World<A: App, F: Front<A>> {
    settings: Settings,
    front: F,
    clients: Vec<Client<A>>,
    people: Vec<Walker<A>>,
    schedule: Schedule<Due<A>>,
    rng: Rng,
    trace: Trace,
    referee: Referee<super::referee::Rules>,
    now: Time,
    callbacks: u64,
    dropped: u64,
    quota_failure: bool,
    meter: Option<skein_world::domain::heap::Meter>,
    heap_peak: u64,
    heap_bound: u64,
}
impl<A: App, F: Front<A>> World<A, F> {
    /// Creates one seeded loop in front of the supplied engine boundary.
    #[must_use]
    pub fn new(settings: Settings, front: F) -> Self {
        let heap_bound = A::worst_case(&A::limits());
        let meter = if settings.memory { Some(skein_world::domain::heap::Meter::new()) } else { None };
        Self {
            meter,
            heap_bound,
            heap_peak: 0,
            rng: Rng::new(settings.seed),
            settings,
            front,
            clients: Vec::new(),
            people: Vec::new(),
            schedule: Schedule::new(),
            trace: Trace::default(),
            referee: Referee::new(super::referee::Rules::default()),
            now: Time::ZERO,
            callbacks: 0,
            dropped: 0,
            quota_failure: false,
        }
    }
    /// Starts a device with an empty small store.
    pub fn client(&mut self, party: u64, address: A::Address) -> ClientId {
        self.make_client(party, address, Vec::new())
    }
    fn make_client(&mut self, party: u64, address: A::Address, kept: Vec<A::Record>) -> ClientId {
        if let Some(meter) = &self.meter {
            meter.start();
        }
        let (root, view) = A::new(self.rng.next_u64());
        let id = ClientId(self.clients.len());
        self.clients.push(Client {
            root,
            view,
            model: Model::default(),
            kept: kept.clone(),
            deferred: Vec::new(),
            history: vec![address.clone()],
            at: 0,
            party,
            epoch: 0,
        });
        self.front.bind(id, party);
        self.callback(id, Some(A::start(address, kept, 0)));
        self.check_heap();
        id
    }
    /// Starts another tab with this tab’s address and kept records.
    pub fn tab(&mut self, of: ClientId) -> ClientId {
        let c = &self.clients[of.0];
        self.make_client(c.party, c.history[c.at].clone(), c.kept.clone())
    }
    /// Adds a person whose script acts only through the visible tree.
    pub fn person(&mut self, client: ClientId, steps: Vec<Step<A::Address>>) -> PersonId {
        let id = PersonId(self.people.len());
        self.people.push(Walker {
            client,
            person: PersonKind::Script(Person::new(steps.into_boxed_slice())),
            done: false,
            wake: None,
        });
        id
    }
    /// Adds a seeded person with a bounded number of actions.
    pub fn random_person(&mut self, client: ClientId) -> PersonId {
        let id = PersonId(self.people.len());
        self.people.push(Walker {
            client,
            person: PersonKind::Random {
                person: RandomPerson::new(self.rng.next_u64(), self.settings.word_bytes),
                left: self.settings.random_steps,
            },
            done: false,
            wake: None,
        });
        id
    }
    /// Restarts this tab from its kept records and advances its delivery epoch.
    pub fn reload(&mut self, id: ClientId) {
        self.front.reload(id, self.now);
        let c = &mut self.clients[id.0];
        c.epoch += 1;
        let (root, view) = A::new(self.rng.next_u64());
        c.root = root;
        c.view = view;
        c.model = Model::default();
        c.deferred.clear();
        let event = A::start(c.history[c.at].clone(), c.kept.clone(), 0);
        self.callback(id, Some(event));
    }
    /// Navigates and records an address in the shell history.
    pub fn go(&mut self, id: ClientId, address: A::Address) {
        self.history(id, address.clone(), false);
        self.callback(id, Some(A::went(address)));
    }
    /// Moves to the previous history entry when present.
    pub fn back(&mut self, id: ClientId) {
        if self.clients[id.0].at > 0 {
            self.clients[id.0].at -= 1;
            let address = self.clients[id.0].history[self.clients[id.0].at].clone();
            self.callback(id, Some(A::went(address)));
        }
    }
    /// Moves to the next history entry when present.
    pub fn forward(&mut self, id: ClientId) {
        if self.clients[id.0].at + 1 < self.clients[id.0].history.len() {
            self.clients[id.0].at += 1;
            let address = self.clients[id.0].history[self.clients[id.0].at].clone();
            self.callback(id, Some(A::went(address)));
        }
    }
    /// Reports a platform act and mirrors a local field edit before diffing.
    pub fn act(&mut self, id: ClientId, node: NodeId, act: Act) {
        if let Act::Input { value } = &act {
            self.clients[id.0].model.input(node, value);
        }
        if let Some(event) = A::act(&mut self.clients[id.0].view, node, act) {
            self.callback(id, Some(event));
        }
    }
    /// Delivers a shell event through the same callback path.
    pub fn event(&mut self, id: ClientId, event: A::Event) {
        self.callback(id, Some(event));
    }
    /// Makes the next callback’s store writes fail before dispatch.
    pub fn fail_store(&mut self) {
        self.quota_failure = true;
    }
    /// Injects a transport fault into the front.
    pub fn fault(&mut self, fault: FrontFault) {
        self.front.fault(self.now, fault);
    }
    /// Exposes the truth boundary for fixture changes.
    pub fn front(&mut self) -> &mut F {
        &mut self.front
    }
    /// Returns the visible accessibility tree.
    #[must_use]
    pub fn tree(&self, id: ClientId) -> &Tree {
        A::tree(&self.clients[id.0].view)
    }
    /// Returns the records that survived successful store writes.
    #[must_use]
    pub fn kept(&self, id: ClientId) -> &[A::Record] {
        &self.clients[id.0].kept
    }
    /// Returns the loop’s monotonic time.
    #[must_use]
    pub fn now(&self) -> Time {
        self.now
    }
    fn history(&mut self, id: ClientId, address: A::Address, replace: bool) {
        let c = &mut self.clients[id.0];
        if replace {
            c.history[c.at] = address;
        } else if c.history[c.at] != address {
            c.history.truncate(c.at + 1);
            c.history.push(address);
            c.at += 1;
        }
    }
    fn callback(&mut self, id: ClientId, event: Option<A::Event>) {
        let env =
            Env { now: self.now, wall: Wall::from_nanos(1_000_000_000_000 + self.now.as_nanos()), limits: A::limits() };
        let mut out = Queue::with_capacity(A::max_out(&env.limits));
        let c = &mut self.clients[id.0];
        match event {
            Some(event) => A::step(&mut c.root, &env, event, &mut out),
            None => A::fire(&mut c.root, &env, &mut out),
        }
        A::take(&mut c.root, &env, &mut out);
        let mut wires = Vec::new();
        let mut shows = Vec::new();
        let mut signs = Vec::new();
        let mut failed = self.quota_failure;
        self.quota_failure = false;
        while let Some(request) = out.pop() {
            match A::duty(request) {
                Duty::Store(write) => c.deferred.push(write),
                Duty::Wire(request) => wires.push(request),
                Duty::Show { address, replace } => shows.push((address, replace)),
                Duty::SignIn { provider: _, back } => signs.push(back),
                Duty::Fact => {}
            }
        }
        let writes = std::mem::take(&mut c.deferred);
        for write in writes {
            if failed {
                c.deferred.push(write);
                continue;
            }
            match write {
                StoreWrite::Save(record) => {
                    let key = A::record_key(&record);
                    if let Some(old) = c.kept.iter_mut().find(|old| A::record_key(old) == key) {
                        *old = record;
                    } else if c.kept.len() < self.settings.store_records {
                        c.kept.push(record);
                    } else {
                        failed = true;
                        c.deferred.push(StoreWrite::Save(record));
                    }
                }
                StoreWrite::Erase(key) => c.kept.retain(|record| A::record_key(record) != key),
            }
        }
        for request in wires {
            if failed {
                if let Some(event) = A::unsent(request) {
                    self.schedule.send(
                        self.now.saturating_add(Duration::from_nanos(1)),
                        Due { client: id, epoch: self.clients[id.0].epoch, event },
                    );
                }
            } else {
                self.front.wire(id, self.now, env.wall, request);
            }
        }
        for (address, replace) in shows {
            self.history(id, address, replace);
        }
        for back in signs {
            self.front.sign_in(id, self.now, env.wall);
            self.schedule.send(
                self.now.saturating_add(self.settings.latency.draw(&mut self.rng)),
                Due { client: id, epoch: self.clients[id.0].epoch, event: A::went(back) },
            );
        }
        let c = &mut self.clients[id.0];
        let mut patches = Queue::with_capacity(A::patch_limit(&c.view));
        A::build(&mut c.view, &c.root, &mut patches);
        while let Some(patch) = patches.pop() {
            c.model.apply(patch);
        }
        c.model.check(A::tree(&c.view));
        self.callbacks += 1;
        self.trace.log(self.now, format!("client {} epoch {} tree {}", id.0, c.epoch, A::tree(&c.view).len()));
    }
    fn check_heap(&mut self) {
        if let Some(meter) = &self.meter {
            let measured = meter.end();
            let bound = self
                .heap_bound
                .checked_mul(u64::try_from(self.clients.len()).expect("client count fits"))
                .expect("world heap bound fits");
            self.heap_peak = self.heap_peak.max(meter.check(measured, bound, &"client world iteration"));
        }
    }
    fn walk_people(&mut self) -> bool {
        let mut progress = false;
        for index in 0..self.people.len() {
            if self.people[index].done {
                continue;
            }
            let id = self.people[index].client;
            let next = match &mut self.people[index].person {
                PersonKind::Script(person) => {
                    let before = person.at();
                    let next = person.next(&mut TreeFace::new(A::tree(&self.clients[id.0].view)), self.now);
                    progress |= before != person.at();
                    next
                }
                PersonKind::Random { person, left } => {
                    if *left == 0 {
                        Next::Done
                    } else {
                        *left -= 1;
                        progress = true;
                        match person.next(A::tree(&self.clients[id.0].view)) {
                            Some(RandomDoing::Act { node, act }) => Next::Do(Doing::Act { node, act }),
                            Some(RandomDoing::Reload) => Next::Do(Doing::Reload),
                            Some(RandomDoing::Back) => {
                                self.back(id);
                                Next::Wait { until: None }
                            }
                            None => Next::Wait { until: None },
                        }
                    }
                }
            };
            match next {
                Next::Do(Doing::Act { node, act }) => self.act(id, node, act),
                Next::Do(Doing::Go { address }) => self.go(id, address),
                Next::Do(Doing::Reload) => self.reload(id),
                Next::Wait { until } => self.people[index].wake = until,
                Next::Done => self.people[index].done = true,
                Next::Failed { step, why } => panic!(
                    "seed {} client {} step {step}: {why}; tree {:?}",
                    self.settings.seed,
                    id.0,
                    A::tree(&self.clients[id.0].view)
                        .nodes()
                        .iter()
                        .filter_map(|n| n.name.as_ref().map(|s| String::from_utf8_lossy(s)))
                        .collect::<Vec<_>>()
                ),
            }
        }
        progress
    }
    /// Runs callbacks, people and deadlines until the requested boundary.
    pub fn run(&mut self, until: Until) -> Outcome {
        let mut iterations = 0;
        for _ in 0..self.settings.iterations {
            if let Some(meter) = &self.meter {
                meter.start();
            }
            iterations += 1;
            let wall = Wall::from_nanos(1_000_000_000_000 + self.now.as_nanos());
            for (client, event) in self.front.poll(self.now, wall) {
                self.schedule.send(
                    self.now.saturating_add(self.settings.latency.draw(&mut self.rng)),
                    Due { client, epoch: self.clients[client.0].epoch, event },
                );
            }
            for observed in self.front.observed() {
                self.referee.observe(self.now, observed, &mut Vec::new());
            }
            self.referee.fire(self.now, &mut Vec::new());
            self.referee.assert_holding(self.settings.seed);
            while let Some(due) = self.schedule.next(self.now) {
                if due.epoch == self.clients[due.client.0].epoch {
                    self.front.received(due.client, &due.event);
                    self.callback(due.client, Some(due.event));
                } else {
                    self.dropped += 1;
                }
            }
            for index in 0..self.clients.len() {
                if A::next_deadline(&self.clients[index].root).is_some_and(|at| at <= self.now) {
                    self.callback(ClientId(index), None);
                }
            }
            let progress = self.walk_people();
            let done = self.people.iter().all(|person| person.done);
            let stop = match until {
                Until::Done => done && self.schedule.is_empty(),
                Until::Time(at) => self.now >= at,
                Until::Steps(count) => iterations >= count,
            };
            self.check_heap();
            if stop {
                break;
            }
            if progress {
                self.now = self.now.saturating_add(Duration::from_nanos(1));
                continue;
            }
            let next = [
                self.schedule.next_time(),
                self.front.next_due(),
                self.clients.iter().filter_map(|c| A::next_deadline(&c.root)).min(),
                self.people.iter().filter(|p| !p.done).filter_map(|p| p.wake).min(),
            ]
            .into_iter()
            .flatten()
            .min();
            self.now = next.map_or(self.now.saturating_add(Duration::from_millis(1)), |at| {
                at.max(self.now.saturating_add(Duration::from_nanos(1)))
            });
        }
        assert!(
            iterations < self.settings.iterations,
            "seed {} exceeded iteration bound at {:?}, walkers {:?}, tree {:?}",
            self.settings.seed,
            self.now,
            self.people
                .iter()
                .map(|p| match &p.person {
                    PersonKind::Script(p) => p.at(),
                    PersonKind::Random { left, .. } => *left,
                })
                .collect::<Vec<_>>(),
            self.clients
                .iter()
                .map(|c| A::tree(&c.view)
                    .nodes()
                    .iter()
                    .filter_map(|n| n.name.as_ref().map(|s| String::from_utf8_lossy(s)))
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>()
        );
        Outcome {
            verdict: self.referee.verdict(),
            judged: self.referee.judged(),
            now: self.now,
            iterations,
            callbacks: self.callbacks,
            dropped: self.dropped,
            trace: self.trace.lines().to_vec(),
            heap_peak: self.heap_peak,
            heap_bound: self.heap_bound * u64::try_from(self.clients.len()).expect("client count fits"),
        }
    }
}
