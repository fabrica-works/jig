use super::*;
use jig_core_world::fixtures::unchanged;

#[test]
fn a_kept_delegate_is_woken_by_its_last_dependencys_met() {
    let mut world = plan_world(873);
    let parent = world.assignments[0];
    let kept = member(&world, tasks::Executor::Agent { charter: 1 }, b"kept report");
    let kept = delegate(&mut world, parent, 1, kept);
    let kept_run = running(&world, kept);
    let dependency = member(&world, tasks::Executor::Agent { charter: 1 }, b"prerequisite");
    let dependency = delegate(&mut world, parent, 2, dependency);
    let dependency_run = running(&world, dependency);
    amend_delegate(&mut world, parent, kept, dependency);
    assert_eq!(live_row(&world, kept).waiting_on.as_ref(), [dependency]);
    finish(&mut world, kept_run, b"saved report");
    assert_eq!(live_row(&world, kept).phase, tasks::Phase::Active(tasks::Active::Idle));
    let parked_milestone = live_row(&world, kept).milestone;
    finish(&mut world, dependency_run, b"prerequisite done");
    assert!(live_row(&world, kept).waiting_on.is_empty());
    let met: Vec<_> = live_row(&world, kept)
        .inbox
        .iter()
        .filter(|word| matches!(word.kind, tasks::MessageKind::Met { .. }))
        .collect();
    assert_eq!(met.len(), 1);
    let met = met.first().expect("one numbered notice");
    assert_eq!(met.kind, tasks::MessageKind::Met { dependency, removed: false });
    assert_eq!(met.from, tasks::Party::Task(dependency));
    assert_eq!(live_row(&world, kept).due_reason, tasks::Why::Message { message: met.number });
    for phase in [tasks::Active::Due, tasks::Active::Preparing] {
        assert!(
            world.store.rows.values().any(|row| matches!(
                row,
                root::Record::Core(core::Record::Tasks(tasks::Stored::Milestone(milestone)))
                    if milestone.task == kept
                        && milestone.position > parked_milestone
                        && milestone.phase == tasks::Phase::Active(phase)
            )),
            "Met made the kept delegate {phase:?}"
        );
    }
    // M2's dependency read carries this story to the next assignment.
}

fn amend_delegate(world: &mut World, parent: (u64, u64), target: u64, dependency: u64) {
    let mut amendment = unchanged(b"wait for prerequisite");
    amendment.dependencies = Some(Box::new([dependency]));
    named_for(world, parent, 3, core::NamedAction::Amend { target, revision: 0, amendment });
}

fn named_for(world: &mut World, run: (u64, u64), completion: u32, action: core::NamedAction) {
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9100 + u64::from(completion))),
        key: core::CallKey { task: run.0, attempt: run.1, completion, position: 0 },
        action,
    }));
}

#[test]
fn an_agents_circular_dependency_amendment_names_the_circle() {
    let mut world = plan_world(874);
    let parent = world.assignments[0];
    let first = member(&world, tasks::Executor::Agent { charter: 1 }, b"first report");
    let first = delegate(&mut world, parent, 1, first);
    let mut second = member(&world, tasks::Executor::Agent { charter: 1 }, b"second report");
    second.dependencies = Box::new([core::Dependency::Existing(first)]);
    let second = delegate(&mut world, parent, 2, second);
    amend_delegate(&mut world, parent, first, second);
    let core::CallPart::ControlRefused(problem) = &world.answers.last().expect("amendment answer").2 else {
        panic!("circle refused");
    };
    assert_eq!(problem.why, tasks::Refusal::Circle { cut: false });
    assert_eq!(problem.blocked_by.as_deref(), Some([first, second].as_slice()));
    assert!(live_row(&world, first).dependencies.is_empty());
}

