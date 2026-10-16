use jig_core_world::fixtures::{live_row, person_unchanged};
use jig_core_world::stories::reads::{request, world};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::Duration;

const PROJECT: u32 = 1;
const OWNER: u64 = 1;
const CANCEL: u8 = 1;
const AMEND: u8 = 2;

#[test]
fn a_partys_cancel_with_a_grace_ends_the_chat_when_the_grace_ends() {
    let mut world = world(1102);
    let task = world.assigned.as_ref().expect("chat").task;
    request(
        &mut world,
        CANCEL,
        people::Ask::Cancel {
            project: PROJECT,
            task,
            reason: b"wrap up".as_slice().into(),
            grace: Some(Duration::from_secs(5)),
        },
    );
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::Cancelled { task })));
    assert_eq!(live_row(&world, task).grace.as_ref().expect("finish requested").reason.as_ref(), b"wrap up");
    assert_eq!(world.inbound.last().expect("the run hears the finish word").words.as_ref(), b"wrap up");
    world.advance(Duration::from_secs(5));
    world.end(tasks::End::Parked, 0);
    let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) =
        world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task))))
    else {
        panic!("the chat ended")
    };
    let tasks::Phase::Ended(tasks::Ending::Cancelled { by, reason, .. }) = &row.phase else { panic!("cancelled") };
    assert_eq!(*by, tasks::Party::Person(OWNER));
    assert_eq!(reason.as_ref(), b"wrap up");
}

#[test]
fn a_maintainer_tracks_a_task_by_amendment() {
    let mut world = jig_core_world::stories::offers::world(1103, Some(people::Role::Maintainer));
    let task = world.assigned.as_ref().expect("chat").task;
    let revision = live_row(&world, task).revisions.spec;
    request(
        &mut world,
        AMEND,
        people::Ask::Amend {
            project: PROJECT,
            task,
            revision,
            amendment: people::Amendment {
                tracked: Some(people::Tracking::Tracked { priority: 7 }),
                ..person_unchanged(b"tracked")
            },
            release: None,
        },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Amended { task, released: false }))
    );
    assert_eq!(live_row(&world, task).tracked, Some(7));
}

#[test]
fn a_grace_past_the_limit_is_refused_naming_it() {
    let mut world = world(1104);
    let task = world.assigned.as_ref().expect("chat").task;
    request(
        &mut world,
        CANCEL,
        people::Ask::Cancel {
            project: PROJECT,
            task,
            reason: b"wrap up".as_slice().into(),
            grace: Some(Duration::from_secs(601)),
        },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Limit {
            which: people::LimitKind::Grace
        })))
    );
    assert_eq!(live_row(&world, task).grace, None);
}
