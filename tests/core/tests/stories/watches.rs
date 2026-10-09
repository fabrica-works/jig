use jig_core::{
    self as core,
    documents::{Count, Document, PhaseLine, Subject},
};
use jig_core_tasks as tasks;
use jig_core_views as views;
use jig_core_world::{
    effects::Cut,
    watches::{latest, open, world},
};
use jig_test_domain as root;
use skein_lib::Token;

#[test]
fn a_party_watching_a_chat_sees_each_phase_after_its_commit() {
    let mut world = world(701, true);
    let Document::Task(task) = latest(&world, 2) else { panic!("task page") };
    assert!(matches!(task.chip.phase, PhaseLine::Active(tasks::Active::Running { .. })));
    world.store_delay = 4;
    world.end(tasks::End::Parked, 0);
    let Document::Task(task) = latest(&world, 2) else { panic!("idle page") };
    assert_eq!(task.chip.phase, PhaseLine::Active(tasks::Active::Idle));
    world.say(70);
    world.retry_due();
    let Document::Task(task) = latest(&world, 2) else { panic!("running page") };
    assert!(matches!(task.chip.phase, PhaseLine::Active(tasks::Active::Running { .. })));
    world.watches.as_ref().expect("watches").finish();
}

#[test]
fn a_slow_watcher_of_a_tree_sees_fewer_deliveries_and_the_last_state() {
    let mut world = world(702, true);
    world.watches.as_mut().expect("watches").manual(3);
    world.turn(1, 1, None, b"first");
    // This first replacement reaches the party, which leaves it in flight.
    for turn in 2..=4 {
        world.turn(turn, u64::from(turn), None, b"later");
    }
    world.end(tasks::End::Parked, 4);
    world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(3) }));
    let watches = world.watches.as_ref().expect("watches");
    let slow = watches.deliveries.iter().filter(|(watcher, _, _)| *watcher == 3).count();
    let fast = watches.deliveries.iter().filter(|(watcher, _, _)| *watcher == 2).count();
    assert!(slow < fast, "one replacement coalesces intervening commits: slow={slow} fast={fast}");
    let Document::Tree(tree) = latest(&world, 3) else { panic!("tree page") };
    assert_eq!(tree.root.phase, PhaseLine::Active(tasks::Active::Idle));
    watches.finish();
}

#[test]
fn a_delivery_never_shows_what_a_crash_took_back() {
    let mut world = world(703, true);
    let task = world.assigned.as_ref().expect("assignment").task;
    open(&mut world, 5, Subject::Run { task });
    let before = world.watches.as_ref().expect("watches").deliveries.len();
    let next = world.store.applied + 1;
    world.cut = Some(Cut::Submitted(next));
    world.turn(1, 1, None, b"uncommitted turn");
    assert_eq!(world.reached_cut, Some(Cut::Submitted(next)));
    assert_eq!(world.watches.as_ref().expect("watches").deliveries.len(), before);
    world.restart(703, false);
    open(&mut world, 6, Subject::Run { task });
    let Document::Run(run) = latest(&world, 6) else { panic!("reopened run") };
    assert_eq!(run.committed_turn, 0);
    assert!(world.store.rows.values().all(|row| !matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) if turn.task == task && turn.turn == 1)));
    world.watches.as_ref().expect("watches").finish();
}

#[test]
fn a_run_s_text_streams_by_turn_and_is_replaced_by_the_committed_turn() {
    let mut world = world(704, true);
    let assignment = world.assigned.as_ref().expect("assignment");
    let (task, attempt) = (assignment.task, assignment.attempt);
    open(&mut world, 5, Subject::Run { task });
    let committed = world.store.applied;
    for (offset, text) in [(0, b"hello ".as_slice()), (6, b"world".as_slice())] {
        world.send(root::Event::Text { task, attempt, turn: 1, offset, text: text.into() });
        world.send(root::Event::Core(core::Event::StreamDelivered { watcher: Token::new(5) }));
    }
    assert_eq!(world.store.applied, committed, "provisional text writes nothing");
    let streams = &world.watches.as_ref().expect("watches").streams;
    assert_eq!(streams.len(), 2);
    assert_eq!((streams[0].turn, streams[0].from, streams[0].to), (views::TurnName { attempt, turn: 1 }, 0, 6));
    assert_eq!((streams[1].from, streams[1].to, streams[1].text.as_ref()), (6, 11, b"world".as_slice()));
    world.store_delay = 3;
    world.turn(1, 1, None, b"whole committed turn");
    let Document::Run(run) = latest(&world, 5) else { panic!("committed run") };
    assert_eq!(run.committed_turn, 1);
    assert!(run.writing.is_none());
    world.watches.as_ref().expect("watches").finish();
}

#[test]
fn a_watch_on_a_task_that_ends_ends_saying_so() {
    let mut world = world(705, true);
    world.store_delay = 3;
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        0,
    );
    let watches = world.watches.as_ref().expect("watches");
    assert!(watches.endings.contains(&(2, views::End::Gone)));
    assert!(watches.endings.contains(&(3, views::End::Gone)));
    let Document::Frame(frame) = latest(&world, 1) else { panic!("updated frame") };
    assert_eq!(frame.updates, Count::Known(1));
    let Document::Inbox(inbox) = latest(&world, 4) else { panic!("updated inbox") };
    assert_eq!(inbox.cached_updates, Count::Known(1));
    assert_eq!(inbox.updates.len(), 1);
    watches.finish();
}
