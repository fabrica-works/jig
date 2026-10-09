//! The project task collection has its own live subject and filtered watch.
use super::*;

const ALL: TaskFilter = TaskFilter { phase: None, executor: None, goal: None, asked_by_me: false };

fn at(core: &mut Core, limits: &Limits, wall: u64, event: Event) -> Box<[Request]> {
    let Requests::Out(mut out) =
        step(core, &Env { now: Time::ZERO, wall: Wall::from_nanos(wall), limits: *limits }, event);
    let mut requests = List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        requests.push(request).expect("bounded outputs");
    }
    requests.into_boxed()
}
fn rows(core: &Core, limits: &Limits, filter: TaskFilter, window: Window) -> Tasks {
    let Document::Tasks(value) =
        make(core, limits, 1, &Subject::Tasks { project: 1, filter }, &window).expect("task collection")
    else {
        panic!("tasks document")
    };
    value
}
fn fixture() -> (Core, Limits) {
    let (mut core, mut limits) = tree_core();
    let (auth, auth_limits) = watch_core();
    core.people = auth.people;
    limits.people = auth_limits.people;
    for (number, when, creator) in [(1, 10, 1), (3, 20, 2)] {
        let batch: Box<[tasks::New]> = if number == 1 {
            let first = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, Some(1));
            let mut second = new_row(&core, 2, tasks::Funder::Period { project: 1, period: 1 }, None);
            second.dependencies = Box::new([1]);
            Box::new([first, second])
        } else {
            Box::new([new_row(&core, number, tasks::Funder::Period { project: 1, period: 1 }, None)])
        };
        let mut out = Queue::with_capacity(tasks::max_out(&limits.tasks));
        tasks::step(
            &mut core.tasks,
            &Env { now: Time::ZERO, wall: Wall::from_nanos(when), limits: limits.tasks },
            tasks::Event::Make {
                reply_to: ReplyTo::new(Token::new(number)),
                creator: tasks::Party::Person(creator),
                batch,
            },
            &mut out,
        );
        let mut observed = List::with_capacity(out.len());
        while let Some(request) = out.pop() {
            if let tasks::Request::Save { record: tasks::Stored::Live(row) } = &request {
                let mut projections = Queue::with_capacity(room_max(&limits).expect("projection room").writes);
                projections::save(&mut core, &limits, row, &mut projections);
            }
            observed.push(request).expect("observed outputs");
        }
        assert!(core.tasks.task(number).is_some(), "task {number}: {observed:?}");
    }
    (core, limits)
}

#[test]
fn project_tasks_filter_owner_facts_and_page_in_last_change_order() {
    let (core, limits) = fixture();
    let first = rows(&core, &limits, ALL, window());
    assert_eq!(first.rows.len(), 1);
    assert_eq!(first.rows.get(0).expect("first").chip.number, 3);
    assert!(first.more);
    let next = rows(&core, &limits, ALL, Window { rows: 2, after: Some(Cursor::Number { number: 3 }) });
    assert_eq!(next.rows.get(0).expect("second").chip.number, 2);
    assert_eq!(next.rows.get(1).expect("third").chip.number, 1);
    assert!(!next.more);
    assert_eq!(next.rows.get(1).expect("goal").goal, Some(1));
    let mine = rows(&core, &limits, TaskFilter { asked_by_me: true, ..ALL }, Window { rows: 2, after: None });
    assert_eq!(mine.rows.len(), 2);
    assert!(mine.rows.get(0).expect("mine").asked_by_me);
    let goal = rows(&core, &limits, TaskFilter { goal: Some(1), ..ALL }, window());
    assert_eq!(goal.rows.get(0).expect("goal").chip.number, 1);
    let waiting = rows(&core, &limits, TaskFilter { phase: Some(TaskPhase::Waiting), ..ALL }, window());
    assert_eq!(waiting.rows.get(0).expect("waiting").chip.number, 2);
    assert!(!waiting.more);
    let none = rows(
        &core,
        &limits,
        TaskFilter { executor: Some(tasks::Executor::Procedure { connector: 1, code: 1 }), ..ALL },
        window(),
    );
    assert!(none.rows.is_empty());
    let agents = rows(
        &core,
        &limits,
        TaskFilter { executor: Some(tasks::Executor::Agent { charter: 1 }), ..ALL },
        Window { rows: 2, after: None },
    );
    assert_eq!(agents.rows.len(), 2);
    assert!(agents.more);
    assert_eq!(core.tasks.task(1).expect("unchanged by reads").changed, Wall::from_nanos(10));
    assert_eq!(
        make(&core, &limits, 1, &Subject::Tasks { project: 2, filter: ALL }, &window()),
        Err(Missing::NotAllowed)
    );
}

