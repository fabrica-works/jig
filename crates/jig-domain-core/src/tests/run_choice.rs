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
pub(super) fn host(core: &mut Core, limits: &Limits, channel: Token, names: &[&[u8]]) {
    host_with_logins(core, limits, channel, names, Box::new([]));
}
fn host_with_logins(core: &mut Core, limits: &Limits, channel: Token, names: &[&[u8]], logins: Box<[fleet::Login]>) {
    let mut bounds = fixtures::bounds();
    bounds.models =
        names.iter().map(|name| fleet::ModelBound { name: (*name).into(), window: 8192, output: 1024 }).collect();
    let mut out = Queue::with_capacity(fleet::max_out(&limits.fleet));
    fleet::step(
        &mut core.fleet,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet },
        fleet::Event::Hello {
            worker: u32::from(channel == SECOND),
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
        fleet::Event::Logins { channel, logins },
        &mut out,
    );
}
fn ready(none: bool) -> (Core, Limits) {
    let (base, mut limits) = core();
    limits.accounts.accounts = 4;
    limits.accounts.holder_accounts = 4;
    limits.accounts.credentials = 3;
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
    let mut core = Core::new(config, &limits).expect("fixture core configuration");
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

const OWNER: u64 = 1;
const OTHER_PERSON: u64 = 2;
const PERSONAL_SOURCE: u32 = 3;
fn login(name: &[u8], state: fleet::LoginState) -> fleet::Login {
    fleet::Login { name: fleet::SourceName::new(name).unwrap(), state }
}
fn inventory(core: &mut Core, limits: &Limits, channel: Token, logins: Box<[fleet::Login]>) {
    let mut out = Queue::with_capacity(fleet::max_out(&limits.fleet));
    fleet::step(
        &mut core.fleet,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet },
        fleet::Event::Logins { channel, logins },
        &mut out,
    );
}
fn placement_fleet(core: &Core, limits: &Limits) -> fleet::Domain {
    let mut config = config(41);
    config.settings.charters[0].hosts = core.settings.charters[0].hosts;
    config.settings.charters[0].policy = core.settings.charters[0].policy.clone();
    config.settings.charters[0].policy.alternatives = core.settings.models[1..].into();
    let mut fleet = fleet::Domain::new(&limits.fleet, fleet_needs::configured(&config, limits));
    fleet.configure_workers(
        fleet::Membership::new(config.worker_names, limits.fleet.configured_workers, limits.fleet.worker_name_bytes)
            .unwrap(),
    );
    fleet
}
fn placement_fixture(
    main_holder: accounts::Holder,
    login_holder: accounts::Holder,
    personal_first: bool,
) -> (Core, Limits) {
    placement_with_accounts(main_holder, login_holder, personal_first, Vec::new())
}
fn placement_with_accounts(
    main_holder: accounts::Holder,
    login_holder: accounts::Holder,
    personal_first: bool,
    extra: Vec<accounts::Stored>,
) -> (Core, Limits) {
    let (mut core, limits) = ready(false);
    core.settings.charters[0].policy.alternatives = Box::new([]);
    core.accounts = accounts::Domain::new(&limits.accounts, core.settings.llm_providers.clone());
    let program_account = if main_holder == login_holder { 0 } else { 2 };
    let mut records = vec![
        accounts::Stored::Account {
            number: 0,
            provider: 1,
            holder: main_holder,
            place: 0,
            uses: Some(Box::new([Use::Project(1)])),
            subject: Box::new([]),
            display: Box::new([]),
        },
        accounts::Stored::Source {
            account: 0,
            number: 1,
            place: u32::from(personal_first),
            kind: accounts::Kind::Key,
            name: Box::new([]),
            held: None,
        },
        accounts::Stored::Source {
            account: program_account,
            number: PERSONAL_SOURCE,
            place: u32::from(!personal_first),
            kind: accounts::Kind::Program,
            name: b"personal".as_slice().into(),
            held: None,
        },
        accounts::Stored::Issued {
            number: PERSONAL_SOURCE,
            holder: login_holder,
            name: b"personal".as_slice().into(),
            signed_out: false,
        },
    ];
    if program_account != 0 {
        records.insert(
            1,
            accounts::Stored::Account {
                number: program_account,
                provider: 2,
                holder: login_holder,
                place: 0,
                uses: None,
                subject: Box::new([]),
                display: Box::new([]),
            },
        );
    }
    records.extend(extra);
    for record in records {
        account(&mut core, &limits, accounts::Event::Restore { record });
    }
    account(&mut core, &limits, accounts::Event::Want);
    granted(&mut core, &limits, 1);
    core.people = people::Domain::new(&limits.people, Box::new([]), u16::MAX);
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    for person in [OWNER, OTHER_PERSON] {
        people::step(
            &mut core.people,
            &env,
            people::Event::Restore {
                record: people::Stored::Person {
                    number: person,
                    identity: people::Identity {
                        key: people::IdentityKey { provider: 0, subject: person.to_be_bytes().into() },
                        login: b"person".as_slice().into(),
                        name: b"Person".as_slice().into(),
                    },
                },
            },
            &mut out,
        );
    }
    people::step(
        &mut core.people,
        &env,
        people::Event::Restore {
            record: people::Stored::Roles {
                project: 1,
                holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Owner }]),
            },
        },
        &mut out,
    );
    people::step(&mut core.people, &env, people::Event::Restored, &mut out);
    core.fleet = placement_fleet(&core, &limits);
    let mut out = Queue::with_capacity(fleet::max_out(&limits.fleet));
    fleet::step(
        &mut core.fleet,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet },
        fleet::Event::Sources { sources: Box::new([fleet::SourceName::new(b"personal").unwrap()]) },
        &mut out,
    );
    host_with_logins(
        &mut core,
        &limits,
        FIRST,
        &[b"scripted", b"other"],
        Box::new([login(b"personal", fleet::LoginState::Usable)]),
    );
    host_with_logins(
        &mut core,
        &limits,
        SECOND,
        &[b"scripted"],
        Box::new([login(b"personal", fleet::LoginState::NotSignedIn)]),
    );
    (core, limits)
}
#[test]
fn another_person_s_kept_login_keeps_a_run_off_its_host() {
    let (mut core, limits) =
        placement_fixture(accounts::Holder::Deployment, accounts::Holder::Party(OTHER_PERSON), false);
    for state in [
        fleet::LoginState::Usable,
        fleet::LoginState::SignedInOtherwise(fleet::LoginMethod::new(b"api-key").unwrap()),
        fleet::LoginState::Failed { reason: fleet::SourceFailure::Exited, detail: Box::new([]) },
        fleet::LoginState::Failed { reason: fleet::SourceFailure::Deadline, detail: Box::new([]) },
    ] {
        inventory(&mut core, &limits, FIRST, Box::new([login(b"personal", state)]));
        assert_eq!(core.run_choice(TASK).unwrap().placement.channel, SECOND);
    }
}
#[test]
fn a_person_s_own_run_takes_the_host_keeping_their_logins() {
    let (core, _) = placement_fixture(accounts::Holder::Party(OWNER), accounts::Holder::Party(OWNER), false);
    assert_eq!(core.run_choice(TASK).unwrap().placement.channel, FIRST);
}
#[test]
fn a_program_source_is_chosen_only_where_its_login_is_usable() {
    let (core, _) = placement_fixture(accounts::Holder::Party(OWNER), accounts::Holder::Party(OWNER), true);
    let choice = core.run_choice(TASK).unwrap();
    assert_eq!((choice.placement.channel, choice.accounts[0].source), (FIRST, PERSONAL_SOURCE));
}
#[test]
fn an_unusable_login_lets_the_next_source_serve_there() {
    let (mut core, limits) = placement_fixture(accounts::Holder::Party(OWNER), accounts::Holder::Party(OWNER), true);
    inventory(&mut core, &limits, FIRST, Box::new([login(b"personal", fleet::LoginState::NotSignedIn)]));
    let choice = core.run_choice(TASK).unwrap();
    assert_eq!((choice.placement.channel, choice.accounts[0].source), (FIRST, 1));
}
#[test]
fn a_deployment_held_login_restricts_no_run() {
    let (core, _) = placement_fixture(accounts::Holder::Party(OWNER), accounts::Holder::Deployment, false);
    assert_eq!(core.run_choice(TASK).unwrap().placement.channel, FIRST);
}
#[test]
fn a_direct_run_takes_a_worker_without_kept_logins() {
    let (mut core, limits) =
        placement_fixture(accounts::Holder::Deployment, accounts::Holder::Party(OTHER_PERSON), false);
    inventory(&mut core, &limits, FIRST, Box::new([login(b"personal", fleet::LoginState::NotSignedIn)]));
    assert_eq!(core.run_choice(TASK).unwrap().placement.channel, FIRST);
}
#[test]
fn an_unnamed_kept_login_closes_a_host() {
    let (mut core, limits) = placement_fixture(accounts::Holder::Deployment, accounts::Holder::Party(OWNER), false);
    core.accounts = accounts::Domain::new(&limits.accounts, core.settings.llm_providers.clone());
    restore_key_account(&mut core, &limits, 0, 1, 1);
    account(&mut core, &limits, accounts::Event::Want);
    granted(&mut core, &limits, 1);
    assert_eq!(core.run_choice(TASK).unwrap().placement.channel, SECOND);
}
#[test]
fn one_person_s_deployment_places_every_run_on_their_host() {
    let (mut core, _) = placement_fixture(accounts::Holder::Deployment, accounts::Holder::Party(OTHER_PERSON), false);
    core.people = super::core().0.people;
    assert_eq!(core.run_choice(TASK).unwrap().placement.channel, FIRST);
}
#[test]
fn two_people_s_kept_logins_close_a_host() {
    let (mut core, limits) = placement_with_accounts(
        accounts::Holder::Party(OWNER),
        accounts::Holder::Party(OWNER),
        false,
        vec![accounts::Stored::Issued {
            number: 4,
            holder: accounts::Holder::Party(OTHER_PERSON),
            name: b"other".as_slice().into(),
            signed_out: false,
        }],
    );
    let mut out = Queue::with_capacity(fleet::max_out(&limits.fleet));
    fleet::step(
        &mut core.fleet,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet },
        fleet::Event::Sources {
            sources: Box::new([
                fleet::SourceName::new(b"personal").unwrap(),
                fleet::SourceName::new(b"other").unwrap(),
            ]),
        },
        &mut out,
    );
    inventory(
        &mut core,
        &limits,
        FIRST,
        Box::new([login(b"personal", fleet::LoginState::Usable), login(b"other", fleet::LoginState::Usable)]),
    );
    inventory(
        &mut core,
        &limits,
        SECOND,
        Box::new([login(b"personal", fleet::LoginState::NotSignedIn), login(b"other", fleet::LoginState::NotSignedIn)]),
    );
    assert_eq!(core.run_choice(TASK).unwrap().placement.channel, SECOND);
}
#[test]
fn the_engine_never_takes_a_program_source() {
    let (mut core, limits) = placement_fixture(accounts::Holder::Party(OWNER), accounts::Holder::Party(OWNER), true);
    core.settings.charters[0].hosts = fleet::Kinds::Engine;
    let mut limits = limits;
    limits.fleet.engine_slots = 1;
    core.fleet = placement_fleet(&core, &limits);
    host(&mut core, &limits, Token::new(0), &[b"scripted"]);
    let choice = core.run_choice(TASK).unwrap();
    assert_eq!((choice.placement.kind, choice.accounts[0].source), (fleet::HostKind::Engine, 1));
}

