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
    // Keep the wall time fixed so the brief names this exact amendment time.
    let (now, _) = world.clocks();
    world.set_clock(now, skein_lib::Wall::from_nanos(123_456));
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
    let next_run = running(&world, kept);
    assert_ne!(next_run, kept_run);
    let assignment = world.assigned.as_ref().expect("kept delegate assigned again");
    assert_eq!((assignment.task, assignment.attempt), next_run);
    let dependency_text = assignment
        .brief
        .iter()
        .find_map(|section| match section {
            jig_domain_brief::GatherPlaced::Core { kind: jig_domain_brief::Core::Results, text } => Some(text),
            jig_domain_brief::GatherPlaced::Core { .. }
            | jig_domain_brief::GatherPlaced::CoreMissing { .. }
            | jig_domain_brief::GatherPlaced::Connector { .. }
            | jig_domain_brief::GatherPlaced::Missing { .. } => None,
        })
        .expect("dependency section in next run");
    assert_eq!(
        dependency_text.as_ref(),
        format!("waited for task {dependency}, added 123456 by task {}: done\ntask {dependency}: report\nprerequisite done\n\n", parent.0).as_bytes(),
    );
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
    let mut policy = configuration.core.seed_projects[0].policy.policy.clone();
    let owner = policy.roles[0].clone();
    let mut member = owner.clone();
    member.number = people::Role::Member.number();
    // Amend is bit 8 (authority::Requests); Member still has no project-wide reach.
    member.requests = authority::Requests(8);
    policy.roles = Box::new([owner, member]);
    let mut domain = authority::Domain::new(rules, limits.core.authority).expect("bounded member policy");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    configuration.core.seed_projects[0].policy.policy = domain.policy(1).expect("checked seed policy").clone();
    configuration.core.authority =
        jig_domain_authority::Domain::new(domain.rules().clone(), *domain.limits()).expect("checked rules");
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
        held: None,
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

// These stories observe the store load and externally durable rows, rather than
// duplicating the tasks child's place-reach table.
fn ended_dependency_world(seed: u64) -> (World, (u64, u64), u64, u64) {
    let mut world = plan_world(seed);
    let parent = world.assignments[0];
    let target = member(&world, tasks::Executor::Agent { charter: 1 }, b"report");
    let target = delegate(&mut world, parent, 1, target);
    let dependency = member(&world, tasks::Executor::Agent { charter: 1 }, b"prerequisite");
    let dependency = delegate(&mut world, parent, 2, dependency);
    (world, parent, target, dependency)
}

fn assert_met(world: &World, target: u64, dependency: u64, by: tasks::Party) {
    let row = live_row(world, target);
    assert_eq!(row.dependencies.as_ref(), [dependency]);
    assert!(row.waiting_on.is_empty());
    let added = row.added.first().expect("added dependency retained");
    assert_eq!((added.task, added.by), (dependency, by));
}

fn assert_ended_refusal(world: &World, dependency: u64, how: tasks::Status) {
    let core::CallPart::ControlRefused(problem) = &world.answers.last().expect("amendment answer").2 else {
        panic!("ended dependency refused");
    };
    assert_eq!(problem.why, tasks::Refusal::DependencyEnded { how });
    assert_eq!(problem.blocked_by.as_deref(), Some([dependency].as_slice()));
}

