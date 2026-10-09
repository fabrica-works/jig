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
            watchers_per_party: 8,
            subjects_per_change: 64,
            text_bytes: 64,
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

fn tree_core() -> (Core, Limits) {
    let (old, mut limits) = core();
    limits.tasks.tasks = 16;
    limits.tasks.project_tasks = 16;
    limits.tasks.tree_tasks = 16;
    limits.tasks.depth = 4;
    limits.tasks.delegates = 8;
    limits.tasks.batch = 8;
    limits.tasks.inbox_messages = 16;
    limits.tasks.inbox_bytes = 2048;
    let mut config = config(17);
    config.settings.chat_authority.delegation.tasks = 16;
    config.settings.chat_authority.delegation.depth = 4;
    let mut core = Core::new(config, &limits);
    core.people = old.people;
    send_task(&mut core, &limits, tasks::Event::Restored);
    send_task(
        &mut core,
        &limits,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(1)), project: 1, period: 1, budget: 1000 },
    );
    (core, limits)
}

fn send_task(core: &mut Core, limits: &Limits, event: tasks::Event) {
    let mut out = Queue::with_capacity(tasks::max_out(&limits.tasks));
    tasks::step(&mut core.tasks, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks }, event, &mut out);
    while let Some(request) = out.pop() {
        if let tasks::Request::Save { record: tasks::Stored::Live(row) } = request {
            let mut projections = Queue::with_capacity(room_max(limits).unwrap().writes);
            projections::save(core, limits, &row, &mut projections);
        }
    }
}

fn new_row(core: &Core, number: u64, funder: tasks::Funder, tracked: Option<u32>) -> tasks::New {
    let mut authority = task_authority(&core.settings.chat_authority);
    let budget = match funder {
        tasks::Funder::Task(_) => 10,
        tasks::Funder::Pool { .. } | tasks::Funder::Recurring { .. } | tasks::Funder::Period { .. } => 100,
    };
    authority.budget.spend = budget;
    tasks::New {
        number,
        project: 1,
        executor: tasks::Executor::Agent { charter: 1 },
        spec: tasks::Spec {
            words: b"Work\nDetailed instructions".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 64 },
        authority,
        numbers: tasks::Numbers { budget, spent: 0, spent_below: 0, reserved: 0 },
        funder,
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        recurring: None,
        tracked,
    }
}

fn make_tree() -> (Core, Limits) {
    let (mut core, limits) = tree_core();
    let root = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, Some(4));
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(1)),
            creator: tasks::Party::Person(1),
            batch: Box::new([root]),
        },
    );
    let children = [
        new_row(&core, 2, tasks::Funder::Task(1), None),
        new_row(&core, 3, tasks::Funder::Task(1), None),
        new_row(&core, 4, tasks::Funder::Task(1), None),
    ];
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(1)),
            creator: tasks::Party::Task(1),
            batch: Box::new(children),
        },
    );
    assert!(core.tasks.task(4).is_some());
    send_task(&mut core, &limits, tasks::Event::Hold { task: 3, why: tasks::Hold::Budget });
    (core, limits)
}

#[test]
fn documents_tree_folds_every_unshown_phase_and_keeps_held_counts() {
    let (core, limits) = make_tree();
    let Document::Tree(tree) = make(&core, &limits, 1, &Subject::Tree { task: 1 }, &window()).unwrap() else {
        panic!()
    };
    assert_eq!(tree.rows.len(), 1);
    assert_eq!(tree.rows.get(0).unwrap().chip.number, 2);
    assert_eq!(tree.held_below, 1);
    assert_eq!(tree.folded.get(0).unwrap().held_below, 1);
    assert_eq!(tree.folded.get(0).unwrap().phases.held, 1);
    assert_eq!(tree.folded.get(0).unwrap().phases.starting, 1);
    assert_eq!(tree.next, Some(Cursor::Number { number: 2 }));
    let Document::Tree(next) =
        make(&core, &limits, 1, &Subject::Tree { task: 1 }, &Window { rows: 2, after: tree.next }).unwrap()
    else {
        panic!()
    };
    assert_eq!(next.rows.get(0).unwrap().chip.number, 3);
    assert_eq!(next.rows.get(1).unwrap().chip.number, 4);
    assert_eq!(next.next, None);
    assert_eq!(next.held_below, 1);
}

#[test]
fn documents_task_carries_overview_lineage_and_the_hold_reason() {
    let (core, limits) = make_tree();
    let Document::Task(task) = make(&core, &limits, 1, &Subject::Task { task: 3 }, &window()).unwrap() else {
        panic!()
    };
    assert_eq!(task.words.bytes.as_ref(), b"Work\nDetailed instructions");
    assert_eq!(task.goal, Some(1));
    assert_eq!(task.lineage.get(0).unwrap().number, 1);
    assert_eq!(task.budget.left, 10);
    assert_eq!(task.budget.funder, tasks::Funder::Task(1));
    let hold = task.hold.unwrap();
    assert_eq!(hold.reason, tasks::Hold::Budget);
    assert_eq!(hold.revision, core.tasks.task(3).unwrap().revisions.hold);
    assert_eq!(hold.before, PriorPhase::Active(tasks::Active::Due));
}

