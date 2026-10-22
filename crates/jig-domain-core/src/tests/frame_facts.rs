//! Supplied Frame facts keep absent names, unopened pools and actual current ledgers distinct.
use super::*;
const CREATING_PROJECT: u32 = 1;
const CREATING_PARTY: u64 = 1;
const CREATING_CHARTER: u32 = 1;
const CREATING_PROCEDURE: u32 = 1;

fn frame(core: &Core, limits: &Limits) -> Frame {
    let Document::Frame(value) =
        make(core, limits, 1, &Subject::Frame { party: 1 }, &window(), Wall::EPOCH).expect("frame")
    else {
        panic!("frame document")
    };
    value
}

#[test]
fn frame_projects_keep_unopened_pools_absent_and_use_actual_current_funding() {
    let (mut core, limits) = tree_core();
    let mut given = core.authority.role(1, 0).unwrap().authority.clone();
    given.delegation.tasks = 1;
    given.delegation.depth = 1;
    set_role_authority(&mut core.authority, given);
    let initial = frame(&core, &limits);
    let project = initial.projects.get(0).expect("project role");
    assert_eq!(project.name, None);
    assert_eq!(project.pool, None);
    let period = core.period(1);
    let source = tasks::Funder::Pool { project: 1, person: 1, period };
    send_task(
        &mut core,
        &limits,
        tasks::Event::CarvePool { reply_to: ReplyTo::new(Token::new(0)), project: 1, person: 1, period, budget: 177 },
    );
    let actual = core.tasks.funding(source).expect("actual pool");
    let joined = frame(&core, &limits);
    let pool = joined.projects.get(0).expect("project role").pool.expect("retained pool");
    assert_eq!(pool.source, source);
    assert_eq!(pool.parent, actual.parent);
    assert_eq!(pool.numbers, actual.numbers);
    assert_eq!(pool.left, 177);
    assert_ne!(pool.numbers.budget, core.pool_budget(1, 1));
}

#[test]
fn frame_joins_all_four_exact_need_counts_without_truncating_them_to_document_lines() {
    let (mut core, limits) = core();
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    for (task, kind) in [
        (1, people::EntryKind::Question { message: 1 }),
        (2, people::EntryKind::Proposal { number: 2 }),
        (3, people::EntryKind::Escalation { revision: 3 }),
        (4, people::EntryKind::PersonTask),
    ] {
        people::step(
            &mut core.people,
            &env,
            people::Event::Waiting {
                task,
                entries: Box::new([people::Entry {
                    task,
                    project: 1,
                    whom: people::Whom::Person(1),
                    kind,
                    at: Wall::EPOCH,
                }]),
            },
            &mut out,
        );
        while out.pop().is_some() {}
    }
    let value = frame(&core, &limits);
    assert_eq!(value.need_counts, people::Needs { questions: 1, proposals: 1, escalations: 1, person_tasks: 1 });
    assert_eq!(value.needs, 4);
    assert_eq!(value.projects.len(), 1);
}

#[test]
fn frame_watch_receives_pool_ledger_changes_after_their_decision() {
    let (mut core, mut limits) = tree_core();
    let (auth, auth_limits) = watch_core();
    core.people = auth.people;
    limits.people = auth_limits.people;
    limits.documents.lines = 2;
    let policy = core.authority.policy(1).expect("configured policy").clone();
    core.authority = authority::Domain::new(
        core.authority.rules().clone(),
        authority::Limits { projects: 2, ..*core.authority.limits() },
    )
    .expect("two project authority");
    let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
    for project in [1, 2] {
        authority::step(&mut core.authority, authority::Event::Policy { project, policy: policy.clone() }, &mut facts);
        assert_eq!(facts.pop(), Some(authority::PolicyFact::Added { project }));
    }
    let watcher = Token::new(30);
    let period = core.period(1);
    watch_outputs(&mut core, &limits, Event::Watch { watcher, sign_in: 10, subject: Watched::Frame, window: window() });
    watch_outputs(&mut core, &limits, Event::Delivered { watcher });
    watch_outputs(
        &mut core,
        &limits,
        Event::Tasks(tasks::Event::CarvePool {
            reply_to: ReplyTo::new(Token::new(0)),
            project: 1,
            person: 1,
            period,
            budget: 177,
        }),
    );
    let row = new_row(&core, 1, tasks::Funder::Pool { project: 1, person: 1, period }, None);
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(91)),
            creator: tasks::Party::Person(1),
            batch: Box::new([row]),
        },
    );
    let changed = finish_watch_outputs(&mut core, &limits);
    let Request::Held(value) = &changed[0] else { panic!("held frame") };
    let Held::Delivery { document, .. } = value.as_ref() else { panic!("frame delivery") };
    let Document::Frame(value) = document.as_ref() else { panic!("frame") };
    assert_eq!(value.projects.get(0).expect("project 1").pool.expect("actual pool").numbers.budget, 177);
    watch_outputs(&mut core, &limits, Event::Delivered { watcher });
    watch_outputs(&mut core, &limits, Event::Period { project: 1, period: 2, budget: 1000, next: None });
    let next = finish_watch_outputs(&mut core, &limits);
    let Request::Held(value) = &next[0] else { panic!("changed frame") };
    let Held::Delivery { document, .. } = value.as_ref() else { panic!("frame delivery") };
    let Document::Frame(value) = document.as_ref() else { panic!("frame") };
    assert_eq!(value.projects.get(0).expect("project 1").pool, None, "the new creation period has no pool yet");
    assert!(
        core.tasks.funding(tasks::Funder::Pool { project: 1, person: 1, period }).is_some(),
        "the prior pool remains an actual owner ledger"
    );
}

