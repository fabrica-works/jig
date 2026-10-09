use jig_core as core;
use jig_core_people as people;
use jig_core_tasks as tasks;
use jig_core_world::effects::World;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

fn lines(world: &World, task: u64) -> Vec<tasks::ActivityLine> {
    world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(row))) if row.task == task => {
                Some(row.clone())
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .collect()
}

#[test]
#[expect(clippy::too_many_lines, reason = "one real-store story keeps its complete run and archive assertions")]
fn a_chat_s_activity_holds_the_person_s_words_its_turns_and_its_delegate_s_result() {
    let (configuration, limits) = jig_core_world::faults::plan_fixture(506);
    let mut world = World::configured(506, false, configuration, limits);
    let parent = world.assignments[0];
    world.turn(1, 2, None, b"the first committed turn");
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(5001)),
        sign_in: world.sign_in.expect("signed-in person"),
        key: [51; 16],
        ask: people::Ask::Say {
            project: 1,
            task: parent.0,
            words: b"the person's complete words".as_slice().into(),
            release: false,
        },
    })));
    let message = world
        .people_answers
        .iter()
        .rev()
        .find_map(|reply| match reply {
            people::Reply::Outcome(people::Outcome::Said { message, .. }) => Some(*message),
            people::Reply::SignedIn { .. }
            | people::Reply::SignedOut
            | people::Reply::Outcome(_)
            | people::Reply::Again(_)
            | people::Reply::Refused(_) => None,
        })
        .expect("numbered person message");
    world.turn(2, 5, Some(message), b"the second committed turn");
    let mut authority = world.assigned.as_ref().expect("assigned parent").run.authority.clone();
    authority.tools = tasks::Tools(0);
    authority.grants = Box::new([]);
    authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    authority.budget.spend = 20;
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(5002)),
        key: world.call_key(1),
        batch: Box::new([core::Delegate {
            executor: tasks::Executor::Agent { charter: 1 },
            spec: tasks::Spec {
                words: b"a delegate's work".as_slice().into(),
                parameters: Box::new([]),
                inputs: Box::new([]),
            },
            contract: tasks::Contract::Report { words: 128 },
            authority,
            symbolic_grants: Box::new([]),
            dependencies: Box::new([]),
            wake: tasks::WakePolicy::DEFAULT,
        }]),
        stubs: Box::new([]),
    }));
    let core::CallPart::Delegated(children) = &world.answers.last().expect("delegate answer").2 else {
        panic!("delegate was refused: {:?}", world.answers);
    };
    let child = *world.assignments.iter().find(|(task, _)| *task == children[0]).expect("delegate assigned");
    assert_ne!(parent.0, child.0);
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"the delegate's complete result".as_slice().into() },
            cancel_delegates: false,
        },
        1,
    );
    let heard = world
        .inbound
        .iter()
        .find(|word| word.from == tasks::Party::Task(child.0) && matches!(word.kind, tasks::MessageKind::Result(_)))
        .expect("delivered delegate result")
        .clone();
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: parent.0,
        attempt: parent.1,
        cumulative: 6,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
    });
    let activity = lines(&world, parent.0);
    for (index, row) in activity.iter().enumerate() {
        assert_eq!(row.position, u64::try_from(index).expect("activity count fits u64") + 1);
    }
    let semantic: Vec<_> =
        activity.iter().filter(|row| !matches!(row.line, tasks::Line::Milestone { .. })).map(|row| &row.line).collect();
    assert_eq!(
        semantic,
        vec![
            &tasks::Line::RunBegan { attempt: parent.1, why: tasks::Why::Made, resumed: false },
            &tasks::Line::Turn { attempt: parent.1, turn: 1, read: None },
            &tasks::Line::Words {
                message,
                by: heard_person(&world),
                kind: tasks::MessageKind::Words,
                words: b"the person's complete words".as_slice().into()
            },
            &tasks::Line::Turn { attempt: parent.1, turn: 2, read: Some(message) },
            &tasks::Line::Heard(heard),
            &tasks::Line::RunEnded { attempt: parent.1, end: tasks::EndKind::Finished, spent: 6 },
        ]
    );
    for (turn, bytes) in [(1, b"the first committed turn".as_slice()), (2, b"the second committed turn".as_slice())] {
        let key = root::Key::Core(core::Key::Core(core::CoreKey::Turn { task: parent.0, attempt: parent.1, turn }));
        let root::Record::Core(core::Record::Core(core::CoreRecord::Turn(row))) = &world.store.rows[&key] else {
            panic!("turn archive")
        };
        assert_eq!(row.transcript.as_ref(), bytes);
        assert!(world.commits.iter().any(|commit| commit.iter().any(|write| matches!(write, root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Turn(row)))) if row.task == parent.0 && row.attempt == parent.1 && row.turn == turn)) && commit.iter().any(|write| matches!(write, root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(row)))) if row.task == parent.0 && row.line == (tasks::Line::Turn { attempt: parent.1, turn, read: if turn == 1 { None } else { Some(message) } })) )));
    }
    assert!(world.commits.iter().any(|commit| {
        let terminal = commit.iter().any(|write| match write {
            root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(row)))) => {
                row.task == parent.0 && row.attempt == parent.1 && row.cumulative == 6
            }
            root::Write::Save(_) | root::Write::Erase(_) => false,
        });
        let line = commit.iter().any(|write| match write {
            root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(row)))) => {
                row.task == parent.0
                    && row.line == tasks::Line::RunEnded { attempt: parent.1, end: tasks::EndKind::Finished, spent: 6 }
            }
            root::Write::Save(_) | root::Write::Erase(_) => false,
        });
        terminal && line
    }));
    let key = root::Key::Core(core::Key::Tasks(tasks::Key::Ended(parent.0)));
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(ended))) = &world.store.rows[&key] else {
        panic!("ended task")
    };
    assert_eq!(ended.activity, activity.last().expect("final activity").position);
    world.restart_with(jig_core_world::faults::plan_fixture(506).0);
    assert_eq!(lines(&world, parent.0), activity);
}

fn heard_person(world: &World) -> tasks::Party {
    let person = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::People(people::Stored::SignIn { person, .. })) => Some(*person),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("retained sign-in");
    tasks::Party::Person(person)
}
