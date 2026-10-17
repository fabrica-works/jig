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
