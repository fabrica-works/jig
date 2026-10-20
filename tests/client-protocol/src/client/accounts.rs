//! Inputs and commit pacing for LLM account sign-in protocol stories.
use super::{Duration, Story, client, fixtures};
use jig_domain_accounts as accounts;

impl Story {
    #[must_use]
    pub fn account_sign_in(seed: u64, offered: bool) -> Self {
        let mut config = jig_core_world::world::config(seed);
        config.core.settings.llm_providers = accounts::Providers {
            providers: Box::new([accounts::Provider {
                name: b"Fake".as_slice().into(),
                signed_in_plan: true,
                manage_usage: Some(b"https://usage.example/".as_slice().into()),
                number: 3,
                models: Box::new([config.core.settings.models[0].name.clone()]),
                offers: accounts::Offers { sign_in: offered, key: false, programs: Box::new([]) },
            }]),
        };
        let mut limits = fixtures::owner_limits();
        limits.core.accounts.accounts = 2;
        limits.core.people.sign_ins = 3;
        let mut story = Self::configured(seed, &limits, config);
        story.sign_in();
        assert!(story.run_until(|s| s.client.link().state == client::Link::Live, Duration::from_secs(1)));
        story
    }
    /// The fake accounts component supplies the newly signed-in source's first grant.
    pub fn grant_account_sign_in(&mut self, source: u32) {
        self.engine.driver.world.send(jig_test_domain::Event::Core(jig_domain_core::Event::Account(
            accounts::Event::Component(accounts::FromComponent::Granted {
                form: accounts::Form::Credential(accounts::Scheme::Bearer),
                account: source,
                generation: 1,
                valid: Duration::from_secs(60),
            }),
        )));
    }
    /// Account key inputs observed after the fake secrets store kept their values.
    #[must_use]
    pub fn account_key_inputs(&self) -> &[jig_domain_core::Carried] {
        &self.engine.account_keys
    }
    #[must_use]
    pub fn malformed_requests(&self) -> u32 {
        self.engine.not_records
    }
    #[must_use]
    pub fn erased_carried(&self) -> &[u64] {
        &self.engine.erasures
    }
    /// Queue complete client account sign-in inputs as one burst into the existing core world.
    pub fn hold_account_sign_in_burst(&mut self) {
        self.engine.account_sign_in_burst = Some(Vec::new());
    }
    #[must_use]
    pub fn account_sign_in_burst_len(&self) -> usize {
        self.engine.account_sign_in_burst.as_ref().map_or(0, Vec::len)
    }
    pub fn release_account_sign_in_burst(&mut self) {
        let events = self.engine.account_sign_in_burst.take().expect("held account sign-in burst");
        self.engine.driver.world.send_all(events);
    }
}

impl Story {
    /// Room for the deployment's account and two client-created own key accounts.
    #[must_use]
    pub fn named_accounts(seed: u64) -> Self {
        let mut limits = fixtures::owner_limits();
        limits.core.accounts.accounts = 3;
        limits.core.accounts.holder_accounts = 2;
        limits.core.people.sign_ins = 3;
        // Two keys, Create, SetUses, Stop, Move, Stop and Release, retained together.
        limits.core.people.requests = 8;
        limits.core.people.projects = 2;
        limits.core.tasks.batch = 2;
        // The two public Create members and the later project-funded copy.
        limits.core.tasks.project_tasks = 3;
        limits.core.tasks.tasks = 3;
        limits.core.authority.batch = 2;
        let mut config = fixtures::batch_config(seed);
        let mut policy = config.core.authority.policy(1).expect("story project policy").clone();
        // Leave 100 in the project's period for the Move story's directly funded task.
        policy.roles[0].period_spend = policy.period_spend - 100;
        let mut out = skein_lib::Queue::with_capacity(jig_domain_authority::POLICY_MAX_OUT);
        jig_domain_authority::step(
            &mut config.core.authority,
            jig_domain_authority::Event::Policy { project: 1, policy },
            &mut out,
        );
        assert_eq!(out.pop(), Some(jig_domain_authority::PolicyFact::Changed { project: 1 }));
        let mut story = Self::configured(seed, &limits, config);
        story.sign_in();
        assert!(story.run_until(|s| s.client.link().state == client::Link::Live, Duration::from_secs(1)));
        story.grant_deployment_role();
        story
    }
    /// Script the lower account input; public Resume still crosses the real client path.
    pub fn limit_account_source(&mut self, source: u32) {
        self.engine.driver.world.send(jig_test_domain::Event::Core(jig_domain_core::Event::Account(
            accounts::Event::Limited { account: source, reset: None },
        )));
    }
    /// An application's project-funded task, copied from a client's ordinary bounded spec.
    pub fn project_paid_copy(&mut self, task: u64, number: u64) {
        use jig_domain_tasks as tasks;
        let row = self.task_record(task).expect("source task").clone();
        let period = match row.funder {
            tasks::Funder::Pool { period, .. } | tasks::Funder::Period { period, .. } => period,
            tasks::Funder::Task(_) | tasks::Funder::Recurring { .. } => panic!("direct source task"),
        };
        let mut authority = row.authority;
        authority.budget.spend = 20;
        self.engine.driver.world.send(jig_test_domain::Event::Core(jig_domain_core::Event::Tasks(
            tasks::Event::Make {
                reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(u64::MAX - 3)),
                creator: tasks::Party::Deployment { project: row.project },
                batch: Box::new([tasks::New {
                    deadline: tasks::Deadline::Named,
                    number,
                    project: row.project,
                    executor: row.executor,
                    llm: None,
                    spec: row.spec,
                    contract: row.contract,
                    authority,
                    numbers: tasks::Numbers { budget: 20, spent: 0, spent_below: 0, reserved: 0 },
                    funder: tasks::Funder::Period { project: row.project, period },
                    dependencies: Box::new([]),
                    holdings: Box::new([]),
                    wake: row.wake,
                    recurring: None,
                    tracked: None,
                    label: row.label,
                    flags: row.flags,
                }]),
            },
        )));
    }
}

impl Story {
    /// Public program requests have no secret-bearing component below accounts.
    #[must_use]
    pub fn program_sources(seed: u64) -> Self {
        let mut config = jig_core_world::world::config(seed);
        config.core.worker_names = Box::new([b"unavailable".as_slice().into()]);
        config.core.settings.llm_providers = accounts::Providers {
            providers: Box::new([accounts::Provider {
                number: 3,
                name: b"Fake".as_slice().into(),
                signed_in_plan: false,
                manage_usage: Some(b"https://usage.example/".as_slice().into()),
                models: Box::new([]),
                offers: accounts::Offers {
                    sign_in: false,
                    key: false,
                    programs: Box::new([b"claude_code".as_slice().into()]),
                },
            }]),
        };
        let mut limits = fixtures::owner_limits();
        limits.core.people.sign_ins = 3;
        limits.core.accounts.accounts = 2;
        limits.core.accounts.holder_accounts = 2;
        limits.core.people.requests = 8;
        let mut story =
            Self::configured_peers(seed, &limits, config, jig_core_world::peers::Peers::new(Vec::new(), Vec::new()));
        story.sign_in();
        assert!(story.run_until(|s| s.client.link().state == client::Link::Live, Duration::from_secs(1)));
        story
    }
}
