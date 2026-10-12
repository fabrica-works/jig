use jig_core_world::{
    effects::World,
    fixtures::{CREDENTIAL_RESOURCE, WORKSPACE_CREDENTIAL},
    world,
};
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_test_domain as root;
use skein_lib::{Duration, Token};

fn configured(seed: u64) -> World {
    let (config, limits) = world::with_workspace_credential(seed);
    let mut world = World::configured(seed, false, config, limits);
    let (now, wall) = world.clocks();
    world.set_clock(now, wall);
    world
}
fn grant(world: &mut World, generation: u64) {
    world.send(root::Event::Credential { number: WORKSPACE_CREDENTIAL, generation, valid: Duration::from_secs(60) });
}
fn delegated(seed: u64) -> World {
    let mut world = configured(seed);
    grant(&mut world, 3);
    world.delegate_on_resource(CREDENTIAL_RESOURCE);
    world
}
fn hello(world: &mut World) {
    let hosting = world
        .assignments
        .iter()
        .map(|(task, attempt)| fleet::Hosted {
            run: Token::new(*task),
            attempt: Token::new(*attempt),
            phase: fleet::Phase::Active,
        })
        .collect();
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        channel: Token::new(7),
        hello: fleet::Hello {
            bounds: jig_core_world::fixtures::bounds(),
            stop_bound: Duration::from_millis(100),
            slots: 2,
            workstreams: Box::new([]),
            hosting,
        },
    })));
}
fn received(world: &World) -> (Token, fleet::Grant) {
    let (_, task, _, grant) = world.received_grant().expect("host received a credential grant");
    (task, grant)
}
#[test]
fn a_run_needing_a_credential_waits_until_it_is_granted() {
    let mut world = configured(150);
    world.delegate_on_resource(CREDENTIAL_RESOURCE);
    assert_eq!(world.assignments.len(), 1);
    grant(&mut world, 3);
    assert_eq!(world.assignments.len(), 2);
}
#[test]
fn the_assignment_carries_the_credential_named_by_its_table() {
    let world = delegated(151);
    let assignment = world.assigned.as_ref().expect("the delegate is assigned");
    assert!(assignment.grants.contains(&accounts::Grant {
        table: accounts::GrantTable::Application,
        account: WORKSPACE_CREDENTIAL,
        generation: 3,
        valid: Duration::from_secs(60),
    }));
}
#[test]
fn a_new_credential_generation_reaches_the_live_run() {
    let mut world = delegated(152);
    let task = world.assigned.as_ref().expect("delegate assigned").task;
    grant(&mut world, 4);
    assert_eq!(
        received(&world),
        (
            Token::new(task),
            fleet::Grant {
                table: fleet::GrantTable::Application,
                account: WORKSPACE_CREDENTIAL,
                generation: 4,
                valid: Duration::from_secs(60),
            }
        )
    );
}
#[test]
fn an_adopted_run_is_sent_its_credential_again() {
    let mut world = delegated(153);
    let task = world.assigned.as_ref().expect("delegate assigned").task;
    world.restart_with(world::with_workspace_credential(153).0);
    grant(&mut world, 4);
    hello(&mut world);
    assert_eq!(
        received(&world),
        (
            Token::new(task),
            fleet::Grant {
                table: fleet::GrantTable::Application,
                account: WORKSPACE_CREDENTIAL,
                generation: 4,
                valid: Duration::from_secs(60),
            }
        )
    );
}
