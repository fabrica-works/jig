//! Unit fixtures use the core world's small configuration and child entry points.
#![expect(clippy::too_many_lines, reason = "tests state complete bounded fixtures")]
use crate::documents::*;
use crate::*;
use skein_lib::{Duration, Env, ReplyTo, Time, Token, Wall};
fn limits() -> Limits {
    let retry = tasks::Retry { retries: 1, base: Duration::from_millis(10), max: Duration::from_secs(1) };
    let tasks = tasks::Limits {
        tasks: 2,
        funders: 4,
        project_tasks: 2,
        tree_tasks: 2,
        depth: 1,
        delegates: 1,
        references: 2,
        subscriptions: 2,
        batch: 1,
        dependencies: 1,
        holdings: 2,
        hold_kinds: 2,
        resource_reports: 64,
        pools: 2,
        hold_segments: 4,
        hold_bytes: 128,
        hold_waiters: 2,
        hold_wait: Duration::from_secs(10),
        inputs: 1,
        spec_bytes: 64,
        parameters: 1,
        result_bytes: 128,
        inbox_messages: 4,
        inbox_bytes: 192,
        message_bytes: 64,
        proposal_stall: Duration::from_millis(10),
        escalation_stall: Duration::from_secs(3600),
        saved_resources: 2,
        contract_choices: 1,
        charters: 1,
        authority_grants: 1,
        authority_segments: 1,
        authority_bytes: 32,
        executor_kinds: 2,
        retries: tasks::Retries {
            transient: retry,
            permanent: retry,
            run: retry,
            agent: retry,
            lost: retry,
            invalid: retry,
        },
        facts: 2,
    };
    let people = people::Limits {
        people: 2,
        inbox_entries: 2,
        sign_ins: 2,
        projects: 1,
        holdings: 2,
        goals: 4,
        initial_owners: 1,
        requests: 4,
        pending: 2,
        waiters: 2,
        identity_bytes: 32,
        words: 64,
        amendment_bytes: 512,
        sign_in_lifetime: Duration::from_secs(60),
        request_retention: Duration::from_secs(120),
        facts: 2,
    };
    let fleet = fleet::Limits {
        workers: 1,
        engine_slots: 0,
        slots: 1,
        workstreams: 1,
        attempts: 2,
        calls: 1,
        call_name_bytes: 64,
        turns: 2,
        grace: Duration::from_secs(5),
        facts: 2,
    };
    Limits {
        documents: DocumentLimits {
            rows: 32,
            text_bytes: 256,
            chips: 32,
            lines: 32,
            versions: Versions { minimum: 1, maximum: 1 },
        },
        connectors: 2,
        resume_bytes: 256,
        run_bytes: 256,
        policy_bytes: 8192,
        escalation_reason_bytes: 128,
        load_slots: 2,
        call_records: 2,
        call_answer_bytes: 256,
        tasks,
        authority: authority_limits(),
        people,
        fleet,
        brief: jig_core_brief::Limits { briefs: 2, sections: 3, read_bytes: 256, brief_bytes: 384 },
        brief_parts: 2,
        brief_core_budgets: CoreBriefBudgets { task: 128, dependencies: 128, attempts: 128, plan: 128, notes: 64 },
        accounts: accounts::Limits {
            accounts: 1,
            refresh_margin: Duration::from_secs(1),
            backoff_base: Duration::from_millis(10),
            backoff_max: Duration::from_secs(1),
            rejected_interval: Duration::from_millis(10),
            spent_attention: Duration::from_secs(1),
            facts: 2,
        },
        views: jig_core_views::Limits {
            runs: 2,
            watchers: 4,
            backlog: 2,
            report_bytes: 64,
            snapshot_bytes: 128,
            facts: 8,
        },
        notes: jig_core_notes::Limits {
            scopes: 4,
            entries_per_scope: 1,
            pattern_bytes: 64,
            description_bytes: 64,
            body_bytes: 256,
            references: 1,
            load_rows: 1,
            lines: 1,
            recalled: 1,
        },
    }
}
fn authority_limits() -> authority::Limits {
    authority::Limits {
        projects: 1,
        roles: 1,
        requirements: 1,
        facts: 2,
        grants: 1,
        executors: 2,
        segments: 1,
        segment_bytes: 32,
        implications: 0,
        batch: 1,
        accounts: 1,
        writes: 1,
    }
}
fn authority_value(spend: u64, kinds: Box<[authority::Executor]>) -> authority::Authority {
    authority::Authority {
        tools: authority::Tools(1023),
        grants: Box::new([]),
        delegation: authority::Delegation { kinds, tasks: 3, depth: 2 },
        budget: authority::Budget { spend, deadline: None },
        notes: authority::Scopes(0),
        note_resources: Box::new([]),
    }
}
fn config(seed: u64) -> Config {
    let ceiling = authority_value(1000, Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(1)]));
    let rules = authority::Rules {
        ceiling: ceiling.clone(),
        period_spend: 1000,
        minimum_run_spend: 1,
        maximum_run_spend: 100,
        implies: authority::Implies::new(Box::new([]), 0).expect("empty implication configuration"),
        requirements: Box::new([]),
    };
    let mut domain = authority::Domain::new(rules, authority_limits()).expect("valid authority configuration");
    let policy = authority::Policy {
        projections: Box::new([]),
        escalation_role: Some(0),
        ceiling,
        period_spend: 1000,
        roles: Box::new([authority::Role {
            number: 0,
            authority: authority_value(
                500,
                Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(1)]),
            ),
            period_spend: 500,
            requests: authority::Requests::ALL,
            decides: authority::Proposals::ALL,
        }]),
        requirements: Box::new([]),
    };
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }), "policy admitted");
    let mut chat_authority = authority_value(100, Box::new([authority::Executor::Procedure(1)]));
    chat_authority.delegation.tasks = 1;
    chat_authority.delegation.depth = 1;
    Config {
        deployment: [31; 16],
        seed,
        owners: Box::new([people::InitialOwner {
            project: 1,
            identity: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
        }]),
        authority: domain,
        projects: {
            let mut projects = List::with_capacity(1);
            projects.push(1).expect("one project");
            projects
        },
        permission_roles: {
            let mut projects: Map<u32, Box<[people::PermissionRole]>> = Map::with_capacity(1);
            projects
                .insert(
                    1,
                    Box::new([
                        people::PermissionRole { connector: 1, permission: 3, role: 1 },
                        people::PermissionRole { connector: 1, permission: 2, role: 2 },
                        people::PermissionRole { connector: 1, permission: 1, role: 3 },
                    ]),
                )
                .expect("one configured permission policy");
            projects
        },
        connectors: Box::new([1, 2]),
        settings: Settings {
            deployment_provider: 3,
            recurring_connector: 1,
            charter: 1,
            run: RunPolicy {
                instructions: b"Complete the task within its authority.".as_slice().into(),
                waiting: Duration::from_secs(1),
                resume: true,
                turns: 8,
                time: Duration::from_secs(60),
                model: Model {
                    dialect: 1,
                    account: 1,
                    endpoint: 1,
                    name: b"scripted".as_slice().into(),
                    max_tokens: 1024,
                    input_price: 1,
                    cached_price: 1,
                    output_price: 1,
                    price_unit: 1000,
                },
                alternatives: Box::new([]),
                inspect: true,
                modify: true,
                shell: true,
                agents: true,
                call_timeout: Duration::from_secs(10),
            },
            resume_bytes: 256,
            period: 1,
            period_budget: 1000,
            person_budget: 500,
            chat_authority,
            tools: ToolFamilies::standard(),
            account: 1,
            account_generation: 1,
            account_valid: Some(Duration::from_secs(60)),
        },
    }
}

