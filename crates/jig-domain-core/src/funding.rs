//! Project periods from the root and person allotments from policy
//! (protocol/configuration.md, 3; domain/authority.md, 7).
use crate::Core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

impl Core {
    /// The latest period the root has opened for this project, restored with its ledger.
    #[must_use]
    pub fn period(&self, project: u32) -> u64 {
        let mut current = 0;
        for (funder, _) in self.tasks.funding_records() {
            match funder {
                tasks::Funder::Period { project: owner, period } => {
                    if *owner == project {
                        current = current.max(*period);
                    }
                }
                tasks::Funder::Pool { .. } | tasks::Funder::Recurring { .. } | tasks::Funder::Task(_) => {}
            }
        }
        current
    }
    pub(crate) fn period_budget(&self, project: u32) -> u64 {
        match self.tasks.funding(tasks::Funder::Period { project, period: self.period(project) }) {
            Some(row) => row.numbers.budget,
            None => 0,
        }
    }
    pub(crate) fn pool_budget(&self, project: u32, person: u64) -> u64 {
        match self.people.role(person, project) {
            Some(role) => match self.authority.role(project, role.number()) {
                Some(policy) => policy.period_spend,
                None => 0,
            },
            None => 0,
        }
    }
    /// The most a person's next task can take from its role and remaining pool.
    pub(crate) fn chat_authority(
        &self,
        project: u32,
        person: u64,
        role: people::Role,
        tasks_left: u32,
    ) -> jig_domain_authority::Authority {
        let mut given = self.authority.role(project, role.number()).expect("configured role policy").authority.clone();
        let (_, pool) = self.goal_pool(project, person);
        given.delegation.tasks = given.delegation.tasks.min(tasks_left).saturating_sub(1);
        given.delegation.depth = given.delegation.depth.saturating_sub(1);
        let ask = jig_domain_authority::PersonAsk {
            project,
            role: role.number(),
            pool: crate::translate::authority_numbers(pool),
            tasks_left,
            request: jig_domain_authority::PersonRequest::Watch,
        };
        given.budget.spend = jig_domain_authority::as_allowed(&self.authority, &ask, 0, None);
        given
    }
}
