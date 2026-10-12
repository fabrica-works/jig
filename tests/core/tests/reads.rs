use jig_core_world::{
    effects::Cut,
    reads::{Fence, ask, complete, latest},
    stories::reads::{conversation, request, world},
};
use jig_domain_core::{documents, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

fn a_read_waits_for_the_commit_in_flight_before_it() {
    assert_eq!(Fence(7).answered(6), Err("read preceded its durable prerequisite"));
    assert_eq!(Fence(7).restarted(6), Err("crash retracted an observed read"));
    let mut world = world(761);
    let task = world.assigned.as_ref().expect("chat").task;
    let number = world.store.applied + 1;
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    world.cut = Some(Cut::Submitted(number));
    request(
        &mut world,
        1,
        people::Ask::Say { project: 1, task, words: b"not yet acknowledged".as_slice().into(), release: false },
    );
    assert_eq!(world.reached_cut, Some(Cut::Submitted(number)));
    assert!(world.store.applied < number);
    ask(&mut world, 1, reads::Read::Task { task });
    assert!(world.reads.as_ref().expect("party").receipts.is_empty());
    let acknowledged = world.store.release_held().expect("one held commit completion");
    world.send(root::Event::Committed { number: acknowledged });
    let receipt = &world.reads.as_ref().expect("party").receipts[0];
    assert_eq!(receipt.1, number);
    assert!(matches!(receipt.2, reads::Answer::Document(_)));
    world.restart(761, false);
    assert_eq!(world.reads.as_ref().expect("party").receipts.len(), 1, "the party keeps the read across a crash");
}

fn a_task_read_after_it_ended_says_how_and_by_whom() {
    let mut world = world(762);
    let task = world.assigned.as_ref().expect("chat").task;
    request(&mut world, 2, people::Ask::Cancel { project: 1, task, reason: b"withdrawn".as_slice().into() });
    world.end(tasks::End::Parked, 0);
    ask(&mut world, 2, reads::Read::Task { task });
    let reads::Answer::Historical(value) = latest(&world) else { panic!("retained task") };
    assert_eq!(value.by, Some(tasks::Party::Person(1)));
    assert!(value.at.is_some());
    let reads::Past::Task { document, result } = &value.value else { panic!("full retained task") };
    assert_eq!(document.chip.number, task);
    let documents::Ending::Cancelled { by, at, reason, .. } = &result.ending else { panic!("cancellation") };
    assert_eq!(*by, tasks::Party::Person(1));
    assert!(*at > skein_lib::Wall::EPOCH);
    assert_eq!(reason.bytes.as_ref(), b"withdrawn");
}

fn a_conversation_reads_whole_from_its_end_after_its_words_were_taken() {
    let mut world = world(763);
    let task = world.assigned.as_ref().expect("chat").task;
    let message = world.say(42);
    world.turn(1, 1, Some(message), b"took the complete words");
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"finished".as_slice().into() },
            cancel_delegates: false,
        },
        1,
    );
    let first = conversation(&mut world, task, 10);
    assert!(first.windows(2).all(|rows| rows[0].position > rows[1].position));
    let line = first
        .iter()
        .find(|line| matches!(line.line, tasks::Line::Words { message: number, .. } if number == message))
        .expect("the original words survived their unread entry");
    let tasks::Line::Words { words, .. } = &line.line else { panic!() };
    assert_eq!(words.as_ref(), [42]);
    world.restart(763, false);
    assert_eq!(
        conversation(&mut world, task, 50),
        first,
        "a crash cannot retract the immutable conversation the party read"
    );
}

fn an_inbox_longer_than_the_engine_holds_pages_from_the_store() {
    let mut world = world(764);
    let mut tasks = Vec::new();
    for number in 0..3 {
        if number != 0 {
            request(
                &mut world,
                10 + number,
                people::Ask::StartChat { project: 1, words: b"next chat".as_slice().into() },
            );
        }
        tasks.push(world.assigned.as_ref().expect("chat").task);
        world.end(
            tasks::End::Finished {
                result: tasks::TaskResult::Report { words: b"result".as_slice().into() },
                cancel_delegates: false,
            },
            0,
        );
    }
    let mut from = None;
    let mut found = Vec::new();
    for page in 0..8 {
        ask(
            &mut world,
            100 + page,
            reads::Read::Inbox {
                group: reads::Group::Updates,
                filter: reads::Filter { project: Some(1), goal: None },
                from: from.take(),
            },
        );
        let reads::Answer::Page(value) = latest(&world) else { panic!("inbox page") };
        assert!(value.rows.len() <= 2);
        for row in &value.rows {
            let reads::PageRow::Inbox(entry) = row else { panic!("inbox owner row") };
            assert_eq!(entry.project, Some(1));
            assert!(entry.at.is_some());
            let documents::TaskLink::Retained { chip, .. } = &entry.task else { panic!("kept ended identity") };
            found.push(chip.number);
        }
        from.clone_from(&value.next);
        if from.is_none() {
            break;
        }
    }
    tasks.reverse();
    assert_eq!(
        found, tasks,
        "the visible cached reference is joined and older results are neither omitted nor repeated"
    );
}

fn a_read_when_loads_are_full_is_busy() {
    let mut world = world(765);
    world.reads.as_mut().expect("party").hold_loads = true;
    ask(&mut world, 200, reads::Read::Task { task: 1000 });
    assert_eq!(world.reads.as_ref().expect("party").loads.len(), 1);
    ask(&mut world, 201, reads::Read::Task { task: 1001 });
    assert_eq!(latest(&world), &reads::Answer::None(reads::Why::Busy));
    assert_eq!(world.reads.as_ref().expect("party").loads.len(), 1, "busy keeps no second load");
    complete(&mut world);
    assert_eq!(latest(&world), &reads::Answer::None(reads::Why::Unknown));
    ask(&mut world, 202, reads::Read::Task { task: 1002 });
    assert_eq!(world.reads.as_ref().expect("party").loads.len(), 1, "the terminal freed its load slot");
    complete(&mut world);
}

#[test]
fn five_independent_read_stories_fit_the_shared_process_budget() {
    a_read_waits_for_the_commit_in_flight_before_it();
    a_task_read_after_it_ended_says_how_and_by_whom();
    a_conversation_reads_whole_from_its_end_after_its_words_were_taken();
    an_inbox_longer_than_the_engine_holds_pages_from_the_store();
    a_read_when_loads_are_full_is_busy();
    jig_core_world::stories::reads::a_backward_wall_correction_keeps_reply_paging_whole_after_restart();
}

#[test]
fn conversation_owner_replay_keeps_every_retained_page_after_durability_and_restart() {
    jig_core_world::stories::reads::conversation_owner_replay_keeps_every_retained_page_after_durability_and_restart();
}
