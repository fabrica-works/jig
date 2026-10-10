use jig_core::{
    self as core,
    documents::{Count, Document, PhaseLine, Subject},
};
use jig_core_tasks as tasks;
use jig_core_views as views;
use jig_core_world::{
    effects::Cut,
    watches::{latest, open, world},
};
use jig_test_domain as root;
use skein_lib::Token;

#[test]
fn a_party_watching_a_chat_sees_each_phase_after_its_commit() {
    let mut world = world(701, true);
    let Document::Task(task) = latest(&world, 2) else { panic!("task page") };
    assert!(matches!(task.chip.phase, PhaseLine::Active(tasks::Active::Running { .. })));
    world.store_delay = 4;
    world.end(tasks::End::Parked, 0);
    let Document::Task(task) = latest(&world, 2) else { panic!("idle page") };
    assert_eq!(task.chip.phase, PhaseLine::Active(tasks::Active::Idle));
    world.say(70);
    world.retry_due();
    let Document::Task(task) = latest(&world, 2) else { panic!("running page") };
    assert!(matches!(task.chip.phase, PhaseLine::Active(tasks::Active::Running { .. })));
    world.watches.as_ref().expect("watches").finish();
}

#[test]
fn a_slow_watcher_of_a_tree_sees_fewer_deliveries_and_the_last_state() {
    let mut world = world(702, true);
    world.watches.as_mut().expect("watches").manual(3);
    world.turn(1, 1, None, b"first");
    // This first replacement reaches the party, which leaves it in flight.
    for turn in 2..=4 {
        world.turn(turn, u64::from(turn), None, b"later");
    }
    world.end(tasks::End::Parked, 4);
    world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(3) }));
    let watches = world.watches.as_ref().expect("watches");
    let slow = watches.deliveries.iter().filter(|(watcher, _, _)| *watcher == 3).count();
    let fast = watches.deliveries.iter().filter(|(watcher, _, _)| *watcher == 2).count();
    assert!(slow < fast, "one replacement coalesces intervening commits: slow={slow} fast={fast}");
    let Document::Tree(tree) = latest(&world, 3) else { panic!("tree page") };
    assert_eq!(tree.root.phase, PhaseLine::Active(tasks::Active::Idle));
    watches.finish();
    inbox_across_projects_stories();
}

#[test]
fn a_delivery_never_shows_what_a_crash_took_back() {
    let mut world = world(703, true);
    let task = world.assigned.as_ref().expect("assignment").task;
    open(&mut world, 5, Subject::Run { task });
    let before = world.watches.as_ref().expect("watches").deliveries.len();
    let next = world.store.applied + 1;
    world.cut = Some(Cut::Submitted(next));
    world.turn(1, 1, None, b"uncommitted turn");
    assert_eq!(world.reached_cut, Some(Cut::Submitted(next)));
    assert_eq!(world.watches.as_ref().expect("watches").deliveries.len(), before);
    world.restart(703, false);
    open(&mut world, 6, Subject::Run { task });
    let Document::Run(run) = latest(&world, 6) else { panic!("reopened run") };
    assert_eq!(run.committed_turn, 0);
    assert!(world.store.rows.values().all(|row| !matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) if turn.task == task && turn.turn == 1)));
    world.watches.as_ref().expect("watches").finish();
}

#[test]
fn a_run_s_text_streams_by_turn_and_is_replaced_by_the_committed_turn() {
    let mut world = world(704, true);
    let assignment = world.assigned.as_ref().expect("assignment");
    let (task, attempt) = (assignment.task, assignment.attempt);
    open(&mut world, 5, Subject::Run { task });
    let committed = world.store.applied;
    for (offset, text) in [(0, b"hello ".as_slice()), (6, b"world".as_slice())] {
        world.send(root::Event::Text { task, attempt, turn: 1, offset, text: text.into() });
        world.send(root::Event::Core(core::Event::StreamDelivered { watcher: Token::new(5) }));
    }
    assert_eq!(world.store.applied, committed, "provisional text writes nothing");
    let streams = &world.watches.as_ref().expect("watches").streams;
    assert_eq!(streams.len(), 2);
    assert_eq!((streams[0].turn, streams[0].from, streams[0].to), (views::TurnName { attempt, turn: 1 }, 0, 6));
    assert_eq!((streams[1].from, streams[1].to, streams[1].text.as_ref()), (6, 11, b"world".as_slice()));
    world.store_delay = 3;
    world.turn(1, 1, None, b"whole committed turn");
    let Document::Run(run) = latest(&world, 5) else { panic!("committed run") };
    assert_eq!(run.committed_turn, 1);
    assert!(run.writing.is_none());
    world.watches.as_ref().expect("watches").finish();
}

#[test]
fn a_watch_on_a_task_that_ends_ends_saying_so() {
    let mut world = world(705, true);
    world.store_delay = 3;
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        0,
    );
    let watches = world.watches.as_ref().expect("watches");
    assert!(watches.endings.contains(&(2, views::End::Gone)));
    assert!(watches.endings.contains(&(3, views::End::Gone)));
    let Document::Frame(frame) = latest(&world, 1) else { panic!("updated frame") };
    assert_eq!(frame.updates, Count::Known(1));
    let Document::Inbox(inbox) = latest(&world, 4) else { panic!("updated inbox") };
    assert_eq!(inbox.cached_updates, Count::Known(1));
    assert_eq!(inbox.updates.len(), 1);
    watches.finish();
}

