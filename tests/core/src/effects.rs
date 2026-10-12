//! A scripted caller and two independent systems exercise the root boundary.
use crate::Store;
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use jig_test_system::{Fault, System};
use skein_lib::{Duration, Env, List, Queue, ReplyTo, Time, Token, Wall};
use std::collections::VecDeque;

/// A scenario's cut around one store transaction, before application or reply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cut {
    /// Submission has left the root but is not durable.
    Submitted(u64),
    /// The rows are durable but the root has not received the completion.
    Durable(u64),
}

/// Root, store and separate systems, with an answer channel that may disappear.
#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independently held peer traffic and crash cuts")]
pub struct World {
    pub domain: root::Domain,
    pub store: Store,
    pub systems: [System; 2],
    protocol: crate::protocol::Stand,
    pub host_calls: Vec<(Token, u64, u64, fleet::Call)>,
    pub host_answers: Vec<(Box<[u8]>, core::SettledCall)>,
    pub inbound: Vec<tasks::Word>,
    pub answers: Vec<(Token, core::CallKey, core::CallPart)>,
    pub assigned: Option<root::Assignment>,
    pub assignments: Vec<(u64, u64)>,
    pub cancellations: Vec<(u64, u64)>,
    pub busy_turns: Vec<(u64, u64, u32)>,
    pub busy_calls: Vec<Box<[u8]>>,
    pub busy_events: Vec<root::Event>,
    pub stop_deliveries: Vec<core::Held>,
    pub commits: Vec<Vec<root::Write>>,
    pub projections: Vec<(usize, u16, core::ProjectionFeed)>,
    pub hold_projection_settlements: bool,
    pub sign_in: Option<u64>,
    pub people_answers: Vec<people::Reply>,
    pub application_answers: Vec<(Token, u16, connector::ApplicationOutcome)>,
    pub procedure: Option<u64>,
    /// Whole held protocol inputs received by the independent procedure connector.
    pub procedure_inputs: Vec<(u64, u16, u16, Box<tasks::RunContext>)>,
    /// A procedure connector can withhold its reply while a story drives its step.
    pub hold_procedures: bool,
    pub pending_reads: Vec<(u16, connector::SystemRequest)>,
    pub lost_answers: u32,
    pub hold_writes: bool,
    pub recovery: connector::Recovery,
    pub hold_lookups: bool,
    pub pending_lookups: Vec<(u16, connector::SystemRequest)>,
    pub pending_writes: Vec<(u16, connector::SystemRequest)>,
    pub lose_answer: bool,
    pub trace: Vec<String>,
    pub restart_steps: Vec<core::RestartStep>,
    pub restore_ranges: Vec<core::RestoreRange>,
    pub hold_restart_reads: bool,
    /// Withhold requested whole store pages during a real cold restore.
    pub hold_restore_pages: bool,
    /// Delay before a submitted store transaction becomes durable.
    pub store_delay: u32,
    /// Whether the root reported a failed commit and stopped.
    pub stopped: bool,
    /// Independent scripted hosts and clients, when this scenario uses them.
    pub peers: Option<crate::peers::Peers>,
    /// Client-facing outputs; their protocol adapter runs in the joint world.
    pub client_peer: Option<crate::client_peer::ClientPeer>,
    /// Whole applied store snapshots, enabled only for external durable observers.
    pub store_peer: Option<crate::store_peer::StorePeer>,
    /// Independent document consumers, enabled only in watch stories.
    pub watches: Option<crate::peers::WatchParties>,
    /// Independent offer observer, enabled for immediately acting scripted parties.
    pub offers: Option<crate::referee::offers::Referee>,
    /// Independent read parties and owner-range store transport, enabled in read stories.
    pub reads: Option<crate::reads::Parties>,
    /// Referee observations, independent of the root state.
    pub observer: crate::observations::Observer,
    heap: crate::observations::Heap,
    restoring: Option<Restore>,
    restart_reads: std::collections::BTreeMap<u16, core::RestartStep>,
    events: VecDeque<root::Event>,
    limits: root::Limits,
    /// Pause at a selected transaction boundary, with its traffic retained.
    pub cut: Option<Cut>,
    /// The boundary at which the last drain paused.
    pub reached_cut: Option<Cut>,
    hold_kinds: Vec<(u16, Box<[tasks::Kind]>)>,
    wall: u64,
    wall_lag: u64,
    supplied_wall: Option<Wall>,
}

#[derive(Debug)]
struct Restore {
    step: core::RestartStep,
    first: root::Key,
    last: root::Key,
    rows: Vec<root::Record>,
    remaining: std::collections::VecDeque<(root::Key, root::Key)>,
}

fn grant() -> authority::Grant {
    authority::Grant {
        connector: 1,
        kind: 5,
        pattern: authority::Pattern { segments: Box::new([]), last: authority::Last::Open(Box::new([])) },
    }
}

#[must_use]
pub fn fixture(seed: u64, requirement: bool) -> (root::Config, root::Limits) {
    let mut limits = crate::world::limits();
    limits.core.authority.segments = 2;
    limits.core.people.requests = 16;
    limits.connector.write_lifetime = Duration::from_secs(1);
    let mut configuration = crate::world::config(seed);
    let mut rules = configuration.core.authority.rules().clone();
    rules.ceiling.grants = Box::new([grant()]);
    let mut policy = configuration.core.authority.policy(1).expect("configured policy").clone();
    policy.ceiling.grants = Box::new([grant()]);
    policy.roles[0].authority.grants = Box::new([grant()]);
    if requirement {
        policy.requirements = Box::new([authority::Requirement {
            connector: 1,
            kind: 5,
            pattern: grant().pattern,
            judge: authority::Judge { connector: 2, requirement: 9, parameters: 0 },
            guard: authority::Guard::Observed { freshness: Duration::from_secs(60) },
            must_be_guarded: false,
        }]);
    }
    let mut domain = authority::Domain::new(rules, limits.core.authority).expect("bounded effect rules");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.settings.chat_authority.grants = Box::new([grant()]);
    configuration.core.settings.chat_authority.budget.spend = 300;
    configuration.core.authority = domain;
    configuration.first.procedures = Box::new([connector::ProcedureSpec {
        number: 1,
        actions: Box::new([
            connector::ProcedureAction::Effect { kind: 5, purpose: 19, target: 13 },
            connector::ProcedureAction::Finish { result: connector::ProcedureResult::Succeeded },
        ]),
        max_steps: 3,
        stall: Duration::from_secs(1),
    }]);
    // Both connectors name the same resource; their facts come from distinct
    // systems. Connector two is the observer, connector one is the writer.
    configuration.second.prefix = configuration.first.prefix.clone();
    configuration.second.resources.clone_from(&configuration.first.resources);
    configuration.second.pools.clone_from(&configuration.first.pools);
    configuration.second.requirements =
        Box::new([connector::RequirementSpec { number: 9, guarded: false, freshness: Duration::from_secs(60) }]);
    (configuration, limits)
}

