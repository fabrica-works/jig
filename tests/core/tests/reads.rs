use jig_core_world::{
    effects::Cut,
    reads::{Fence, ask, complete, latest},
    stories::reads::{conversation, request, world},
};
use jig_domain_core::{self as core, documents, reads};
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
    a_read_after_the_sign_in_ended_is_answered_signed_out();
    a_read_whose_sign_in_ends_while_it_loads_is_signed_out();
    jig_core_world::stories::reads::a_backward_wall_correction_keeps_reply_paging_whole_after_restart();
}

#[test]
fn conversation_owner_replay_keeps_every_retained_page_after_durability_and_restart() {
    jig_core_world::stories::reads::conversation_owner_replay_keeps_every_retained_page_after_durability_and_restart();
}

#[test]
fn updates_page_from_the_index_newest_first_after_restart() {
    let mut world = world(780);
    let task = world.assigned.as_ref().expect("chat").task;
    world.turn(1, 1, None, b"first reply");
    world.advance(skein_lib::Duration::from_secs(1));
    world.turn(2, 2, None, b"second reply");
    world.restart(780, false);
    let group = reads::Group::Updates;
    let filter = reads::Filter { project: Some(1), goal: None };
    let mut from = Some(reads::Cursor::Store { scope: reads::Scope::Inbox { group, filter }, after: None });
    let mut found = Vec::new();
    for page in 0..4 {
        ask(&mut world, 700 + page, reads::Read::Inbox { group, filter, from: from.take() });
        let reads::Answer::Page(value) = latest(&world) else { panic!("updates page") };
        for row in &value.rows {
            let reads::PageRow::Inbox(entry) = row else { panic!("update") };
            assert!(matches!(entry.task, documents::TaskLink::Retained { ref chip, .. } if chip.number == task));
            let documents::InboxObject::Reply { message } = entry.object else { panic!("reply") };
            found.push(message);
        }
        from.clone_from(&value.next);
        if from.is_none() {
            break;
        }
    }
    assert_eq!(found, [2, 1]);
}

#[test]
fn a_forgotten_chats_result_leaves_the_updates_index() {
    let mut world = world(781);
    let task = world.assigned.as_ref().expect("chat").task;
    world.turn(1, 1, None, b"reply");
    request(&mut world, 71, people::Ask::MarkSeen { seen: people::Seen::Chat { task, through: 1 } });
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        1,
    );
    let key = world.store.rows.keys().find(|key| matches!(key, root::Key::Core(core::Key::Tasks(tasks::Key::Update { task: number, object: tasks::UpdateObject::Result, .. })) if number.value() == task)).expect("result index").clone();
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(ended))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))]
    else {
        panic!("ended chat")
    };
    let through = ended.result_position;
    let reply = world.store.rows.keys().find(|key| matches!(key, root::Key::Core(core::Key::Tasks(tasks::Key::Update { task: number, object: tasks::UpdateObject::Reply { .. }, .. })) if number.value() == task)).expect("reply index").clone();
    request(&mut world, 70, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(!world.store.rows.contains_key(&key));
    assert!(world.store.rows.contains_key(&reply));
    let group = reads::Group::Updates;
    let filter = reads::Filter { project: Some(1), goal: None };
    let scope = reads::Scope::Inbox { group, filter };
    ask(&mut world, 730, reads::Read::Inbox { group, filter, from: Some(reads::Cursor::Store { scope, after: None }) });
    let reads::Answer::Page(page) = latest(&world) else { panic!("forgotten updates page") };
    assert_eq!(page.rows.len(), 0);
    assert_eq!(page.next, None);
}

#[test]
fn the_results_group_pages_results_only_from_the_index() {
    let mut world = world(782);
    world.turn(1, 1, None, b"reply");
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        1,
    );
    world.restart(782, false);
    let scope =
        reads::Scope::Inbox { group: reads::Group::Results, filter: reads::Filter { project: Some(1), goal: None } };
    ask(
        &mut world,
        720,
        reads::Read::Inbox {
            group: reads::Group::Results,
            filter: reads::Filter { project: Some(1), goal: None },
            from: Some(reads::Cursor::Store { scope, after: None }),
        },
    );
    let reads::Answer::Page(value) = latest(&world) else { panic!("results page") };
    assert_eq!(value.rows.len(), 1);
    let reads::PageRow::Inbox(entry) = value.rows.get(0).expect("result") else { panic!("inbox result") };
    assert!(matches!(entry.object, documents::InboxObject::Result { .. }));
}