#[test]
fn documents_goal_progress_counts_every_phase_including_ended_plan_rows() {
    let (mut core, limits) = make_tree();
    let phases = [
        tasks::Phase::Waiting,
        tasks::Phase::Active(tasks::Active::Idle),
        tasks::Phase::Active(tasks::Active::Due),
        tasks::Phase::Active(tasks::Active::Running { attempt: 1 }),
        tasks::Phase::Active(tasks::Active::BackingOff { until: Wall::EPOCH }),
        tasks::Phase::Closing(tasks::Closing {
            stage: tasks::Stage::Delegates,
            ending: tasks::Ending::Failed { reason: Box::new([]) },
        }),
        tasks::Phase::Held { was: tasks::Was::Waiting, why: tasks::Hold::Budget },
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: Box::new([]) })),
        tasks::Phase::Ended(tasks::Ending::Failed { reason: Box::new([]) }),
        tasks::Phase::Ended(tasks::Ending::Cancelled {
            reason: Box::new([]),
            result: None,
            by: tasks::Party::Person(1),
            at: Wall::EPOCH,
        }),
    ];
    let mut prototype = core.tasks.task(2).unwrap().clone();
    let mut out = Queue::with_capacity(room_max(&limits).unwrap().writes);
    for (index, phase) in phases.into_iter().enumerate() {
        prototype.number = u64::try_from(index).unwrap() + 2;
        prototype.phase = phase;
        projections::save(&mut core, &limits, &prototype, &mut out);
        while out.pop().is_some() {}
    }
    let Document::Goals(goals) = make(&core, &limits, 1, &Subject::Goals { project: 1 }, &window()).unwrap() else {
        panic!()
    };
    let line = goals.lines.get(0).unwrap();
    assert_eq!(
        line.phases,
        PhaseCounts {
            waiting: 1,
            idle: 1,
            starting: 2,
            running: 1,
            backing_off: 1,
            closing: 1,
            held: 1,
            done: 1,
            failed: 1,
            cancelled: 1
        }
    );
    assert_eq!(line.held_below, 1);
    assert_eq!(line.budget.numbers.reserved, 30);
    assert_eq!(line.budget.left, 70);
}

#[test]
fn documents_goals_follow_priority_and_resume_at_the_named_cursor() {
    let (mut core, limits) = make_tree();
    let goal = new_row(&core, 5, tasks::Funder::Period { project: 1, period: 1 }, Some(2));
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(1)),
            creator: tasks::Party::Person(1),
            batch: Box::new([goal]),
        },
    );
    let Document::Goals(goals) = make(&core, &limits, 1, &Subject::Goals { project: 1 }, &window()).unwrap() else {
        panic!()
    };
    assert_eq!(goals.lines.get(0).unwrap().chip.number, 5);
    assert_eq!(goals.next, Some(Cursor::Number { number: 5 }));
    let Document::Goals(next) =
        make(&core, &limits, 1, &Subject::Goals { project: 1 }, &Window { rows: 1, after: goals.next }).unwrap()
    else {
        panic!()
    };
    assert_eq!(next.lines.get(0).unwrap().chip.number, 1);
    assert_eq!(next.next, None);
}

#[test]
fn documents_part_keys_are_preserved_or_refused_whole() {
    let (mut core, mut limits) = tree_core();
    let mut row = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    row.authority.grants = Box::new([tasks::Grant {
        connector: 1,
        kind: 1,
        pattern: tasks::Pattern {
            segments: Box::new([b"whole".as_slice().into()]),
            last: tasks::Last::Exact(b"end".as_slice().into()),
        },
    }]);
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(1)),
            creator: tasks::Party::Person(1),
            batch: Box::new([row]),
        },
    );
    limits.documents.text_bytes = 3;
    assert_eq!(make(&core, &limits, 1, &Subject::Task { task: 1 }, &window()), Err(Missing::TooLarge));
    limits.documents.text_bytes = 5;
    let Document::Task(task) = make(&core, &limits, 1, &Subject::Task { task: 1 }, &window()).unwrap() else {
        panic!()
    };
    assert_eq!(task.authority.grants.get(0).unwrap().segments.get(0).unwrap().as_ref(), b"whole");
    assert_eq!(task.authority.grants.get(0).unwrap().last, PatternEnd::Exact(b"end".as_slice().into()));
    assert!(document_worst_case(&limits.documents).is_some());
    assert!(worst_case(&limits).is_some());
}

#[test]
fn documents_proposal_tracks_the_current_holder_and_funding_after_a_pass() {
    let (mut core, limits) = make_tree();
    let next = new_row(&core, 5, tasks::Funder::Task(1), None);
    send_task(
        &mut core,
        &limits,
        tasks::Event::Propose {
            reply_to: ReplyTo::new(Token::new(50)),
            proposal: tasks::Proposal {
                number: 10,
                proposer: 2,
                project: 1,
                action: tasks::ProposalAction::Batch(Box::new([next])),
                reason: Box::from(&b"More room"[..]),
                as_holder: true,
                state: tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Task(1), since: Wall::EPOCH },
            },
        },
    );
    let target = Subject::Proposal { proposer: Proposer::Task(2), number: 10 };
    let Document::Proposal(proposal) = make(&core, &limits, 1, &target, &window()).unwrap() else { panic!() };
    assert_eq!(proposal.reason.bytes.as_ref(), b"More room");
    assert_eq!(proposal.waiting.holder, tasks::ProposalHolder::Task(1));
    assert_eq!(proposal.funding.get(0).unwrap().source, tasks::Funder::Task(1));
    assert_eq!(proposal.funding.get(0).unwrap().needed, 10);
    assert_eq!(proposal.funding.get(0).unwrap().left, Some(70));
    assert_eq!(proposal.funding.get(0).unwrap().after, Some(60));
    let ProposalAction::Batch(batch) = proposal.action else { panic!() };
    assert_eq!(batch.get(0).unwrap().number, 5);
    assert_eq!(batch.get(0).unwrap().authority.ceiling.spend, 10);
    send_task(
        &mut core,
        &limits,
        tasks::Event::DecideProposal {
            reply_to: ReplyTo::new(Token::new(51)),
            proposer: 2,
            proposal: 10,
            message: None,
            by: tasks::Party::Task(1),
            decision: tasks::ProposalDecision::Pass { holder: tasks::ProposalHolder::Person(1) },
        },
    );
    let Document::Proposal(proposal) = make(&core, &limits, 1, &target, &window()).unwrap() else { panic!() };
    assert_eq!(proposal.waiting.holder, tasks::ProposalHolder::Person(1));
    send_task(
        &mut core,
        &limits,
        tasks::Event::WithdrawProposal { reply_to: ReplyTo::new(Token::new(52)), proposer: 2, proposal: 10 },
    );
    assert_eq!(make(&core, &limits, 1, &target, &window()), Err(Missing::Ended));
}

