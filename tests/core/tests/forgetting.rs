//! A task's retention class decides what its reading erases (people.md, 6).
const CHAT_MARK: u8 = 63;
use jig_core_world::{
    effects::World,
    stories::reads::{forgettable_world, request, world},
};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

const TURN: u32 = 1;
const PARTY: u64 = 1;
const MARK_TURNS: u8 = 61;
const MARK_RESULTS: u8 = 62;

fn ended(task: u64) -> root::Key {
    root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))
}

fn finish(world: &mut World) -> (u64, u64) {
    let task = world.assigned.as_ref().expect("conversation").task;
    world.turn(TURN, 1, None, b"reply");
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        1,
    );
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) = &world.store.rows[&ended(task)] else {
        panic!("ended conversation")
    };
    (task, row.result_position)
}

#[test]
fn a_conversation_forgotten_once_read_leaves_no_row_once_read() {
    let mut world = forgettable_world(830);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: people::Seen::Chat { task, through: 1 } });
    assert!(world.store.rows.contains_key(&ended(task)));
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(!world.store.rows.contains_key(&ended(task)));
    for key in world.store.rows.keys() {
        if let root::Key::Core(core::Key::Tasks(
            tasks::Key::EndedByParty { task: number, .. } | tasks::Key::EndedByProject { task: number, .. },
        )) = key
        {
            assert_ne!(*number, task);
        }
    }
}

#[test]
fn a_kept_chat_stays_after_its_party_read_it() {
    let mut world = world(831);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: people::Seen::Chat { task, through: 1 } });
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(world.store.rows.contains_key(&ended(task)));
    assert!(world.store.rows.keys().any(|key| matches!(key, root::Key::Core(core::Key::Tasks(tasks::Key::EndedByParty { task: number, .. })) if *number == task)));
}

#[test]
fn a_forgettable_conversation_read_in_the_other_order_is_forgotten() {
    let mut world = forgettable_world(832);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(world.store.rows.contains_key(&ended(task)));
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: people::Seen::Chat { task, through: 1 } });
    assert!(!world.store.rows.contains_key(&ended(task)));
}

#[test]
fn a_results_mark_forgets_in_the_commit_that_moves_its_position() {
    let mut world = forgettable_world(833);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: people::Seen::Chat { task, through: 1 } });
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    let erases: Vec<_> =
        world.commits.iter().filter(|commit| commit.contains(&root::Write::Erase(ended(task)))).collect();
    assert_eq!(erases.len(), 1);
    assert!(erases[0].contains(&root::Write::Save(root::Record::Core(core::Record::People(
        people::Stored::ReadResults { party: PARTY, through },
    )))));
}

#[test]
fn a_conversation_read_before_a_restart_is_forgotten_by_its_results_mark() {
    let mut world = forgettable_world(837);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: people::Seen::Chat { task, through: 1 } });
    world.restart(837, false);
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(!world.store.rows.contains_key(&ended(task)));
    assert!(
        !world
            .store
            .rows
            .contains_key(&root::Key::Core(core::Key::People(people::Key::EndedRead { task, party: PARTY })))
    );
}
#[test]
fn a_results_mark_while_a_chat_mark_loads_is_busy() {
    let mut world = forgettable_world(838);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: people::Seen::Chat { task, through: 1 } });
    world.store_delay = 3;
    let chat = jig_core_world::stories::reads::request_event(
        &world,
        CHAT_MARK,
        people::Ask::MarkSeen { seen: people::Seen::Chat { task, through: 1 } },
    );
    let results = jig_core_world::stories::reads::request_event(
        &world,
        MARK_RESULTS,
        people::Ask::MarkSeen { seen: people::Seen::Results { through } },
    );
    world.send_all(vec![chat, results]);
    assert!(
        world.people_answers.contains(&people::Reply::Again(people::Again::Busy)),
        "answers: {:?}; rejected inputs: {:?}",
        world.people_answers,
        world.busy_events
    );
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(!world.store.rows.contains_key(&ended(task)));
    assert!(
        !world
            .store
            .rows
            .contains_key(&root::Key::Core(core::Key::People(people::Key::EndedRead { task, party: PARTY })))
    );
}