fn core() -> (Core, Limits) {
    let limits = limits();
    let mut core = Core::new(config(41), &limits);
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    people::step(
        &mut core.people,
        &env,
        people::Event::Restore {
            record: people::Stored::Person {
                number: 1,
                identity: people::Identity {
                    key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
                    login: b"owner".as_slice().into(),
                    name: b"A Person".as_slice().into(),
                },
            },
        },
        &mut out,
    );
    people::step(
        &mut core.people,
        &env,
        people::Event::Restore {
            record: people::Stored::Roles {
                project: 1,
                holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
            },
        },
        &mut out,
    );
    people::step(&mut core.people, &env, people::Event::Restored, &mut out);
    (core, limits)
}

fn make_task(core: &mut Core, limits: &Limits, number: u64) {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks };
    let mut out = Queue::with_capacity(tasks::max_out(&limits.tasks));
    tasks::step(&mut core.tasks, &env, tasks::Event::Restored, &mut out);
    while out.pop().is_some() {}
    tasks::step(
        &mut core.tasks,
        &env,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(1)), project: 1, period: 1, budget: 1000 },
        &mut out,
    );
    while out.pop().is_some() {}
    let row = tasks::New {
        number,
        project: 1,
        executor: tasks::Executor::Agent { charter: 1 },
        spec: tasks::Spec {
            words: b"A task title\nThe specification".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 64 },
        authority: task_authority(&core.settings.chat_authority),
        numbers: tasks::Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
        funder: tasks::Funder::Period { project: 1, period: 1 },
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        recurring: None,
        tracked: None,
    };
    tasks::step(
        &mut core.tasks,
        &env,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(1)),
            creator: tasks::Party::Person(1),
            batch: Box::new([row]),
        },
        &mut out,
    );
    assert!(core.tasks.task(number).is_some());
}

