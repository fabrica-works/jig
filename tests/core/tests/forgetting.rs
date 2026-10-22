//! A task's retention class decides what its reading erases (people.md, 6).
const CONVERSATION_MARK: u8 = 63;
use jig_core_world::{
    effects::{Cut, World},
    stories::reads::{forgettable_world, request, request_event, world},
};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

const TURN: u32 = 1;
const PARTY: u64 = 1;
const MARK_TURNS: u8 = 61;
const MARK_RESULTS: u8 = 62;

fn ended(task: u64) -> root::Key {
    root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))
}

fn finish(world: &mut World) -> (u64, u64) {
    let task = world.assigned.as_ref().expect("conversation").task;
    world.turn(TURN, 1, None, b"reply");
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        1,
    );
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) = &world.store.rows[&ended(task)] else {
        panic!("ended conversation")
    };
    (task, row.result_position)
}

#[test]
fn a_conversation_forgotten_once_read_leaves_no_row_once_read() {
    let mut world = forgettable_world(830);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } });
    assert!(world.store.rows.contains_key(&ended(task)));
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(!world.store.rows.contains_key(&ended(task)));
    for key in world.store.rows.keys() {
        if let root::Key::Core(core::Key::Tasks(
            tasks::Key::EndedByParty { task: number, .. } | tasks::Key::EndedByProject { task: number, .. },
        )) = key
        {
            assert_ne!(*number, task);
        }
    }
}

#[test]
fn a_kept_chat_stays_after_its_party_read_it() {
    let mut world = world(831);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } });
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(world.store.rows.contains_key(&ended(task)));
    assert!(world.store.rows.keys().any(|key| matches!(key, root::Key::Core(core::Key::Tasks(tasks::Key::EndedByParty { task: number, .. })) if *number == task)));
}

#[test]
fn a_forgettable_conversation_read_in_the_other_order_is_forgotten() {
    let mut world = forgettable_world(832);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(world.store.rows.contains_key(&ended(task)));
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } });
    assert!(!world.store.rows.contains_key(&ended(task)));
}

#[test]
fn a_results_mark_forgets_in_the_commit_that_moves_its_position() {
    let mut world = forgettable_world(833);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } });
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    let erases: Vec<_> =
        world.commits.iter().filter(|commit| commit.contains(&root::Write::Erase(ended(task)))).collect();
    assert_eq!(erases.len(), 1);
    assert!(erases[0].contains(&root::Write::Save(root::Record::Core(core::Record::People(
        people::Stored::ReadResults { party: PARTY, through },
    )))));
}

#[test]
fn a_conversation_read_before_a_restart_is_forgotten_by_its_results_mark() {
    let mut world = forgettable_world(837);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } });
    world.restart(837, false);
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(!world.store.rows.contains_key(&ended(task)));
    assert!(
        !world
            .store
            .rows
            .contains_key(&root::Key::Core(core::Key::People(people::Key::EndedRead { task, party: PARTY })))
    );
}
#[test]
fn a_results_mark_is_busy_while_a_conversation_loads() {
    let mut world = forgettable_world(838);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } });
    world.store_delay = 3;
    let chat = jig_core_world::stories::reads::request_event(
        &world,
        CONVERSATION_MARK,
        people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } },
    );
    let results = jig_core_world::stories::reads::request_event(
        &world,
        MARK_RESULTS,
        people::Ask::MarkSeen { seen: people::Seen::Results { through } },
    );
    let other_chat = request_event(
        &world,
        MARK_TURNS,
        people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } },
    );
    let replies = world.people_answers.len();
    world.send_all(vec![chat, other_chat, results]);
    assert_eq!(
        world.people_answers[replies..]
            .iter()
            .filter(|reply| **reply == people::Reply::Again(people::Again::Busy))
            .count(),
        2
    );
    assert!(
        world.people_answers.contains(&people::Reply::Again(people::Again::Busy)),
        "answers: {:?}; rejected inputs: {:?}",
        world.people_answers,
        world.busy_events
    );
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(!world.store.rows.contains_key(&ended(task)));
    assert!(
        !world
            .store
            .rows
            .contains_key(&root::Key::Core(core::Key::People(people::Key::EndedRead { task, party: PARTY })))
    );
}

fn assert_no_mark_answers(world: &World) {
    for key in [MARK_TURNS, MARK_RESULTS] {
        assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Answer(
            people::RequestKey { person: PARTY, key: [key; 16] },
        )))));
    }
    assert!(!world.commits.iter().flatten().any(|write| matches!(write,
        root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Answer { key, .. })))
            if key.key == [MARK_TURNS; 16] || key.key == [MARK_RESULTS; 16]
    )));
}