#[test]
fn documents_proposal_actions_keep_effect_parts_and_amendment_before_and_after() {
    let (mut core, limits) = make_tree();
    let mut changed = core.tasks.task(2).unwrap().spec.clone();
    changed.words = Box::from(&b"New question"[..]);
    let mut wider = core.tasks.task(2).unwrap().authority.clone();
    wider.budget.spend = 15;
    let actions = [
        tasks::ProposalAction::Effect {
            connector: 2,
            authority: wider.clone(),
            attempt: 7,
            completion: 8,
            position: 9,
            purpose: 11,
        },
        tasks::ProposalAction::Amend {
            task: 2,
            amendment: tasks::Amendment {
                spec: Some(changed),
                wake: None,
                dependencies: None,
                authority: Some(wider.clone()),
                reason: Box::from(&b"Reconsider"[..]),
            },
        },
        tasks::ProposalAction::Widen { task: 2, authority: wider },
        tasks::ProposalAction::Release { task: 3 },
    ];
    for (offset, action) in actions.into_iter().enumerate() {
        let number = u64::try_from(offset).unwrap() + 20;
        send_task(
            &mut core,
            &limits,
            tasks::Event::Propose {
                reply_to: ReplyTo::new(Token::new(number)),
                proposal: tasks::Proposal {
                    number,
                    proposer: 2,
                    project: 1,
                    action,
                    reason: Box::from(&b"Reason"[..]),
                    as_holder: false,
                    state: tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Task(1), since: Wall::EPOCH },
                },
            },
        );
        let Document::Proposal(proposal) =
            make(&core, &limits, 1, &Subject::Proposal { proposer: Proposer::Task(2), number }, &window()).unwrap()
        else {
            panic!()
        };
        match proposal.action {
            ProposalAction::Effect { part, purpose, .. } => {
                assert_eq!(part.connector, 2);
                assert_eq!(part.key, PartKey::Number(number));
                assert_eq!(purpose, 11);
                assert!(proposal.funding.is_empty());
            }
            ProposalAction::Amend { before, after, .. } => {
                assert_eq!(before.spec.words.bytes.as_ref(), b"Work\nDetailed instructions");
                assert_eq!(after.spec.words.bytes.as_ref(), b"New question");
                assert_eq!(after.authority.ceiling.spend, 15);
                assert_eq!(proposal.funding.get(0).unwrap().needed, 5);
            }
            ProposalAction::Widen { before, after, .. } => {
                assert_eq!(before.ceiling.spend, 10);
                assert_eq!(after.ceiling.spend, 15);
            }
            ProposalAction::Release { hold, .. } => assert_eq!(hold.unwrap().reason, tasks::Hold::Budget),
            ProposalAction::Batch(_) => panic!(),
        }
        send_task(
            &mut core,
            &limits,
            tasks::Event::WithdrawProposal { reply_to: ReplyTo::new(Token::new(90)), proposer: 2, proposal: number },
        );
    }
}

#[test]
fn documents_every_escalation_reason_keeps_the_task_story_and_independent_revision() {
    let reasons = [
        tasks::Hold::Failures(tasks::Class::Lost),
        tasks::Hold::Stalled,
        tasks::Hold::Dependency(2),
        tasks::Hold::Stopped,
        tasks::Hold::StoppedBy { party: 1 },
        tasks::Hold::Drift,
        tasks::Hold::Effects,
        tasks::Hold::EffectFailed,
        tasks::Hold::Uncertain { entry: 4 },
        tasks::Hold::Budget,
        tasks::Hold::Deadline,
        tasks::Hold::HoldsWaited,
        tasks::Hold::ResourceUnavailable,
        tasks::Hold::ResourceAccess,
        tasks::Hold::PoolLost { pool: 6 },
        tasks::Hold::Procedure,
    ];
    for reason in reasons {
        let (mut core, limits) = make_tree();
        send_task(&mut core, &limits, tasks::Event::Hold { task: 2, why: reason });
        let Document::Escalation(value) = make(&core, &limits, 1, &Subject::Escalation { task: 2 }, &window()).unwrap()
        else {
            panic!()
        };
        let hold = value.task.hold.unwrap();
        assert_eq!(hold.reason, reason);
        assert_eq!(hold.revision, core.tasks.task(2).unwrap().revisions.hold);
        assert_eq!(hold.before, PriorPhase::Active(tasks::Active::Due));
        assert_eq!(value.task.budget.numbers.budget, 10);
        assert_eq!(value.task.lineage.get(0).unwrap().number, 1);
    }
}

