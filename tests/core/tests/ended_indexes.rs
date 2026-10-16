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
