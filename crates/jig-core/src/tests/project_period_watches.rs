//! A global configured period refreshes each project's existing watch.
//! `client/domain/contract.md`, 7.2; `domain/engine.md`, 11.

use super::*;

fn period_core(configured: u64) -> (Core, Limits) {
    let (auth, mut limits) = watch_core();
    limits.tasks.funders = 8;
    limits.views.watchers = 4;
    limits.views.watchers_per_party = 4;
    limits.documents.lines = 4;
    let mut config = config(61);
    config.settings.period = configured;
    let policy = config.authority.policy(1).unwrap().clone();
    config.authority = authority::Domain::new(
        config.authority.rules().clone(),
        authority::Limits { projects: 2, ..*config.authority.limits() },
    )
    .unwrap();
    let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
    for project in [1, 2] {
        authority::step(
            &mut config.authority,
            authority::Event::Policy { project, policy: policy.clone() },
            &mut facts,
        );
        assert_eq!(facts.pop(), Some(authority::PolicyFact::Added { project }));
    }
    config.projects = List::with_capacity(2);
    config.projects.push(1).unwrap();
    config.projects.push(2).unwrap();
    let mut core = Core::new(config, &limits);
    core.people = auth.people;
    send_task(&mut core, &limits, tasks::Event::Restored);
    let opened = watch_outputs(
        &mut core,
        &limits,
        Event::Tasks(tasks::Event::OpenPeriod {
            reply_to: ReplyTo::new(Token::new(1)),
            project: 1,
            period: 1,
            budget: 1000,
        }),
    );
    assert!(opened.iter().any(|request| matches!(request, Request::Write(_))));
    let carved = watch_outputs(
        &mut core,
        &limits,
        Event::Tasks(tasks::Event::CarvePool {
            reply_to: ReplyTo::new(Token::new(2)),
            project: 1,
            person: 1,
            period: 1,
            budget: 177,
        }),
    );
    assert!(carved.iter().any(|request| matches!(request, Request::Write(_))));
    assert_eq!(finish_watch_outputs(&mut core, &limits).len(), 1);
    (core, limits)
}

fn project_delivery(out: &[Request], watcher: Token) -> Option<Project> {
    let mut delivered = None;
    for request in out {
        if let Request::Held(held) = request {
            if let Held::Delivery { watcher: actual, document } = held.as_ref() {
                if *actual == watcher {
                    let Document::Project(project) = document.as_ref() else { panic!("watched project") };
                    assert!(delivered.is_none(), "one whole delivery per watcher");
                    delivered = Some(project.clone());
                }
            }
        }
    }
    delivered
}

fn open_project(core: &mut Core, limits: &Limits, watcher: Token) -> Project {
    let initial = watch_outputs(
        core,
        limits,
        Event::Watch { watcher, sign_in: 10, subject: Subject::Project { project: 1 }, window: window() },
    );
    project_delivery(&initial, watcher).expect("authenticated first project delivery")
}

fn period_outputs(core: &mut Core, limits: &Limits, project: u32, period: u64, budget: u64) -> Box<[Request]> {
    let first = watch_outputs(core, limits, Event::Period { project, period, budget, next: None });
    let finished = finish_watch_outputs(core, limits);
    let mut out = List::with_capacity(u32::try_from(first.len() + finished.len()).unwrap());
    for request in first.into_iter().chain(finished) {
        out.push(request).unwrap();
    }
    out.into_boxed()
}

fn assert_observed_pool(project: &Project, before: &Project) {
    assert_eq!(project.number, 1);
    assert_eq!(project.current_period, before.current_period);
    assert_eq!(project.pools, before.pools);
    assert_eq!(project.configured_pools.allocated(), 4);
    assert!(!project.configured_pools_cut);
    assert!(!project.cut);
    assert_eq!(project.pools.len(), 1);
    let pool = project.pools.get(0).unwrap();
    assert_eq!(pool.source, tasks::Funder::Pool { project: 1, person: 1, period: 1 });
    assert_eq!(pool.parent, Some(tasks::Funder::Period { project: 1, period: 1 }));
    assert_eq!(pool.numbers, tasks::Numbers { budget: 177, spent: 0, spent_below: 0, reserved: 0 });
    assert_eq!(pool.left, 177);
    assert!(!pool.closed);
    assert_eq!(pool.began, None);
    assert_eq!(pool.next, None);
}

