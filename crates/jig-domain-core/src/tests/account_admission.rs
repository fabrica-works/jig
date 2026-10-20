//! Shared account admission uses the financial chain at each public core entry point.

use super::*;
fn family() -> (Core, Limits) {
    let (mut core, limits) = tree_core();
    core.accounts = accounts::Domain::new(&limits.accounts, core.settings.llm_providers.clone());
    let mut out = Queue::with_capacity(accounts::max_out(&limits.accounts));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts };
    for record in [
        accounts::Stored::Account {
            number: 0,
            provider: 1,
            holder: accounts::Holder::Party(1),
            place: 0,
            uses: None,
            subject: Box::new([]),
            display: Box::new([]),
        },
        accounts::Stored::Source {
            account: 0,
            number: 1,
            place: 0,
            kind: accounts::Kind::Key,
            name: Box::new([]),
            held: None,
        },
    ] {
        accounts::step(&mut core.accounts, &env, accounts::Event::Restore { record }, &mut out);
    }
    let root = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([root]));
    let child = new_row(&core, 2, tasks::Funder::Task(1), None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Task(1), Box::new([child]));
    (core, limits)
}
fn choice() -> tasks::LlmChoice {
    tasks::LlmChoice {
        account: Some(0),
        main: tasks::ModelEffort { model: 0, effort: Some(0) },
        subagents: Box::new([]),
    }
}
#[test]
fn a_party_amendment_uses_its_targets_financial_payer() {
    let (core, limits) = family();
    let mut edit = fixtures::unchanged(b"choose");
    edit.llm = Some(choice());
    assert_eq!(
        core.amend_admit(1, Some(people::Role::Owner), 1, 2, &edit, limits.tasks.depth, &limits.tasks),
        Err(people::Refusal::Account { account: 0, why: people::AccountWhy::Uses })
    );
}
#[test]
fn a_task_amendment_uses_its_targets_financial_payer() {
    let (core, limits) = family();
    let mut edit = fixtures::unchanged(b"choose");
    edit.llm = Some(choice());
    assert_eq!(
        core.task_amend_admit(1, 2, &edit, limits.tasks.depth, &limits.tasks),
        Err(TaskAmendDenied::Refused {
            task: 2,
            why: tasks::Refusal::Account { account: 0, why: tasks::AccountWhy::Uses }
        })
    );
}
#[test]
fn a_delegate_preflight_uses_the_parents_financial_payer() {
    let (mut core, limits) = family();
    core.proofs.insert(1, RunProof::claimed(1, 1, 1, None, fleet::HostKind::Worker)).unwrap();
    let member = new_row(&core, 3, tasks::Funder::Task(1), None);
    let delegate = Delegate {
        executor: member.executor,
        spec: member.spec,
        contract: member.contract,
        authority: member.authority,
        llm: Some(choice()),
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: member.wake,
        tracked: None,
        label: 0,
        flags: tasks::Flags::PLAIN,
        budget: people::Budget::Named,
    };
    let key = CallKey { task: 1, attempt: 1, completion: 1, position: 0 };
    assert_eq!(
        core.delegate_preflight(&limits.tasks, key, &[delegate]),
        Err(CallPart::DelegationRefused(tasks::Problem::new(
            None,
            tasks::Refusal::Account { account: 0, why: tasks::AccountWhy::Uses }
        )))
    );
}