#[test]
fn documents_question_words_are_whole_or_refused_and_answered_questions_end() {
    let (mut core, mut limits) = make_tree();
    send_task(
        &mut core,
        &limits,
        tasks::Event::Message {
            reply_to: ReplyTo::new(Token::new(60)),
            project: 1,
            task: 1,
            word: tasks::Word {
                number: 60,
                from: tasks::Party::Task(2),
                kind: tasks::MessageKind::Question,
                words: Box::from(&b"Whole question?"[..]),
                at: Wall::from_nanos(9),
                hits: 1,
                eligible: true,
            },
        },
    );
    let target = Subject::Question { task: 1, message: 60 };
    let Document::Question(question) = make(&core, &limits, 1, &target, &window()).unwrap() else { panic!() };
    assert_eq!(question.words.as_ref(), b"Whole question?");
    assert_eq!(question.at, Wall::from_nanos(9));
    limits.documents.text_bytes = 5;
    assert_eq!(make(&core, &limits, 1, &target, &window()), Err(Missing::TooLarge));
    limits.documents.text_bytes = 256;
    send_task(
        &mut core,
        &limits,
        tasks::Event::Message {
            reply_to: ReplyTo::new(Token::new(61)),
            project: 1,
            task: 1,
            word: tasks::Word {
                number: 61,
                from: tasks::Party::Person(1),
                kind: tasks::MessageKind::Answer { question: 60 },
                words: Box::from(&b"Yes"[..]),
                at: Wall::from_nanos(10),
                hits: 1,
                eligible: true,
            },
        },
    );
    assert_eq!(make(&core, &limits, 1, &target, &window()), Err(Missing::Ended));
}

#[test]
fn documents_person_task_has_the_current_revision_holder_and_answer_form() {
    let (mut core, limits) = make_tree();
    let mut row = new_row(&core, 5, tasks::Funder::Period { project: 1, period: 1 }, None);
    row.executor = tasks::Executor::Person(tasks::PersonAddress::Role(0));
    row.contract =
        tasks::Contract::Verdict { choices: Box::new([tasks::Verdict { code: 3, words: 32, followups: 0 }]) };
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(70)),
            creator: tasks::Party::Person(1),
            batch: Box::new([row]),
        },
    );
    send_task(
        &mut core,
        &limits,
        tasks::Event::TakePerson { reply_to: ReplyTo::new(Token::new(71)), task: 5, person: 1 },
    );
    let Document::PersonTask(task) = make(&core, &limits, 1, &Subject::PersonTask { task: 5 }, &window()).unwrap()
    else {
        panic!()
    };
    assert_eq!(task.address, tasks::PersonAddress::Role(0));
    assert_eq!(task.taken_by, Some(1));
    assert_eq!(task.revision, core.tasks.task(5).unwrap().revisions.person);
    let ContractLine::Verdict { choices, cut } = task.contract else { panic!() };
    assert!(!cut);
    assert_eq!(choices.get(0).unwrap().code, 3);
}

#[test]
fn documents_retained_results_and_ended_delegates_preserve_cancellation_facts() {
    let (core, limits) = make_tree();
    let mut row = core.tasks.task(2).unwrap().clone();
    row.phase = tasks::Phase::Ended(tasks::Ending::Cancelled {
        reason: Box::from(&b"Done enough"[..]),
        result: Some(tasks::TaskResult::Report { words: Box::from(&b"Partial work"[..]) }),
        by: tasks::Party::Person(1),
        at: Wall::from_nanos(19),
    });
    row.ended_at = Some(Wall::from_nanos(19));
    let result = result_from_row(&core, &limits, 1, &row).unwrap();
    assert_eq!(result.at, Some(Wall::from_nanos(19)));
    assert_eq!(result.task.phase, PhaseLine::Ended);
    let Ending::Cancelled { by, at, result, .. } = result.ending else { panic!() };
    assert_eq!(by, tasks::Party::Person(1));
    assert_eq!(at, Wall::from_nanos(19));
    let ResultValue::Report(words) = result.unwrap() else { panic!() };
    assert_eq!(words.bytes.as_ref(), b"Partial work");
    let delegate = ended_delegate_from_row(&limits, &row).unwrap();
    assert_eq!(delegate.chip.number, 2);
    assert_eq!(delegate.at, row.ended_at);
    for ending in [
        tasks::Ending::Done(tasks::TaskResult::Verdict { code: 3, words: Box::from(&b"Choice"[..]) }),
        tasks::Ending::Failed { reason: Box::from(&b"Failure"[..]) },
        tasks::Ending::Done(tasks::TaskResult::Change {
            connector: 2,
            kind: 3,
            resource: 4,
            words: Box::from(&b"Changed"[..]),
        }),
    ] {
        row.phase = tasks::Phase::Ended(ending);
        let result = result_from_row(&core, &limits, 1, &row).unwrap();
        match result.ending {
            Ending::Done(ResultValue::Verdict { code, .. }) => assert_eq!(code, 3),
            Ending::Done(ResultValue::Change { part, .. }) => assert_eq!(part.key, PartKey::Number(4)),
            Ending::Failed(reason) => assert_eq!(reason.bytes.as_ref(), b"Failure"),
            Ending::Done(ResultValue::Report(_) | ResultValue::Failure(_)) | Ending::Cancelled { .. } => panic!(),
        }
    }
}

