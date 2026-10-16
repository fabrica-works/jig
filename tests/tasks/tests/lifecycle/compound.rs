use jig_domain_tasks::{Event, Operation, Party};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::{Duration, List};
const ROOT: u64 = 1;
const CHILD: u64 = 2;
const REQUESTER: u64 = 9;
const PROJECT: u32 = 1;
const FINISH: u64 = 1;

#[test]
fn a_compound_closing_a_task_and_moving_its_delegate_survives_a_restart() {
    let mut world = World::new(204, LIMITS);
    world.make(Party::Person(REQUESTER), vec![task(ROOT, &[])]);
    let mut child = task(CHILD, &[]);
    child.funder = jig_domain_tasks::Funder::Task(ROOT);
    world.make(Party::Task(ROOT), vec![child]);
    let mut operations = List::with_capacity(2);
    operations
        .push(Operation::Cancel {
            task: ROOT,
            reason: b"finish".as_slice().into(),
            grace: Some(Duration::from_secs(5)),
            message: Some(FINISH),
        })
        .expect("compound fixture");
    operations.push(Operation::Move { task: CHILD, reason: b"keep".as_slice().into() }).expect("compound fixture");
    let reply_to = world.to();
    world.send(Event::Compound {
        reply_to,
        person: REQUESTER,
        project: PROJECT,
        operations,
        period: 0,
        pool_budget: 1_000,
        period_budget: 10_000,
    });
    assert_eq!(world.replies.last_key_value().expect("compound fixture").1, &Reply::Done);
    world.restart();
    assert_eq!(world.record(ROOT).grace.as_ref().expect("compound fixture").message, FINISH);
    assert_eq!(world.record(CHILD).requester, Party::Person(REQUESTER));
    assert_eq!(world.record(CHILD).root, CHILD);
}
