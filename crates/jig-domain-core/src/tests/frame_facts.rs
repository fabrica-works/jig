//! Supplied Frame facts keep absent names, unopened pools and actual current ledgers distinct.
use super::*;

fn frame(core: &Core, limits: &Limits) -> Frame {
    let Document::Frame(value) = make(core, limits, 1, &Subject::Frame { party: 1 }, &window()).expect("frame") else {
        panic!("frame document")
    };
    value
}

#[test]
fn frame_projects_keep_unopened_pools_absent_and_use_actual_current_funding() {
    let (mut core, limits) = tree_core();
    let initial = frame(&core, &limits);
    let project = initial.projects.get(0).expect("project role");
    assert_eq!(project.name, None);
    assert_eq!(project.pool, None);
    assert_eq!(project.chat_budget, core.settings.chat_authority.budget.spend);
    let period = core.settings.period;
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
    assert_ne!(pool.numbers.budget, core.settings.person_budget);
    assert_eq!(joined.charters.get(0).expect("charters").chat, core.settings.chat);
    assert_eq!(joined.charters.get(0).expect("charters").goals.as_slice(), core.tasks.charters());
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
    let period = core.settings.period;
    watch_outputs(
        &mut core,
        &limits,
        Event::Watch { watcher, sign_in: 10, subject: Subject::Frame { party: 1 }, window: window() },
    );
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
    let changed = finish_watch_outputs(&mut core, &limits);
    let Request::Held(value) = &changed[0] else { panic!("held frame") };
    let Held::Delivery { document, .. } = value.as_ref() else { panic!("frame delivery") };
    let Document::Frame(value) = document.as_ref() else { panic!("frame") };
    assert_eq!(
        value.projects.get(1).expect("project 1 follows deployment role").pool.expect("actual pool").numbers.budget,
        177
    );
    watch_outputs(&mut core, &limits, Event::Delivered { watcher });
    watch_outputs(&mut core, &limits, Event::Period { project: 2, period: 2, budget: 1000, next: None });
    let next = finish_watch_outputs(&mut core, &limits);
    let Request::Held(value) = &next[0] else { panic!("changed frame") };
    let Held::Delivery { document, .. } = value.as_ref() else { panic!("frame delivery") };
    let Document::Frame(value) = document.as_ref() else { panic!("frame") };
    assert_eq!(value.projects.get(1).expect("project 1").pool, None, "the new creation period has no pool yet");
    assert!(
        core.tasks.funding(tasks::Funder::Pool { project: 1, person: 1, period }).is_some(),
        "the prior pool remains an actual owner ledger"
    );
}