fn window() -> Window {
    Window { rows: 1, after: None }
}

#[test]
fn documents_frame_carries_roles_retention_and_configured_charters() {
    let (core, limits) = core();
    let Document::Frame(frame) = make(&core, &limits, 1, &Subject::Frame { party: 1 }, &window()).unwrap() else {
        panic!()
    };
    assert_eq!(frame.party, 1);
    assert_eq!(frame.kind, people::Kind::Person);
    assert_eq!(frame.projects.as_slice(), &[people::ProjectRole { project: 1, role: people::Role::Owner }]);
    assert_eq!(frame.retention, limits.people.request_retention);
    assert_eq!(frame.charters.as_slice()[0].goals.as_slice(), &[1]);
    assert_eq!(frame.versions, limits.documents.versions);
    assert_eq!(frame.needs, 0);
    assert_eq!(frame.updates, Count::Known(0));
}

#[test]
fn documents_chip_cuts_the_title_and_keeps_its_phase_facts() {
    let (mut core, mut limits) = core();
    make_task(&mut core, &limits, 1);
    limits.documents.text_bytes = 6;
    let mut tasks = List::with_capacity(1);
    tasks.push(1).unwrap();
    let Document::Chips(chips) = make(&core, &limits, 1, &Subject::Chips { tasks }, &window()).unwrap() else {
        panic!()
    };
    let chip = &chips.as_slice()[0];
    assert_eq!(chip.title, Text { bytes: b"A task".as_slice().into(), cut: true });
    assert_eq!(chip.executor, ExecutorKind::Agent);
    assert_eq!(chip.phase, PhaseLine::Active(tasks::Active::Due));
}

#[test]
fn documents_refuse_a_party_without_a_project_role() {
    let (mut core, limits) = core();
    make_task(&mut core, &limits, 1);
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    people::step(
        &mut core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::Roles { project: 1, holdings: Box::new([]) },
        &mut out,
    );
    let mut tasks = List::with_capacity(1);
    tasks.push(1).unwrap();
    assert_eq!(make(&core, &limits, 1, &Subject::Chips { tasks }, &window()), Err(Missing::NotAllowed));
    assert_eq!(make(&core, &limits, 1, &Subject::Frame { party: 2 }, &window()), Err(Missing::NotAllowed));
}

#[test]
fn documents_run_has_only_the_live_proofs_committed_progress() {
    let (mut core, limits) = core();
    make_task(&mut core, &limits, 1);
    core.proofs
        .insert(
            1,
            RunProof {
                task: 1,
                attempt: 7,
                transcript_from: 0,
                host: fleet::HostKind::Engine,
                offered: None,
                turn: Some(TurnProof { turn: 3, cumulative: 0, read: None }),
                terminal: None,
            },
        )
        .unwrap();
    assert_eq!(
        make(&core, &limits, 1, &Subject::Run { task: 1 }, &window()),
        Ok(Document::Run(Run { task: 1, attempt: 7, committed_turn: 3, writing: None }))
    );
    core.proofs.remove(&1);
    assert_eq!(make(&core, &limits, 1, &Subject::Run { task: 1 }, &window()), Err(Missing::Ended));
}

#[test]
fn documents_reject_oversized_whole_requests_and_invalid_bounds() {
    let (core, mut limits) = core();
    let empty = Subject::Chips { tasks: List::with_capacity(0) };
    assert_eq!(make(&core, &limits, 1, &empty, &Window { rows: 33, after: None }), Err(Missing::TooLarge));
    assert!(document_worst_case(&limits.documents).is_some());
    limits.documents.lines = 0;
    assert_eq!(document_worst_case(&limits.documents), None);
    limits.documents.lines = u32::MAX;
    limits.documents.chips = u32::MAX;
    assert_eq!(document_worst_case(&limits.documents), None);
}
