use super::*;

#[test]
#[should_panic(expected = "one account per charter")]
fn a_charter_whose_alternative_names_another_account_is_refused() {
    let mut config = config(1);
    let mut alternative = config.settings.charters[0].policy.model.clone();
    alternative.account = 2;
    config.settings.charters[0].policy.alternatives = Box::new([alternative]);
    let _core = Core::new(config, &limits());
}

#[test]
#[should_panic(expected = "charter default spend holds reserve floor")]
fn a_charter_whose_default_spend_misses_its_floor_is_refused() {
    let mut config = config(1);
    config.settings.charters[0].spend = 1;
    let _core = Core::new(config, &limits());
}

pub(super) fn context(task: u64, charter: u32) -> Box<tasks::RunContext> {
    Box::new(tasks::RunContext {
        task,
        last_message: 0,
        inbox: Box::new([]),
        delegates: Box::new([]),
        dependencies: Box::new([]),
        saved: Box::new([]),
        previous_attempt: 0,
        ever_turned: false,
        tries: tasks::Tries::NONE,
        invalid_result: None,
        project: 1,
        executor: tasks::Executor::Agent { charter },
        spec: tasks::Spec { words: b"task".as_slice().into(), parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: 64 },
        requester: tasks::Party::Person(1),
        authority: task_authority(&config(1).authority.role(1, 0).unwrap().authority),
        numbers: tasks::Numbers { budget: 20, spent: 0, spent_below: 0, reserved: 0 },
    })
}

#[test]
fn take_due_skips_a_task_whose_charters_account_is_unusable() {
    let mut config = config(1);
    let mut limits = limits();
    limits.tasks.charters = 2;
    limits.accounts.accounts = 2;
    let mut second = config.settings.charters[0].clone();
    second.number = 2;
    second.policy.model.account = 2;
    config.settings.charters = Box::new([config.settings.charters[0].clone(), second]);
    let mut core = Core::new(config, &limits);
    let mut out = Queue::with_capacity(accounts::max_out(&limits.accounts));
    accounts::step(
        &mut core.accounts,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts },
        accounts::Event::Want,
        &mut out,
    );
    while out.pop().is_some() {}
    accounts::step(
        &mut core.accounts,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts },
        accounts::Event::Component(accounts::FromComponent::Granted {
            form: accounts::Form::Credential(accounts::Scheme::Bearer),
            account: 2,
            generation: 1,
            valid: Duration::from_secs(60),
        }),
        &mut out,
    );
    core.due.push(context(1, 1));
    core.due.push(context(2, 2));
    assert!(core.due_ready());
    assert_eq!(core.take_due().expect("second ready").task, 2);
    assert!(!core.due_ready());
    assert_eq!(core.due.pop().expect("first retains order").task, 1);
}