impl World {
    #[must_use]
    pub fn new(seed: u64, requirement: bool) -> World {
        Self::with_entries(seed, requirement, crate::world::limits().connector.entries)
    }

    /// A finite connector outbox that can be filled by a pending write.
    #[must_use]
    pub fn with_entries(seed: u64, requirement: bool, entries: u32) -> World {
        let (configuration, mut limits) = fixture(seed, requirement);
        limits.connector.entries = entries;
        Self::configured(seed, requirement, configuration, limits)
    }

    /// A story may configure its programs, policy and worker capacity.
    #[must_use]
    pub fn configured(seed: u64, requirement: bool, configuration: root::Config, limits: root::Limits) -> World {
        let world = Self::configured_unassigned(seed, requirement, configuration, limits);
        assert!(world.assigned.is_some(), "the task hub and fleet assigned the caller: {:?}", world.trace);
        world
    }

    /// Build a story whose initial chat may wait or be held before assignment.
    #[must_use]
    pub fn configured_unassigned(
        seed: u64,
        requirement: bool,
        configuration: root::Config,
        limits: root::Limits,
    ) -> World {
        // This domain only journals roles before the exact effect configuration
        // is installed and restored below; its bootstrap needs no effect grants.
        let mut bootstrap = crate::world::config(seed);
        if *bootstrap.core.authority.limits() != limits.core.authority {
            let rules = bootstrap.core.authority.rules().clone();
            let policy = bootstrap.core.authority.policy(1).expect("bootstrap policy").clone();
            bootstrap.core.authority = authority::Domain::new(rules, limits.core.authority).expect("bootstrap bounds");
            let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
            authority::step(&mut bootstrap.core.authority, authority::Event::Policy { project: 1, policy }, &mut facts);
        }
        let mut world = Self::empty(bootstrap, &limits);
        world.send(root::Event::Core(core::Event::People(people::Event::Roles { project: 1, holdings: Box::new([]) })));
        let projects: Vec<_> = configuration.core.projects.iter().copied().collect();
        world.observer = crate::observations::Observer::new(&configuration);
        world.observer.durable(world.wall, &world.store);
        world.domain = root::Domain::new(configuration, &limits);
        world.send(root::Event::RestartBegin);
        for project in projects {
            if project != 1 {
                world.send(root::Event::Core(core::Event::People(people::Event::Roles {
                    project,
                    holdings: Box::new([]),
                })));
            }
        }
        world.events.extend([
            root::Event::Core(core::Event::Account(accounts::Event::Add {
                account: 1,
                generation: 1,
                valid: Some(Duration::from_secs(60)),
            })),
            root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
                channel: Token::new(7),
                hello: fleet::Hello {
                    bounds: crate::fixtures::bounds(),
                    stop_bound: Duration::from_millis(100),
                    slots: limits.core.fleet.slots,
                    workstreams: Box::new([]),
                    hosting: Box::new([]),
                },
            })),
            root::Event::Core(core::Event::SignIn {
                reply_to: ReplyTo::new(Token::new(101)),
                identity: people::Identity {
                    key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                    login: b"person".as_slice().into(),
                    name: b"Person".as_slice().into(),
                },
            }),
        ]);
        world.drain();
        if requirement {
            world.waiting_judge();
        }
        world
    }

    fn empty(configuration: root::Config, limits: &root::Limits) -> World {
        let observer = crate::observations::Observer::new(&configuration);
        let heap = crate::observations::Heap::new(limits);
        World {
            domain: root::Domain::new(configuration, limits),
            store: Store::new(),
            systems: [System::new(), System::new()],
            protocol: crate::protocol::Stand::new(limits.core),
            host_calls: Vec::new(),
            host_answers: Vec::new(),
            inbound: Vec::new(),
            answers: Vec::new(),
            assigned: None,
            assignments: Vec::new(),
            cancellations: Vec::new(),
            busy_turns: Vec::new(),
            busy_calls: Vec::new(),
            busy_events: Vec::new(),
            stop_deliveries: Vec::new(),
            commits: Vec::new(),
            projections: Vec::new(),
            hold_projection_settlements: false,
            sign_in: None,
            people_answers: Vec::new(),
            application_answers: Vec::new(),
            procedure: None,
            procedure_inputs: Vec::new(),
            hold_procedures: false,
            pending_reads: Vec::new(),
            lost_answers: 0,
            hold_writes: false,
            recovery: connector::Recovery::Keyed,
            hold_lookups: false,
            pending_lookups: Vec::new(),
            pending_writes: Vec::new(),
            lose_answer: false,
            trace: Vec::new(),
            restart_steps: Vec::new(),
            restore_ranges: Vec::new(),
            hold_restart_reads: false,
            hold_restore_pages: false,
            store_delay: 0,
            stopped: false,
            peers: None,
            client_peer: None,
            store_peer: None,
            watches: None,
            offers: None,
            reads: None,
            observer,
            heap,
            restoring: None,
            restart_reads: std::collections::BTreeMap::new(),
            events: VecDeque::new(),
            limits: *limits,
            cut: None,
            reached_cut: None,
            hold_kinds: Vec::new(),
            wall: 0,
            wall_lag: 0,
            supplied_wall: None,
        }
    }

    /// Compose the testing root with independently scripted workers and parties.
    #[must_use]
    pub fn scripted(configuration: root::Config, limits: root::Limits, peers: crate::peers::Peers) -> World {
        Self::scripted_at(configuration, limits, peers, None)
    }

    /// Select a cut before even the first sign-in and assignment have committed.
    #[must_use]
    pub fn scripted_at(
        configuration: root::Config,
        limits: root::Limits,
        peers: crate::peers::Peers,
        cut: Option<Cut>,
    ) -> World {
        Self::scripted_clock_at(configuration, &limits, peers, cut, None)
    }

    /// Run the same root/store/peer machinery with the enclosing world's clocks.
    /// Internal draining preserves these values until the caller supplies a new pair.
    #[must_use]
    pub fn scripted_clock(
        configuration: root::Config,
        limits: root::Limits,
        peers: crate::peers::Peers,
        now: Time,
        wall: Wall,
    ) -> World {
        Self::scripted_clock_at(configuration, &limits, peers, None, Some((now, wall)))
    }

    fn scripted_clock_at(
        configuration: root::Config,
        limits: &root::Limits,
        peers: crate::peers::Peers,
        cut: Option<Cut>,
        clock: Option<(Time, Wall)>,
    ) -> World {
        let mut world = Self::empty(configuration, limits);
        if let Some((now, wall)) = clock {
            world.set_clock(now, wall);
        }
        world.cut = cut;
        world.peers = Some(peers);
        let row =
            root::Record::Core(core::Record::People(people::Stored::Roles { project: 1, holdings: Box::new([]) }));
        world.store.rows.insert(row.key(), row);
        world.events.push_back(root::Event::RestartBegin);
        world.events.push_back(root::Event::Core(core::Event::Account(accounts::Event::Add {
            account: 1,
            generation: 1,
            valid: Some(Duration::from_secs(60)),
        })));
        world.drain();
        world
    }

    /// Supply monotonic time and wall time independently, including wall corrections.
    pub fn set_clock(&mut self, now: Time, wall: Wall) {
        if self.supplied_wall.is_some() {
            assert!(now.as_nanos() >= self.wall, "elapsed time is monotonic");
        }
        self.wall = now.as_nanos();
        self.supplied_wall = Some(wall);
    }

    /// The pair actually supplied to the root and its children.
    #[must_use]
    pub fn clocks(&self) -> (Time, Wall) {
        let env = self.env();
        (env.now, env.wall)
    }

    /// Earliest armed owner deadline at the current supplied clock pair.
    /// Absolute Wall values are sampled again after a correction or delivery.
    #[must_use]
    pub fn next_due(&self) -> Option<Time> {
        let (now, wall) = self.clocks();
        let mut next = None;
        for (_, deadline) in self.domain.core_deadlines() {
            if let Some(deadline) = deadline {
                next = Some(next.map_or(deadline, |prior: Time| prior.min(deadline)));
            }
        }
        for deadline in self
            .domain
            .connector_deadlines()
            .into_iter()
            .map(|(_, deadline)| deadline)
            .chain([self.domain.effect_deadline()])
            .flatten()
        {
            let remaining = deadline.as_nanos().saturating_sub(wall.as_nanos());
            let due = Time::from_nanos(now.as_nanos().checked_add(remaining).expect("world deadline fits"));
            next = Some(next.map_or(due, |prior| prior.min(due)));
        }
        next
    }

    /// Fire only currently due owners through their ordinary root inputs.
    /// The caller supplies clocks; neither draining nor timer delivery advances them.
    pub fn fire_due(&mut self, now: Time, wall: Wall) {
        self.set_clock(now, wall);
        let mut events = Vec::new();
        for (timer, deadline) in self.domain.core_deadlines() {
            if deadline.is_some_and(|deadline| deadline <= now) {
                events.push(root::Event::Timer(timer));
            }
        }
        for (number, deadline) in self.domain.connector_deadlines() {
            if deadline.is_some_and(|deadline| deadline <= wall) {
                events.push(root::Event::ConnectorTimer { number });
            }
        }
        if self.domain.effect_deadline().is_some_and(|deadline| deadline <= wall) {
            events.push(root::Event::Core(core::Event::EffectDeadline));
        }
        self.send_all(events);
    }

    /// Advance simulated time, firing fleet and task deadlines through the root.
    pub fn advance(&mut self, by: Duration) {
        self.wall = self.wall.checked_add(by.as_nanos()).expect("world time fits");
        self.supplied_wall = self
            .supplied_wall
            .map(|wall| Wall::from_nanos(wall.as_nanos().checked_add(by.as_nanos()).expect("supplied wall fits")));
        self.events.push_back(root::Event::Timer(core::Timer::Fleet));
        self.events.push_back(root::Event::Timer(core::Timer::Tasks));
        self.events.push_back(root::Event::Timer(core::Timer::People));
        self.events.push_back(root::Event::ConnectorTimer { number: 1 });
        self.events.push_back(root::Event::ConnectorTimer { number: 2 });
        self.drain();
    }

    /// Correct wall time backwards while elapsed time continues monotonically.
    pub fn correct_wall_backwards(&mut self, by: Duration) {
        self.wall_lag = self.wall_lag.checked_add(by.as_nanos()).expect("wall correction fits");
    }

    fn env(&self) -> Env<root::Limits> {
        Env {
            now: Time::from_nanos(self.wall),
            wall: self.supplied_wall.unwrap_or_else(|| Wall::from_nanos(self.wall.saturating_sub(self.wall_lag))),
            limits: self.limits,
        }
    }

    /// Queue a burst so independent parties draw their pace between its inputs.
    pub fn send_all(&mut self, events: Vec<root::Event>) {
        self.events.extend(events);
        self.drain();
    }

    pub fn send(&mut self, event: root::Event) {
        self.events.push_back(event);
        self.drain();
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one iteration keeps commit application, peer observations and root outputs in order"
    )]
    pub fn drain(&mut self) {
        for _ in 0..300 {
            if self.stopped {
                return;
            }
            if self.supplied_wall.is_none() {
                self.wall += 1_000_000;
            }
            self.observer.observe(self.wall, crate::referee::Observed::Tick);
            let env = self.env();
            self.observer.systems(self.wall, &self.systems);
            if let Some(mut event) = self.events.pop_front() {
                self.observer.inbound(&self.store, self.wall, &mut event);
                if let Some(offers) = &mut self.offers {
                    offers.inbound(&mut event);
                }
                if let Some(watches) = &mut self.watches {
                    watches.inbound(&event);
                }
                if let Some(reads) = &mut self.reads {
                    reads.inbound(&event);
                }
                self.trace.push(format!("input {event:?}"));
                root::step(&mut self.domain, &env, event);
            }
            let mut out = Queue::with_capacity(128);
            root::release(&mut self.domain, &env, &mut out);
            let empty = out.is_empty();
            for _ in 0..out.len() {
                let request = out.pop().expect("root output count");
                self.trace.push(format!("output {request:?}"));
                match request {
                    root::Request::Restart(step) => self.restart_step(step),
                    root::Request::Commit { number, mut writes } => {
                        let mut rows = Vec::new();
                        while let Some(write) = writes.pop() {
                            rows.push(write);
                        }
                        if let Some(watches) = &mut self.watches {
                            watches.commit(number, &rows);
                        }
                        if let Some(offers) = &mut self.offers {
                            offers.committed(number);
                        }
                        if let Some(reads) = &mut self.reads {
                            reads.commit(number);
                        }
                        self.commits.push(rows.clone());
                        self.store.submit(number, crate::store_writes(rows), self.store_delay);
                        self.observer.submitted(self.wall, &self.store);
                        if self.cut == Some(Cut::Submitted(number)) {
                            self.reached_cut = self.cut.take();
                            return;
                        }
                    }
                    root::Request::Deliver(delivery) => self.delivered(delivery),
                    root::Request::Now(core::Now::StartPreparation { task, .. }) => {
                        let rows: Vec<_> = self
                            .store
                            .rows
                            .values()
                            .filter_map(|record| match record {
                                root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn)))
                                    if turn.task == task =>
                                {
                                    Some(turn.clone())
                                }
                                root::Record::Core(_) | root::Record::Connector { .. } => None,
                            })
                            .collect();
                        for row in rows {
                            self.events.push_back(root::Event::TranscriptLoaded {
                                task,
                                rows: Box::new([row]),
                                done: false,
                            });
                        }
                        self.events.push_back(root::Event::TranscriptLoaded { task, rows: Box::new([]), done: true });
                    }
                    root::Request::Now(core::Now::Call { to, run, attempt, call }) => {
                        let token = to.into_token();
                        if self.peers.is_none()
                            && let Some(event) = self.protocol.decode(token, run, attempt, &call)
                        {
                            self.events.push_back(event);
                        } else if let Some(peers) = &mut self.peers {
                            self.events.extend(peers.opaque_answer(ReplyTo::new(token), run, attempt, &call));
                        } else {
                            self.host_calls.push((token, run.raw(), attempt.raw(), call));
                        }
                    }
                    root::Request::Now(core::Now::EffectAnswer { to, key, part }) => {
                        let token = to.into_token();
                        if let Some(event) = self
                            .peers
                            .as_mut()
                            .map_or(&mut self.protocol, |peers| &mut peers.protocol)
                            .answer(token, key, &part, false)
                        {
                            self.events.push_back(event);
                        }
                        self.answers.push((token, key, part));
                    }
                    root::Request::Now(core::Now::Account(accounts::Request::Refresh { account, generation })) => {
                        self.events.push_back(root::Event::Core(core::Event::Account(accounts::Event::Refreshed {
                            account,
                            generation,
                            valid: Duration::from_secs(60),
                        })));
                    }
                    root::Request::Now(core::Now::SeenChat { request, task }) => {
                        let row = match self.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task))))
                        {
                            Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) => {
                                Some(row.clone())
                            }
                            Some(_) | None => None,
                        };
                        self.events.push_back(root::Event::Core(core::Event::SeenChatLoaded {
                            request,
                            row,
                            busy: false,
                        }));
                    }
                    root::Request::Now(core::Now::HistoricalPersonTask { request, person, project, task }) => {
                        let key = root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)));
                        let row = match self.store.rows.get(&key) {
                            Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) => {
                                Some(row.clone())
                            }
                            Some(_) | None => None,
                        };
                        self.events.push_back(root::Event::Core(core::Event::HistoricalPersonTask {
                            request,
                            person,
                            project,
                            task,
                            row,
                            busy: false,
                        }));
                    }
                    root::Request::Now(core::Now::HistoricalProposal {
                        request,
                        person,
                        project,
                        proposer,
                        proposal,
                    }) => {
                        self.events.push_back(crate::historical::proposal(
                            &self.store,
                            request,
                            person,
                            project,
                            proposer,
                            proposal,
                        ));
                    }
                    root::Request::Now(core::Now::HistoricalEscalation {
                        request,
                        person,
                        project,
                        task,
                        revision,
                    }) => {
                        self.events.push_back(crate::historical::escalation(
                            &self.store,
                            request,
                            person,
                            project,
                            task,
                            revision,
                        ));
                    }
                    root::Request::Now(now) => {
                        if let Some(peer) = &mut self.client_peer
                            && crate::client_peer::ClientPeer::immediate(&now)
                        {
                            peer.now.push(now);
                            continue;
                        }
                        if let Some(watches) = &mut self.watches {
                            watches.now(&now);
                        }
                    }
                    root::Request::Busy { event } => self.busy_events.push(*event),
                    root::Request::Stop => self.stopped = true,
                }
            }
            if !self.stopped {
                match self.store.tick(false) {
                    Ok(Some(number)) => self.events.push_front(root::Event::Committed { number }),
                    Ok(None) => {}
                    Err(number) => self.events.push_front(root::Event::Failed { number }),
                }
                self.observer.durable(self.wall, &self.store);
                if let Some(peer) = &mut self.store_peer {
                    peer.capture(Time::from_nanos(self.wall), &self.store);
                }
                if self.cut == Some(Cut::Durable(self.store.applied)) {
                    self.reached_cut = self.cut.take();
                    return;
                }
                self.observer.systems(self.wall, &self.systems);
                self.restore_pages();
                if let Some(watches) = &mut self.watches {
                    self.events.extend(watches.tick());
                }
                if self.domain.ready()
                    && let Some(peers) = &mut self.peers
                {
                    self.events.extend(peers.tick(&self.store, env.now));
                }
            }
            if let Some(peers) = &self.peers {
                self.observer.workers(self.wall, &peers.workers);
            }
            self.domain.reclaim();
            self.observer.observe(self.wall, self.heap.observed());
            let held_restore = self.hold_restore_pages && self.restoring.is_some() && self.store.loading();
            if empty
                && self.events.is_empty()
                && self.store.pending.is_empty()
                && (!self.store.loading() || held_restore)
                && (self.domain.quiescent()
                    || held_restore
                    || !self.store.held.is_empty()
                    || self.reads.as_ref().is_some_and(crate::reads::Parties::paused))
                && self.watches.as_ref().is_none_or(crate::peers::WatchParties::quiescent)
            {
                return;
            }
        }
        panic!("effect world did not settle: {:?}", self.trace);
    }

    #[expect(clippy::too_many_lines, reason = "one exhaustive delivery handler drives the script's independent peers")]
    fn delivered(&mut self, mut delivery: root::Delivery) {
        self.observer.delivered(self.wall, &mut delivery);
        if let Some(offers) = &mut self.offers {
            offers.delivered(&mut delivery);
        }
        if let Some(watches) = &mut self.watches {
            watches.released(&delivery, self.store.applied);
        }
        if let Some(peer) = &mut self.client_peer {
            match peer.capture(self.store.applied, delivery) {
                Ok(()) => return,
                Err(unowned) => delivery = unowned,
            }
        }
        if let Some(peers) = &mut self.peers {
            peers.delivered(&delivery);
            if let root::Delivery::Core(core::Held::PeopleReply { to, reply }) = delivery {
                peers.reply(to, reply);
                self.people_answers.push(reply);
                return;
            }
        }
        match delivery {
            root::Delivery::ApplicationRead { reply, number, answer } => {
                if let Some(reads) = &mut self.reads {
                    reads.application_replied(reply, number, answer, &self.store);
                }
            }
            root::Delivery::Core(core::Held::Read { reply, answer }) => {
                if let Some(reads) = &mut self.reads {
                    reads.replied(reply, answer, &self.store);
                }
            }
            root::Delivery::Core(core::Held::ReadLoad { load, range }) => {
                if let Some(reads) = &mut self.reads {
                    self.events.extend(reads.load(load, *range, &self.store));
                } else if self.client_peer.is_some() {
                    let result = crate::reads::page(&self.store, &range);
                    self.events.push_back(root::Event::Core(core::Event::ReadLoaded { load, result }));
                }
            }
            root::Delivery::Core(
                core::Held::Applied { .. }
                | core::Held::ApplicationDelivery { .. }
                | core::Held::ApplicationWatchEnded { .. },
            ) => {
                panic!("root assembles connector outcomes before journaling")
            }
            root::Delivery::Application { to, number, outcome } => {
                self.application_answers.push((to.into_token(), number, outcome));
            }
            root::Delivery::Projection { connector, feed } => {
                if feed.closing && !self.hold_projection_settlements {
                    self.events.push_back(root::Event::Core(core::Event::ProjectionSettled {
                        goal: feed.goal.number,
                        connector,
                    }));
                }
                self.projections.push((self.commits.len(), connector, *feed));
            }
            root::Delivery::Restart(step) => self.restart_step(step),
            root::Delivery::CallBusy { name, .. } => self.busy_calls.push(name),
            root::Delivery::Core(core::Held::TurnBusy { run, attempt, turn, .. }) => {
                self.busy_turns.push((run.raw(), attempt.raw(), turn));
            }
            root::Delivery::Core(
                held @ (core::Held::Drained | core::Held::WatchEnded { .. } | core::Held::Acknowledge { .. }),
            ) => {
                self.stop_deliveries.push(held);
            }
            root::Delivery::CallAnswer { name, call, .. } => self.host_answers.push((name, call)),
            root::Delivery::Message { word, .. } => self.inbound.push(word),
            root::Delivery::Assigned { assignment, .. } => {
                self.assignments.push((assignment.task, assignment.attempt));
                self.assigned = Some(assignment);
            }
            root::Delivery::Core(core::Held::Cancel { run, attempt, .. }) => {
                self.cancellations.push((run.raw(), attempt.raw()));
            }
            root::Delivery::Procedure { task, step, connector: number, code, context } => {
                self.procedure = Some(task);
                let project = context.project;
                self.procedure_inputs.push((step, number, code, context));
                if self.hold_procedures {
                    return;
                }
                self.events.push_back(root::Event::Connector {
                    number,
                    event: connector::Event::Procedure {
                        project,
                        task,
                        number: code,
                        resource: jig_test_connector_world::path(1, 1),
                        signal: connector::ProcedureSignal::Activate,
                    },
                });
            }
            root::Delivery::LostRead { connector, resource, call, .. } => {
                let event = self.systems[usize::from(connector - 1)].answer(call, Fault::None);
                self.events.push_back(root::Event::WriterRead { connector, resource, event });
            }
            root::Delivery::System {
                connector: number,
                call: connector::SystemRequest::ReadFact { resource, observed },
            } => {
                let call = connector::SystemRequest::ReadFact { resource, observed };
                if self.restart_reads.contains_key(&number) && !self.hold_restart_reads {
                    let event = self.systems[usize::from(number - 1)].answer(call, Fault::None);
                    self.events.push_back(root::Event::Connector { number, event: connector::Event::System(event) });
                    let step = self.restart_reads.remove(&number).expect("restart read");
                    self.events.push_back(root::Event::RestartDone(step));
                } else {
                    self.pending_reads.push((number, call));
                }
            }
            root::Delivery::System { connector: number, call } => {
                if let connector::SystemRequest::Apply { entry, key, .. } = &call {
                    assert!(self.store.rows.values().any(|row| matches!(row, root::Record::Connector { number: owner, record: connector::Record::Outbox(row) } if *owner == number && row.number == *entry && row.key == *key)), "system sees only a committed outbox attempt");
                    assert_eq!(key.deployment, [31; 16]);
                }
                if self.hold_lookups && matches!(call, connector::SystemRequest::Look { .. }) {
                    self.pending_lookups.push((number, call));
                    return;
                }
                if self.hold_writes && matches!(call, connector::SystemRequest::Apply { .. }) {
                    self.pending_writes.push((number, call));
                    return;
                }
                let event = self.systems[usize::from(number - 1)].answer(call, Fault::None);
                self.events.push_back(root::Event::Connector { number, event: connector::Event::System(event) });
            }
            root::Delivery::Core(core::Held::PeopleReply {
                reply: reply @ people::Reply::SignedIn { sign_in, .. },
                ..
            }) => {
                let initial = self.sign_in.is_none();
                self.sign_in = Some(sign_in);
                self.people_answers.push(reply);
                if !initial {
                    return;
                }
                self.events.push_back(root::Event::Core(core::Event::People(people::Event::Ask {
                    reply_to: ReplyTo::new(Token::new(102)),
                    sign_in,
                    key: [5; 16],
                    ask: people::Ask::StartChat { project: 1, words: b"make an effect".as_slice().into() },
                })));
            }
            root::Delivery::Core(core::Held::NotesLoad { owner, range }) => {
                let mut records = List::with_capacity(self.limits.core.notes.load_rows);
                let mut more = false;
                for row in self.store.rows.values() {
                    let root::Record::Core(core::Record::Notes(record)) = row else { continue };
                    let selected = match (&range, record) {
                        (jig_domain_notes::Range::Entry { name }, jig_domain_notes::Record::Entry(entry)) => {
                            *name == entry.name
                        }
                        (jig_domain_notes::Range::Lines { scope, after }, jig_domain_notes::Record::Line(line)) => {
                            *scope == line.scope && after.is_none_or(|previous| line.name > previous)
                        }
                        _ => false,
                    };
                    if selected && records.push(record.clone()).is_err() {
                        more = true;
                    }
                }
                self.events.push_back(root::Event::Core(core::Event::Notes(jig_domain_notes::Event::Loaded {
                    owner,
                    rows: jig_domain_notes::Rows { records },
                    more,
                })));
            }
            root::Delivery::Core(core::Held::CallAnswer { to, key, part }) => {
                let token = to.into_token();
                assert!(
                    self.store.rows.contains_key(&root::Key::Core(core::Key::Core(core::CoreKey::Call(key)))),
                    "state-changing answer has its committed record"
                );
                if self.lose_answer {
                    self.lost_answers += 1;
                } else {
                    if let Some(event) = self
                        .peers
                        .as_mut()
                        .map_or(&mut self.protocol, |peers| &mut peers.protocol)
                        .answer(token, key, &part, true)
                    {
                        self.events.push_back(event);
                    }
                    self.answers.push((token, key, part));
                }
            }
            root::Delivery::Core(core::Held::PeopleReply { reply, .. }) => self.people_answers.push(reply),
            root::Delivery::ApplicationDocument { .. }
            | root::Delivery::Core(
                core::Held::Delivery { .. }
                | core::Held::SettledCall { .. }
                | core::Held::Assign { .. }
                | core::Held::Inbound { .. }
                | core::Held::Relayed { .. }
                | core::Held::Relay { .. }
                | core::Held::Result { .. }
                | core::Held::AcknowledgeTurn { .. }
                | core::Held::TaskTurnKept { .. }
                | core::Held::TaskTerminalAcknowledged { .. }
                | core::Held::NotesWritten { .. }
                | core::Held::NotesDeleted { .. }
                | core::Held::StopRun { .. }
                | core::Held::Refuse { .. }
                | core::Held::MakeEffect { .. },
            ) => {}
            root::Delivery::Fleet(_) => panic!("root releases its fleet continuations internally"),
        }
    }

    #[must_use]
    pub fn call_key(&self, completion: u32) -> core::CallKey {
        let assignment = self.assigned.as_ref().expect("assigned caller");
        core::CallKey { task: assignment.task, attempt: assignment.attempt, completion, position: 0 }
    }

    #[must_use]
    pub fn effect() -> connector::Effect {
        connector::Effect {
            kind: 5,
            resources: Box::new([jig_test_connector_world::path(1, 1)]),
            purpose: 19,
            condition: None,
            target: 13,
            state: 13,
        }
    }

    pub fn call(&mut self, completion: u32, right: u64) {
        self.send(root::Event::EffectCall {
            to: ReplyTo::new(Token::new(right)),
            key: self.call_key(completion),
            number: 1,
            effect: Self::effect(),
            deadline: Wall::from_nanos(self.wall + 1_000_000_000),
            proposal: None,
        });
    }

    /// The caller's independently stored expense, including made effects.
    #[must_use]
    pub fn spent(&self) -> u64 {
        let task = self.call_key(1).task;
        match self.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))) {
            Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) => row.numbers.spent,
            row => panic!("live caller's stored balance: {row:?}"),
        }
    }

    /// Expense debited to the accepting person's finite pool.
    #[must_use]
    pub fn holder_spent(&self) -> u64 {
        match self.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ledger(tasks::Funder::Pool {
            project: 1,
            person: 1,
            period: 1,
        })))) {
            Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(row)))) => row.numbers.spent,
            row => panic!("holder's stored balance: {row:?}"),
        }
    }

    /// Explicitly propose the connector payload instead of making it.
    pub fn propose(&mut self, completion: u32) -> u64 {
        self.send(root::Event::EffectCall {
            to: ReplyTo::new(Token::new(220)),
            key: self.call_key(completion),
            number: 1,
            effect: Self::effect(),
            deadline: Wall::from_nanos(self.wall + 1_000_000_000),
            proposal: Some(b"ask the holder".as_slice().into()),
        });
        match self.answers.last() {
            Some((_, _, core::CallPart::Proposed { proposal })) => *proposal,
            answer => panic!("effect proposed: {answer:?}; {:?}", self.trace),
        }
    }

    /// The signed-in person decides the caller's proposal through people.
    pub fn decide_proposal(&mut self, proposal: u64, accept: bool, request: u8) {
        let decision = if accept {
            people::ProposalDecision::Accept
        } else {
            people::ProposalDecision::Reject { reason: b"not now".as_slice().into() }
        };
        self.send(root::Event::Core(core::Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(u64::from(request) + 230)),
            sign_in: self.sign_in.expect("signed-in person"),
            key: [request; 16],
            ask: people::Ask::DecideProposal {
                project: 1,
                proposer: people::Proposer::Task(self.call_key(1).task),
                proposal,
                decision,
            },
        })));
    }

    /// Cross the call's absolute deadline while its committed write is pending.
    pub fn deadline(&mut self) {
        self.wall += 1_000_000_000;
        self.send(root::Event::Core(core::Event::EffectDeadline));
    }

    /// Deliver the system answers held past the caller's deadline.
    pub fn release_writes(&mut self) {
        self.hold_writes = false;
        for (number, call) in std::mem::take(&mut self.pending_writes) {
            let event = self.systems[usize::from(number - 1)].answer(call, Fault::None);
            self.events.push_back(root::Event::Connector { number, event: connector::Event::System(event) });
        }
        self.drain();
    }

    pub fn delegate(&mut self) {
        let assignment = self.assigned.as_ref().expect("assigned caller");
        let mut authority = assignment.run.authority.clone();
        authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
        authority.budget.spend = 20;
        authority.tools = tasks::Tools(0);
        self.send(root::Event::Core(core::Event::DelegateValidated {
            to: ReplyTo::new(Token::new(201)),
            key: self.call_key(1),
            batch: Box::new([core::Delegate {
                executor: tasks::Executor::Procedure { connector: 1, code: 1 },
                spec: tasks::Spec {
                    words: b"make a checked effect".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                },
                contract: tasks::Contract::Report { words: 128 },
                authority,
                symbolic_grants: Box::new([]),
                dependencies: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
            }]),
            stubs: Box::new([]),
        }));
        assert!(self.procedure.is_some(), "delegated procedure admitted: {:?}", self.trace);
    }

    /// Read the observer's system, then give the idle procedure another step.
    pub fn observed(&mut self, state: u64) {
        self.systems[1].other_hand(&jig_test_connector_world::path(1, 1), state);
        let reads = std::mem::take(&mut self.pending_reads);
        for (number, call) in reads {
            let event = self.systems[usize::from(number - 1)].answer(call, Fault::None);
            self.events.push_back(root::Event::Connector { number, event: connector::Event::System(event) });
        }
        self.drain();
        if let Some(task) = self.procedure {
            self.send(root::Event::Core(core::Event::Tasks(tasks::Event::WakeProcedure { task })));
        }
    }

    /// Rebuild all live components from the fake store, preserving the systems.
    fn restart_step(&mut self, step: core::RestartStep) {
        self.restart_steps.push(step);
        match step {
            core::RestartStep::LoadCore | core::RestartStep::LoadInbox | core::RestartStep::RestoreConnector { .. } => {
                self.load_restart(step);
            }
            core::RestartStep::ReadAfresh { connector: number } => {
                self.restart_reads.insert(number, step);
                self.events.push_back(root::Event::Connector {
                    number,
                    event: connector::Event::Restart(connector::RestartStep::FreshRead {
                        resource: jig_test_connector_world::path(1, 1),
                    }),
                });
            }
            core::RestartStep::SettleOutbox { connector: number } => {
                self.events.push_back(root::Event::Connector {
                    number,
                    event: connector::Event::Restart(connector::RestartStep::SettleOutbox),
                });
            }
            core::RestartStep::AdoptRuns | core::RestartStep::Open => panic!("root performs its synchronous step"),
        }
    }

    fn load_restart(&mut self, step: core::RestartStep) {
        let mut intervals = std::collections::VecDeque::new();
        if step == core::RestartStep::LoadCore {
            self.restore_ranges.extend(core::RestoreRange::LIVE);
            for range in core::RestoreRange::LIVE {
                let mut keys = self.store.rows.keys().filter(|key| match key {
                    root::Key::Core(key) => range.admits(key),
                    root::Key::Connector { .. } => false,
                });
                if let Some(first) = keys.next() {
                    let last = keys.next_back().unwrap_or(first);
                    intervals.push_back((first.clone(), last.clone()));
                }
            }
        } else {
            let mut interval = None;
            for key in self.store.rows.keys() {
                let admitted = match (step, key) {
                    (core::RestartStep::LoadInbox, root::Key::Core(key)) => core::inbox_restore_key(key),
                    (core::RestartStep::RestoreConnector { connector }, root::Key::Connector { number, .. }) => {
                        connector == *number
                    }
                    _ => false,
                };
                if admitted {
                    match &mut interval {
                        Some((_, last)) => *last = key.clone(),
                        None => interval = Some((key.clone(), key.clone())),
                    }
                } else if let Some(bounds) = interval.take() {
                    intervals.push_back(bounds);
                }
            }
            if let Some(bounds) = interval {
                intervals.push_back(bounds);
            }
        }
        if let Some((first, last)) = intervals.pop_front() {
            self.store.load(1, &first, &last, None, 2);
            self.restoring = Some(Restore { step, first, last, rows: Vec::new(), remaining: intervals });
        } else {
            self.restored(step, &[]);
        }
    }

    fn restore_pages(&mut self) {
        if self.hold_restore_pages {
            return;
        }
        loop {
            let pages = self.store.tick_pages();
            if pages.is_empty() {
                break;
            }
            for (owner, page) in pages {
                assert_eq!(owner, 1, "one restart page at a time");
                let mut restore = self.restoring.take().expect("requested restart page");
                let live = restore.step == core::RestartStep::LoadCore;
                let rows = page.rows.len() + if live { restore.rows.len() % 2 } else { 0 };
                if !live || rows == 2 || (page.next.is_none() && restore.remaining.is_empty()) {
                    self.trace.push(format!("store page {:?} {rows} rows", restore.step));
                }
                restore.rows.extend(page.rows);
                if let Some(after) = page.next {
                    self.store.load(1, &restore.first, &restore.last, Some(&after), 2);
                    self.restoring = Some(restore);
                } else if let Some((first, last)) = restore.remaining.pop_front() {
                    let room = if live { 2 - restore.rows.len() % 2 } else { 2 };
                    self.store.load(1, &first, &last, None, u32::try_from(room).expect("two-row page"));
                    restore.first = first;
                    restore.last = last;
                    self.restoring = Some(restore);
                } else {
                    self.restored(restore.step, &restore.rows);
                }
            }
        }
    }

    /// Deliver every outstanding original restore page through the usual route.
    pub fn finish_restore_pages(&mut self) {
        self.hold_restore_pages = false;
        self.drain();
    }

    fn restored(&mut self, step: core::RestartStep, rows: &[root::Record]) {
        let env = self.env();
        if step == core::RestartStep::LoadInbox {
            for row in rows {
                assert!(root::restore_record(&mut self.domain, &env, row.clone()), "restored inbox {row:?}");
            }
            self.events.push_back(root::Event::RestartDone(step));
            return;
        }
        for row in rows {
            assert!(root::restore_record(&mut self.domain, &env, row.clone()), "restored {row:?}");
        }
        if step == core::RestartStep::LoadCore {
            self.events.push_back(root::Event::Core(core::Event::People(people::Event::Restored)));
        }
        self.events.push_back(root::Event::RestartDone(step));
    }

    /// Install the connector's fixed hold vocabulary, retained across cold starts.
    pub fn configure_holds(&mut self, connector: u16, kinds: Box<[tasks::Kind]>) {
        self.hold_kinds.push((connector, kinds.clone()));
        self.send(root::Event::Core(core::Event::Tasks(tasks::Event::Kinds { connector, kinds })));
    }

    /// Let every previously durable completion through its ordered barrier.
    pub fn release_commits(&mut self) {
        while let Some(number) = self.store.release_held() {
            self.events.push_back(root::Event::Committed { number });
        }
        self.drain();
    }

    pub fn finish_restart_reads(&mut self) {
        let reads = std::mem::take(&mut self.pending_reads);
        for (number, call) in reads {
            let event = self.systems[usize::from(number - 1)].answer(call, Fault::None);
            self.events.push_back(root::Event::Connector { number, event: connector::Event::System(event) });
            if let Some(step) = self.restart_reads.remove(&number) {
                self.events.push_back(root::Event::RestartDone(step));
            }
        }
        self.hold_restart_reads = false;
        self.drain();
    }

    pub fn restart(&mut self, seed: u64, requirement: bool) {
        let mut config = fixture(seed, requirement).0;
        for kind in &mut config.first.kinds {
            if kind.kind == 5 {
                kind.recovery = self.recovery;
            }
        }
        self.restart_with(config);
    }

    /// Cold start with the same scenario configuration, including its host slots.
    pub fn restart_with(&mut self, config: root::Config) {
        self.observer.observe(self.wall, crate::referee::Observed::Restart);
        self.store.crash();
        if let Some(watches) = &mut self.watches {
            watches.restart(self.store.applied);
        }
        if let Some(reads) = &mut self.reads {
            reads.restart(self.store.applied);
        }
        self.observer.cold(self.wall, &self.store);
        self.stopped = false;
        self.restoring = None;
        self.domain = root::Domain::new(config, &self.limits);
        for (connector, kinds) in self.hold_kinds.clone() {
            self.domain.configure_holds(&self.env(), connector, kinds);
        }
        self.events.clear();
        self.pending_reads.clear();
        self.pending_writes.clear();
        self.pending_lookups.clear();
        self.restart_reads.clear();
        self.restart_steps.clear();
        self.restore_ranges.clear();
        if let Some(peers) = &mut self.peers {
            peers.restart();
        }
        self.cut = None;
        self.send(root::Event::RestartBegin);
        self.lose_answer = false;
    }
}