#[test]
fn a_period_in_another_project_refreshes_the_watched_configured_target_after_durability() {
    let (mut core, limits) = period_core(1);
    let watcher = Token::new(30);
    let before = open_project(&mut core, &limits, watcher);
    assert_eq!(before.configured_period, 1);
    assert_eq!(before.configured_pools, before.pools);
    assert!(project_delivery(&watch_outputs(&mut core, &limits, Event::Delivered { watcher }), watcher).is_none());
    let out = period_outputs(&mut core, &limits, 2, 2, 1000);
    let mut funding_writes = 0;
    for request in &out {
        if let Request::Write(Write::Save(Record::Tasks(record))) = request {
            let tasks::Stored::Ledger(row) = record else { panic!("no task in the period fixture") };
            let tasks::Funder::Period { project, period } = row.funder else { panic!("no local pool mutation") };
            assert_eq!(project, 2, "the watched project emitted no funding write");
            assert_eq!(period, 2);
            funding_writes += 1;
        }
    }
    assert!(funding_writes > 0);
    let after = project_delivery(&out, watcher).expect("cross-project change is held");
    assert_eq!(after.configured_period, 2);
    assert!(after.configured_pools.is_empty());
    assert_observed_pool(&after, &before);
    let room = room_max(&limits).unwrap();
    let mut journal = skein_lib::Journal::<Write, Box<Held>>::new(&skein_lib::JournalLimits {
        commits: 1,
        writes: room.writes,
        held: room.held,
        now: 1,
        release: room.held,
    });
    let mut decision = journal.decision(&skein_lib::JournalRoom { writes: room.writes, held: room.held }).unwrap();
    let mut writes = 0;
    for request in out {
        match request {
            Request::Write(write) => {
                decision.write(write).unwrap();
                writes += 1;
            }
            Request::Held(held) => decision.hold(held).unwrap(),
            Request::Now(_) | Request::Decided => {}
            Request::Ask { .. } => panic!("no connector or recurring work in this fixture"),
        }
    }
    assert!(writes > 0, "the new project period is a durable decision");
    journal.accept(decision);
    let commit = journal.commit().expect("the period's own commit");
    let mut released = Queue::with_capacity(room.held);
    assert_eq!(journal.release(&mut released), skein_lib::Released::None);
    assert!(released.is_empty());
    journal.committed(commit.number);
    assert_eq!(journal.release(&mut released), skein_lib::Released::Some);
    let mut observed = 0;
    while let Some(held) = released.pop() {
        if let Held::Delivery { watcher: actual, document } = held.as_ref() {
            assert_eq!(*actual, watcher);
            assert_eq!(document.as_ref(), &Document::Project(after.clone()));
            observed += 1;
        }
    }
    assert_eq!(observed, 1);
    assert!(project_delivery(&watch_outputs(&mut core, &limits, Event::Delivered { watcher }), watcher).is_none());
}

#[test]
fn changed_configured_periods_coalesce_to_the_latest_whole_project_behind_an_inflight_delivery() {
    let (mut core, limits) = period_core(1);
    let watcher = Token::new(30);
    let first = open_project(&mut core, &limits, watcher);
    for period in [2, 3, 4] {
        let out = period_outputs(&mut core, &limits, 2, period, 1000);
        assert!(project_delivery(&out, watcher).is_none(), "the initial delivery is still in flight");
    }
    assert_eq!(first.configured_period, 1, "the already made delivery stays whole");
    let out = watch_outputs(&mut core, &limits, Event::Delivered { watcher });
    let current = project_delivery(&out, watcher).expect("one coalesced delivery");
    assert_eq!(current.configured_period, 4);
    assert!(current.configured_pools.is_empty());
    assert_observed_pool(&current, &first);
    assert!(project_delivery(&watch_outputs(&mut core, &limits, Event::Delivered { watcher }), watcher).is_none());
    assert!(project_delivery(&finish_watch_outputs(&mut core, &limits), watcher).is_none());
}

#[test]
fn an_equal_period_in_another_project_keeps_the_watched_configured_target_unchanged() {
    let (mut core, limits) = period_core(1);
    let watcher = Token::new(30);
    let before = open_project(&mut core, &limits, watcher);
    watch_outputs(&mut core, &limits, Event::Delivered { watcher });
    let out = period_outputs(&mut core, &limits, 2, 1, 700);
    assert!(project_delivery(&out, watcher).is_none());
    let Document::Project(after) = make(&core, &limits, 1, &Subject::Project { project: 1 }, &window()).unwrap() else {
        panic!("project read");
    };
    assert_eq!(after, before);
}

#[test]
fn zero_older_over_budget_and_unknown_project_periods_do_not_refresh_a_project_watch() {
    for (project, period, budget) in [(2, 0, 1000), (2, 1, 1000), (2, 3, 1001), (9, 3, 1000)] {
        let (mut core, limits) = period_core(2);
        let watcher = Token::new(30);
        let before = open_project(&mut core, &limits, watcher);
        assert_eq!(before.configured_period, 2);
        watch_outputs(&mut core, &limits, Event::Delivered { watcher });
        assert!(project_delivery(&period_outputs(&mut core, &limits, project, period, budget), watcher).is_none());
        let Document::Project(after) = make(&core, &limits, 1, &Subject::Project { project: 1 }, &window()).unwrap()
        else {
            panic!("unchanged project read");
        };
        assert_eq!(after, before);
    }
}

#[test]
fn the_first_positive_configured_period_refreshes_a_watch_that_started_at_zero() {
    let (mut core, limits) = period_core(0);
    let watcher = Token::new(30);
    let before = open_project(&mut core, &limits, watcher);
    assert_eq!(before.configured_period, 0);
    assert!(before.configured_pools.is_empty());
    assert_eq!(before.current_period.as_ref().unwrap().source, tasks::Funder::Period { project: 1, period: 1 });
    watch_outputs(&mut core, &limits, Event::Delivered { watcher });
    let after =
        project_delivery(&period_outputs(&mut core, &limits, 2, 1, 1000), watcher).expect("zero target changed");
    assert_eq!(after.configured_period, 1);
    assert_eq!(after.configured_pools, before.pools);
    assert_observed_pool(&after, &before);
}
