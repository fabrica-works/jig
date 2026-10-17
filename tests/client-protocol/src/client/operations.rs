//! External peer messages and durable task phases used by operation stories.
use super::Story;
use jig_client::{Key, contract};
use jig_domain_tasks as tasks;
use skein_lib::Duration;
const RUNNING_WAIT: Duration = Duration::from_secs(1);
impl Story {
    pub fn running_task(&mut self) -> u64 {
        let task = self.start_task();
        assert!(self.run_until(|s| s.phase(task) == Some(contract::Phase::Running), RUNNING_WAIT));
        task
    }
    #[must_use]
    pub fn heard_asked_to_finish(&self, task: u64) -> bool {
        let Some(message) = self.task_record(task).and_then(|r| r.grace.as_ref()).map(|g| g.message) else {
            return false;
        };
        self.engine
            .driver
            .world
            .peers
            .as_ref()
            .expect("scripted workers")
            .workers
            .iter()
            .any(|worker| worker.messages.iter().any(|(number, _, word, _)| *number == task && *word == message))
    }
    #[must_use]
    pub fn phase(&self, task: u64) -> Option<contract::Phase> {
        Some(match &self.task_record(task)?.phase {
            tasks::Phase::Waiting => contract::Phase::Waiting,
            tasks::Phase::Active(value) => match value {
                tasks::Active::Idle => contract::Phase::Idle,
                tasks::Active::Due => contract::Phase::Due,
                tasks::Active::Preparing => contract::Phase::Preparing,
                tasks::Active::Claimed { .. } => contract::Phase::Claimed,
                tasks::Active::Running { .. } => contract::Phase::Running,
                tasks::Active::BackingOff { .. } => contract::Phase::BackingOff,
            },
            tasks::Phase::Closing(_) => contract::Phase::Closing,
            tasks::Phase::Held { .. } => contract::Phase::Held,
            tasks::Phase::Ended(tasks::Ending::Done(_)) => contract::Phase::Done,
            tasks::Phase::Ended(tasks::Ending::Failed { .. }) => contract::Phase::Failed,
            tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) => contract::Phase::Cancelled,
        })
    }
    #[must_use]
    pub fn elapsed_since_answer(&self, key: Key) -> Duration {
        Duration::from_nanos(self.now.as_nanos().saturating_sub(self.front.told_at(key).as_nanos()))
    }
    fn task_record(&self, task: u64) -> Option<&tasks::TaskRecord> {
        self.engine.driver.world.store.rows.values().find_map(|row| match row {
            jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(
                tasks::Stored::Live(value) | tasks::Stored::Ended(value),
            )) if value.number == task => Some(value.as_ref()),
            jig_test_domain::Record::Core(_) | jig_test_domain::Record::Connector { .. } => None,
        })
    }
}

impl Story {
    #[must_use]
    pub fn hold_revision(&self, task: u64) -> contract::Revision {
        contract::Revision { value: self.task_record(task).expect("live task").revisions.hold }
    }
    #[must_use]
    pub fn activity_message(&self, task: u64, message: u64) -> Option<Vec<u8>> {
        self.engine.driver.world.store.rows.values().find_map(|record| match record {
            jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(line)))
                if line.task == task =>
            {
                match &line.line {
                    tasks::Line::Words { message: number, words, .. } if *number == message => Some(words.to_vec()),
                    tasks::Line::Heard(word) if word.number == message => Some(word.words.to_vec()),
                    tasks::Line::Words { .. }
                    | tasks::Line::Heard(_)
                    | tasks::Line::Notified { .. }
                    | tasks::Line::RunBegan { .. }
                    | tasks::Line::RunEnded { .. }
                    | tasks::Line::Turn { .. }
                    | tasks::Line::Decided { .. }
                    | tasks::Line::Milestone { .. } => None,
                }
            }
            jig_test_domain::Record::Core(_) | jig_test_domain::Record::Connector { .. } => None,
        })
    }
}

impl Story {
    #[must_use]
    pub fn worker_started_with(&self, task: u64, message: u64) -> bool {
        self.engine.driver.world.peers.as_ref().expect("workers").workers.iter().any(|worker| {
            worker.seen.iter().any(|assignment| assignment.task == task && assignment.messages.contains(&message))
        })
    }
}

impl Story {
    /// The core fixture admits two roots, optionally with no escalation policy.
    #[must_use]
    pub fn live_batch(seed: u64, holder: bool) -> Self {
        let mut limits = crate::fixtures::owner_limits();
        limits.core.people.sign_ins = 3;
        limits.core.tasks.batch = 2;
        limits.core.authority.batch = 2;
        let mut config = crate::fixtures::batch_config(seed);
        if !holder {
            use jig_domain_authority as a;
            let mut policy = config.core.authority.policy(1).expect("project policy").clone();
            policy.escalation_role = None;
            policy.roles[0].decides = a::Proposals(0);
            let mut out = skein_lib::Queue::with_capacity(a::POLICY_MAX_OUT);
            a::step(&mut config.core.authority, a::Event::Policy { project: 1, policy }, &mut out);
            assert_eq!(out.pop(), Some(a::PolicyFact::Changed { project: 1 }));
        }
        let mut story = Self::configured(seed, &limits, config);
        story.sign_in();
        assert!(story.run_until(|s| s.client.link().state == jig_client::Link::Live, RUNNING_WAIT));
        story
    }
    #[must_use]
    pub fn task_budget(&self, task: u64) -> Option<u64> {
        self.task_record(task).map(|v| v.numbers.budget)
    }
    #[must_use]
    pub fn worker_budget(&self, task: u64) -> Option<u64> {
        self.engine
            .driver
            .world
            .peers
            .as_ref()
            .expect("workers")
            .workers
            .iter()
            .flat_map(|w| &w.seen)
            .find(|v| v.task == task)
            .map(|v| v.budget)
    }
}

impl Story {
    #[must_use]
    pub fn party_proposal(&self, number: u64) -> Option<&tasks::PersonProposal> {
        self.engine.driver.world.store.rows.values().find_map(|row| stored_party_proposal(row, number))
    }
}

fn stored_party_proposal(row: &jig_test_domain::Record, number: u64) -> Option<&tasks::PersonProposal> {
    match row {
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(
            tasks::Stored::PersonProposal(value) | tasks::Stored::DecidedProposal(value),
        )) if value.number == number => Some(value.as_ref()),
        jig_test_domain::Record::Core(_) | jig_test_domain::Record::Connector { .. } => None,
    }
}

impl Story {
    /// A batch fixture whose owner may create a person task at their held role.
    #[must_use]
    pub fn live_person_batch(seed: u64) -> Self {
        let mut story =
            Self::configured(seed, &crate::fixtures::query_limits(), crate::fixtures::person_batch_config(seed));
        story.sign_in();
        assert!(story.run_until(|s| s.client.link().state == jig_client::Link::Live, RUNNING_WAIT));
        story
    }
}
