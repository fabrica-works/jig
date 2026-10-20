use super::*;

#[test]
fn a_run_waits_while_a_credential_it_needs_is_unusable() {
    let (mut core, limits) = core();
    make_task(&mut core, &limits, 1);
    let mut placed = Queue::with_capacity(fleet::max_out(&limits.fleet));
    fleet::step(
        &mut core.fleet,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet },
        fleet::Event::Hello {
            channel: Token::new(1),
            hello: fleet::Hello {
                bounds: fixtures::bounds(),
                stop_bound: Duration::ZERO,
                slots: 1,
                workstreams: Box::new([]),
                hosting: Box::new([]),
            },
        },
        &mut placed,
    );
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts };
    let mut out = Queue::with_capacity(accounts::max_out(&limits.accounts));
    accounts::step(&mut core.accounts, &env, accounts::Event::Want, &mut out);
    while out.pop().is_some() {}
    accounts::step(
        &mut core.accounts,
        &env,
        accounts::Event::Component(accounts::FromComponent::Granted {
            form: accounts::Form::Credential(accounts::Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        }),
        &mut out,
    );
    while out.pop().is_some() {}
    assert_eq!(core.workspace_credentials.insert(1, Box::new([7])), Ok(None));
    let context = charters::context(1, 1);
    assert_eq!(core.run_admission(&context, Wall::EPOCH, Box::new([])), RunAdmission::Wait(RunWait::Credential));
    core.due.push(context);
    assert!(!core.due_ready());
    accounts::step(
        &mut core.accounts,
        &env,
        accounts::Event::Credential { number: 7, generation: 1, valid: Duration::from_secs(60) },
        &mut out,
    );
    let context = core.take_due().expect("credential frees the waiting task");
    let RunAdmission::Allow(_) = core.run_admission(&context, Wall::EPOCH, Box::new([])) else {
        panic!("credential permits the current allowance")
    };
}