#[test]
fn filtered_task_watches_coalesce_enter_leave_and_ending_then_revoke_visibility() {
    let (mut core, limits) = fixture();
    let watcher = Token::new(30);
    let filter = TaskFilter { phase: Some(TaskPhase::Held), ..ALL };
    let initial = watch_outputs(
        &mut core,
        &limits,
        Event::Watch {
            watcher,
            sign_in: 10,
            subject: Subject::Tasks { project: 1, filter },
            window: Window { rows: 2, after: None },
        },
    );
    let Request::Held(held) = &initial[0] else { panic!("held document") };
    let Held::Delivery { document, .. } = held.as_ref() else { panic!("delivery") };
    let Document::Tasks(value) = document.as_ref() else { panic!("tasks") };
    assert!(value.rows.is_empty());
    for (number, wall) in [(1, 30), (3, 40)] {
        at(&mut core, &limits, wall, Event::Tasks(tasks::Event::Hold { task: number, why: tasks::Hold::Budget }));
        assert_eq!(finish_watch_outputs(&mut core, &limits).len(), 1, "in-flight document coalesces");
    }
    let current = watch_outputs(&mut core, &limits, Event::Delivered { watcher });
    let Request::Held(held) = &current[0] else { panic!("held current") };
    let Held::Delivery { document, .. } = held.as_ref() else { panic!("current delivery") };
    let Document::Tasks(value) = document.as_ref() else { panic!("tasks") };
    assert_eq!(value.rows.len(), 2);
    assert_eq!(value.rows.get(0).expect("newest").chip.number, 3);
    watch_outputs(&mut core, &limits, Event::Delivered { watcher });
    at(
        &mut core,
        &limits,
        50,
        Event::Tasks(tasks::Event::Control {
            reply_to: ReplyTo::new(Token::new(60)),
            by: tasks::Party::Person(1),
            task: 3,
            control: tasks::Control::Cancel { reason: Box::new([]) },
        }),
    );
    let changed = finish_watch_outputs(&mut core, &limits);
    let Request::Held(held) = &changed[0] else { panic!("changed collection") };
    let Held::Delivery { document, .. } = held.as_ref() else { panic!("delivery") };
    let Document::Tasks(value) = document.as_ref() else { panic!("tasks") };
    assert_eq!(value.rows.len(), 1, "closing task leaves held filter");
    watch_outputs(&mut core, &limits, Event::Delivered { watcher });
    at(&mut core, &limits, 60, Event::Tasks(tasks::Event::EffectsSettled { task: 3 }));
    at(&mut core, &limits, 70, Event::Tasks(tasks::Event::Settled { task: 3 }));
    assert!(core.tasks.task(3).is_none());
    let removal = finish_watch_outputs(&mut core, &limits);
    assert!(removal.len() > 1, "ending refreshes the project collection even if the row no longer matches");
    watch_outputs(&mut core, &limits, Event::Delivered { watcher });
    watch_outputs(&mut core, &limits, Event::People(people::Event::Roles { project: 1, holdings: Box::new([]) }));
    let ended = finish_watch_outputs(&mut core, &limits);
    let Request::Held(held) = &ended[0] else { panic!("visibility end") };
    let Held::WatchEnded { why, .. } = held.as_ref() else { panic!("watch ended") };
    assert_eq!(*why, views::End::NotAllowed);
}
