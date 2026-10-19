use jig_core_world::{
    fixtures::{
        ask_as,
        deadlines::{self as f, CREATED},
        live_row, refusal,
    },
    stories::offers::{member_configuration, world_with, world_with_policy},
};
use jig_domain_people as p;
use jig_domain_tasks as tasks;
use skein_lib::Wall;

#[test]
fn a_span_starts_at_create_admission() {
    let mut world = world_with(301, Some(p::Role::Member), |_| {});
    let sign_in = world.sign_in.expect("signed in");
    f::clock(&mut world, 10);
    f::create(&mut world, sign_in, 31);
    assert_eq!(live_row(&world, CREATED).authority.budget.deadline, Some(Wall::from_nanos(15_000_000_000)));
}
#[test]
fn a_pending_span_survives_restart_until_acceptance() {
    let (mut world, _, owner, proposal) = f::pending(302);
    assert_eq!(f::person_proposal(&world, proposal).batch()[0].deadline, tasks::Deadline::Span(f::SPAN));
    world.restart_with(member_configuration(302));
    assert_eq!(f::person_proposal(&world, proposal).batch()[0].deadline, tasks::Deadline::Span(f::SPAN));
    f::clock(&mut world, 30);
    f::decide(&mut world, owner, 33, proposal, p::ProposalDecision::Accept);
    assert_eq!(live_row(&world, CREATED).authority.budget.deadline, Some(Wall::from_nanos(35_000_000_000)));
}
#[test]
fn a_delayed_commit_does_not_move_the_deadline() {
    let mut world = world_with(303, Some(p::Role::Member), |_| {});
    let sign_in = world.sign_in.expect("signed in");
    f::clock(&mut world, 10);
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    f::create(&mut world, sign_in, 31);
    assert!(!world.store.held.is_empty());
    f::clock(&mut world, 30);
    world.release_commits();
    assert_eq!(live_row(&world, CREATED).authority.budget.deadline, Some(Wall::from_nanos(15_000_000_000)));
}
#[test]
fn a_span_past_the_ceiling_is_refused_when_proposed() {
    let mut world = world_with_policy(304, Some(p::Role::Member), |_, _, policy| {
        policy.ceiling.budget.deadline = Some(Wall::from_nanos(14_000_000_000));
        for role in &mut policy.roles {
            role.authority.budget.deadline = policy.ceiling.budget.deadline;
        }
    });
    let sign_in = world.sign_in.expect("signed in");
    f::clock(&mut world, 10);
    ask_as(
        &mut world,
        sign_in,
        32,
        p::Ask::Propose {
            project: 1,
            reason: b"span".as_slice().into(),
            action: p::Action::Batch(Box::new([f::span_member()])),
        },
    );
    refusal(&world, p::Refusal::Authority { lacking: p::Lack::Shape, proposable: Some(p::Accepter::Nobody) });
}
#[test]
fn a_resolved_span_beyond_the_creator_is_refused() {
    for (seconds, expected) in [(15, None), (16, Some(p::Accepter::Role(0)))] {
        let (mut world, member, _) = f::finite_world(305);
        f::clock(&mut world, seconds);
        f::create(&mut world, member, 31);
        if let Some(proposable) = expected {
            refusal(&world, p::Refusal::Authority { lacking: p::Lack::Deadline, proposable: Some(proposable) });
        } else {
            assert_eq!(live_row(&world, CREATED).authority.budget.deadline, Some(Wall::from_nanos(20_000_000_000)));
        }
    }
}
#[test]
fn an_aged_span_is_beyond_its_accepter_and_passed_up() {
    let (mut world, member, owner) = f::finite_world(306);
    let proposal = f::task_proposal(&mut world, member);
    assert_eq!(f::holder(&world), tasks::ProposalHolder::Person(1));
    assert_eq!(f::acceptance(&mut world, proposal), jig_domain_core::acts::Admission::Allowed);
    let before = live_row(&world, 1).numbers;
    f::clock(&mut world, 30);
    assert_eq!(f::holder(&world), tasks::ProposalHolder::Person(1));
    let why = p::Refusal::Authority { lacking: p::Lack::Deadline, proposable: Some(p::Accepter::Role(0)) };
    assert_eq!(f::acceptance(&mut world, proposal), jig_domain_core::acts::Admission::Refused(why));
    f::task_decide(&mut world, member, 33, proposal, p::ProposalDecision::Accept);
    refusal(&world, why);
    assert_eq!(live_row(&world, 1).numbers, before);
    assert!(
        !world
            .store
            .rows
            .contains_key(&jig_test_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(CREATED))))
    );
    f::task_decide(&mut world, member, 34, proposal, p::ProposalDecision::Pass);
    assert_eq!(f::holder(&world), tasks::ProposalHolder::Policy { project: 1, kind: tasks::ProposalKind::Batch });
    f::clock(&mut world, 40);
    f::task_decide(&mut world, owner, 35, proposal, p::ProposalDecision::Accept);
    assert_eq!(live_row(&world, CREATED).authority.budget.deadline, Some(Wall::from_nanos(45_000_000_000)));
}
#[test]
fn an_aged_span_past_the_ceiling_makes_no_task() {
    let mut world = world_with_policy(307, Some(p::Role::Member), |_, _, policy| {
        policy.ceiling.budget.deadline = Some(Wall::from_nanos(20_000_000_000));
        for role in &mut policy.roles {
            role.authority.budget.deadline = policy.ceiling.budget.deadline;
        }
    });
    let member = world.sign_in.expect("member");
    let owner = jig_core_world::fixtures::second_party(&mut world, b"second-party", p::Role::Owner);
    f::clock(&mut world, 10);
    let proposal = f::propose(&mut world, member, 32, Box::new([f::span_member()]));
    f::clock(&mut world, 30);
    let before = live_row(&world, 1).numbers;
    f::decide(&mut world, owner, 33, proposal, p::ProposalDecision::Accept);
    refusal(&world, p::Refusal::Authority { lacking: p::Lack::Deadline, proposable: Some(p::Accepter::Nobody) });
    assert_eq!(live_row(&world, 1).numbers, before);
    assert!(
        !world
            .store
            .rows
            .contains_key(&jig_test_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(CREATED))))
    );
}

#[test]
fn holder_rechecks_use_their_own_wall() {
    type Stimulus = fn(&mut jig_core_world::effects::World, u64, u64, u64);
    for (name, stimulate) in
        [("pass", f::pass as Stimulus), ("reroute", f::reroute as Stimulus), ("stall", f::stall as Stimulus)]
    {
        for (seconds, expected) in [
            (30, tasks::ProposalHolder::Policy { project: 1, kind: tasks::ProposalKind::Batch }),
            (100, tasks::ProposalHolder::Person(1)),
        ] {
            let (mut world, member, owner) = f::finite_world(308);
            let proposal = f::task_proposal(&mut world, member);
            f::clock(&mut world, seconds);
            stimulate(&mut world, member, proposal, owner);
            assert_eq!(f::holder(&world), expected, "{name} at {seconds}");
        }
    }
}
