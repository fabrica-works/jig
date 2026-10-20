//! The run's data check is pure; the core world only tests routing.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "small input tables are ordinary Rust tests"
)]
use super::*;
const TASK: u64 = 1;
const FIRST: Token = Token::new(7);
const SECOND: Token = Token::new(11);

fn account(core: &mut Core, limits: &Limits, event: accounts::Event) {
    let mut out = Queue::with_capacity(accounts::max_out(&limits.accounts));
    accounts::step(
        &mut core.accounts,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts },
        event,
        &mut out,
    );
}
fn granted(core: &mut Core, limits: &Limits, source: u32) {
    account(
        core,
        limits,
        accounts::Event::Component(accounts::FromComponent::Granted {
            form: accounts::Form::Credential(accounts::Scheme::Bearer),
            account: source,
            generation: 1,
            valid: Duration::from_secs(60),
        }),
    );
}
fn host(core: &mut Core, limits: &Limits, channel: Token, names: &[&[u8]]) {
    let mut bounds = fixtures::bounds();
    bounds.models =
        names.iter().map(|name| fleet::ModelBound { name: (*name).into(), window: 8192, output: 1024 }).collect();
    let mut out = Queue::with_capacity(fleet::max_out(&limits.fleet));
    fleet::step(
        &mut core.fleet,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet },
        fleet::Event::Hello {
            channel,
            hello: fleet::Hello {
                bounds,
                stop_bound: Duration::ZERO,
                slots: 1,
                workstreams: Box::new([]),
                hosting: Box::new([]),
            },
        },
        &mut out,
    );
    fleet::step(
        &mut core.fleet,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet },
        fleet::Event::Logins { channel, logins: Box::new([]) },
        &mut out,
    );
}
fn ready(none: bool) -> (Core, Limits) {
    let (base, mut limits) = core();
    limits.accounts.accounts = 2;
    limits.fleet.workers = 2;
    limits.run_bytes = 4096;
    let mut config = config(41);
    let mut same = scripted_model();
    same.name = b"same".as_slice().into();
    let mut other = scripted_model();
    other.name = b"other".as_slice().into();
    config.settings.models = Box::new([scripted_model(), same.clone(), other.clone()]);
    config.settings.charters[0].policy.alternatives = Box::new([same, other]);
    config.settings.llm_providers.providers[0].models =
        Box::new([b"scripted".as_slice().into(), b"same".as_slice().into()]);
    let provider = accounts::Provider {
        name: b"Fake".as_slice().into(),
        signed_in_plan: !none,
        manage_usage: Some(b"https://usage.example/".as_slice().into()),
        number: 2,
        models: Box::new([b"other".as_slice().into()]),
        offers: accounts::Offers { sign_in: !none, key: false, programs: Box::new([]) },
    };
    config.settings.llm_providers.providers = Box::new([config.settings.llm_providers.providers[0].clone(), provider]);
    let mut core = Core::new(config, &limits);
    core.people = base.people;
    restore_key_account(&mut core, &limits, 0, 1, 1);
    if !none {
        restore_key_account(&mut core, &limits, 3, 2, 2);
    }
    make_task(&mut core, &limits, TASK);
    account(&mut core, &limits, accounts::Event::Want);
    granted(&mut core, &limits, 1);
    host(&mut core, &limits, FIRST, &[b"scripted", b"same"]);
    host(&mut core, &limits, SECOND, &[b"scripted", b"same", b"other"]);
    (core, limits)
}

#[test]
fn optional_unavailable_provider_is_omitted_and_same_provider_shares_one_source() {
    let (core, _) = ready(false);
    let choice = core.run_choice(TASK).unwrap();
    assert_eq!(choice.placement.channel, FIRST);
    assert_eq!(choice.accounts.as_ref(), [authority::AccountBinding { provider: 1, account: 0, source: 1 }]);
    assert_eq!(choice.policy.alternatives.iter().map(|model| model.name.as_ref()).collect::<Vec<_>>(), [b"same"]);
}