#[test]
fn the_same_seen_key_after_forgetting_rechecks_absence_without_writing_history() {
    let mut world = forgettable_world(839);
    let (task, through) = finish(&mut world);
    let seen = people::Seen::Conversation { task, through: 1 };
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen });
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::MarkedSeen { seen })));
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    assert!(!world.store.rows.contains_key(&ended(task)));
    assert_no_mark_answers(&world);
    for restart in [false, true] {
        if restart {
            world.restart(839, false);
            assert!(world.domain.ready());
        }
        let rows = world.store.rows.clone();
        let commits = world.commits.len();
        let replies = world.people_answers.len();
        request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen });
        assert_eq!(
            &world.people_answers[replies..],
            &[people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown))]
        );
        assert_eq!(world.commits.len(), commits, "reprocessing Unknown makes no commit");
        assert_eq!(world.store.rows, rows, "no position or history is recreated");
        assert_no_mark_answers(&world);
    }
}

#[test]
fn a_seen_mark_and_its_waiter_wait_for_the_atomic_forgetting_commit() {
    let mut world = forgettable_world(840);
    let (task, through) = finish(&mut world);
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
    let seen = people::Seen::Conversation { task, through: 1 };
    let first = request_event(&world, MARK_TURNS, people::Ask::MarkSeen { seen });
    let second = root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(40_000)),
        sign_in: world.sign_in.expect("signed in"),
        key: [MARK_TURNS; 16],
        ask: people::Ask::MarkSeen { seen },
    }));
    let replies = world.people_answers.len();
    let number = world.store.applied + 1;
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    world.cut = Some(Cut::Submitted(number));
    world.send_all(vec![first, second]);
    assert_eq!(world.reached_cut, Some(Cut::Submitted(number)));
    assert_eq!(world.people_answers.len(), replies, "no reply before durable submission");
    assert!(world.store.rows.contains_key(&ended(task)));
    let writes = world.commits.last().expect("mark decision");
    assert!(writes.contains(&root::Write::Erase(ended(task))));
    assert!(writes.contains(&root::Write::Save(root::Record::Core(core::Record::People(people::Stored::EndedRead {
        task,
        party: PARTY,
        through: 1
    },)))));
    world.drain();
    assert_eq!(world.store.applied, number);
    assert!(!world.store.rows.contains_key(&ended(task)));
    assert_eq!(world.people_answers.len(), replies, "durable but unacknowledged commit still holds both replies");
    let commits = world.commits.len();
    let results = people::Seen::Results { through };
    request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: results });
    assert_eq!(world.commits.len(), commits, "an unchanged results mark writes nothing");
    assert_eq!(world.people_answers.len(), replies, "a write-free Seen still waits behind the prior commit");
    let acknowledged = world.store.release_held().expect("held mark commit");
    world.send(root::Event::Committed { number: acknowledged, fill: 0 });
    assert_eq!(
        &world.people_answers[replies..],
        &[
            people::Reply::Outcome(people::Outcome::MarkedSeen { seen }),
            people::Reply::Outcome(people::Outcome::MarkedSeen { seen }),
            people::Reply::Outcome(people::Outcome::MarkedSeen { seen: results }),
        ]
    );
    world.drain();
    assert_eq!(world.people_answers.len(), replies + 3, "one terminal per caller");
    assert_no_mark_answers(&world);
}

#[test]
#[should_panic(expected = "Seen reply preceded its durable position/forget commit")]
fn the_seen_observer_rejects_a_reply_before_the_position_commit() {
    let mut world = forgettable_world(843);
    let (task, _) = finish(&mut world);
    let seen = people::Seen::Conversation { task, through: 1 };
    let number = world.store.applied + 1;
    world.cut = Some(Cut::Submitted(number));
    request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen });
    assert_eq!(world.reached_cut, Some(Cut::Submitted(number)));
    let mut premature = root::Delivery::Core(core::Held::PeopleReply {
        to: skein_lib::ReplyTo::new(skein_lib::Token::new(30_000 + u64::from(MARK_TURNS))),
        reply: people::Reply::Outcome(people::Outcome::MarkedSeen { seen }),
    });
    world.observer.delivered(0, &mut premature);
}