fn ended_chats(world: &mut jig_core_world::effects::World) -> Vec<u64> {
    let mut chats = Vec::new();
    for number in 0..3 {
        if number != 0 {
            request(world, 80 + number, people::Ask::StartChat { project: 1, words: b"next".as_slice().into() });
        }
        chats.push(world.assigned.as_ref().expect("chat").task);
        world.end(
            tasks::End::Finished {
                result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
                cancel_delegates: false,
            },
            0,
        );
    }
    chats
}
fn conversation_numbers(answer: &reads::Answer) -> Vec<u64> {
    let reads::Answer::Page(page) = answer else { panic!("conversation page") };
    page.rows
        .iter()
        .map(|row| {
            let reads::PageRow::Conversation(row) = row else { panic!("conversation") };
            row.chip.number
        })
        .collect()
}
#[test]
fn conversations_page_both_ways_from_the_index() {
    let mut world = world(783);
    let chats = ended_chats(&mut world);
    world.restart(783, false);
    ask(
        &mut world,
        740,
        reads::Read::Conversations {
            from: Some(reads::Cursor::Store { scope: reads::Scope::Conversations, after: None }),
        },
    );
    assert_eq!(conversation_numbers(latest(&world)), [chats[2], chats[1]]);
    let reads::Answer::Page(first) = latest(&world) else { panic!("first page") };
    let from = first.next.clone();
    ask(&mut world, 741, reads::Read::Conversations { from });
    assert_eq!(conversation_numbers(latest(&world)), [chats[0]]);
    let reads::Answer::Page(last) = latest(&world) else { panic!("last page") };
    let from = last.before.clone();
    ask(&mut world, 742, reads::Read::Conversations { from });
    assert_eq!(conversation_numbers(latest(&world)), [chats[2], chats[1]]);
}
#[test]
fn a_forgotten_chat_leaves_both_conversation_families() {
    let mut world = world(784);
    let task = world.assigned.as_ref().expect("chat").task;
    world.turn(1, 1, None, b"reply");
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        1,
    );
    let newest =
        root::Key::Core(core::Key::Tasks(tasks::Key::ConversationNewest { party: 1, task: tasks::Newest::of(task) }));
    let oldest = root::Key::Core(core::Key::Tasks(tasks::Key::ConversationOldest { party: 1, task }));
    assert!(world.store.rows.contains_key(&newest));
    assert!(world.store.rows.contains_key(&oldest));
    let through = result_position(&world, task);
    request(&mut world, 91, people::Ask::MarkSeen { seen: people::Seen::Chat { task, through: 1 } });
    request(&mut world, 92, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(!world.store.rows.contains_key(&newest));
    assert!(!world.store.rows.contains_key(&oldest));
    ask(
        &mut world,
        743,
        reads::Read::Conversations {
            from: Some(reads::Cursor::Store { scope: reads::Scope::Conversations, after: None }),
        },
    );
    assert_eq!(conversation_numbers(latest(&world)), []);
}

fn result_position(world: &jig_core_world::effects::World, task: u64) -> u64 {
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))]
    else {
        panic!("ended chat")
    };
    row.result_position
}

fn a_read_after_the_sign_in_ended_is_answered_signed_out() {
    let mut world = world(784);
    end_sign_in(&mut world);
    ask(&mut world, 800, reads::Read::Frame);
    assert_eq!(latest(&world), &reads::Answer::None(reads::Why::SignedOut));
}

fn a_read_whose_sign_in_ends_while_it_loads_is_signed_out() {
    let mut world = world(785);
    world.reads.as_mut().expect("party").hold_loads = true;
    ask(&mut world, 801, reads::Read::Task { task: 1000 });
    assert_eq!(world.reads.as_ref().expect("party").loads.len(), 1);
    end_sign_in(&mut world);
    complete(&mut world);
    assert_eq!(latest(&world), &reads::Answer::None(reads::Why::SignedOut));
    assert!(world.reads.as_ref().expect("party").loads.is_empty());
}