#[test]
fn usable_retained_provider_constrains_the_concrete_host() {
    let (mut core, limits) = ready(false);
    granted(&mut core, &limits, 2);
    let mut stale = core.tasks.run_context(TASK).unwrap();
    stale.payer = tasks::Payer::Party(999);
    stale.numbers.budget = 1;
    let RunAdmission::Allow(allowed) = core.run_admission(&stale, Wall::EPOCH, Box::new([])) else {
        panic!("current data allows")
    };
    assert_eq!(allowed.spend, 100);
    assert_eq!(allowed.choice.placement.channel, SECOND);
    assert_eq!(
        allowed.choice.accounts.as_ref(),
        [
            authority::AccountBinding { provider: 1, account: 0, source: 1 },
            authority::AccountBinding { provider: 2, account: 3, source: 2 },
        ]
    );
}

#[test]
fn a_none_provider_is_retained_without_a_credential_grant() {
    let (core, _) = ready(true);
    let choice = core.run_choice(TASK).unwrap();
    assert_eq!(choice.placement.channel, SECOND);
    assert_eq!(
        choice.accounts.as_ref(),
        [
            authority::AccountBinding { provider: 1, account: 0, source: 1 },
            authority::AccountBinding { provider: 2, account: 2, source: 3 },
        ]
    );
    assert_eq!(core.accounts.source_grant(3, Time::ZERO), None);
}

#[test]
fn an_unknown_reset_wait_has_no_readiness_spin_after_a_long_advance() {
    let (mut core, limits) = ready(false);
    account(&mut core, &limits, accounts::Event::Limited { account: 1, reset: None });
    core.run_now = Time::ZERO.saturating_add(Duration::from_secs(1_000_000));
    core.due.push(core.tasks.run_context(TASK).unwrap());
    assert!(!core.due_ready());
    assert_eq!(
        core.run_choice(TASK),
        Err(RunWait::Account {
            account: Some(0),
            source: Some(1),
            why: accounts::WaitWhy::Source(accounts::State::Limited { until: None })
        })
    );
}

#[test]
fn restored_bindings_refuse_duplicates_and_wrong_surviving_associations() {
    let (mut core, limits) = ready(false);
    let mut cases = Vec::new();
    let binding = authority::AccountBinding { provider: 1, account: 0, source: 1 };
    cases.push((vec![binding], true));
    cases.push((vec![binding, binding], false));
    cases.push((vec![authority::AccountBinding { source: 2, ..binding }], false));
    cases.push((vec![authority::AccountBinding { provider: 2, ..binding }], false));
    cases.push((vec![authority::AccountBinding { source: 99, ..binding }], true));
    cases.push((vec![], false));
    for (accounts, accepted) in cases {
        let mut proof = RunProof::claimed(TASK, 7, 7, None, fleet::HostKind::Worker);
        proof.placement = Some(FIRST);
        proof.accounts = accounts.into();
        core.restoring_proofs
            .insert(TASK, RestoringProof { attempt: 7, turn: 0, run_spent: 0, last_answer: None })
            .unwrap();
        assert_eq!(core.restore_proof(proof, limits.tasks.result_bytes), accepted);
        core.proofs.remove(&TASK);
    }
}

#[test]
fn adoption_uses_recorded_sources_through_refresh_and_removal() {
    let (mut core, limits) = ready(false);
    granted(&mut core, &limits, 2);
    let mut proof = RunProof::claimed(TASK, 7, 7, None, fleet::HostKind::Worker);
    proof.placement = Some(FIRST);
    proof.accounts = Box::new([authority::AccountBinding { provider: 2, account: 3, source: 2 }]);
    core.proofs.insert(TASK, proof).unwrap();
    for usable in [true, false] {
        if !usable {
            account(&mut core, &limits, accounts::Event::Limited { account: 2, reset: None });
        }
        core.adopt(&Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, TASK, 7, 0);
        let fleet::Event::Adopt { holds, grants, .. } = core.adopted.pop().unwrap() else {
            panic!("recorded adoption")
        };
        assert_eq!(holds.as_ref(), [fleet::GrantName { table: fleet::GrantTable::Accounts, account: 2 }]);
        assert_eq!(grants.len(), usize::from(usable));
        if let Some(grant) = grants.first() {
            assert_eq!(grant.account, 2);
        }
        core.unreported_restored.remove(&TASK);
    }
    core.proofs.get_mut(&TASK).unwrap().accounts[0] =
        authority::AccountBinding { provider: 1, account: 99, source: 99 };
    core.adopt(&Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, TASK, 7, 0);
    let fleet::Event::Adopt { grants, .. } = core.adopted.pop().unwrap() else { panic!("removed source history") };
    assert!(grants.is_empty(), "the usable source 1 cannot replace removed source 99");
}