#[test]
fn documents_notes_carry_the_child_revision_and_cut_display_text_only() {
    let (core, mut limits) = make_tree();
    limits.documents.text_bytes = 5;
    let mut references = List::with_capacity(1);
    references.push(2).unwrap();
    let entry = notes::Entry {
        name: 8,
        scope: notes::Scope::Project { project: 1 },
        description: Box::from(&b"Description"[..]),
        body: Box::from(&b"Full body"[..]),
        references,
        author: notes::Author::Task { task: 2, attempt: 1 },
        revision: 7,
        changed: Wall::from_nanos(100),
    };
    let note = note_from_entry(&core, &limits, 1, &entry).unwrap();
    assert_eq!(note.revision, 7);
    assert_eq!(note.writer, entry.author);
    assert_eq!(note.tasks.get(0), Some(&2));
    assert!(note.description.cut);
    assert!(note.body.cut);
    assert_eq!(note.changed, Some(entry.changed));
    assert_eq!(make(&core, &limits, 1, &Subject::Note { note: 8 }, &window()), Err(Missing::Ended));
}

#[test]
fn documents_history_retains_every_change_kind_and_a_passed_proposal_state() {
    let (_, limits) = make_tree();
    let changes = [
        tasks::Change::Cancelled,
        tasks::Change::Released,
        tasks::Change::Amended,
        tasks::Change::Prioritised { priority: 3 },
        tasks::Change::Moved,
        tasks::Change::Proposed,
        tasks::Change::ProposalPassed,
        tasks::Change::ProposalAccepted,
        tasks::Change::ProposalRejected,
        tasks::Change::ProposalWithdrawn,
    ];
    for change in changes {
        let row = tasks::History {
            task: 2,
            revision: 5,
            by: tasks::Party::Person(1),
            change,
            reason: Box::from(&b"Why"[..]),
            proposal: Some(Box::new(tasks::Proposal {
                number: 6,
                proposer: 2,
                project: 1,
                action: tasks::ProposalAction::Release { task: 2 },
                reason: Box::from(&b"Why"[..]),
                as_holder: false,
                state: tasks::ProposalState::Pending {
                    holder: tasks::ProposalHolder::Person(1),
                    since: Wall::from_nanos(8),
                },
            })),
        };
        let history = history_from_row(&limits, &row).unwrap();
        assert_eq!(history.change, change);
        assert_eq!(history.by, row.by);
        assert_eq!(history.reason.bytes.as_ref(), b"Why");
        assert_eq!(
            history.proposal.unwrap().state,
            ProposalHistoryState::Pending(ProposalWaiting {
                holder: tasks::ProposalHolder::Person(1),
                since: Wall::from_nanos(8)
            })
        );
    }
}

#[test]
fn documents_project_exposes_two_roles_members_pool_and_resource_parts() {
    let (mut core, limits) = make_tree();
    let mut policy = core.authority.policy(1).unwrap().clone();
    let mut second = policy.roles[0].clone();
    second.number = 1;
    policy.roles = Box::new([policy.roles[0].clone(), second]);
    let mut admitted = authority_limits();
    admitted.roles = 2;
    core.authority = authority::Domain::new(core.authority.rules().clone(), admitted).unwrap();
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut core.authority, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    let first_identity = core.people.identity(1).unwrap().clone();
    core.people = people::Domain::new(&limits.people, Box::new([]), core.settings.deployment_provider);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut restored = Queue::with_capacity(people::max_out(&limits.people));
    for record in [
        people::Stored::Person { number: 1, identity: first_identity },
        people::Stored::Person {
            number: 2,
            identity: people::Identity {
                key: people::IdentityKey {
                    provider: core.settings.deployment_provider,
                    subject: Box::from(&b"service"[..]),
                },
                login: Box::from(&b"service"[..]),
                name: Box::from(&b"Service"[..]),
            },
        },
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([
                people::Holding { person: 1, role: people::Role::Owner },
                people::Holding { person: 2, role: people::Role::Maintainer },
            ]),
        },
    ] {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut restored);
        while restored.pop().is_some() {}
    }
    people::step(&mut core.people, &env, people::Event::Restored, &mut restored);
    send_task(
        &mut core,
        &limits,
        tasks::Event::CarvePool {
            reply_to: ReplyTo::new(Token::new(80)),
            project: 1,
            person: 1,
            period: 1,
            budget: 200,
        },
    );
    core.resource_roles
        .insert(
            resources::Key {
                project: 1,
                name: tasks::Name { connector: 2, path: Box::new([Box::from(&b"resource"[..])]) },
            },
            connector::ResourceRole::Owned,
        )
        .unwrap();
    let Document::Project(project) = make(&core, &limits, 1, &Subject::Project { project: 1 }, &window()).unwrap()
    else {
        panic!()
    };
    assert_eq!(project.policy.roles.len(), 2);
    assert_eq!(project.members.get(0).unwrap().role, people::Role::Owner);
    assert_eq!(project.members.get(1).unwrap().kind, people::Kind::Service);
    assert_eq!(project.members.get(1).unwrap().role, people::Role::Maintainer);
    assert_eq!(project.current_period.unwrap().numbers.reserved, 300);
    assert_eq!(project.pools.get(0).unwrap().left, 200);
    assert_eq!(project.resources.get(0).unwrap().part.connector, 2);
    assert_eq!(project.resources.get(0).unwrap().role, ResourceRole::Owned);
    assert_eq!(project.spend.get(0).unwrap().chip.number, 1);
    assert_eq!(project.spend.get(0).unwrap().numbers.reserved, 30);
}