fn end_sign_in(world: &mut jig_core_world::effects::World) {
    world.send(root::Event::Core(core::Event::People(people::Event::SignOut {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(80_000)),
        sign_in: world.sign_in.expect("original sign-in"),
    })));
}

fn chat_task(world: &jig_core_world::effects::World) -> u64 {
    world.assigned.as_ref().expect("chat").task
}

fn message(
    world: &mut jig_core_world::effects::World,
    source: u64,
    target: u64,
    completion: u32,
    kind: tasks::MessageKind,
    words: &[u8],
) -> u64 {
    world.send(root::Event::Core(core::Event::NamedAction {
        to: skein_lib::ReplyTo::new(skein_lib::Token::new(40_000 + u64::from(completion))),
        key: run_key(world, source, completion),
        action: core::NamedAction::Message { target, kind, words: words.into() },
    }));
    match world.answers.last() {
        Some((_, _, core::CallPart::Sent { message })) => *message,
        other => panic!("message admitted: {other:?}"),
    }
}

fn question(world: &mut jig_core_world::effects::World, target: u64, completion: u32) -> u64 {
    message(world, chat_task(world), target, completion, tasks::MessageKind::Question, b"original question?")
}

fn finish_chat(world: &mut jig_core_world::effects::World, task: u64) {
    let key = run_key(world, task, 0);
    let read = live_task(world, task).inbox.last().map(|word| word.number);
    world.send(root::Event::Answer {
        read,
        refused: Box::new([]),
        channel: skein_lib::Token::new(7),
        task,
        attempt: key.attempt,
        cumulative: 0,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
    });
}

fn past(world: &jig_core_world::effects::World) -> &reads::Past {
    let reads::Answer::Historical(value) = latest(world) else { panic!("retained document: {:?}", latest(world)) };
    &value.value
}

#[test]
fn a_question_read_after_its_task_ended_finds_both_lines_by_key() {
    let (mut world, task) = question_world(791, None);
    let sender = chat_task(&world);
    let message = question(&mut world, task, 1);
    request(
        &mut world,
        91,
        people::Ask::AnswerQuestion { project: 1, task, question: message, words: b"the answer".as_slice().into() },
    );
    finish_chat(&mut world, sender);
    finish_chat(&mut world, task);
    ask(&mut world, 901, reads::Read::Question { task, message });
    let reads::Past::Question { question, answer, ending } = past(&world) else { panic!("question") };
    assert_eq!(question.words.as_ref(), b"original question?");
    let answer = answer.as_ref().expect("answer by key");
    assert!(matches!(&answer.line, tasks::Line::Words { kind: tasks::MessageKind::Answer { question }, words, .. }
        if *question == message && words.as_ref() == b"the answer"));
    assert!(ending.is_some());
}

#[test]
fn an_escalation_read_shows_its_latest_decision_and_its_hold() {
    let mut world = world(792);
    let task = chat_task(&world);
    request(&mut world, 92, people::Ask::Stop { project: 1, task });
    world.end(tasks::End::Parked, 0);
    request(
        &mut world,
        93,
        people::Ask::DecideEscalation {
            project: 1,
            task,
            revision: 1,
            decision: people::EscalationDecision::Reject { reason: b"stay stopped".as_slice().into() },
        },
    );
    request(&mut world, 94, people::Ask::Cancel { project: 1, task, reason: b"end".as_slice().into() });
    ask(&mut world, 902, reads::Read::Escalation { task });
    let reads::Past::Escalation { decision, hold, .. } = past(&world) else { panic!("escalation") };
    assert_eq!(decision.revision, 1);
    assert_eq!(decision.decision, people::EscalationDecision::Reject { reason: b"stay stopped".as_slice().into() });
    assert!(hold.is_some());
}

#[test]
fn a_task_proposal_read_finds_its_history_row_by_key() {
    let mut world = world(793);
    let task = chat_task(&world);
    let proposal = world.propose(1);
    world.decide_proposal(proposal, false, 95);
    finish_chat(&mut world, task);
    ask(&mut world, 903, reads::Read::Proposal { proposer: documents::Proposer::Task(task), proposal });
    let reads::Past::Proposal(value) = past(&world) else { panic!("proposal") };
    assert_eq!(value.number, proposal);
    assert!(matches!(&value.state, documents::ProposalHistoryState::Rejected { by, reason }
        if *by == tasks::Party::Person(1) && reason.bytes.as_ref() == b"not now"));
}