#[test]
fn a_retained_provider_binds_an_account_the_rule_allows() {
    let mut extra = Vec::new();
    for (number, source, holder) in [(2, 7, accounts::Holder::Deployment), (5, 6, accounts::Holder::Party(OWNER))] {
        extra.push(accounts::Stored::Account {
            number,
            provider: 2,
            holder,
            place: number,
            uses: Some(Box::new([Use::Project(1)])),
            subject: Box::new([]),
            display: Box::new([]),
        });
        extra.push(accounts::Stored::Source {
            account: number,
            number: source,
            place: 0,
            kind: accounts::Kind::Key,
            name: Box::new([]),
            held: None,
        });
    }
    let (mut core, limits) =
        placement_with_accounts(accounts::Holder::Party(OWNER), accounts::Holder::Party(OWNER), false, extra);
    core.settings.charters[0].policy.alternatives = Box::new([core.settings.models[2].clone()]);
    granted(&mut core, &limits, 7);
    granted(&mut core, &limits, 6);
    let choice = core.run_choice(TASK).unwrap();
    assert_eq!(
        choice.accounts.as_ref(),
        [
            authority::AccountBinding { provider: 1, account: 0, source: 1 },
            authority::AccountBinding { provider: 2, account: 5, source: 6 }
        ]
    );
}
