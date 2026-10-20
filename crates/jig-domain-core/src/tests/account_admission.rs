//! Shared account admission uses the financial chain at each public core entry point.

use super::*;
const ACCOUNT: u32 = 0;
const SOURCE: u32 = 1;
const PROJECT: u32 = 1;
const HOLDER: u64 = 1;
const RECURRING_TASK: u64 = 100;
const PERIOD: u64 = 1;

fn restore_account(core: &mut Core, limits: &Limits, uses: Option<Box<[Use]>>) {
    core.accounts = accounts::Domain::new(&limits.accounts, core.settings.llm_providers.clone());
    let mut out = Queue::with_capacity(accounts::max_out(&limits.accounts));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts };
    for record in [
        accounts::Stored::Account {
            number: ACCOUNT,
            provider: 1,
            holder: accounts::Holder::Party(HOLDER),
            place: 0,
            uses,
            subject: Box::new([]),
            display: Box::new([]),
        },
        accounts::Stored::Source {
            account: ACCOUNT,
            number: SOURCE,
            place: 0,
            kind: accounts::Kind::Key,
            name: Box::new([]),
            held: None,
        },
    ] {
        accounts::step(&mut core.accounts, &env, accounts::Event::Restore { record }, &mut out);
    }
}
fn family() -> (Core, Limits) {
    let (mut core, limits) = tree_core();
    restore_account(&mut core, &limits, None);
    let root = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([root]));
    let child = new_row(&core, 2, tasks::Funder::Task(1), None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Task(1), Box::new([child]));
    (core, limits)
}
fn choice() -> tasks::LlmChoice {
    tasks::LlmChoice {
        account: Some(ACCOUNT),
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
        Err(people::Refusal::Account { account: ACCOUNT, why: people::AccountWhy::Uses })
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
            why: tasks::Refusal::Account { account: ACCOUNT, why: tasks::AccountWhy::Uses }
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
            tasks::Refusal::Account { account: ACCOUNT, why: tasks::AccountWhy::Uses }
        )))
    );
}

#[test]
fn an_accepter_can_fund_the_proposed_private_account() {
    let (mut core, limits) = family();
    core.people = watch_core().0.people;
    let mut member = new_row(&core, 3, tasks::Funder::Period { project: 1, period: 1 }, None);
    member.llm = Some(choice());
    member.authority.delegation.tasks = 0;
    let action = tasks::ProposalAction::Batch(Box::new([member.clone()]));
    core.proposal_admit_action(1, &action, Wall::EPOCH).expect("proposal has no payer yet");
    let proposal = tasks::PersonProposal {
        number: 1,
        proposer: 2,
        project: 1,
        action: tasks::PersonAction::Batch(Box::new([member])),
        reason: Box::new([]),
        state: tasks::PersonProposalState::Pending { since: Wall::EPOCH },
    };
    assert_eq!(
        core.goal_accept_allowed(&proposal, 1, 1, people::Role::Owner, limits.tasks.tasks, Wall::EPOCH),
        Ok(tasks::Funder::Pool { project: 1, person: 1, period: 1 })
    );
}

#[test]
fn a_non_agent_executor_refuses_an_explicit_account() {
    let (core, _) = family();
    for executor in
        [tasks::Executor::Procedure { connector: 1, code: 1 }, tasks::Executor::Person(tasks::PersonAddress::Person(1))]
    {
        let mut member = new_row(&core, 3, tasks::Funder::Period { project: 1, period: 1 }, None);
        member.executor = executor;
        member.llm = Some(choice());
        assert_eq!(
            core.resolve_llm(&mut [member]),
            Err(tasks::Problem::new(Some(3), tasks::Refusal::Llm { model: 0, why: tasks::LlmFault::Procedure }))
        );
    }
}

#[test]
fn a_recurring_hint_uses_the_project_payer() {
    let (core, _) = family();
    let mut member = new_row(&core, 1, tasks::Funder::Pool { project: 1, person: 1, period: 1 }, None);
    member.llm = Some(choice());
    let mut parent = new_row(&core, 3, tasks::Funder::Period { project: 1, period: 1 }, None);
    parent.executor = tasks::Executor::Procedure { connector: RECURRING, code: 1 };
    parent.recurring = Some(Box::new(tasks::RecurringTemplate {
        key: 1,
        batch: Box::new([member]),
        overlap: tasks::RecurringOverlap::Skip,
    }));
    assert_eq!(
        core.resolve_llm(&mut [parent]),
        Err(tasks::Problem::new(Some(3), tasks::Refusal::Account { account: ACCOUNT, why: tasks::AccountWhy::Uses }))
    );
}

fn narrowed_recurring() -> (Core, Limits, u64) {
    let (mut core, mut limits) = tree_core();
    let (authenticated, auth_limits) = watch_core();
    core.people = authenticated.people;
    limits.people = auth_limits.people;
    restore_account(&mut core, &limits, Some(Box::new([Use::Project(PROJECT)])));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts };
    let mut out = Queue::with_capacity(accounts::max_out(&limits.accounts));
    accounts::step(&mut core.accounts, &env, accounts::Event::Want, &mut out);
    let mut parent = new_row(&core, RECURRING_TASK, tasks::Funder::Period { project: PROJECT, period: PERIOD }, None);
    parent.executor = tasks::Executor::Procedure { connector: RECURRING, code: 1 };
    parent.numbers = tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
    parent.authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Charter(1)]);
    let mut member = new_row(&core, 1, tasks::Funder::Pool { project: 1, person: 1, period: 1 }, None);
    member.llm = Some(choice());
    member.authority.delegation.kinds = Box::new([]);
    member.authority.delegation.tasks = 0;
    member.authority.delegation.depth = 0;
    parent.recurring = Some(Box::new(tasks::RecurringTemplate {
        key: 1,
        batch: Box::new([member]),
        overlap: tasks::RecurringOverlap::Skip,
    }));
    core.resolve_llm(core::slice::from_mut(&mut parent)).unwrap();
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(1)),
            creator: tasks::Party::Deployment { project: 1 },
            batch: Box::new([parent]),
        },
    );
    while out.pop().is_some() {}
    accounts::step(
        &mut core.accounts,
        &env,
        accounts::Event::Change {
            request: Token::new(2),
            holder: accounts::Holder::Party(HOLDER),
            change: accounts::Change::SetUses { account: ACCOUNT, uses: Some(Box::new([])) },
        },
        &mut out,
    );
    step(
        &mut core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        Event::Tasks(tasks::Event::TickRecurring { task: RECURRING_TASK, period: PERIOD }),
    );
    let number = core.tasks.task(RECURRING_TASK).unwrap().recurring.as_ref().unwrap().last_batch[0];
    (core, limits, number)
}

#[test]
fn a_recurring_occurrence_waits_for_its_narrowed_account() {
    let (core, _, number) = narrowed_recurring();
    assert_eq!(
        core.run_choice(number),
        Err(RunWait::InvalidAccount { account: ACCOUNT, why: tasks::AccountWhy::Uses })
    );
}

#[test]
fn a_recurring_task_document_names_its_invalid_account_reason() {
    let (core, limits, number) = narrowed_recurring();
    let Document::Task(doc) = make(&core, &limits, 1, &Subject::Task { task: number }, &window(), Wall::EPOCH).unwrap()
    else {
        panic!("task document")
    };
    assert_eq!(
        doc.run_wait,
        Some(TaskRunWait::Account {
            account: Some(ACCOUNT),
            source: None,
            why: TaskWaitWhy::Invalid(tasks::AccountWhy::Uses),
        })
    );
}