#[test]
fn an_agents_failed_dependency_amendment_names_the_ended_task() {
    let mut world = plan_world(875);
    let parent = world.assignments[0];
    let dependency = member(&world, tasks::Executor::Agent { charter: 1 }, b"failed prerequisite");
    let dependency = delegate(&mut world, parent, 1, dependency);
    let dependency_run = running(&world, dependency);
    let kept = member(&world, tasks::Executor::Agent { charter: 1 }, b"live report");
    let kept = delegate(&mut world, parent, 2, kept);
    let kept_run = running(&world, kept);
    // Once the dependency ends, explicit introductions keep its identity
    // within the parent's reach and retain its historical result stub.
    named_for(&mut world, parent, 4, core::NamedAction::Introduce { left: kept, right: dependency });
    named_for(&mut world, kept_run, 1, core::NamedAction::Introduce { left: parent.0, right: dependency });
    world.send(root::Event::Answer {
        read: None,
        refused: Box::new([]),
        channel: Token::new(7),
        task: dependency_run.0,
        attempt: dependency_run.1,
        cumulative: 0,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Failure { reason: b"prerequisite failed".as_slice().into() },
            cancel_delegates: false,
        },
    });
    amend_delegate(&mut world, parent, kept, dependency);
    let core::CallPart::ControlRefused(problem) = &world.answers.last().expect("amendment answer").2 else {
        panic!("ended dependency refused");
    };
    assert_eq!(problem.why, tasks::Refusal::DependencyEnded { how: tasks::Status::Failed });
    assert_eq!(problem.blocked_by.as_deref(), Some([dependency].as_slice()));
    assert!(live_row(&world, kept).dependencies.is_empty());
}

fn member_world(seed: u64) -> World {
    let (mut configuration, mut limits) = jig_core_world::faults::plan_fixture(seed);
    // The owner remains configured, alongside the member whose reach is its own tree.
    limits.core.authority.roles = 2;
    limits.core.tasks.dependencies = 2; // The retained outside task and one newly added inside task.
    let rules = configuration.core.authority.rules().clone();
    let mut policy = configuration.core.authority.policy(1).expect("plan policy").clone();
    let owner = policy.roles[0].clone();
    let mut member = owner.clone();
    member.number = people::Role::Member.number();
    // Amend is bit 8 (authority::Requests); Member still has no project-wide reach.
    member.requests = authority::Requests(8);
    policy.roles = Box::new([owner, member]);
    let mut domain = authority::Domain::new(rules, limits.core.authority).expect("bounded member policy");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    configuration.core.authority = domain;
    jig_core_world::peers::journal(&mut limits);
    World::configured(seed, false, configuration, limits)
}

fn dependency_amendment(dependencies: &[u64]) -> people::Amendment {
    let mut edit = amendment(b"report");
    edit.spec = None;
    edit.dependencies = Some(dependencies.into());
    edit
}

fn members_dependency_world(seed: u64) -> (World, u64, u64, u64, u64, u64) {
    let mut world = member_world(seed);
    let parent = world.assignments[0];
    let kept = member(&world, tasks::Executor::Agent { charter: 1 }, b"report");
    let kept = delegate(&mut world, parent, 1, kept);
    let inside = member(&world, tasks::Executor::Agent { charter: 1 }, b"inside prerequisite");
    let inside = delegate(&mut world, parent, 2, inside);
    let member_sign_in = world.sign_in.expect("requester signed in");
    let tasks::Party::Person(person) = live_row(&world, parent.0).requester else { panic!("person's tree") };
    world.send(root::Event::Core(core::Event::SignIn {
        reply_to: ReplyTo::new(Token::new(6100)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: 8_u64.to_be_bytes().into() },
            login: b"second".as_slice().into(),
            name: b"other requester".as_slice().into(),
        },
    }));
    let owner_sign_in = world.sign_in.expect("other requester signed in");
    let Some(people::Reply::SignedIn { person: other, .. }) = world.people_answers.last().cloned() else {
        panic!("second sign-in");
    };
    ask_as(
        &mut world,
        member_sign_in,
        201,
        people::Ask::SetRoles {
            project: 1,
            holdings: Box::new([
                people::Holding { person, role: people::Role::Member },
                people::Holding { person: other, role: people::Role::Owner },
            ]),
        },
    );
    let people::Ask::Create { mut batch, project } =
        jig_core_world::fixtures::chat(&world, 1, b"outside prerequisite".as_slice().into())
    else {
        panic!("chat creation")
    };
    batch[0].budget = people::Budget::Named;
    batch[0].authority.spend = 20;
    let outside = people::Ask::Create { project, batch };
    ask_as(&mut world, owner_sign_in, 202, outside);
    let Some(people::Reply::Outcome(people::Outcome::Created { first: outside, .. })) = world.people_answers.last()
    else {
        panic!("other person's task created: {:?}", world.people_answers.last());
    };
    let outside = *outside;
    (world, kept, inside, outside, member_sign_in, owner_sign_in)
}

