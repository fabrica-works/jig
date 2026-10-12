//! Charter fixtures and agent actions shared by the core world's stories.

use crate::effects::World;
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, Queue, ReplyTo, Token};

#[must_use]
pub fn research_charter() -> core::Charter {
    let mut charter = crate::world::config(0).core.settings.charters[0].clone();
    charter.number = 2;
    charter.policy.instructions = b"Research the question.".as_slice().into();
    charter.policy.model.account = 2;
    charter.policy.model.name = b"research".as_slice().into();
    charter
}

#[must_use]
pub fn two_charters(seed: u64) -> (root::Config, root::Limits) {
    let (mut config, mut limits) = crate::effects::fixture(seed, false);
    limits.core.tasks.charters = 2;
    limits.core.tasks.executor_kinds = 3;
    limits.core.fleet.slots = 2;
    limits.core.authority.executors = 3;
    limits.core.accounts.accounts = 2;
    let kinds: Box<[authority::Executor]> =
        Box::new([authority::Executor::Charter(1), authority::Executor::Charter(2), authority::Executor::Procedure(1)]);
    let mut rules = config.core.authority.rules().clone();
    rules.ceiling.delegation.kinds.clone_from(&kinds);
    let mut policy = config.core.authority.policy(1).expect("project policy").clone();
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    policy.roles[0].authority.delegation.kinds.clone_from(&kinds);

    config.core.authority = authority::Domain::new(rules, limits.core.authority).expect("charter policy");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut config.core.authority, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    config.core.settings.charters = Box::new([config.core.settings.charters[0].clone(), research_charter()]);
    let room = core::room_max(&limits.core).expect("charter room");
    let connectors = jig_test_connector::MAX_OUT * 2;
    limits.journal.writes = limits.journal.writes.max(room.writes + connectors);
    limits.journal.held = limits.journal.held.max(room.held + connectors);
    (config, limits)
}

impl World {
    /// Make this charter's account usable without changing another charter's grant.
    pub fn grant_account(&mut self, account: u32) {
        self.send(root::Event::Core(core::Event::Account(accounts::Event::Component(
            accounts::FromComponent::Granted { account, generation: 1, valid: Duration::from_secs(60) },
        ))));
    }

    /// Delegate one agent task from the currently assigned chat.
    pub fn delegate_agent(&mut self, charter: u32, words: &[u8]) {
        let mut authority = self.assigned.as_ref().expect("assigned caller").run.authority.clone();
        authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
        authority.budget.spend = 20;
        self.send(root::Event::Core(core::Event::DelegateValidated {
            to: ReplyTo::new(Token::new(201)),
            key: self.call_key(1),
            batch: Box::new([core::Delegate {
                executor: tasks::Executor::Agent { charter },
                spec: tasks::Spec { words: words.into(), parameters: Box::new([]), inputs: Box::new([]) },
                contract: tasks::Contract::Report { words: 128 },
                authority,
                symbolic_grants: Box::new([]),
                dependencies: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
            }]),
            stubs: Box::new([]),
        }));
    }
}