#[test]
fn documents_lifecycle_history_keeps_holds_and_endings() {
    let (_, limits) = make_tree();
    let row = tasks::Milestone {
        task: 2,
        position: 3,
        phase: tasks::Phase::Held { was: tasks::Was::Active(tasks::Active::Due), why: tasks::Hold::Deadline },
    };
    let line = milestone_from_row(&limits, &row);
    assert_eq!(line.position, 3);
    assert_eq!(
        line.phase,
        MilestonePhase::Held { before: PriorPhase::Active(tasks::Active::Due), why: tasks::Hold::Deadline }
    );
    assert_eq!(line.at, None);
    let line = milestone_from_row(
        &limits,
        &tasks::Milestone {
            task: 2,
            position: 4,
            phase: tasks::Phase::Ended(tasks::Ending::Failed { reason: Box::from(&b"Failure"[..]) }),
        },
    );
    let MilestonePhase::Ended(Ending::Failed(words)) = line.phase else { panic!() };
    assert_eq!(words.bytes.as_ref(), b"Failure");
}

fn restore_document_party(core: &mut Core, limits: &Limits, projects: &[u32]) {
    let identity = core.people.identity(1).unwrap().clone();
    core.people = people::Domain::new(&limits.people, Box::new([]), core.settings.deployment_provider);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    people::step(
        &mut core.people,
        &env,
        people::Event::Restore { record: people::Stored::Person { number: 1, identity } },
        &mut out,
    );
    while out.pop().is_some() {}
    for project in projects {
        people::step(
            &mut core.people,
            &env,
            people::Event::Restore {
                record: people::Stored::Roles {
                    project: *project,
                    holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
                },
            },
            &mut out,
        );
        while out.pop().is_some() {}
    }
    people::step(&mut core.people, &env, people::Event::Restored, &mut out);
    assert!(core.people.identity(1).is_some());
}
#[test]
fn documents_inbox_keeps_two_projects_groups_order_and_exact_cursor() {
    let (mut core, mut limits) = make_tree();
    limits.people.projects = 2;
    limits.people.inbox_entries = 8;
    restore_document_party(&mut core, &limits, &[1, 2]);
    send_task(
        &mut core,
        &limits,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(101)), project: 2, period: 1, budget: 1000 },
    );
    let mut task = new_row(&core, 5, tasks::Funder::Period { project: 2, period: 1 }, None);
    task.project = 2;
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(102)),
            creator: tasks::Party::Person(1),
            batch: Box::new([task]),
        },
    );
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    let needs = [
        people::Entry {
            task: 1,
            project: 1,
            whom: people::Whom::Person(1),
            kind: people::EntryKind::Question { message: 9 },
            at: Wall::from_nanos(30),
        },
        people::Entry {
            task: 1,
            project: 1,
            whom: people::Whom::Person(1),
            kind: people::EntryKind::Question { message: 8 },
            at: Wall::from_nanos(20),
        },
        people::Entry {
            task: 5,
            project: 2,
            whom: people::Whom::Role { project: 2, role: 0 },
            kind: people::EntryKind::PersonTask,
            at: Wall::from_nanos(10),
        },
    ];
    people::step(
        &mut core.people,
        &env,
        people::Event::Waiting { task: 1, entries: Box::new([needs[0], needs[1]]) },
        &mut out,
    );
    people::step(
        &mut core.people,
        &env,
        people::Event::Waiting {
            task: 5,
            entries: Box::new([
                needs[2],
                people::Entry {
                    task: 5,
                    project: 2,
                    whom: people::Whom::Person(1),
                    kind: people::EntryKind::Reply { message: 7 },
                    at: Wall::from_nanos(40),
                },
            ]),
        },
        &mut out,
    );
    let Document::Inbox(first) =
        make(&core, &limits, 1, &Subject::Inbox { party: 1 }, &Window { rows: 2, after: None }).unwrap()
    else {
        panic!()
    };
    assert_eq!(first.needs.len(), 2);
    assert_eq!(first.cached_needs, 3);
    assert!(first.updates.is_empty());
    assert_eq!(first.needs.get(0).unwrap().project, Some(2));
    assert_eq!(first.needs.get(1).unwrap().object, InboxObject::Question { message: 8 });
    let Document::Inbox(next) =
        make(&core, &limits, 1, &Subject::Inbox { party: 1 }, &Window { rows: 2, after: first.next }).unwrap()
    else {
        panic!()
    };
    assert_eq!(next.needs.get(0).unwrap().object, InboxObject::Question { message: 9 });
    assert_eq!(next.updates.get(0).unwrap().object, InboxObject::Reply { message: 7 });
    assert_eq!(next.next, None);
    assert_eq!(make(&core, &limits, 1, &Subject::Inbox { party: 2 }, &window()), Err(Missing::NotAllowed));
}
#[test]
fn documents_cached_inbox_results_preserve_positions_and_unknown_store_metadata() {
    let (mut core, limits) = make_tree();
    core.people.remember_result(&limits.people, 1, people::ResultRef { task: 100, position: 30 });
    core.people.remember_result(&limits.people, 1, people::ResultRef { task: 99, position: 31 });
    let Document::Inbox(inbox) =
        make(&core, &limits, 1, &Subject::Inbox { party: 1 }, &Window { rows: 2, after: None }).unwrap()
    else {
        panic!()
    };
    assert_eq!(inbox.updates.get(0).unwrap().object, InboxObject::Result { position: 31 });
    assert_eq!(inbox.updates.get(0).unwrap().project, None);
    assert_eq!(inbox.updates.get(0).unwrap().at, None);
    assert_eq!(inbox.updates.get(0).unwrap().unread, Some(true));
    assert!(!inbox.updates_time_complete);
    assert_eq!(inbox.cached_updates, Count::AtLeast(2));
}
#[test]
fn documents_conversations_show_chats_and_exclude_goals() {
    let (mut core, limits) = make_tree();
    let row = new_row(&core, 5, tasks::Funder::Period { project: 1, period: 1 }, None);
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(110)),
            creator: tasks::Party::Person(1),
            batch: Box::new([row]),
        },
    );
    let Document::Conversations(conversations) =
        make(&core, &limits, 1, &Subject::Conversations { party: 1 }, &window()).unwrap()
    else {
        panic!()
    };
    assert_eq!(conversations.lines.len(), 1);
    assert_eq!(conversations.lines.get(0).unwrap().chip.number, 5);
    assert_eq!(conversations.lines.get(0).unwrap().budget.numbers.budget, 100);
    assert_eq!(conversations.lines.get(0).unwrap().replies_unread, None);
}
#[test]
fn documents_deployment_keeps_lost_worker_attempts_and_translates_the_grace_clock() {
    let (mut core, mut limits) = make_tree();
    limits.people.projects = 2;
    assert_eq!(make(&core, &limits, 1, &Subject::Deployment, &window()), Err(Missing::NotAllowed));
    restore_document_party(&mut core, &limits, &[0]);
    let env = Env { now: Time::from_nanos(100), wall: Wall::from_nanos(1000), limits: limits.fleet };
    let mut out = Queue::with_capacity(fleet::max_out(&limits.fleet));
    fleet::step(
        &mut core.fleet,
        &env,
        fleet::Event::Hello {
            channel: Token::new(10),
            hello: fleet::Hello {
                stop_bound: Duration::from_secs(1),
                slots: 1,
                workstreams: Box::new([]),
                hosting: Box::new([fleet::Hosted {
                    run: Token::new(2),
                    attempt: Token::new(9),
                    phase: fleet::Phase::Active,
                }]),
            },
        },
        &mut out,
    );
    while out.pop().is_some() {}
    let clock = DocumentClock { now: env.now, wall: env.wall };
    let Document::Deployment(deployment) =
        make_with_clock(&core, &limits, 1, &Subject::Deployment, &window(), clock).unwrap()
    else {
        panic!()
    };
    assert_eq!(deployment.hosts.len(), 1);
    assert_eq!(deployment.hosts.get(0).unwrap().used, 1);
    assert_eq!(deployment.hosts.get(0).unwrap().free, 0);
    assert_eq!(deployment.attempts.get(0).unwrap().attempt, 9);
    fleet::step(&mut core.fleet, &env, fleet::Event::Lost { channel: Token::new(10) }, &mut out);
    let Document::Deployment(deployment) =
        make_with_clock(&core, &limits, 1, &Subject::Deployment, &window(), clock).unwrap()
    else {
        panic!()
    };
    assert!(deployment.hosts.is_empty());
    assert_eq!(deployment.workers.used, 0);
    assert_eq!(deployment.attempts.len(), 1);
    let RunLocation::Adrift { until, remaining } = deployment.attempts.get(0).unwrap().location else { panic!() };
    assert_eq!(until, Some(Wall::from_nanos(1000 + limits.fleet.grace.as_nanos())));
    assert_eq!(remaining, Some(limits.fleet.grace));
    let Document::Deployment(without_clock) = make(&core, &limits, 1, &Subject::Deployment, &window()).unwrap() else {
        panic!()
    };
    assert_eq!(without_clock.attempts.get(0).unwrap().location, RunLocation::Adrift { until: None, remaining: None });
}
#[test]
fn documents_deployment_accounts_are_secret_free_and_show_unusable_state() {
    let (mut core, mut limits) = make_tree();
    limits.people.projects = 2;
    restore_document_party(&mut core, &limits, &[0]);
    let env = Env { now: Time::from_nanos(100), wall: Wall::from_nanos(1000), limits: limits.accounts };
    let mut out = Queue::with_capacity(accounts::MAX_OUT);
    accounts::step(
        &mut core.accounts,
        &env,
        accounts::Event::Add { account: 1, generation: 1, valid: Some(Duration::from_secs(60)) },
        &mut out,
    );
    while out.pop().is_some() {}
    accounts::step(
        &mut core.accounts,
        &env,
        accounts::Event::Exhausted { account: 1, retry_after: Duration::from_secs(5) },
        &mut out,
    );
    let Document::Deployment(deployment) = make_with_clock(
        &core,
        &limits,
        1,
        &Subject::Deployment,
        &window(),
        DocumentClock { now: env.now, wall: env.wall },
    )
    .unwrap() else {
        panic!()
    };
    let account = deployment.accounts.get(0).unwrap();
    assert!(!account.usable);
    assert_eq!(account.number, 1);
    assert_eq!(account.state, accounts::State::Fresh);
    assert_eq!(account.spent_until, Some(Wall::from_nanos(1000 + Duration::from_secs(5).as_nanos())));
    assert!(!account.waiting.is_empty());
}

