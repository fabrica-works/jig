use jig_core_world::{
    reads::{ask, latest},
    stories::reads::{request, world},
};
use jig_domain_core::{self as core, documents, reads, waits::WaitsOn};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

#[test]
fn a_stopped_chat_reads_as_waiting_on_its_party() {
    let mut world = world(1201);
    let task = world.assigned.as_ref().expect("chat").task;
    request(&mut world, 1, people::Ask::Stop { project: 1, task });
    ask(&mut world, 2, reads::Read::Task { task });
    let reads::Answer::Document(document) = latest(&world) else { panic!("a live task") };
    let documents::Document::Task(value) = document.as_ref() else { panic!("the task's document") };
    assert!(matches!(value.waits_on, WaitsOn::Party {
        whom: people::Whom::Person(1), entries: 1, task: waiting,
        kind: people::EntryKind::Escalation { .. },
    } if waiting == task));
}

fn delegate_question(world: &mut jig_core_world::effects::World, parent: u64) {
    let mut authority = world.assigned.as_ref().expect("parent run").run.authority.clone();
    authority.budget.spend = 20;
    authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(4001)),
        key: world.call_key(1),
        batch: Box::new([core::Delegate {
            executor: tasks::Executor::Agent { charter: 1 },
            spec: tasks::Spec {
                words: b"ask the requester".as_slice().into(),
                parameters: Box::new([]),
                inputs: Box::new([]),
            },
            contract: tasks::Contract::Report { words: 64 },
            authority,
            symbolic_grants: Box::new([]),
            dependencies: Box::new([]),
            wake: tasks::WakePolicy::DEFAULT,
        }]),
        stubs: Box::new([]),
    }));
    let core::CallPart::Delegated(numbers) = &world.answers.last().expect("delegation answered").2 else {
        panic!("admitted delegate")
    };
    let child = numbers[0];
    world.end(tasks::End::Parked, 0);
    assert_eq!(world.assigned.as_ref().expect("delegate assigned").task, child);
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(4002)),
        key: world.call_key(1),
        action: core::NamedAction::Message {
            target: parent,
            kind: tasks::MessageKind::Question,
            words: b"what next?".as_slice().into(),
        },
    }));
    assert!(matches!(world.answers.last(), Some((_, _, core::CallPart::Sent { .. }))));
}

#[test]
fn what_each_live_task_waits_on_matches_the_reference_from_the_store() {
    let mut world = world(1202);
    let parent = world.assigned.as_ref().expect("chat").task;
    delegate_question(&mut world, parent);
    request(&mut world, 1, people::Ask::Stop { project: 1, task: parent });
    let live: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) => Some(row.number),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .collect();
    for task in live {
        let expected = jig_core_world::reference::waits_on(&world.store.rows, task);
        ask(&mut world, 10 + task, reads::Read::Task { task });
        let reads::Answer::Document(document) = latest(&world) else { panic!("live task {task}") };
        let documents::Document::Task(value) = document.as_ref() else { panic!("task document") };
        assert_eq!(value.waits_on, expected, "task {task}");
    }
}

#[test]
fn me_reads_the_pool_left_in_its_creating_bounds() {
    let mut world = world(1203);
    ask(&mut world, 20, reads::Read::Frame);
    let reads::Answer::Document(document) = latest(&world) else { panic!("me document") };
    let documents::Document::Frame(value) = document.as_ref() else { panic!("me") };
    let project = value.projects.iter().find(|project| project.project == 1).expect("project");
    let Some(core::acts::Bounds::Goal { pool_left, .. }) = &project.creating else { panic!("creating") };
    assert_eq!(*pool_left, 200);
}
