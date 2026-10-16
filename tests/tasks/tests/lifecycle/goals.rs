use jig_domain_tasks::{Amendment, Control, End, Endings, Event, Key, Newest, Party, Stored, TaskResult, Tracking};
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

const FIRST_GOAL: u64 = 1;
const SECOND_GOAL: u64 = 2;
const THIRD_GOAL: u64 = 3;
const REQUESTER: u64 = 1;
const TRACK_MESSAGE: u64 = 1;

#[test]
fn goals_per_project_count_only_live_tracked_tasks_after_a_restart() {
    let mut world = World::new(181, LIMITS);
    let mut first = task(FIRST_GOAL, &[]);
    first.tracked = Some(1);
    let mut second = task(SECOND_GOAL, &[]);
    second.tracked = Some(2);
    world.make(Party::Person(REQUESTER), vec![first, second, task(THIRD_GOAL, &[])]);
    let change = Amendment {
        tracked: Some(Tracking::Tracked { priority: 3 }),
        addressed: None,
        instructions: None,
        release: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        reason: Box::new([]),
    };
    let call = world.to().into_token().raw();
    world.send(Event::Amend {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(call)),
        by: Party::Person(REQUESTER),
        task: THIRD_GOAL,
        message: TRACK_MESSAGE,
        stop_run: false,
        amendment: change.clone(),
    });
    assert_eq!(
        world.replies.get(&call),
        Some(&jig_tasks_world::Reply::Refused(jig_domain_tasks::Problem::new(
            Some(THIRD_GOAL),
            jig_domain_tasks::Refusal::Goals
        )))
    );
    finish(&mut world, FIRST_GOAL, TaskResult::Report { words: Box::new([]) });
    world.restart();
    let call = world.to().into_token().raw();
    world.send(Event::Amend {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(call)),
        by: Party::Person(REQUESTER),
        task: THIRD_GOAL,
        message: TRACK_MESSAGE,
        stop_run: false,
        amendment: change,
    });
    assert_eq!(world.replies.get(&call), Some(&jig_tasks_world::Reply::Done));
    assert_eq!(world.record(THIRD_GOAL).tracked, Some(3));
    assert_eq!(world.record(THIRD_GOAL).revisions.spec, 0);
}