fn watch_core() -> (Core, Limits) {
    let (mut core, mut limits) = core();
    limits.people.projects = 2;
    limits.people.requests = 0;
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
                turn: None,
                terminal: None,
            },
        )
        .unwrap();
    let identity = core.people.identity(1).unwrap().clone();
    core.people = people::Domain::new(&limits.people, Box::new([]), core.settings.deployment_provider);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    let records = [
        people::Stored::Person { number: 1, identity },
        people::Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()) },
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
        people::Stored::Roles {
            project: 0,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Observer }]),
        },
    ];
    for record in records {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut out);
        while out.pop().is_some() {}
    }
    people::step(&mut core.people, &env, people::Event::Restored, &mut out);
    (core, limits)
}

fn watch_outputs(core: &mut Core, limits: &Limits, event: Event) -> Box<[Request]> {
    match step(core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }, event) {
        Requests::Out(mut out) => {
            let mut requests = List::with_capacity(out.len());
            while let Some(request) = out.pop() {
                requests.push(request).unwrap();
            }
            requests.into_boxed()
        }
    }
}

fn watch_subjects() -> [Subject; 9] {
    let mut chips = List::with_capacity(1);
    chips.push(1).unwrap();
    [
        Subject::Frame { party: 1 },
        Subject::Task { task: 1 },
        Subject::Tree { task: 1 },
        Subject::Chips { tasks: chips },
        Subject::Inbox { party: 1 },
        Subject::Goals { project: 1 },
        Subject::Conversations { party: 1 },
        Subject::Run { task: 1 },
        Subject::Deployment,
    ]
}