#[test]
fn a_default_chooses_a_usable_provider_once_with_its_first_effort() {
    let (mut core, limits) = ready(false);
    granted(&mut core, &limits, 2);
    account(&mut core, &limits, accounts::Event::Limited { account: 1, reset: None });
    let mut batch = [new_row(&core, 2, tasks::Funder::Period { project: 1, period: 1 }, None)];
    core.resolve_llm(&mut batch).unwrap();
    assert_eq!(batch[0].llm.as_ref().unwrap().main, tasks::ModelEffort { model: 2, effort: Some(0) });
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new(batch));
    account(&mut core, &limits, accounts::Event::Limited { account: 2, reset: None });
    assert_eq!(core.chosen_policy(2).model.name.as_ref(), b"other");
}

#[test]
fn unknown_program_login_facts_also_hold_a_direct_run_off_workers() {
    let (mut core, limits) = ready(false);
    core.accounts = accounts::Domain::new(&limits.accounts, core.settings.llm_providers.clone());
    restore_key_account(&mut core, &limits, 0, 1, 1);
    account(
        &mut core,
        &limits,
        accounts::Event::Restore {
            record: accounts::Stored::Account {
                number: 2,
                provider: 2,
                holder: accounts::Holder::Party(2),
                place: 0,
                uses: None,
                subject: Box::new([]),
                display: Box::new([]),
            },
        },
    );
    account(
        &mut core,
        &limits,
        accounts::Event::Restore {
            record: accounts::Stored::Source {
                account: 2,
                number: 3,
                place: 0,
                kind: accounts::Kind::Program,
                name: b"personal".as_slice().into(),
                held: None,
            },
        },
    );
    account(&mut core, &limits, accounts::Event::Want);
    granted(&mut core, &limits, 1);
    assert_eq!(core.run_choice(TASK), Err(RunWait::Host));
}

fn terminal_after_zero_reset(account_terminal: bool) -> tasks::Tries {
    let (mut core, limits) = ready(false);
    account(&mut core, &limits, accounts::Event::Limited { account: 1, reset: Some(Time::ZERO) });
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(FIRST), task: TASK });
    send_task(
        &mut core,
        &limits,
        tasks::Event::Claim {
            reply_to: ReplyTo::new(FIRST),
            task: TASK,
            attempt: 7,
            resumed: false,
            budget: 10,
            account: Some(0),
            writes: Box::new([]),
        },
    );
    let mut proof = RunProof::claimed(TASK, 7, 7, None, fleet::HostKind::Worker);
    proof.placement = Some(FIRST);
    proof.accounts = Box::new([authority::AccountBinding { provider: 1, account: 0, source: 1 }]);
    proof.account_failure = Some(1);
    proof.account_terminal = account_terminal;
    core.proofs.insert(TASK, proof).unwrap();
    let _out = step(
        &mut core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        Event::AnswerPayload {
            run: Token::new(TASK),
            attempt: Token::new(7),
            payload: Token::new(99),
            cumulative: 1,
            end: tasks::End::Failed(tasks::Class::Run),
            saved: None,
            invalid_saved: false,
            read: None,
            refused: Box::new([]),
        },
    );
    core.tasks.task(TASK).unwrap().tries
}

#[test]
fn a_terminal_limit_with_a_zero_reset_spares_tries() {
    assert_eq!(terminal_after_zero_reset(true), tasks::Tries::NONE);
}