#[test]
fn a_seen_reply_can_precede_a_later_marks_commit_for_the_same_position() {
    for conversation in [false, true] {
        let mut world = world(844);
        let task = world.assigned.as_ref().expect("conversation").task;
        world.turn(1, 1, None, b"first reply");
        world.turn(2, 2, None, b"second reply");
        let first = if conversation {
            people::Seen::Conversation { task, through: 1 }
        } else {
            people::Seen::Results { through: 1 }
        };
        let second = if conversation {
            people::Seen::Conversation { task, through: 2 }
        } else {
            people::Seen::Results { through: 2 }
        };
        let replies = world.people_answers.len();
        let number = world.store.applied + 1;
        world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
        request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen: first });
        assert_eq!(world.store.applied, number);
        assert_eq!(world.people_answers.len(), replies, "first reply waits for acknowledgement");

        world.cut = Some(Cut::Submitted(number + 1));
        request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: second });
        assert_eq!(world.reached_cut, Some(Cut::Submitted(number + 1)));
        assert_eq!(world.store.applied, number, "the later mark is not durable");
        assert_eq!(world.people_answers.len(), replies);

        let mut observer = world.observer.clone();
        let mut premature = root::Delivery::Core(core::Held::PeopleReply {
            to: skein_lib::ReplyTo::new(skein_lib::Token::new(30_000 + u64::from(MARK_RESULTS))),
            reply: people::Reply::Outcome(people::Outcome::MarkedSeen { seen: second }),
        });
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            observer.delivered(0, &mut premature);
        }))
        .expect_err("the later reply must still require its own commit");
        assert_eq!(
            failure.downcast_ref::<&str>().copied(),
            Some("Seen reply preceded its durable position/forget commit")
        );

        let acknowledged = world.store.release_held().expect("first mark acknowledgement");
        assert_eq!(acknowledged, number);
        world.cut = Some(Cut::Durable(number + 1));
        world.send(root::Event::Committed { number: acknowledged, fill: 0 });
        assert_eq!(world.reached_cut, Some(Cut::Durable(number + 1)));
        assert_eq!(
            &world.people_answers[replies..],
            &[people::Reply::Outcome(people::Outcome::MarkedSeen { seen: first })],
            "the first reply is released before the later mark becomes durable"
        );
        world.drain();
        assert_eq!(
            &world.people_answers[replies..],
            &[
                people::Reply::Outcome(people::Outcome::MarkedSeen { seen: first }),
                people::Reply::Outcome(people::Outcome::MarkedSeen { seen: second }),
            ]
        );
    }
}

#[test]
fn an_unchanged_ended_seen_mark_still_requires_its_own_position_commit() {
    for through in [0, 1] {
        let mut world = world(845);
        let (task, _) = finish(&mut world);
        request(
            &mut world,
            MARK_TURNS,
            people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } },
        );
        let seen = people::Seen::Conversation { task, through };
        let replies = world.people_answers.len();
        let number = world.store.applied + 1;
        world.cut = Some(Cut::Submitted(number));
        request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen });
        assert_eq!(world.reached_cut, Some(Cut::Submitted(number)));
        assert_eq!(world.store.applied, number - 1);
        assert_eq!(world.people_answers.len(), replies);
        assert!(world.commits.last().expect("unchanged ended mark").contains(&root::Write::Save(root::Record::Core(
            core::Record::People(people::Stored::EndedRead { task, party: PARTY, through: 1 })
        ))));

        let mut observer = world.observer.clone();
        let mut premature = root::Delivery::Core(core::Held::PeopleReply {
            to: skein_lib::ReplyTo::new(skein_lib::Token::new(30_000 + u64::from(MARK_RESULTS))),
            reply: people::Reply::Outcome(people::Outcome::MarkedSeen { seen }),
        });
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            observer.delivered(0, &mut premature);
        }))
        .expect_err("even a lower/equal ended mark rewrites the position");
        assert_eq!(
            failure.downcast_ref::<&str>().copied(),
            Some("Seen reply preceded its durable position/forget commit")
        );
        world.drain();
        assert_eq!(&world.people_answers[replies..], &[people::Reply::Outcome(people::Outcome::MarkedSeen { seen })]);
    }
}

#[test]
fn seen_forgetting_restores_only_the_committed_position_and_absence_at_both_crash_cuts() {
    for cut_durable in [false, true] {
        let mut world = forgettable_world(841);
        let (task, through) = finish(&mut world);
        request(&mut world, MARK_RESULTS, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
        let seen = people::Seen::Conversation { task, through: 1 };
        let replies = world.people_answers.len();
        let number = world.store.applied + 1;
        let cut = if cut_durable { Cut::Durable(number) } else { Cut::Submitted(number) };
        world.cut = Some(cut);
        request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen });
        assert_eq!(world.reached_cut, Some(cut));
        assert_eq!(world.people_answers.len(), replies);
        world.restart(841, false);
        assert!(world.domain.ready());
        assert_eq!(world.store.rows.contains_key(&ended(task)), !cut_durable);
        assert_eq!(
            world.store.rows.get(&root::Key::Core(core::Key::People(people::Key::ReadResults { party: PARTY }))),
            Some(&root::Record::Core(core::Record::People(people::Stored::ReadResults { party: PARTY, through })))
        );
        let replies = world.people_answers.len();
        request(&mut world, MARK_TURNS, people::Ask::MarkSeen { seen });
        let expected = if cut_durable {
            people::Outcome::Refused(people::Refusal::Unknown)
        } else {
            people::Outcome::MarkedSeen { seen }
        };
        assert_eq!(&world.people_answers[replies..], &[people::Reply::Outcome(expected)]);
        assert!(!world.store.rows.contains_key(&ended(task)));
        assert_no_mark_answers(&world);
    }
}
