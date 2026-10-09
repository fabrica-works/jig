//! Last-change facts are owned by accepted row mutations, including cold restore.
use super::revisions::{Harness, task, to};
use crate::{
    Contract, Control, EscalationHolder, Event, Executor, Funder, Hold, Key, Numbers, Party, RecurringOverlap,
    RecurringTemplate, Request, Stored,
};
use skein_lib::Wall;

#[test]
fn changed_tracks_creation_real_changes_backward_walls_restore_and_ending() {
    let mut test = Harness::new();
    test.env.wall = Wall::from_nanos(100);
    test.send(Event::Make { reply_to: to(), creator: Party::Person(9), batch: Box::new([task(1, &[])]) });
    assert_eq!(test.domain.task(1).expect("created").changed, test.env.wall);
    test.env.wall = Wall::from_nanos(200);
    test.send(Event::Hold { task: 1, why: Hold::Stopped });
    assert_eq!(test.domain.task(1).expect("held").changed, test.env.wall);
    test.env.wall = Wall::from_nanos(50);
    test.send(Event::Hold { task: 1, why: Hold::Budget });
    assert_eq!(test.domain.task(1).expect("changed hold").changed, test.env.wall);
    test.route(1, EscalationHolder::Person(9), 1);
    test.env.wall = Wall::from_nanos(300);
    test.restart();
    assert_eq!(test.domain.task(1).expect("restored").changed, Wall::from_nanos(50));
    test.send(Event::Control {
        reply_to: to(),
        by: Party::Person(9),
        task: 1,
        control: Control::Cancel { reason: Box::new([]) },
    });
    test.send(Event::EffectsSettled { task: 1 });
    test.env.wall = Wall::from_nanos(400);
    test.send(Event::Settled { task: 1 });
    let Stored::Ended(row) = test.stored.get(&Key::Ended(1)).expect("ended snapshot") else { panic!("ended record") };
    assert_eq!(row.changed, Wall::from_nanos(400));
    assert_eq!(row.ended_at, Some(row.changed));
}

#[test]
fn duplicate_live_publication_does_not_replace_the_last_change_wall() {
    let mut test = Harness::new();
    let mut master = task(100, &[]);
    master.executor = Executor::Procedure { connector: 1, code: 1 };
    master.contract = Contract::Report { words: 0 };
    master.numbers = Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
    master.recurring =
        Some(Box::new(RecurringTemplate { key: 1, batch: Box::new([task(1, &[])]), overlap: RecurringOverlap::Skip }));
    test.env.wall = Wall::from_nanos(100);
    test.send(Event::Make { reply_to: to(), creator: Party::Deployment { project: 1 }, batch: Box::new([master]) });
    test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 1, budget: 1000 });
    test.env.wall = Wall::from_nanos(200);
    test.send(Event::TickRecurring { task: 100, period: 1 });
    let before = test.domain.task(100).expect("due recurring").clone();
    assert_eq!(before.funder, Funder::Period { project: 1, period: 1 });
    test.env.wall = Wall::from_nanos(300);
    let out = test.send(Event::TickRecurring { task: 100, period: 1 });
    let mut saved = false;
    for request in out {
        if let Request::Save { record: Stored::Live(row) } = request {
            assert_eq!(row.as_ref(), &before);
            saved = true;
        }
    }
    assert!(saved, "the no-op path really publishes a live snapshot");
    test.restart();
    test.env.wall = Wall::from_nanos(400);
    test.send(Event::TickRecurring { task: 100, period: 1 });
    assert_eq!(test.domain.task(100).expect("restored recurring").changed, before.changed);
}
