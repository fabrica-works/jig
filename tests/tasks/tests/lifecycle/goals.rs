use jig_domain_tasks::{Control, End, Endings, Event, Key, Newest, Party, Stored, TaskResult};
use jig_tasks_world::{LIMITS, World, task};

fn finish(world: &mut World, number: u64, result: TaskResult) {
    world.claim(number, number);
    world.terminal(number, End::Finished { result, cancel_delegates: false });
    world.settle(number);
}
fn subtree() -> World {
    let mut world = World::new(180, LIMITS);
    let mut goal = task(1, &[]);
    goal.tracked = Some(1);
    world.make(Party::Person(1), vec![goal]);
    world.make(Party::Task(1), vec![task(2, &[]), task(4, &[])]);
    world.make(Party::Task(2), vec![task(3, &[])]);
    finish(&mut world, 3, TaskResult::Report { words: Box::new([]) });
    finish(&mut world, 2, TaskResult::Failure { reason: Box::new([1]) });
    let reply_to = world.to();
    world.send(Event::Control {
        reply_to,
        by: Party::Person(1),
        task: 4,
        control: Control::Cancel { reason: Box::new([1]), grace: None, message: None },
    });
    world.settle(4);
    world
}

#[test]
fn a_goal_ending_saves_its_subtrees_phase_counts() {
    let mut world = subtree();
    finish(&mut world, 1, TaskResult::Report { words: Box::new([]) });
    assert_eq!(
        world.records.get(&Key::Goal { project: 1, task: Newest::of(1) }),
        Some(&Stored::Goal(jig_domain_tasks::GoalRow {
            project: 1,
            task: 1,
            endings: Endings { done: 2, failed: 1, cancelled: 1 }
        }))
    );
}

#[test]
fn a_delegates_ending_adds_to_its_requesters_ended_below() {
    let mut world = subtree();
    assert_eq!(world.record(1).ended_below, Endings { done: 1, failed: 1, cancelled: 1 });
    world.restart();
    assert_eq!(world.record(1).ended_below, Endings { done: 1, failed: 1, cancelled: 1 });
}
