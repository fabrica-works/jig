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
                models: Box::new([]),
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
