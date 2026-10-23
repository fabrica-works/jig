//! Queries and creation fixtures shared by the real byte stories.
use jig_client::{Handle, Latest, contract as c};
use skein_lib::List;
pub const FIRST_QUERY_TASK: u64 = 1;
pub const SECOND_QUERY_TASK: u64 = 2;
#[must_use]
pub fn task_query(project: u32) -> c::TaskQuery {
    c::TaskQuery {
        project: Some(project),
        requester: None,
        root: None,
        parent: None,
        label: None,
        phase: None,
        executor: None,
        tracked: Some(true),
        order: c::QueryOrder::Priority,
    }
}
#[must_use]
pub fn tracked_tasks() -> List<c::NewTask> {
    let mut tasks = super::create_tasks();
    tasks.get_mut(0).expect("first member").tracked = Some(1);
    tasks
}
#[must_use]
pub fn task_lines(story: &crate::client::Story, handle: Handle) -> Option<&c::Tasks> {
    match story.client.document(handle)?.latest {
        Some(Latest::Jig(c::Delivery::Tasks(tasks))) => Some(tasks),
        _ => None,
    }
}
#[must_use]
pub fn person_tasks() -> List<c::NewTask> {
    let mut tasks = tracked_tasks();
    tasks.get_mut(0).expect("first member").executor =
        c::OwnerExecutor::Person(c::PersonAddress::Party(super::FIRST_OWNER));
    tasks
}
#[must_use]
pub fn person_batch_config(seed: u64) -> jig_test_domain::Config {
    use jig_domain_authority as a;
    let mut config = super::batch_config(seed);
    let mut rules = config.core.authority.rules().clone();
    let mut policy = config.core.seed_projects[0].policy.policy.clone();
    let kinds: Box<[a::Executor]> = Box::new([a::Executor::Charter(1), a::Executor::Role(0)]);
    rules.ceiling.delegation.kinds.clone_from(&kinds);
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    policy.roles[0].authority.delegation.kinds = kinds;
    let mut authority = a::Domain::new(rules, *config.core.authority.limits()).expect("person rules");
    let mut out = skein_lib::Queue::with_capacity(a::POLICY_MAX_OUT);
    a::step(&mut authority, a::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(a::PolicyFact::Added { project: 1 }));
    config.core.seed_projects[0].policy.policy = authority.policy(1).expect("checked seed policy").clone();
    config.core.authority = a::Domain::new(authority.rules().clone(), *authority.limits()).expect("checked rules");
    config
}
#[must_use]
pub fn query_limits() -> jig_test_domain::Limits {
    let mut limits = super::owner_limits();
    limits.core.people.sign_ins = 3;
    limits.core.tasks.batch = 2;
    limits.core.authority.batch = 2;
    super::journal(limits)
}

#[must_use]
pub fn tracked_rows(story: &crate::client::Story, handle: Handle) -> Vec<(u64, Option<u32>)> {
    task_lines(story, handle)
        .expect("query story fixture and delivery")
        .rows
        .iter()
        .map(|r| (r.chip.task, r.tracked))
        .collect()
}
#[must_use]
pub fn tracked_amendment(priority: u32) -> c::Amendment {
    c::Amendment {
        llm: None,
        addressed: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        instructions: None,
        tracked: Some(c::Tracking::Tracked { priority }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_wait_story_creates_a_tracked_person_task() {
        let limits = query_limits();
        let ask = jig_client_world::joint::wire::asks::ask(&c::Ask::Create { project: 1, tasks: person_tasks() }, &[])
            .expect("whole create");
        let world = jig_core_world::effects::World::configured_with(
            super::super::SEED,
            false,
            person_batch_config(super::super::SEED),
            super::super::journal(limits),
            ask,
        );
        assert_eq!(
            world.people_answers.last(),
            Some(&jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::Created {
                first: FIRST_QUERY_TASK,
                last: SECOND_QUERY_TASK,
                given: Some(jig_domain_people::Given { task: FIRST_QUERY_TASK, amount: 20 })
            }))
        );
    }
}