// Intercept exactly one outgoing store demand at the root boundary. The normal
// world transport resumes it after the story has supplied the intervening input.
fn wait_for_dependency(world: &mut World, seed: u64, parent: (u64, u64), target: u64, dependency: u64) -> core::Now {
    let (_, limits) = jig_core_world::faults::plan_fixture(seed);
    let (now, wall) = world.clocks();
    let env = skein_lib::Env { now, wall, limits };
    let mut amendment = unchanged(b"wait for prerequisite");
    amendment.dependencies = Some(Box::new([dependency]));
    let before = world.store.rows.clone();
    let mut event = root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9199)),
        key: core::CallKey { task: parent.0, attempt: parent.1, completion: 99, position: 0 },
        action: core::NamedAction::Amend { target, revision: 0, amendment },
    });
    world.observer.inbound(&world.store, now.as_nanos(), &mut event);
    root::step(&mut world.domain, &env, event);
    let mut out = Queue::with_capacity(root::max_out(&limits));
    root::release(&mut world.domain, &env, &mut out);
    let mut load = None;
    while let Some(output) = out.pop() {
        let root::Request::Deliver(root::Delivery::Core(core::Held::Load(now))) = output else {
            panic!("asking pass emitted more than a load: {output:?}");
        };
        let core::Now::EndedTask { task, .. } = now.as_ref() else { panic!("ended task load") };
        assert_eq!(*task, dependency);
        assert!(load.replace(*now).is_none(), "one load at a time");
    }
    assert_eq!(world.store.rows, before);
    load.expect("amendment loads the ended dependency")
}

#[test]
fn an_ended_dependency_load_waits_then_adds_done_as_met() {
    const SEED: u64 = 878;
    let (mut world, parent, target, dependency) = ended_dependency_world(SEED);
    let run = running(&world, dependency);
    finish(&mut world, run, b"prerequisite done");
    let answers = world.answers.len();
    let load = wait_for_dependency(&mut world, SEED, parent, target, dependency);
    assert_eq!(world.answers.len(), answers);
    assert!(live_row(&world, target).dependencies.is_empty());
    world.store.ask(jig_test_store::StoreOut::Now(load));
    world.drain();
    assert_met(&world, target, dependency, tasks::Party::Task(parent.0));
}

#[test]
fn a_waiting_amendment_rechecks_its_revision_after_loading() {
    const SEED: u64 = 879;
    let (mut world, parent, target, dependency) = ended_dependency_world(SEED);
    let run = running(&world, dependency);
    finish(&mut world, run, b"prerequisite done");
    let load = wait_for_dependency(&mut world, SEED, parent, target, dependency);
    let mut edit = unchanged(b"newer instructions");
    edit.instructions = Some(b"revised while the row loads".as_slice().into());
    named_for(&mut world, parent, 100, core::NamedAction::Amend { target, revision: 0, amendment: edit });
    let before = live_row(&world, target).clone();
    world.store.ask(jig_test_store::StoreOut::Now(load));
    world.drain();
    assert!(matches!(world.answers.last().map(|answer| &answer.2),
        Some(core::CallPart::Moved { task, now: people::Moved::Spec { revision: 1 } }) if *task == target));
    assert_eq!(live_row(&world, target), &before);
}

#[test]
fn a_cancelled_added_dependency_is_refused_by_name() {
    let (mut world, parent, target, dependency) = ended_dependency_world(880);
    named_for(
        &mut world,
        parent,
        4,
        core::NamedAction::Control {
            target: dependency,
            control: tasks::Control::Cancel {
                reason: b"no longer needed".as_slice().into(),
                grace: None,
                message: None,
            },
        },
    );
    let run = running(&world, dependency);
    finish(&mut world, run, b"cancelled run settled");
    let before = live_row(&world, target).clone();
    amend_delegate(&mut world, parent, target, dependency);
    assert_ended_refusal(&world, dependency, tasks::Status::Cancelled);
    assert_eq!(live_row(&world, target), &before);
}

#[test]
fn an_unknown_added_dependency_is_refused_by_name() {
    const UNKNOWN: u64 = u64::MAX;
    let (mut world, parent, target, _) = ended_dependency_world(881);
    let before = live_row(&world, target).clone();
    amend_delegate(&mut world, parent, target, UNKNOWN);
    let core::CallPart::ControlRefused(problem) = &world.answers.last().expect("amendment answer").2 else {
        panic!("unknown dependency refused");
    };
    assert_eq!((problem.task, problem.why), (Some(UNKNOWN), tasks::Refusal::Unknown));
    assert_eq!(live_row(&world, target), &before);
}

