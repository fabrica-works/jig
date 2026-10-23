use jig_conformance::Harness;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_ops_domain as root;
use jig_ops_domain_infrastructure as infra;
use jig_ops_world::conformance::{Config, Ops, Story, chat};
use skein_lib::{ReplyTo, Token};

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

fn provisioning(granted: bool) -> Harness<Ops> {
    let config = Config::new(915, Story::Environment);
    let config = if granted { config } else { config.without_close_grant() };
    let mut world = Harness::<Ops>::new(config, 915);
    world.drain().expect("ops startup");
    world.send(root::Event::Core(core::Event::SignIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(601)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: 8_u64.to_be_bytes().into() },
            login: b"developer".as_slice().into(),
            name: Box::new([]),
        },
    }));
    world.drain().expect("the configured developer owner signs in");
    let pool =
        world.systems.infrastructure(infra::SystemRequest::Pool { pool: infra::Pool(b"staging".as_slice().into()) });
    world.send(root::Event::Infrastructure(infra::Event::System(pool)));
    world.drain().expect("the staging pool reports its available slots");
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(600)),
        sign_in: world.peers.sign_in.expect("developer signed in"),
        key: [91; 16],
        ask: {
            let people::Ask::Create { project, mut batch } = chat(b"@environment_close") else {
                panic!("chat creation fixture");
            };
            for member in &mut batch {
                member.wake.results = people::ResultsWake::Each;
            }
            people::Ask::Create { project, batch }
        },
    })));
    world.drain().expect("a real chat delegates an environment and another live task");
    let live: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) => Some(row),
            root::Record::Core(_) | root::Record::Observability(_) | root::Record::Infrastructure(_) => None,
        })
        .collect();
    assert_eq!(live.len(), 3, "C, E and D");
    assert!(live.iter().any(|row| row.number == 1 && row.flags.conversation));
    assert!(live.iter().filter(|row| row.requester == tasks::Party::Task(1)).count() == 2);
    assert!(world.store.rows.values().any(|row| matches!(
        row,
        root::Record::Infrastructure(infra::Record::Procedure(infra::ProcedureState { chat: Some(1), .. }))
    )));
    world
}

fn pending_closes(world: &Harness<Ops>) -> usize {
    world
        .store
        .rows
        .values()
        .filter(|row| matches!(row, root::Record::Infrastructure(infra::Record::Closing { .. })))
        .count()
}

fn task(world: &Harness<Ops>, number: u64) -> &tasks::TaskRecord {
    world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                if row.number == number =>
            {
                Some(row.as_ref())
            }
            root::Record::Core(_) | root::Record::Observability(_) | root::Record::Infrastructure(_) => None,
        })
        .expect("retained task")
}

fn closed(world: &Harness<Ops>) {
    for number in [1, 3] {
        assert!(
            matches!(&task(world, number).phase,
                tasks::Phase::Ended(tasks::Ending::Cancelled { by: tasks::Party::Deployment { project: 1 }, reason, .. })
                    if reason.as_ref() == b"environment torn down"
            ),
            "deployment cancels task {number}: {:?}",
            task(world, number).phase
        );
    }
    assert_eq!(
        world
            .store
            .rows
            .values()
            .filter(|row| matches!(
                row,
                root::Record::Core(core::Record::People(people::Stored::Acted {
                    outcome: people::Outcome::Compound { .. },
                    ..
                }))
            ))
            .count(),
        1
    );
    assert_eq!(pending_closes(world), 0);
}

#[test]
fn infrastructure_closes_the_chat_its_environment_served_once_torn_down() {
    let mut world = provisioning(true);
    world.advance(5_000_000_000);
    world.drain().expect("E finishes; C delegates T, which tears down its environment");
    assert_eq!(world.peers.close_answers.len(), 1, "infrastructure closes C once T's teardown is made");
    assert!(matches!(world.peers.close_answers[0], people::Outcome::Compound { operations: 3, .. }));
    let grace = task(&world, 1).grace.as_ref().expect("C is asked to finish in its template's grace");
    assert_eq!(grace.by, tasks::Party::Deployment { project: 1 });
    assert_eq!(grace.reason.as_ref(), b"environment torn down");
    let at = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::People(people::Stored::Acted { at, .. })) => Some(*at),
            root::Record::Core(_) | root::Record::Observability(_) | root::Record::Infrastructure(_) => None,
        })
        .expect("saved act's decision time");
    assert_eq!(grace.until.as_nanos(), at.as_nanos() + 1_000_000_000);
    assert!(matches!(task(&world, 3).phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })));
    world.advance(1_000_000_000);
    world.drain().expect("C outlives its grace and is cancelled");
    closed(&world);
}

#[test]
fn a_close_committed_before_a_restart_is_sent_again_and_answered_once() {
    let mut world = provisioning(true);
    world.advance(5_000_000_000);
    for _ in 0..2000 {
        if world.systems.production.used_slots("staging") == 0 {
            break;
        }
        world.iterate().expect("reach teardown's answer");
    }
    assert_eq!(world.systems.production.used_slots("staging"), 0);
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    for _ in 0..200 {
        world.iterate().expect("commit the act but hold its answer");
        if world
            .store
            .rows
            .values()
            .any(|row| matches!(row, root::Record::Core(core::Record::People(people::Stored::Acted { .. }))))
        {
            break;
        }
    }
    assert_eq!(pending_closes(&world), 1);
    assert!(world.peers.close_answers.is_empty());
    assert!(world.store.rows.values().any(|row| matches!(
        row,
        root::Record::Core(core::Record::People(people::Stored::Acted {
            outcome: people::Outcome::Compound { .. },
            ..
        }))
    )));
    world.crash().expect("restart between commit and answer");
    world.drain().expect("infrastructure resends and hears the saved answer");
    assert_eq!(world.peers.close_answers.len(), 1);
    assert_eq!(pending_closes(&world), 0);
    world.advance(1_000_000_000);
    world.drain().expect("restored grace ends");
    closed(&world);
}

#[test]
fn without_the_grant_the_close_is_not_offered_and_kept() {
    let mut world = provisioning(false);
    world.advance(5_000_000_000);
    world.drain().expect("teardown asks for an ungranted close");
    assert_eq!(world.peers.close_answers, [people::Outcome::Refused(people::Refusal::NotOffered)]);
    assert_eq!(pending_closes(&world), 1);
    assert!(!matches!(task(&world, 1).phase, tasks::Phase::Closing(_) | tasks::Phase::Ended(_)));
    assert!(!matches!(task(&world, 3).phase, tasks::Phase::Closing(_) | tasks::Phase::Ended(_)));
    world.crash().expect("later start reconsiders its configuration");
    world.drain().expect("the retained close is sent again once ready");
    assert_eq!(world.peers.close_answers.len(), 2);
    assert_eq!(pending_closes(&world), 1);
}