fn person_amend(world: &mut World, sign_in: u64, request: u8, task: u64, dependencies: &[u64]) {
    let revision = live_row(world, task).revisions.spec;
    ask_as(
        world,
        sign_in,
        request,
        people::Ask::Amend { project: 1, task, revision, release: None, amendment: dependency_amendment(dependencies) },
    );
}

#[test]
fn a_members_added_dependencies_must_be_within_their_tree() {
    let (mut world, kept, inside, outside, member_sign_in, owner_sign_in) = members_dependency_world(876);
    person_amend(&mut world, member_sign_in, 203, kept, &[outside]);
    jig_core_world::fixtures::refusal(&world, people::Refusal::Unknown);
    assert!(live_row(&world, kept).dependencies.is_empty());
    person_amend(&mut world, member_sign_in, 204, kept, &[inside]);
    let amended = people::Reply::Outcome(people::Outcome::Amended { task: kept, released: false });
    assert_eq!(world.people_answers.last(), Some(&amended));
    assert_eq!(live_row(&world, kept).dependencies.as_ref(), [inside]);

    // The owner can add a dependency beyond this member's tree. Keeping or
    // removing it then needs no new reach from the member.
    person_amend(&mut world, owner_sign_in, 205, kept, &[outside]);
    person_amend(&mut world, member_sign_in, 206, kept, &[outside, inside]);
    assert_eq!(world.people_answers.last(), Some(&amended));
    assert_eq!(live_row(&world, kept).dependencies.as_ref(), [outside, inside]);
    person_amend(&mut world, member_sign_in, 207, kept, &[inside]);
    assert_eq!(world.people_answers.last(), Some(&amended));
    assert_eq!(live_row(&world, kept).dependencies.as_ref(), [inside]);
}

#[test]
fn an_earlier_compound_move_brings_an_added_dependency_within_reach() {
    let (mut world, kept, _, outside, member_sign_in, _) = members_dependency_world(877);
    let mut operations = List::with_capacity(2); // One move, followed by one amendment.
    operations
        .push(people::Operation::Move { task: outside, reason: b"continue in my tree".as_slice().into() })
        .expect("move slot");
    operations
        .push(people::Operation::Amend {
            task: kept,
            revision: 0,
            release: None,
            amendment: Box::new(dependency_amendment(&[outside])),
        })
        .expect("amend slot");
    ask_as(&mut world, member_sign_in, 208, people::Ask::Compound { project: 1, operations });
    assert!(
        matches!(
            world.people_answers.last(),
            Some(people::Reply::Outcome(people::Outcome::Compound { operations: 2, .. }))
        ),
        "compound answer: {:?}",
        world.people_answers.last()
    );
    assert_eq!(live_row(&world, kept).dependencies.as_ref(), [outside]);
    assert_eq!(live_row(&world, outside).requester, live_row(&world, world.assignments[0].0).requester);
}
