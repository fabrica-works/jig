use jig_core_world::watches::{open, world};
use jig_domain_core as core;
use jig_domain_views as views;
use jig_test_domain as root;
use skein_lib::Token;

fn report(world: &mut jig_core_world::effects::World, task: u64, attempt: u64, retry: u32, text: &[u8]) {
    world.send(root::Event::Core(core::Event::RunText {
        run: Token::new(task),
        attempt: Token::new(attempt),
        turn: 1,
        retry,
        offset: 0,
        text: text.into(),
        call: None,
    }));
}

#[test]
fn actual_engine_assignment_keeps_an_unwatched_retry_until_its_watch_opens() {
    let mut world = world(799, true);
    let assignment = world.assigned.as_ref().expect("actual live assignment");
    let (task, attempt) = (assignment.task, assignment.attempt);
    let durable = world.store.applied;
    report(&mut world, task, attempt, 0, b"old");
    report(&mut world, task, attempt, 1, b"");
    open(&mut world, 5, core::documents::Subject::Run { task });
    let party = world.watches.as_ref().expect("independent consumer");
    let stream = party.streams.iter().find(|stream| stream.watcher == 5).expect("mandatory empty retry");
    assert_eq!(
        (stream.turn, stream.from, stream.to, stream.text.as_ref()),
        (views::TurnName { attempt, turn: 1, retry: 1 }, 0, 0, b"".as_slice())
    );
    assert_eq!(world.store.applied, durable);
    party.finish();
}

#[test]
fn an_engine_watch_reopens_from_the_retained_retry_after_its_sole_watcher_leaves() {
    let mut world = world(800, true);
    let assignment = world.assigned.as_ref().expect("actual live assignment");
    let (task, attempt) = (assignment.task, assignment.attempt);
    open(&mut world, 5, core::documents::Subject::Run { task });
    let durable = world.store.applied;
    report(&mut world, task, attempt, 0, b"earlier");
    world.send(root::Event::Core(core::Event::StreamDelivered { watcher: Token::new(5) }));
    world.send(root::Event::Core(core::Event::Unwatch { watcher: Token::new(5) }));
    report(&mut world, task, attempt, 1, b"new prefix");
    open(&mut world, 6, core::documents::Subject::Run { task });
    let party = world.watches.as_ref().expect("independent consumer");
    assert!(party.endings.iter().any(|(watcher, end)| *watcher == 5 && *end == views::End::Unwatched));
    let stream = party.streams.iter().find(|stream| stream.watcher == 6).expect("reopened current retry");
    assert_eq!(
        (stream.turn, stream.from, stream.to, stream.text.as_ref(), stream.cut),
        (views::TurnName { attempt, turn: 1, retry: 1 }, 0, 10, b"new prefix".as_slice(), false)
    );
    assert_eq!(world.store.applied, durable, "provisional lifecycle is independent of journal commits");
    party.finish();
}

fn supplied_call(owner: u64, id: &[u8]) -> views::Call {
    views::Call {
        identity: views::CallIdentity { owner: Token::new(owner), sequence: 8, retry: 2, id: id.into() },
        tool: b"wait".as_slice().into(),
        input: b"{}".as_slice().into(),
        cut: false,
        began: None,
    }
}
fn current_run(world: &jig_core_world::effects::World, watcher: u64) -> &core::documents::Run {
    let core::documents::Document::Run(run) = jig_core_world::watches::latest(world, watcher) else {
        panic!("whole public run document")
    };
    run
}

#[test]
fn an_actual_assignment_opens_a_call_only_document_with_whole_opaque_identity() {
    let mut world = world(801, true);
    let assignment = world.assigned.as_ref().expect("actual live assignment");
    let (task, attempt) = (assignment.task, assignment.attempt);
    let durable = world.store.applied;
    let call = supplied_call(u64::MAX, &[0xff, 0, 0x80]);
    world.send(root::Event::Core(core::Event::RunCall {
        run: Token::new(task),
        attempt: Token::new(attempt),
        call: call.clone(),
    }));
    open(&mut world, 5, core::documents::Subject::Run { task });
    let run = current_run(&world, 5);
    assert_eq!((run.task, run.attempt, run.committed_turn), (task, Some(attempt), 0));
    assert!(run.writing.is_none());
    let current = run.call.as_ref().expect("call-only retained independent value");
    assert_eq!(current.identity, call.identity);
    assert_eq!((&current.tool, &current.input, current.cut), (&call.tool, &call.input, false));
    assert!(current.began.is_some());
    assert!(
        world.watches.as_ref().expect("independent watch consumer").streams.iter().all(|stream| stream.watcher != 5)
    );
    assert_eq!(world.store.applied, durable, "current content makes no durable decision");
    world.watches.as_ref().expect("independent watch consumer").finish();
}

#[test]
fn a_real_committed_turn_keeps_current_calls_without_main_writing_on_reopen() {
    let mut world = world(802, true);
    let assignment = world.assigned.as_ref().expect("actual live assignment");
    let (task, attempt) = (assignment.task, assignment.attempt);
    report(&mut world, task, attempt, 0, b"accepted main text");
    let main = supplied_call(91, b"same");
    world.send(root::Event::Core(core::Event::RunCall {
        run: Token::new(task),
        attempt: Token::new(attempt),
        call: main.clone(),
    }));
    let before = world.store.applied;
    world.turn(1, 0, None, b"real committed turn bytes");
    assert!(world.store.applied > before, "actual turn is durable before its document");
    open(&mut world, 5, core::documents::Subject::Run { task });
    let run = current_run(&world, 5);
    assert_eq!(run.committed_turn, 1);
    assert!(run.writing.is_none());
    assert_eq!(run.call.as_ref().expect("current postcommit call").identity, main.identity);
    let first_wall = run.call.as_ref().expect("current postcommit call").began;
    world.send(root::Event::Core(core::Event::Unwatch { watcher: Token::new(5) }));
    let nested = supplied_call(92, b"same");
    let durable = world.store.applied;
    world.send(root::Event::Core(core::Event::RunCall {
        run: Token::new(task),
        attempt: Token::new(attempt),
        call: nested.clone(),
    }));
    open(&mut world, 6, core::documents::Subject::Run { task });
    let reopened = current_run(&world, 6);
    assert_eq!(reopened.committed_turn, 1);
    assert!(reopened.writing.is_none());
    assert_eq!(reopened.call.as_ref().expect("current nested call").identity, nested.identity);
    assert_ne!(reopened.call.as_ref().expect("current nested call").identity, main.identity);
    assert!(first_wall.is_some());
    assert!(reopened.call.as_ref().expect("current nested call").began.is_some());
    assert_eq!(world.store.applied, durable);
    world.watches.as_ref().expect("independent watch consumer").finish();
}
