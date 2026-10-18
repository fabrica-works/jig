//! Policy roles hold roots and receive person-task delegations by policy, not number.
use super::*;

const PROJECT: u32 = 1;
const REQUESTER: u64 = 1;
const TASK: u64 = 1;
const POLICY_ROLE: u32 = 5;

fn holder_policy(core: &mut Core, decides: authority::Proposals, holder: Option<u32>) {
    let mut policy = core.authority.policy(PROJECT).unwrap().clone();
    policy.roles[0].decides = decides;
    policy.escalation_role = holder;
    if let Some(number) = holder {
        let mut added = policy.roles[0].clone();
        added.number = number;
        added.decides = authority::Proposals::ALL;
        policy.roles = Box::new([policy.roles[0].clone(), added]);
    }
    let mut bounds = *core.authority.limits();
    bounds.roles = 2;
    let rules = core.authority.rules().clone();
    core.authority = authority::Domain::new(rules, bounds).unwrap();
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut core.authority, authority::Event::Policy { project: PROJECT, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: PROJECT }));
}

#[test]
fn a_chat_is_refused_while_nobody_holds_its_escalations() {
    let (mut core, _) = core();
    holder_policy(&mut core, authority::Proposals(0), None);
    assert_eq!(
        crate::acts::bounds::create_start(
            &core,
            PROJECT,
            REQUESTER,
            people::Role::Owner,
            tasks::Executor::Agent { charter: 1 },
            10,
            None,
            2
        )
        .unwrap_err(),
        people::Refusal::NoEscalationHolder
    );
}

#[test]
fn a_goal_is_refused_while_nobody_holds_its_escalations() {
    let (mut core, _) = core();
    holder_policy(&mut core, authority::Proposals(0), None);
    assert_eq!(
        crate::acts::bounds::create_start(
            &core,
            PROJECT,
            REQUESTER,
            people::Role::Owner,
            tasks::Executor::Agent { charter: 1 },
            10,
            None,
            2
        )
        .unwrap_err(),
        people::Refusal::NoEscalationHolder
    );
}

#[test]
fn a_policy_role_numbered_five_holds_a_chats_escalations() {
    let (mut core, _) = core();
    holder_policy(&mut core, authority::Proposals(0), Some(POLICY_ROLE));
    crate::acts::bounds::create_start(
        &core,
        PROJECT,
        REQUESTER,
        people::Role::Owner,
        tasks::Executor::Agent { charter: 1 },
        10,
        None,
        2,
    )
    .unwrap();
}

#[test]
fn a_person_task_for_policy_role_five_is_offered_to_delegation() {
    let (mut core, _) = core();
    let executor = core.delegation_executor(PROJECT, tasks::Executor::Person(tasks::PersonAddress::Role(POLICY_ROLE)));
    assert_eq!(executor, Some(authority::Executor::Role(POLICY_ROLE)));
    let mut policy = core.authority.policy(PROJECT).unwrap().clone();
    let mut given = policy.roles[0].authority.clone();
    given.delegation.kinds = Box::new([executor.unwrap()]);
    let mut rules = core.authority.rules().clone();
    rules.ceiling.delegation.kinds = given.delegation.kinds.clone();
    policy.ceiling.delegation.kinds = given.delegation.kinds.clone();
    policy.roles[0].authority = given.clone();
    let bounds = *core.authority.limits();
    core.authority = authority::Domain::new(rules, bounds).unwrap();
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut core.authority, authority::Event::Policy { project: PROJECT, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: PROJECT }));
    let creator = given.clone();
    given.delegation.tasks = 0;
    given.delegation.depth = 0;
    let ask = authority::BatchAsk {
        project: PROJECT,
        creator,
        numbers: authority::Numbers { budget: 500, spent: 0, spent_below: 0, reserved: 0 },
        tasks_left: 2,
        tasks: Box::new([authority::Delegate {
            executor: executor.unwrap(),
            authority: given,
            symbolic: Box::new([]),
        }]),
    };
    let mut findings = Queue::with_capacity(authority::max_out(core.authority.limits()).unwrap());
    assert_eq!(authority::check_batch(&core.authority, &ask, &mut findings).answer, authority::Answer::Allow);
}

#[test]
fn a_restored_person_task_for_policy_role_five_is_supported() {
    let (mut core, limits) = tree_core();
    make_task(&mut core, &limits, TASK);
    let mut row = core.tasks.task(TASK).unwrap().clone();
    row.executor = tasks::Executor::Person(tasks::PersonAddress::Role(POLICY_ROLE));
    assert_eq!(fresh(&mut core.counters, Family::Task), Some(TASK));
    assert_eq!(fresh(&mut core.counters, Family::Person), Some(REQUESTER));
    assert!(core.restore_task_row(&tasks::Stored::Live(Box::new(row))));
}

#[test]
fn a_move_is_refused_while_nobody_holds_its_escalations() {
    let (mut core, limits) = tree_core();
    make_task(&mut core, &limits, TASK);
    holder_policy(&mut core, authority::Proposals(0), None);
    assert_eq!(
        core.move_admit(REQUESTER, Some(people::Role::Owner), PROJECT, TASK, 2, 2),
        Err(people::Refusal::NoEscalationHolder)
    );
}
