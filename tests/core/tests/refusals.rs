use jig_core_world::effects::World;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

fn ask(world: &mut World, request: u8, ask: people::Ask) {
    ask_as(world, world.sign_in.expect("signed-in person"), request, ask);
}

fn ask_as(world: &mut World, sign_in: u64, request: u8, ask: people::Ask) {
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(8000 + u64::from(request))),
        sign_in,
        key: [request; 16],
        ask,
    })));
}

fn hold_revision(world: &World, task: u64) -> u64 {
    match world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))) {
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) => row.revisions.hold,
        row => panic!("live task row missing: {row:?}"),
    }
}

fn second_person(world: &mut World, role: bool) -> u64 {
    world.send(root::Event::Core(core::Event::SignIn {
        reply_to: ReplyTo::new(Token::new(8090)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: Box::new([2]) },
            login: b"second".as_slice().into(),
            name: b"Second".as_slice().into(),
        },
    }));
    let Some(people::Reply::SignedIn { person, sign_in, .. }) = world.people_answers.last().copied() else {
        panic!("second person signed in");
    };
    if role {
        world.send(root::Event::Core(core::Event::People(people::Event::Roles {
            project: 1,
            holdings: Box::new([
                people::Holding { person: 1, role: people::Role::Owner },
                people::Holding { person, role: people::Role::Owner },
            ]),
        })));
    }
    sign_in
}

#[test]
fn deciding_an_escalation_after_its_release_is_refused_as_not_waiting() {
    let mut world = World::new(152, false);
    let task = world.call_key(1).task;
    ask(&mut world, 60, people::Ask::Stop { project: 1, task });
    world.end(tasks::End::Parked, 0);
    let revision = hold_revision(&world, task);
    ask(&mut world, 61, people::Ask::Release { project: 1, task, revision, again: None });
    ask(
        &mut world,
        62,
        people::Ask::DecideEscalation { project: 1, task, revision, decision: people::EscalationDecision::Pass },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWaiting)))
    );
}

#[test]
fn deciding_an_escalation_another_party_decided_says_by_whom() {
    let mut world = World::new(153, false);
    let first = world.sign_in.expect("first person");
    let second = second_person(&mut world, true);
    let task = world.call_key(1).task;
    ask_as(&mut world, first, 60, people::Ask::Stop { project: 1, task });
    world.end(tasks::End::Parked, 0);
    let revision = hold_revision(&world, task);
    ask_as(
        &mut world,
        first,
        61,
        people::Ask::DecideEscalation {
            project: 1,
            task,
            revision,
            decision: people::EscalationDecision::Reject { reason: b"leave held".as_slice().into() },
        },
    );
    ask_as(
        &mut world,
        second,
        62,
        people::Ask::DecideEscalation { project: 1, task, revision, decision: people::EscalationDecision::Pass },
    );
    assert!(matches!(
        world.people_answers.last(),
        Some(people::Reply::Outcome(people::Outcome::EscalationDecided {
            task: 1,
            revision: 1,
            by: 1,
            choice: people::EscalationChoice::Rejected,
            ..
        }))
    ));
}

#[test]
fn deciding_an_escalation_of_an_unseen_project_is_unknown() {
    let mut world = World::new(154, false);
    let task = world.call_key(1).task;
    ask(&mut world, 60, people::Ask::Stop { project: 1, task });
    world.end(tasks::End::Parked, 0);
    let revision = hold_revision(&world, task);
    let other = second_person(&mut world, false);
    ask_as(
        &mut world,
        other,
        62,
        people::Ask::DecideEscalation { project: 1, task, revision, decision: people::EscalationDecision::Pass },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown)))
    );
}
