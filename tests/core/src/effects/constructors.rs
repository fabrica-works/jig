//! Seeded configuration and world construction.
use super::{
    Cut, Duration, Queue, ReplyTo, Store, System, Time, Token, VecDeque, Wall, World, accounts, authority, connector,
    core, fleet, people, root,
};
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
    policy.roles[0].authority.budget.spend = 300;
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
    /// The local scripted host declares the same models as the worker fixtures.
    pub(super) fn hello_engine(&mut self) {
        if self.limits.core.fleet.engine_slots > 0 {
            self.events.push_back(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
                channel: Token::new(0),
                hello: fleet::Hello {
                    bounds: crate::fixtures::bounds(),
                    stop_bound: Duration::ZERO,
                    slots: self.limits.core.fleet.engine_slots,
                    workstreams: Box::new([]),
                    hosting: Box::new([]),
                },
            })));
        }
    }

    /// Restore configuration into a wholly empty store without a manual bootstrap.
    #[must_use]
    pub fn first_start(configuration: root::Config, limits: root::Limits) -> World {
        let mut world = Self::empty(configuration, &limits);
        world.send(root::Event::RestartBegin);
        world
    }

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
        let initial = crate::fixtures::create_chat(
            &crate::fixtures::Creating::new(&configuration.core, limits.core.tasks.tree_tasks),
            1,
            0,
            b"make an effect".as_slice().into(),
        );
        Self::configured_with(seed, requirement, configuration, limits, initial)
    }

    /// Choose the initial authenticated task request.
    #[must_use]
    pub fn configured_with(
        seed: u64,
        requirement: bool,
        configuration: root::Config,
        limits: root::Limits,
        initial: people::Ask,
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
        world.creating = crate::fixtures::Creating::new(&configuration.core, limits.core.tasks.tree_tasks);
        world.initial_ask = initial;
        world.send(root::Event::Core(core::Event::People(people::Event::Roles { project: 1, holdings: Box::new([]) })));
        let projects: Vec<_> = configuration
            .core
            .projects
            .iter()
            .map(|project| {
                (*project, configuration.core.authority.policy(*project).expect("project policy").period_spend)
            })
            .collect();
        world.observer = crate::observations::Observer::new(&configuration);
        world.bootstrap_owners = configuration.core.owners.to_vec();
        world.observer.durable(world.wall, &world.store);
        world.domain = root::Domain::new(configuration, &limits);
        world.send(root::Event::RestartBegin);
        world.hello_engine();
        for (project, budget) in projects {
            world.send(root::Event::Core(core::Event::Period { project, period: 1, budget, next: None }));
            if project != 1 {
                world.send(root::Event::Core(core::Event::People(people::Event::Roles {
                    project,
                    holdings: Box::new([]),
                })));
            }
        }
        world.events.extend([
            root::Event::Core(core::Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
                account: 1,
                generation: 1,
                valid: Duration::from_secs(60),
            }))),
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

    pub(super) fn empty(configuration: root::Config, limits: &root::Limits) -> World {
        let observer = crate::observations::Observer::new(&configuration);
        let heap = crate::observations::Heap::new(limits);
        let creating = crate::fixtures::Creating::new(&configuration.core, limits.core.tasks.tree_tasks);
        let initial_ask = crate::fixtures::create_chat(&creating, 1, 0, b"make an effect".as_slice().into());
        let store = Store::new(limits, &configuration.core.settings.models);
        World {
            bootstrap_owners: Vec::new(),
            domain: root::Domain::new(configuration, limits),
            store,
            systems: [System::new(), System::new()],
            protocol: crate::protocol::Stand::new(limits.core),
            host_calls: Vec::new(),
            host_answers: Vec::new(),
            inbound: Vec::new(),
            answers: Vec::new(),
            assigned: None,
            last_read: None,
            received_grant: None,
            grant_follows_assignment: false,
            renew_on_running: None,
            fail_account_before_workspace: false,
            hold_accounts: false,
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
            initial_ask,
            creating,
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
            tracing: std::env::var_os("JIG_WORLD_TRACE").is_some(),
            restart_steps: Vec::new(),
            restore_ranges: Vec::new(),
            census_ended: 0,
            hold_restart_reads: false,
            hold_restore_pages: false,
            store_delay: 0,
            fill: 0,
            stopped: false,
            period_started: false,
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
        let mut peers = peers;
        peers.creating = Some(world.creating.clone());
        world.peers = Some(peers);
        world.cut = cut;
        world.events.push_back(root::Event::RestartBegin);
        world.hello_engine();
        world.events.push_back(root::Event::Core(core::Event::Account(accounts::Event::Component(
            accounts::FromComponent::Granted { account: 1, generation: 1, valid: Duration::from_secs(60) },
        ))));
        world.drain();
        world
    }
}
