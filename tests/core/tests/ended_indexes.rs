//! The ended indexes commit with the task they name (protocol/store.md, 7.4).
use jig_core_world::stories::reads::world;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

const REQUESTER: u64 = 1;

fn ended_rows(world: &jig_core_world::effects::World) -> Vec<tasks::Key> {
    world
        .store
        .rows
        .keys()
        .filter_map(|key| match key {
            root::Key::Core(core::Key::Tasks(key @ tasks::Key::EndedByParty { .. })) => Some(*key),
            root::Key::Core(_) | root::Key::Connector { .. } => None,
        })
        .collect()
}

#[test]
fn an_ended_chat_is_indexed_by_its_requesting_party_and_label() {
    let mut world = world(820);
    let task = world.assigned.as_ref().expect("chat").task;
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        0,
    );
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))]
    else {
        panic!("ended chat")
    };
    let ended_at = tasks::Newest::of(row.ended_at.expect("ended").as_nanos());
    assert_eq!(ended_rows(&world), [tasks::Key::EndedByParty { party: REQUESTER, label: 0, ended_at, task }]);
}

#[test]
fn an_ended_delegate_of_a_task_saves_no_party_row() {
    let mut world = world(821);
    world.delegate();
    world.observed(13);
    let delegate = world.procedure.expect("ended delegate");
    assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(delegate)))));
    assert_eq!(ended_rows(&world), []);
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        0,
    );
    let root_task = world.assigned.as_ref().expect("root task").task;
    assert_eq!(ended_rows(&world).len(), 1);
    assert!(
        ended_rows(&world).iter().any(|key| matches!(key, tasks::Key::EndedByParty { task, .. } if *task == root_task))
    );
}

#[test]
fn an_ended_goal_is_indexed_by_its_project_and_label() {
    let mut world = world(824);
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        0,
    );
    jig_core_world::stories::reads::request(&mut world, 100, jig_core_world::stories::offers::goal(100));
    let task = world.assigned.as_ref().expect("goal").task;
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        0,
    );
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))]
    else {
        panic!("ended goal")
    };
    let ended_at = tasks::Newest::of(row.ended_at.expect("ended").as_nanos());
    let keys: Vec<_> = world
        .store
        .rows
        .keys()
        .filter_map(|key| match key {
            root::Key::Core(core::Key::Tasks(key @ tasks::Key::EndedByProject { .. })) => Some(*key),
            root::Key::Core(_) | root::Key::Connector { .. } => None,
        })
        .collect();
    assert_eq!(keys, [tasks::Key::EndedByProject { project: 1, label: 0, ended_at, task }]);
    assert!(ended_rows(&world).contains(&tasks::Key::EndedByParty { party: REQUESTER, label: 0, ended_at, task }));
}

#[test]
fn an_ended_chat_saves_no_project_row() {
    let mut world = world(825);
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        0,
    );
    assert_eq!(
        world
            .store
            .rows
            .keys()
            .filter(|key| matches!(key, root::Key::Core(core::Key::Tasks(tasks::Key::EndedByProject { .. }))))
            .count(),
        0
    );
}

fn query_page(
    world: &mut jig_core_world::effects::World,
    query: core::documents::TaskQuery,
    from: Option<core::reads::Cursor>,
) -> core::reads::Page {
    jig_core_world::reads::ask(world, 900, core::reads::Read::Tasks { query, from });
    let core::reads::Answer::Page(page) = jig_core_world::reads::latest(world) else { panic!("query page") };
    *page.clone()
}

fn query_ended(
    world: &mut jig_core_world::effects::World,
    query: core::documents::TaskQuery,
) -> Vec<core::documents::TaskLine> {
    let mut from = Some(core::reads::Cursor::Store { scope: core::reads::Scope::Tasks { query }, after: None });
    let mut lines = Vec::new();
    for _ in 0..8 {
        let page = query_page(world, query, from.take());
        for row in page.rows.into_boxed() {
            let core::reads::PageRow::Line(line) = row else { panic!("task line") };
            lines.push(line);
        }
        from = page.next;
        if from.is_none() {
            return lines;
        }
    }
    panic!("bounded ended group");
}

