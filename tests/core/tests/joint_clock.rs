use jig_core_world::{effects::World, peers::Peers, world};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Time, Token, Wall};

fn sign_in(world: &mut World, number: u64) {
    world.send(root::Event::Core(core::Event::SignIn {
        reply_to: ReplyTo::new(Token::new(number)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: number.to_be_bytes().into() },
            login: b"person".as_slice().into(),
            name: b"Person".as_slice().into(),
        },
    }));
}

#[test]
fn supplied_joint_clocks_survive_internal_commits_and_drive_the_actual_sign_in_expiry() {
    let limits = world::limits();
    let now = Time::from_nanos(17);
    let wall = Wall::from_nanos(1_000_000_000_000);
    let mut world = World::scripted_clock(world::config(8451), limits, Peers::new(Vec::new(), Vec::new()), now, wall);
    sign_in(&mut world, 7);
    let expiry = Wall::from_nanos(
        wall.as_nanos().checked_add(limits.core.people.sign_in_lifetime.as_nanos()).expect("bounded configured expiry"),
    );
    assert!(
        world
            .people_answers
            .iter()
            .any(|reply| matches!(reply, people::Reply::SignedIn { person: 1, expires, .. } if *expires == expiry))
    );
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::People(people::Stored::SignIn { person: 1, expires, .. })) if *expires == expiry)));
    assert!(world.store.applied > 0, "the sign-in was actually committed");
    let applied = world.store.applied;
    world.drain();
    assert_eq!(world.store.applied, applied);
    assert_eq!(world.clocks(), (now, wall), "draining creates no elapsed or wall time");
}

#[test]
fn corrected_joint_wall_is_independent_of_elapsed_time_and_is_persisted_for_a_new_sign_in() {
    let limits = world::limits();
    let mut world = World::scripted_clock(
        world::config(8452),
        limits,
        Peers::new(Vec::new(), Vec::new()),
        Time::from_nanos(17),
        Wall::from_nanos(2_000_000_000_000),
    );
    sign_in(&mut world, 7);
    let prior_expiry = Wall::from_nanos(2_000_000_000_000 + limits.core.people.sign_in_lifetime.as_nanos());
    let now = Time::from_nanos(31);
    let corrected = Wall::from_nanos(1_000_000_000_000);
    world.set_clock(now, corrected);
    sign_in(&mut world, 8);
    let expiry = Wall::from_nanos(
        corrected.as_nanos().checked_add(limits.core.people.sign_in_lifetime.as_nanos()).expect("corrected expiry"),
    );
    assert!(
        world
            .people_answers
            .iter()
            .any(|reply| matches!(reply, people::Reply::SignedIn { person: 2, expires, .. } if *expires == expiry))
    );
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::People(people::Stored::SignIn { person: 2, expires, .. })) if *expires == expiry)));
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::People(people::Stored::SignIn { person: 1, expires, .. })) if *expires == prior_expiry)), "the correction does not rewrite an existing durable expiry");
    assert_eq!(world.clocks(), (now, corrected));
}

#[test]
fn joint_scheduler_expires_the_actual_sign_in_at_its_armed_people_deadline() {
    let limits = world::limits();
    let now = Time::from_nanos(17);
    let wall = Wall::from_nanos(1_000_000_000_000);
    let mut world = World::scripted_clock(world::config(8453), limits, Peers::new(Vec::new(), Vec::new()), now, wall);
    sign_in(&mut world, 7);
    let due = Time::from_nanos(now.as_nanos() + limits.core.people.sign_in_lifetime.as_nanos());
    assert!(world.domain.core_deadlines().contains(&(core::Timer::People, Some(due))));
    let saved = world
        .store
        .rows
        .iter()
        .find_map(|(key, row)| {
            if matches!(row, root::Record::Core(core::Record::People(people::Stored::SignIn { person: 1, .. }))) {
                Some(key.clone())
            } else {
                None
            }
        })
        .expect("actual durable sign-in");
    let applied = world.store.applied;
    world.fire_due(Time::from_nanos(due.as_nanos() - 1), wall);
    assert!(world.store.rows.contains_key(&saved), "not yet due");
    assert_eq!(world.store.applied, applied);
    world.fire_due(due, wall);
    assert!(!world.store.rows.contains_key(&saved), "People fired through the normal durable root route");
    assert!(world.store.applied > applied);
    assert_eq!(world.clocks(), (due, wall), "an elapsed deadline needs no wall jump");
    assert!(world.domain.core_deadlines().contains(&(core::Timer::People, None)));
}

#[test]
fn joint_scheduler_resamples_the_effect_wall_after_correction_and_consumes_the_right_without_a_new_commit() {
    let mut world = World::new(8454, false);
    let now = world.clocks().0;
    let wall = Wall::from_nanos(1_000_000_000_000);
    world.set_clock(now, wall);
    world.hold_writes = true;
    let deadline = Wall::from_nanos(wall.as_nanos() + 500);
    let key = world.call_key(1);
    world.send(root::Event::EffectCall {
        to: ReplyTo::new(Token::new(9001)),
        key,
        number: 1,
        effect: World::effect(),
        deadline,
        proposal: None,
    });
    assert!(world.answers.is_empty());
    assert_eq!(world.domain.effect_deadline(), Some(deadline));
    assert_eq!(world.next_due(), Some(Time::from_nanos(now.as_nanos() + 500)));
    let corrected = Wall::from_nanos(wall.as_nanos() - 100);
    let elapsed = Time::from_nanos(now.as_nanos() + 20);
    world.set_clock(elapsed, corrected);
    assert_eq!(world.next_due(), Some(Time::from_nanos(elapsed.as_nanos() + 600)));
    let applied = world.store.applied;
    let makes = world.systems[0].observed().len();
    world.fire_due(Time::from_nanos(elapsed.as_nanos() + 599), Wall::from_nanos(deadline.as_nanos() - 1));
    assert!(world.answers.is_empty());
    world.fire_due(Time::from_nanos(elapsed.as_nanos() + 600), deadline);
    assert_eq!(world.answers.len(), 1);
    assert_eq!(world.answers[0].0, Token::new(9001));
    assert_eq!(world.answers[0].1, key);
    assert!(matches!(world.answers[0].2, core::CallPart::Effect { outcome: None, .. }));
    assert_eq!(world.store.applied, applied, "deadline returns the previously committed part");
    assert_eq!(world.systems[0].observed().len(), makes, "deadline makes no new external effect");
    assert_eq!(world.domain.effect_deadline(), None);
    world.fire_due(Time::from_nanos(elapsed.as_nanos() + 601), deadline);
    assert_eq!(world.answers.len(), 1, "the original reply right is consumed once");
}