#[test]
fn the_frame_offers_both_charters_for_goals() {
    let (old, mut limits) = tree_core();
    limits.tasks.charters = 2;
    limits.tasks.executor_kinds = 3;
    limits.authority.executors = 3;
    limits.documents.executors = 3;
    let mut config = config(146);
    let mut second = config.settings.charters[0].clone();
    second.number = 2;
    config.settings.charters = Box::new([config.settings.charters[0].clone(), second]);
    let kinds: Box<[authority::Executor]> =
        Box::new([authority::Executor::Charter(1), authority::Executor::Charter(2), authority::Executor::Procedure(1)]);
    let mut rules = config.authority.rules().clone();
    rules.ceiling.delegation.kinds.clone_from(&kinds);
    let mut policy = config.authority.policy(1).expect("project policy").clone();
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    policy.roles[0].authority.delegation.kinds = kinds;
    config.authority = authority::Domain::new(rules, limits.authority).expect("charter policy");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut config.authority, authority::Event::Policy { project: 1, policy }, &mut out);
    let mut core = Core::new(config, &limits).expect("fixture core configuration");
    core.people = old.people;
    let frame = frame(&core, &limits);
    let crate::acts::Creating { within: crate::acts::Bounds::Create { charters, .. }, .. } =
        frame.projects.get(0).expect("project").creating.as_ref().expect("creating")
    else {
        panic!("creating bounds")
    };
    assert_eq!(charters.as_slice(), [1, 2]);
}

pub(super) fn owner_creating(capacity: u32, tasks_left: u32, depth: u32) -> crate::acts::Bounds {
    let mut charters = List::with_capacity(capacity);
    charters.push(1).unwrap();
    let mut procedures = List::with_capacity(capacity);
    for connector in [1, 2] {
        procedures.push(crate::acts::CreateProcedure { connector, code: 1 }).unwrap();
    }
    let mut executors = List::with_capacity(2);
    executors.push(tasks::AuthorityExecutor::Charter(CREATING_CHARTER)).unwrap();
    executors.push(tasks::AuthorityExecutor::Procedure(CREATING_PROCEDURE)).unwrap();
    crate::acts::Bounds::Create {
        authority: Box::new(AuthorityLine {
            models: tasks::Models::ALL,
            tools: tasks::Tools(1023),
            grants: List::with_capacity(0),
            executors,
            tasks: tasks_left,
            depth,
            ceiling: tasks::Budget { spend: 500, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: List::with_capacity(0),
        }),
        pool_left: 500,
        charters,
        procedures,
        latest: None,
        budget: 500,
        maximum_budget: 1000,
        maximum_deadline: None,
    }
}

fn hold(core: &mut Core, limits: &Limits, role: people::Role) {
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    people::step(
        &mut core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::Roles { project: 1, holdings: Box::new([people::Holding { person: 1, role }]) },
        &mut out,
    );
}

#[test]
fn me_carries_a_members_creating_bounds_per_project() {
    let (mut core, limits) = tree_core();
    let mut policy = core.authority.policy(1).unwrap().clone();
    policy.roles[0].number = people::Role::Member.number();
    policy.escalation_role = Some(people::Role::Member.number());
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut core.authority, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Changed { project: 1 }));
    hold(&mut core, &limits, people::Role::Member);
    let value = frame(&core, &limits);
    let Some(creating) = &value.projects.get(0).unwrap().creating else { panic!("member creating bound") };
    assert_eq!(creating.within, owner_creating(limits.documents.lines, 15, 4));
    let crate::acts::Bounds::Create { authority, .. } = &creating.within else { panic!("creating authority") };
    let given =
        core.creating_authority(CREATING_PROJECT, CREATING_PARTY, people::Role::Member, limits.tasks.tree_tasks);
    assert_eq!(**authority, act_authority(&limits, &task_authority(&given)).unwrap());
}

#[test]
fn me_shows_the_models_a_partys_role_may_use_in_each_project() {
    let (mut core, limits) = core();
    let mut policy = core.authority.policy(1).unwrap().clone();
    policy.ceiling.models = authority::Models::Named(0b011);
    policy.roles[0].authority.models = authority::Models::Named(0b110);
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut core.authority, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Changed { project: 1 }));
    let value = frame(&core, &limits);
    assert_eq!(value.projects.get(0).unwrap().models, authority::Models::Named(0b010));
}

#[test]
fn me_carries_no_creating_bounds_for_an_observer() {
    let (mut core, limits) = tree_core();
    hold(&mut core, &limits, people::Role::Observer);
    let value = frame(&core, &limits);
    assert_eq!(value.projects.get(0).unwrap().creating, None);
}