#[test]
fn a_done_former_delegate_named_by_a_sibling_is_met() {
    let (mut world, parent, target, dependency) = ended_dependency_world(882);
    let sibling = member(&world, tasks::Executor::Agent { charter: 1 }, b"live sibling");
    let sibling = delegate(&mut world, parent, 4, sibling);
    named_for(&mut world, parent, 5, core::NamedAction::Introduce { left: sibling, right: dependency });
    let run = running(&world, dependency);
    finish(&mut world, run, b"prerequisite done");
    amend_delegate(&mut world, parent, target, dependency);
    assert_met(&world, target, dependency, tasks::Party::Task(parent.0));
    // The introduction was to the sibling, not to the parent amender.
    assert!(live_row(&world, sibling).references.contains(&dependency));
}

#[test]
fn another_subtrees_ended_task_is_refused_unchanged() {
    let (mut world, parent, _, dependency) = ended_dependency_world(883);
    let run = running(&world, dependency);
    finish(&mut world, run, b"prerequisite done");
    let amender = member(&world, tasks::Executor::Agent { charter: 1 }, b"other subtree");
    let amender = delegate(&mut world, parent, 4, amender);
    let amender_run = running(&world, amender);
    let mut target = member(&world, tasks::Executor::Agent { charter: 1 }, b"other report");
    // The amender's 20 includes its running allowance of 10; the leaf
    // needs only 5 and has no further delegation authority.
    target.authority.budget.spend = 5;
    target.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let target = delegate(&mut world, amender_run, 1, target);
    let before = live_row(&world, target).clone();
    amend_delegate(&mut world, amender_run, target, dependency);
    let core::CallPart::ControlRefused(problem) = &world.answers.last().expect("amendment answer").2 else {
        panic!("outside dependency refused");
    };
    assert_eq!(problem.why, tasks::Refusal::Dependencies);
    assert_eq!(problem.blocked_by.as_deref(), Some([dependency].as_slice()));
    assert_eq!(live_row(&world, target), &before);
}

#[test]
fn a_person_adds_an_ended_dependency_from_its_stored_row() {
    let (mut world, _, target, dependency) = ended_dependency_world(884);
    let run = running(&world, dependency);
    finish(&mut world, run, b"prerequisite done");
    let tasks::Party::Person(person) = live_row(&world, world.assignments[0].0).requester else {
        panic!("person's tree");
    };
    let sign_in = world.sign_in.expect("signed-in requester");
    person_amend(&mut world, sign_in, 209, target, &[dependency]);
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Amended { task: target, released: false }))
    );
    assert_met(&world, target, dependency, tasks::Party::Person(person));
}

#[test]
fn a_members_ended_dependency_outside_their_tree_is_refused_unchanged() {
    let (mut world, kept, inside, outside, member_sign_in, owner_sign_in) = members_dependency_world(886);
    // The fleet's three slots hold the parent, kept and inside: inside's end lets the owner's task run.
    let run = running(&world, inside);
    finish(&mut world, run, b"inside done");
    let tasks::Party::Person(owner) = live_row(&world, outside).requester else { panic!("owner's task") };
    let run = running(&world, outside);
    finish(&mut world, run, b"outside done");
    let ended = |key| world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(key)));
    assert!(!ended(tasks::Key::Live(outside)) && ended(tasks::Key::Ended(outside)), "the owner's task has ended");
    let before = live_row(&world, kept).clone();
    person_amend(&mut world, member_sign_in, 211, kept, &[outside]);
    jig_core_world::fixtures::refusal(&world, people::Refusal::Unknown);
    assert_eq!(live_row(&world, kept), &before, "a refused ended dependency leaves the task unchanged");

    // The owner reaches the whole project, so the same done row is met.
    person_amend(&mut world, owner_sign_in, 212, kept, &[outside]);
    assert_met(&world, kept, outside, tasks::Party::Person(owner));
}