impl World {
    pub fn say(&mut self, key: u8) -> u64 {
        let task = self.assigned.as_ref().expect("assigned chat").task;
        self.send(root::Event::Core(core::Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(3000 + u64::from(key))),
            sign_in: self.sign_in.expect("signed in"),
            key: [key; 16],
            ask: people::Ask::Say { project: 1, task, words: Box::from([key]), release: false },
        })));
        self.people_answers
            .iter()
            .rev()
            .find_map(|reply| match reply {
                people::Reply::Outcome(people::Outcome::Said { message, .. }) => Some(*message),
                people::Reply::SignedIn { .. }
                | people::Reply::SignedOut
                | people::Reply::Outcome(_)
                | people::Reply::Refused(_)
                | people::Reply::Again(_) => None,
            })
            .expect("durable words")
    }

    pub fn turn(&mut self, number: u32, spent: u64, read: Option<u64>, body: &[u8]) {
        let assignment = self.assigned.as_ref().expect("assigned chat");
        self.send(root::Event::Turn {
            channel: Token::new(7),
            task: assignment.task,
            attempt: assignment.attempt,
            turn: number,
            cumulative: spent,
            read,
            transcript: body.into(),
        });
    }

    pub fn end(&mut self, end: tasks::End, cumulative: u64) {
        let assignment = self.assigned.as_ref().expect("assigned chat");
        self.send(root::Event::Answer {
            channel: Token::new(7),
            task: assignment.task,
            attempt: assignment.attempt,
            cumulative,
            end,
        });
    }

    pub fn host_call(&mut self, name: &[u8], tool: &[u8], input: &[u8], writes: bool) -> Token {
        self.relay_call(name, tool, input, writes);
        self.host_calls.last().expect("fleet authenticated call").0
    }
}

impl World {
    pub fn retry_due(&mut self) {
        self.wall += 1_000_000_000;
        self.send(root::Event::Timer(core::Timer::Tasks));
    }
}

impl World {
    #[must_use]
    pub fn wall_time(&self) -> Wall {
        Wall::from_nanos(self.wall)
    }

    pub fn finish_lookups(&mut self) {
        self.hold_lookups = false;
        for (number, call) in std::mem::take(&mut self.pending_lookups) {
            let event = self.systems[usize::from(number - 1)].answer(call, Fault::None);
            self.events.push_back(root::Event::Connector { number, event: connector::Event::System(event) });
        }
        self.drain();
    }

    pub fn waiting_judge(&mut self) {
        self.send(root::Event::Connector {
            number: 2,
            event: connector::Event::System(connector::SystemEvent::Fact {
                resource: jig_test_connector_world::path(1, 1),
                fact: connector::Fact { state: None, observed: Wall::from_nanos(self.wall), pending: true },
                origin: connector::Origin::Other,
            }),
        });
    }
}

mod stopping;