#[test]
fn a_recovered_notice_preserves_an_ordinary_failure() {
    assert_eq!(terminal_after_zero_reset(false).run, 1);
}

#[test]
fn a_retained_due_context_is_refreshed_before_preparation() {
    let (mut core, _) = ready(false);
    let mut stale = core.tasks.run_context(TASK).unwrap();
    stale.payer = tasks::Payer::Party(999);
    stale.numbers.budget = 1;
    core.due.push(stale);
    let current = core.take_due().unwrap();
    assert_eq!(current.payer, tasks::Payer::Project(1));
    assert_eq!(current.numbers.budget, 100);
}

#[test]
fn a_held_tasks_due_context_leaves_the_queue() {
    let (mut core, limits) = ready(false);
    core.due.push(core.tasks.run_context(TASK).unwrap());
    send_task(&mut core, &limits, tasks::Event::Hold { task: TASK, why: tasks::Hold::Uncertain { entry: 1 } });
    assert!(core.take_due().is_none());
    assert!(core.due.is_empty());
}

#[test]
fn a_task_document_preserves_an_unknown_account_reset() {
    let (mut core, limits) = ready(false);
    account(&mut core, &limits, accounts::Event::Limited { account: 1, reset: None });
    let row = core.tasks.task(TASK).unwrap();
    let document = task_from_row(&core, &limits, 1, row, Wall::EPOCH).unwrap();
    assert_eq!(
        document.run_wait,
        Some(TaskRunWait::Account {
            account: Some(0),
            source: Some(1),
            why: TaskWaitWhy::Source(TaskSourceState::Limited { until: None }),
        })
    );
}

#[test]
fn a_task_document_distinguishes_an_application_credential_wait() {
    let (mut core, limits) = ready(false);
    core.workspace_credentials.insert(TASK, Box::new([7])).unwrap();
    let document = task_from_row(&core, &limits, 1, core.tasks.task(TASK).unwrap(), Wall::EPOCH).unwrap();
    assert_eq!(document.run_wait, Some(TaskRunWait::Credential));
}

#[test]
fn a_progress_count_leaves_due_task_watches_unmarked() {
    let (mut core, mut limits) = ready(false);
    limits.fleet.engine_slots = 1;
    core.fleet = fleet::Domain::new(&limits.fleet, fixtures::needs());
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet };
    let mut out = Queue::with_capacity(fleet::max_out(&limits.fleet));
    fleet::step(
        &mut core.fleet,
        &env,
        fleet::Event::Start {
            placement: None,
            charter: 1,
            grants: Box::new([]),
            reply_to: ReplyTo::new(Token::new(1)),
            run: Token::new(2),
            attempt: Token::new(7),
            workstream: 1,
            kinds: fleet::Kinds::Engine,
            assignment: fleet::Assignment { turns: Token::new(0), answered: Token::new(0) },
        },
        &mut out,
    );
    fleet::resume(&mut core.fleet, &env, &mut out);
    fleet::step(
        &mut core.fleet,
        &env,
        fleet::Event::Observe {
            channel: Token::new(0),
            run: Token::new(2),
            attempt: Token::new(7),
            report: fleet::Report::Started,
        },
        &mut out,
    );
    let subject =
        core.view_names.insert(WatchedSubject { subject: Subject::Task { task: TASK }, watchers: 1 }).unwrap();
    core.view_subjects
        .insert(Token::new(99), WatchBinding { party: 1, sign_in: 10, subject, window: window() })
        .unwrap();
    step(
        &mut core,
        &Env { now: Time::ZERO, wall: Wall::from_nanos(1), limits },
        Event::Fleet(fleet::Event::Observe {
            channel: Token::new(0),
            run: Token::new(2),
            attempt: Token::new(7),
            report: fleet::Report::Count { kind: fleet::CountKind::TextArrived, elapsed: Duration::ZERO, count: 1 },
        }),
    );
    assert!(core.changed_views.is_empty());
}