#[test]
fn a_question_older_than_the_kept_lines_reads_as_gone() {
    let bound = 2;
    let (mut world, task) = question_world(794, Some(bound));
    let sender = chat_task(&world);
    let oldest = question(&mut world, task, 1);
    let answer = reply(&mut world, task, sender, oldest, bound + 2);
    take_message(&mut world, task, 1, oldest);
    take_message(&mut world, sender, 1, answer);
    for number in 2..=bound + 1 {
        let message = question(&mut world, task, number);
        let answer = reply(&mut world, task, sender, message, number + bound + 1);
        take_message(&mut world, task, number, message);
        take_message(&mut world, sender, number, answer);
    }
    ask(&mut world, 904, reads::Read::Question { task, message: oldest });
    assert_eq!(latest(&world), &reads::Answer::None(reads::Why::Gone));
}

fn reply(world: &mut jig_core_world::effects::World, source: u64, target: u64, question: u64, completion: u32) -> u64 {
    message(world, source, target, completion, tasks::MessageKind::Answer { question }, b"answer")
}

fn run_key(world: &jig_core_world::effects::World, task: u64, completion: u32) -> core::CallKey {
    let (_, attempt) = world.assignments.iter().rev().find(|(number, _)| *number == task).expect("assigned task");
    core::CallKey { task, attempt: *attempt, completion, position: 0 }
}

fn take_message(world: &mut jig_core_world::effects::World, task: u64, turn: u32, message: u64) {
    world.send(root::Event::Turn {
        opens_window: turn == 1,
        refused: Box::new([]),
        channel: skein_lib::Token::new(7),
        task,
        attempt: run_key(world, task, 0).attempt,
        turn,
        cumulative: 0,
        read: Some(message),
        transcript: b"took the message".as_slice().into(),
    });
}

fn question_world(seed: u64, bound: Option<u32>) -> (jig_core_world::effects::World, u64) {
    let (configuration, mut limits) = jig_core_world::faults::plan_fixture(seed);
    if let Some(bound) = bound {
        limits.core.tasks.inbox_messages = bound;
        limits.core.tasks.run_inbox = bound;
    }
    let mut world = jig_core_world::effects::World::configured(seed, false, configuration, limits);
    world.reads = Some(jig_core_world::reads::Parties::new(world.store.applied));
    let parent = chat_task(&world);
    let mut authority = world.assigned.as_ref().expect("parent").run.authority.clone();
    authority.budget.spend = 20;
    authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: skein_lib::ReplyTo::new(skein_lib::Token::new(49_001)),
        key: run_key(&world, parent, 1),
        batch: Box::new([core::Delegate {
            executor: tasks::Executor::Agent { charter: 1 },
            spec: tasks::Spec {
                words: b"ask the parent".as_slice().into(),
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
    assert_ne!(chat_task(&world), parent, "the delegate runs");
    (world, parent)
}

#[test]
fn a_question_answered_by_its_task_reads_the_line_in_the_delegates_activity() {
    let (mut world, task) = question_world(795, None);
    let delegate = chat_task(&world);
    let message = question(&mut world, task, 1);
    let sent = reply(&mut world, task, delegate, message, 2);
    finish_chat(&mut world, delegate);
    finish_chat(&mut world, task);
    ask(&mut world, 905, reads::Read::Question { task, message });
    let reads::Past::Question { answer, .. } = past(&world) else { panic!("question") };
    let answer = answer.as_ref().expect("delegate's answer by key");
    assert_eq!(answer.task, delegate);
    assert!(
        matches!(&answer.line, tasks::Line::Words { message: number, kind: tasks::MessageKind::Answer { question }, words, .. }
        if *number == sent && *question == message && words.as_ref() == b"answer")
    );
}

fn live_task(world: &jig_core_world::effects::World, task: u64) -> &tasks::TaskRecord {
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))]
    else {
        panic!("live task")
    };
    row
}