fn chats(world: &mut jig_core_world::effects::World) -> Vec<u64> {
    let mut numbers = Vec::new();
    for index in 0..3 {
        if index != 0 {
            jig_core_world::stories::reads::request(
                world,
                80 + index,
                jig_domain_people::Ask::StartChat { project: 1, words: b"indexed chat".as_slice().into() },
            );
        }
        numbers.push(world.assigned.as_ref().expect("chat").task);
        world.turn(1, 1, None, b"unread indexed reply");
        world.advance(skein_lib::Duration::from_secs(1));
        jig_core_world::fixtures::finish_chat(world, 1, b"indexed result");
    }
    numbers.reverse();
    numbers
}

#[test]
fn a_partys_ended_chats_page_newest_first_after_a_restart() {
    let mut world = world(830);
    let expected = chats(&mut world);
    world.restart(830, false);
    let lines = query_ended(&mut world, jig_core_world::fixtures::OWN_TASKS);
    assert_eq!(lines.iter().map(|line| line.chip.number).collect::<Vec<_>>(), expected);
    for line in lines {
        assert_eq!(
            line.conversation,
            Some(core::documents::ConversationRead { seen: 0, unread: core::documents::Count::Known(1) })
        );
    }
}

#[test]
fn a_projects_ended_goals_page_from_the_project_index() {
    let mut world = world(831);
    jig_core_world::fixtures::finish_chat(&mut world, 0, b"chat complete");
    jig_core_world::stories::reads::request(&mut world, 100, jig_core_world::stories::offers::goal(100));
    let goal = world.assigned.as_ref().expect("goal").task;
    jig_core_world::fixtures::finish_chat(&mut world, 0, b"goal complete");
    let query = core::documents::TaskQuery { project: Some(1), requester: None, ..jig_core_world::fixtures::OWN_TASKS };
    assert_eq!(query_ended(&mut world, query).iter().map(|line| line.chip.number).collect::<Vec<_>>(), [goal]);
}

#[test]
fn a_parents_ended_delegates_page_by_task() {
    let mut world = world(832);
    let parent = world.assigned.as_ref().expect("parent").task;
    world.delegate();
    world.observed(13);
    let child = world.procedure.expect("ended delegate");
    let query = core::documents::TaskQuery {
        project: Some(1),
        requester: None,
        parent: Some(parent),
        ..jig_core_world::fixtures::OWN_TASKS
    };
    assert_eq!(query_ended(&mut world, query).iter().map(|line| line.chip.number).collect::<Vec<_>>(), [child]);
}

#[test]
fn a_filter_past_the_prefix_comes_back_cut_and_goes_on() {
    let mut world = world(833);
    chats(&mut world);
    let query = core::documents::TaskQuery { tracked: Some(true), ..jig_core_world::fixtures::OWN_TASKS };
    let page = query_page(
        &mut world,
        query,
        Some(core::reads::Cursor::Store { scope: core::reads::Scope::Tasks { query }, after: None }),
    );
    assert!(page.rows.is_empty());
    assert!(page.cut);
    let Some(core::reads::Cursor::Store { scope, after: Some(position) }) = page.next else {
        panic!("examined continuation")
    };
    assert_eq!(scope, core::reads::Scope::Tasks { query });
    let core::reads::Position::Owner(core::Key::Tasks(tasks::Key::EndedByParty { .. })) = position else {
        panic!("party index")
    };
    let last = query_page(&mut world, query, Some(core::reads::Cursor::Store { scope, after: Some(position) }));
    assert!(last.rows.is_empty());
    assert_eq!(last.next, None);
    assert!(!last.cut);
}

#[test]
fn a_query_naming_no_index_says_it_has_no_ended_part() {
    let mut world = world(834);
    let query = core::documents::TaskQuery {
        phase: Some(core::documents::PhaseClass::Working),
        ..jig_core_world::fixtures::OWN_TASKS
    };
    let page = query_page(&mut world, query, None);
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.next, None);
    let unindexed = core::documents::TaskQuery { requester: None, phase: None, ..query };
    assert_eq!(core::reads::ended::ended_part(&unindexed), core::reads::ended::EndedPart::None);
}