fn inbox_across_projects_stories() {
    use jig_core_authority as authority;
    use jig_core_people as people;
    use jig_core_world::watches::Parties;
    use skein_lib::{List, Map, Queue};
    let (mut config, mut limits) = super::amendment_world_fixture(788);
    limits.core.tasks.tasks = 4;
    limits.core.people.projects = 2;
    limits.core.people.initial_owners = 2;
    limits.core.authority.projects = 2;
    let policy = config.core.authority.policy(1).expect("configured policy").clone();
    let rules = config.core.authority.rules().clone();
    config.core.authority = authority::Domain::new(rules, limits.core.authority).expect("two policies");
    let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
    for project in [1, 2] {
        authority::step(
            &mut config.core.authority,
            authority::Event::Policy { project, policy: policy.clone() },
            &mut facts,
        );
        while facts.pop().is_some() {}
    }
    let first = config.core.owners[0].clone();
    config.core.owners = Box::new([first.clone(), people::InitialOwner { project: 2, identity: first.identity }]);
    config.core.projects = List::with_capacity(2);
    let permissions = config.core.permission_roles.get(&1).expect("permissions").clone();
    config.core.permission_roles = Map::with_capacity(2);
    for project in [1, 2] {
        config.core.projects.push(project).expect("two projects");
        config.core.permission_roles.insert(project, permissions.clone()).expect("two permissions");
    }
    limits.journal.writes = 10_000;
    limits.journal.held = 10_000;
    let mut world = jig_core_world::effects::World::configured(788, false, config, limits);
    world.watches = Some(Parties::new(788, world.store.applied, true));
    open(&mut world, 1, Subject::Frame { party: 1 });
    open(&mut world, 4, Subject::Inbox { party: 1 });
    let first_sign_in = world.sign_in.expect("first party");
    world.send(root::Event::Core(core::Event::SignIn {
        reply_to: skein_lib::ReplyTo::new(Token::new(6090)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: 8_u64.to_be_bytes().into() },
            login: b"other".as_slice().into(),
            name: b"Other".as_slice().into(),
        },
    }));
    let second_sign_in = world.sign_in.expect("second party");
    world.sign_in = Some(first_sign_in);
    super::ask(
        &mut world,
        94,
        people::Ask::SetRoles {
            project: 2,
            holdings: Box::new([
                people::Holding { person: 1, role: people::Role::Owner },
                people::Holding { person: 2, role: people::Role::Owner },
            ]),
        },
    );
    let mut proposals = Vec::new();
    for project in [1, 2] {
        super::ask_as(
            &mut world,
            if project == 1 { first_sign_in } else { second_sign_in },
            u8::try_from(90 + project).expect("request"),
            people::Ask::SetGoal {
                project,
                spec: b"shared goal".as_slice().into(),
                executor: jig_core_people::GoalExecutor::Charter(1),
                budget: 600,
                priority: 0,
                deadline: None,
            },
        );
        let Some(people::Reply::Outcome(people::Outcome::GoalProposed { proposal })) = world.people_answers.last()
        else {
            panic!(
                "goal proposal: {:?}; {:?}",
                world.people_answers,
                &world.trace[world.trace.len().saturating_sub(30)..]
            );
        };
        proposals.push(*proposal);
    }
    a_party_with_roles_in_two_projects_sees_both_in_one_inbox(&world);
    super::ask(
        &mut world,
        93,
        people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Party(1),
            proposal: proposals[0],
            decision: people::ProposalDecision::Reject { reason: b"later".as_slice().into() },
        },
    );
    a_proposal_decided_in_one_project_leaves_the_inbox_across_projects(&world, proposals[1]);
    world.watches.as_ref().expect("parties").finish();
}

fn a_party_with_roles_in_two_projects_sees_both_in_one_inbox(world: &jig_core_world::effects::World) {
    let Document::Inbox(inbox) = latest(world, 4) else { panic!("inbox") };
    assert_eq!(inbox.cached_needs, 2);
    assert_eq!(inbox.needs.len(), 2);
    assert_eq!(inbox.needs.get(0).expect("first").project, Some(1));
    assert_eq!(inbox.needs.get(1).expect("second").project, Some(2));
    let Document::Frame(frame) = latest(world, 1) else { panic!("frame") };
    assert_eq!(frame.needs, 2);
}

fn a_proposal_decided_in_one_project_leaves_the_inbox_across_projects(
    world: &jig_core_world::effects::World,
    other: u64,
) {
    let Document::Inbox(inbox) = latest(world, 4) else { panic!("inbox") };
    assert_eq!(inbox.cached_needs, 1);
    assert_eq!(inbox.needs.len(), 1);
    assert_eq!(inbox.needs.get(0).expect("other proposal").project, Some(2));
    assert_eq!(
        inbox.needs.get(0).expect("other proposal").object,
        core::documents::InboxObject::Proposal { proposer: core::documents::Proposer::Party(2), number: other }
    );
    let Document::Frame(frame) = latest(world, 1) else { panic!("frame") };
    assert_eq!(frame.needs, 1);
}