#[test]
fn an_unkeyed_watch_on_each_subject_makes_its_first_document_as_a_held_output() {
    for subject in watch_subjects() {
        let (mut core, limits) = watch_core();
        let expected = make(&core, &limits, 1, &subject, &window()).unwrap();
        let out = watch_outputs(
            &mut core,
            &limits,
            Event::Watch { watcher: Token::new(30), sign_in: 10, subject, window: window() },
        );
        assert_eq!(out.len(), 2);
        let Request::Held(held) = &out[0] else { panic!("document is held") };
        let Held::Delivery { watcher, document } = held.as_ref() else { panic!("typed delivery") };
        assert_eq!(*watcher, Token::new(30));
        assert_eq!(document.as_ref(), &expected);
        let Request::Decided = &out[1] else { panic!("one decision") };
    }
}

#[test]
fn watch_admission_checks_each_subjects_project_or_owner() {
    for subject in watch_subjects() {
        let (mut core, limits) = watch_core();
        let subject = match subject {
            Subject::Frame { .. } => Subject::Frame { party: 2 },
            Subject::Inbox { .. } => Subject::Inbox { party: 2 },
            Subject::Conversations { .. } => Subject::Conversations { party: 2 },
            other @ (Subject::Task { .. }
            | Subject::Tree { .. }
            | Subject::Chips { .. }
            | Subject::Goals { .. }
            | Subject::Run { .. }
            | Subject::Deployment
            | Subject::Proposal { .. }
            | Subject::Escalation { .. }
            | Subject::Question { .. }
            | Subject::PersonTask { .. }
            | Subject::Result { .. }
            | Subject::Project { .. }
            | Subject::Note { .. }) => other,
        };
        let mut out = Queue::with_capacity(people::max_out(&limits.people));
        for project in [0, 1] {
            people::step(
                &mut core.people,
                &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
                people::Event::Roles { project, holdings: Box::new([]) },
                &mut out,
            );
        }
        let out = watch_outputs(
            &mut core,
            &limits,
            Event::Watch { watcher: Token::new(30), sign_in: 10, subject, window: window() },
        );
        assert_watch_refusal(&out[0], WatchRefusal::NotAllowed);
        assert!(core.view_subjects.is_empty());
    }
}

#[test]
fn signed_out_unknown_and_full_watches_are_refused_without_keyed_work() {
    let (mut core, limits) = watch_core();
    for (sign_in, subject, why) in [
        (99, Subject::Frame { party: 1 }, WatchRefusal::SignedOut),
        (10, Subject::Task { task: 99 }, WatchRefusal::Unknown),
        (10, Subject::Project { project: 1 }, WatchRefusal::Unknown),
    ] {
        let out = watch_outputs(
            &mut core,
            &limits,
            Event::Watch { watcher: Token::new(30), sign_in, subject, window: window() },
        );
        assert_watch_refusal(&out[0], why);
    }
    for watcher in 1..=limits.views.watchers {
        watch_outputs(
            &mut core,
            &limits,
            Event::Watch {
                watcher: Token::new(u64::from(watcher)),
                sign_in: 10,
                subject: Subject::Frame { party: 1 },
                window: window(),
            },
        );
    }
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::Watch { watcher: Token::new(30), sign_in: 10, subject: Subject::Frame { party: 1 }, window: window() },
    );
    assert_watch_refusal(&out[0], WatchRefusal::Busy);
}

#[test]
fn an_unwatch_waits_for_its_delivery_then_ends_as_a_held_output() {
    let (mut core, limits) = watch_core();
    let watcher = Token::new(30);
    watch_outputs(
        &mut core,
        &limits,
        Event::Watch { watcher, sign_in: 10, subject: Subject::Frame { party: 1 }, window: window() },
    );
    let out = watch_outputs(&mut core, &limits, Event::Unwatch { watcher });
    assert_eq!(out.len(), 1);
    let out = watch_outputs(&mut core, &limits, Event::Delivered { watcher });
    let Request::Held(held) = &out[0] else { panic!("ending is held") };
    let Held::WatchEnded { watcher: got, why } = held.as_ref() else { panic!("watch ending") };
    assert_eq!(*got, watcher);
    assert_eq!(*why, views::End::Unwatched);
    assert!(core.view_subjects.is_empty());
}

fn assert_watch_refusal(request: &Request, why: WatchRefusal) {
    let Request::Now(now) = request else { panic!("refusal writes nothing") };
    let Now::WatchRefused { why: got, .. } = now.as_ref() else { panic!("watch refusal") };
    assert_eq!(*got, why);
}
