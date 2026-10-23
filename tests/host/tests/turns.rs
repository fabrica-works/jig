//!  Worker host peers: engine commits, a lossy link, and an
//! agent and workspace that settle through the host's requests.

use jig_domain_host::{AnsweredCall, DeliveryOutcome, Ending, Failure, Reason, RunFailure, SettledAnswer};
use jig_host_world::turn_world::World;
use skein_lib::{Duration, Token};

#[test]
fn a_host_call_reaches_the_engine_with_its_tool_write_flag_and_input() {
    let mut world = World::new();
    world.assign_conversation(Box::new([]), Box::new([]));
    world.call(true);
    let Some(call) = world.relayed_call() else { panic!("the engine received the call") };
    assert_eq!(&*call.name, b"call-one");
    assert_eq!(&*call.tool, b"inspect");
    assert!(call.writes);
    assert_eq!(&*call.input, b"input words");
    assert_eq!(call.deadline, Duration::from_nanos(37));
    assert_eq!(world.answered_call(), Some(&b"call-one"[..]));
}

#[test]
fn a_message_reaches_the_agent_with_its_label_and_words() {
    let mut world = World::new();
    world.assign_conversation(Box::new([]), Box::new([]));
    world.message_from();
    let Some(message) = world.received_message() else { panic!("the agent received the message") };
    assert_eq!(message.name, Token::new(17));
    assert_eq!(&*message.sender, b"requester");
    assert_eq!(&*message.words, b"please check");
    assert!(!message.finish_requested);
}

#[test]
fn a_finish_request_reaches_the_live_agent() {
    let mut world = World::new();
    world.assign_conversation(Box::new([]), Box::new([]));
    world.message_with_finish(true);
    let message = world.received_message().expect("finish request delivered");
    assert!(message.finish_requested);
    assert_eq!(&*message.words, b"please check");
}

#[test]
fn a_resumed_run_starts_with_its_turn_bodies_and_answered_calls() {
    let mut world = World::new();
    let turns: Box<[Box<[u8]>]> = vec![Box::from(&b"first"[..]), Box::from(&b"second"[..])].into_boxed_slice();
    let answered = vec![
        AnsweredCall {
            name: Box::from(&b"call one"[..]),
            tool: Box::from(&b"inspect"[..]),
            answer: SettledAnswer::Host { error: false, body: Box::from(&b"found"[..]) },
        },
        AnsweredCall {
            name: Box::from(&b"call two"[..]),
            tool: Box::from(&b"deliver"[..]),
            answer: SettledAnswer::Delivery {
                outcome: DeliveryOutcome::Delivered,
                evidence: Box::from(&b"receipt for workspace item"[..]),
            },
        },
    ];
    world.assign_conversation(turns, answered.into_boxed_slice());
    let Some(start) = world.started_state() else { panic!("the agent started") };
    assert_eq!(start.activation, 932);
    assert_eq!(&*start.turns, &[Box::from(&b"first"[..]), Box::from(&b"second"[..])]);
    assert_eq!(&start.answered[0].name[..], b"call one");
    assert_eq!(&start.answered[0].tool[..], b"inspect");
    assert_eq!(start.answered[0].answer, SettledAnswer::Host { error: false, body: Box::from(&b"found"[..]) });
    assert_eq!(
        start.answered[1].answer,
        SettledAnswer::Delivery {
            outcome: DeliveryOutcome::Delivered,
            evidence: Box::from(&b"receipt for workspace item"[..]),
        }
    );
}

#[test]
fn committed_turns_replay_after_busy_and_reconnect_then_restore_credit() {
    let mut world = World::new();
    world.assign(true);
    world.deliver();
    world.relay();
    world.turn(1, b"one");
    world.retry_busy(1);
    world.lose_contact();
    world.turn(2, b"two");
    assert!(world.reading(), "the hub ACK leaves room in the agent window");
    world.reconnect();
    world.commit_turn(1);
    assert!(world.reading(), "the exact commit returns one read credit");
    world.commit_turn(2);
    world.finish(2, true);
    world.acknowledge_answer();
    world.assert_settled();
    let stats = world.stats();
    assert!(stats.transmissions >= 4, "busy and hello resent the original turns: {stats:?}");
    assert_eq!((stats.commits, stats.turn_acks, stats.deliveries, stats.relays, stats.replies), (2, 2, 1, 1, 2));
    assert_eq!((stats.saves, stats.releases), (1, 1));
}

#[test]
fn an_answer_made_without_contact_follows_retained_turns_after_hello() {
    let mut world = World::new();
    world.assign(false);
    world.lose_contact();
    world.turn(1, b"kept during outage");
    world.finish(1, false);
    world.reconnect();
    let Some(answer) = world.answer() else { panic!("the host answered") };
    assert!(matches!(answer.ending, Ending::Ended { .. }));
    world.commit_turn(1);
    world.acknowledge_answer();
    world.assert_settled();
    assert_eq!((world.stats().saves, world.stats().releases), (0, 0));
}

#[test]
fn contact_cancel_waits_for_the_agent_then_saves_and_answers() {
    let mut world = World::new();
    world.assign(true);
    world.turn(1, b"kept before cancel");
    world.hold_stop();
    world.lose_contact();
    world.cancel_contact();
    assert!(world.answer().is_none(), "a live agent blocks workspace save and answer");
    world.gone();
    let Some(answer) = world.answer() else { panic!("the cancelled run answers after its agent is gone") };
    assert!(matches!(&answer.ending, Ending::Failed { failure: Failure::Cancelled(Reason::Contact), .. }));
    world.reconnect();
    world.commit_turn(1);
    world.acknowledge_answer();
    world.assert_settled();
    assert_eq!((world.stats().saves, world.stats().releases), (1, 1));
}

#[test]
fn shutdown_and_best_effort_facts_leave_no_live_run() {
    let mut world = World::new();
    world.assign(true);
    world.lose_contact();
    world.activity(count(1));
    world.activity(count(2));
    world.activity(count(3));
    assert_eq!(world.lost_facts(), 1, "the full fact queue drops and counts");
    world.shutdown();
    world.reconnect();
    world.acknowledge_answer();
    world.assert_settled();
    assert_eq!(world.stats().facts, 2);
}

#[test]
fn the_hello_carries_the_roots_stop_bound_workstreams_and_bounds() {
    let mut world = World::new();
    world.reconnect();
    let hello = world.hello().expect("hello first");
    assert_eq!(hello.slots, 1);
    assert_eq!(hello.stop_bound, Duration::from_secs(2));
    assert_eq!(&*hello.workstreams, &[3]);
    assert_eq!(hello.bounds, jig_host_world::fixtures::bounds());
}

#[test]
fn the_hello_lists_a_run_whose_answer_waits_as_answered() {
    let mut world = World::new();
    world.assign(false);
    world.finish(0, false);
    world.reconnect();
    assert_eq!(
        &*world.hello().expect("the root received the hello").hosting,
        &[jig_domain_host::Hosting {
            run: Token::new(31),
            attempt: Token::new(932),
            phase: jig_domain_host::Phase::Answered,
        }]
    );
}

#[test]
fn an_answer_is_sent_again_after_a_hello_until_acknowledged() {
    let mut world = World::new();
    world.assign(false);
    world.lose_contact();
    world.turn(1, b"kept");
    world.finish(1, false);
    assert!(world.link_words.is_empty());
    world.reconnect();
    assert_eq!(world.link_words, ["hello", "turn", "answer"]);
    world.reconnect();
    assert_eq!(&world.link_words[3..], &["hello", "turn", "answer"]);
    world.commit_turn(1);
    world.acknowledge_answer();
    world.reconnect();
    assert_eq!(&world.link_words[6..], &["hello"]);
    world.assert_settled();
}

#[test]
fn an_unacknowledged_answer_keeps_its_runs_slot() {
    let mut world = World::new();
    world.assign(false);
    world.finish(0, false);
    assert_eq!(world.another_assignment(), jig_domain_host::Refusal::Busy);
    assert_eq!(world.slots_taken(), 1);
    world.acknowledge_answer();
    assert_eq!(world.slots_taken(), 0);
}

#[test]
fn a_relayed_error_reaches_the_agent_as_an_error() {
    let mut world = World::new();
    world.assign(false);
    world.relay_answer(jig_domain_host::Relayed::Answer { error: true, text: Box::from(&b"denied"[..]) });
    assert_eq!(world.reply(), Some(&jig_domain_host::Reply::Relayed { error: true, text: Box::from(&b"denied"[..]) }));
}

#[test]
fn a_relayed_busy_reaches_the_agent_as_busy() {
    let mut world = World::new();
    world.assign(false);
    world.relay_answer(jig_domain_host::Relayed::Busy);
    assert_eq!(world.reply(), Some(&jig_domain_host::Reply::Busy));
}

#[test]
fn a_hello_before_an_acknowledgement_lists_the_answer_still_waiting() {
    let settings = jig_host_world::Settings {
        shutdown: Some(jig_host_world::Span::millis(20_000, 120_000)),
        ..jig_host_world::Settings::rough(220)
    };
    let mut world = jig_host_world::World::with_trace(&settings, false);
    world.run(200_000);
    assert!(world.stats().reports > 0);
}

#[test]
fn a_start_carries_its_messages_before_any_later_message() {
    let mut world = World::new();
    let messages = Box::new([
        jig_domain_host::StartMessage {
            name: Token::new(5),
            sender: Box::from(&b"person"[..]),
            words: Box::from(&b"first"[..]),
            finish_requested: false,
        },
        jig_domain_host::StartMessage {
            name: Token::new(6),
            sender: Box::from(&b"person"[..]),
            words: Box::from(&b"second"[..]),
            finish_requested: true,
        },
    ]);
    world.assign_start([0; 16], messages);
    let start = world.started_state().expect("agent started");
    assert_eq!(start.messages[0].name, Token::new(5));
    assert_eq!(&*start.messages[0].sender, b"person");
    assert_eq!(&*start.messages[0].words, b"first");
    assert_eq!(start.messages[1].name, Token::new(6));
    assert_eq!(&*start.messages[1].words, b"second");
    assert!(!start.messages[0].finish_requested);
    assert!(start.messages[1].finish_requested);
    assert!(world.received_message().is_none());
    world.message_from();
    assert_eq!(world.received_message().expect("later message").name, Token::new(17));
}

#[test]
fn a_start_carries_its_identity_unchanged() {
    let mut world = World::new();
    let identity = [4, 9, 2, 8, 1, 7, 3, 6, 0, 5, 10, 15, 11, 14, 12, 13];
    world.assign_start(identity, Box::new([]));
    assert_eq!(world.started_state().expect("agent started").identity, identity);
}

#[test]
fn a_start_beyond_the_inbox_is_refused_as_invalid_messages() {
    let mut world = World::new();
    let messages = (1..=5)
        .map(|name| jig_domain_host::StartMessage {
            name: Token::new(name),
            sender: Box::new([]),
            words: Box::new([]),
            finish_requested: false,
        })
        .collect();
    world.assign_start([0; 16], messages);
    assert!(world.started_state().is_none());
    assert_eq!(world.slots_taken(), 0);
    assert_eq!(
        world.answer().expect("refused").ending,
        Ending::Refused(jig_domain_host::Refusal::Invalid(jig_domain_host::Invalid::Messages))
    );
}

#[test]
fn a_start_message_beyond_the_byte_bound_is_refused() {
    let mut world = World::new();
    let message = jig_domain_host::StartMessage {
        name: Token::new(5),
        sender: Box::from([1; 32]),
        words: Box::from([2; 33]),
        finish_requested: false,
    };
    world.assign_start([0; 16], Box::new([message]));
    assert!(world.started_state().is_none());
    assert_eq!(
        world.answer().expect("refused").ending,
        Ending::Refused(jig_domain_host::Refusal::Invalid(jig_domain_host::Invalid::Messages))
    );
}

#[test]
fn a_refusal_rides_with_the_runs_next_turn() {
    let mut world = World::new();
    world.assign(false);
    world.relay_message(Token::new(1));
    world.refuse(Token::new(1), jig_domain_host::MessageRefusal::Full);
    world.turn(1, b"one");
    assert_eq!(
        &*world.received_turn(1).expect("turn received").refused,
        &[jig_domain_host::Refused { name: Token::new(1), reason: jig_domain_host::MessageRefusal::Full }]
    );
}
#[test]
fn a_refusal_with_no_later_turn_rides_with_the_answer() {
    let mut world = World::new();
    world.assign(false);
    world.relay_message(Token::new(1));
    world.refuse(Token::new(1), jig_domain_host::MessageRefusal::TooLarge);
    world.finish(0, false);
    assert_eq!(
        &*world.answer().expect("answer received").refused,
        &[jig_domain_host::Refused { name: Token::new(1), reason: jig_domain_host::MessageRefusal::TooLarge }]
    );
}
#[test]
fn a_refusal_is_resent_with_its_turn_after_a_hello() {
    let mut world = World::new();
    world.assign(false);
    world.lose_contact();
    world.relay_message(Token::new(1));
    world.refuse(Token::new(1), jig_domain_host::MessageRefusal::NameInUse);
    world.turn(1, b"one");
    world.reconnect();
    world.reconnect();
    assert_eq!(
        &*world.received_turn(1).expect("resent turn").refused,
        &[jig_domain_host::Refused { name: Token::new(1), reason: jig_domain_host::MessageRefusal::NameInUse }]
    );
    assert_eq!(world.stats().transmissions, 2);
}
#[test]
fn the_hubs_own_full_refusal_rides_with_the_next_turn() {
    let mut world = World::new();
    world.preparing();
    world.relay_message(Token::new(1));
    world.relay_message(Token::new(2));
    world.relay_message(Token::new(3));
    world.prepared();
    world.relay_message(Token::new(4));
    world.turn_with(1, b"one", Some(Token::new(4)), false);
    assert_eq!(
        &*world.received_turn(1).expect("turn received").refused,
        &[jig_domain_host::Refused { name: Token::new(3), reason: jig_domain_host::MessageRefusal::Full }]
    );
}
#[test]
fn the_answer_names_the_last_message_its_run_read() {
    let mut world = World::new();
    world.assign(false);
    world.relay_message(Token::new(1));
    world.turn_with(1, b"one", Some(Token::new(1)), false);
    world.finish(1, false);
    assert_eq!(world.answer().expect("answer received").read, Some(Token::new(1)));
}
#[test]
fn a_window_turn_crosses_with_its_flag() {
    let mut world = World::new();
    world.assign_conversation(Box::new([]), Box::new([]));
    world.turn_with(1, b"one", None, true);
    assert!(world.received_turn(1).expect("turn received").opens_window);
}

#[test]
fn refusals_beyond_the_inbox_are_dropped_and_counted() {
    let mut world = World::new();
    world.assign(false);
    for name in 1..=5 {
        world.relay_message(Token::new(name));
        world.refuse(Token::new(name), jig_domain_host::MessageRefusal::Full);
    }
    world.finish(0, false);
    assert_eq!(world.answer().expect("answer received").refused.len(), 4);
    assert_eq!(world.lost_refusals(), 1);
    world.acknowledge_answer();
    assert_eq!(world.lost_refusals(), 1);
}

#[test]
fn a_run_failed_on_its_limited_or_refused_account_answers_with_it() {
    for failure in [
        RunFailure::Limited { account: 7, reset: Some(Duration::from_secs(5)) },
        RunFailure::Limited { account: 7, reset: None },
        RunFailure::Refused { account: 7 },
    ] {
        let mut world = World::new();
        world.assign(false);
        world.fail(0, failure);
        let Some(answer) = world.answer() else { panic!("the host answered") };
        assert!(
            matches!(answer.ending, Ending::Failed { failure: Failure::Run(run), .. } if run == failure),
            "{failure:?}"
        );
        world.acknowledge_answer();
        world.assert_settled();
    }
}

fn live_world() -> World {
    let mut world = World::new();
    world.assign(false);
    world.reports.clear();
    world
}

fn count(count: u64) -> jig_domain_host::Activity {
    jig_domain_host::Activity::Count {
        kind: jig_domain_host::CountKind::TextArrived,
        elapsed: Duration::from_nanos(17),
        count,
    }
}

#[test]
fn an_unauthorized_grant_reaches_the_link_for_a_live_run() {
    let mut world = live_world();
    world.unauthorized();
    assert_eq!(
        world.reports,
        [jig_domain_host::LinkUp::Unauthorized {
            run: Token::new(31),
            attempt: Token::new(932),
            account: 7,
            generation: 4,
        }]
    );
}

#[test]
fn a_limited_account_reaches_the_link_with_its_reset_or_none() {
    let mut world = live_world();
    world.limited(Some(Duration::from_secs(5)));
    world.limited(None);
    assert_eq!(
        world.reports,
        [
            jig_domain_host::LinkUp::Limited {
                run: Token::new(31),
                attempt: Token::new(932),
                account: 7,
                reset: Some(Duration::from_secs(5)),
            },
            jig_domain_host::LinkUp::Limited { run: Token::new(31), attempt: Token::new(932), account: 7, reset: None },
        ]
    );
}

#[test]
fn a_refused_account_reaches_the_link_with_its_code_and_reason() {
    let mut world = live_world();
    world.account_refused();
    assert_eq!(
        world.reports,
        [jig_domain_host::LinkUp::AccountRefused {
            run: Token::new(31),
            attempt: Token::new(932),
            account: 7,
            code: Box::from(&b"forbidden"[..]),
            reason: Box::from(&b"account suspended"[..]),
        }]
    );
}

#[test]
fn a_notice_of_a_fenced_attempt_is_dropped() {
    let mut world = live_world();
    let owner = world.owner();
    world.hold_stop();
    world.shutdown();
    world.reports.clear();
    world.agent_report(jig_domain_host::FromAgent::Unauthorized { owner, account: 7, generation: 4 });
    world.agent_report(jig_domain_host::FromAgent::Limited { owner, account: 7, reset: Some(Duration::from_secs(5)) });
    world.agent_report(jig_domain_host::FromAgent::AccountRefused {
        owner,
        account: 7,
        code: Box::from(&b"forbidden"[..]),
        reason: Box::from(&b"account suspended"[..]),
    });
    assert!(world.reports.is_empty());
    world.gone();
    world.acknowledge_answer();
    world.assert_settled();
}

#[test]
fn a_long_operation_reaches_the_link() {
    let mut world = live_world();
    let activity = jig_domain_host::Activity::Long {
        operation: jig_domain_host::Operation::Completion,
        span: Duration::from_secs(37),
    };
    world.activity(activity);
    assert_eq!(
        world.reports,
        [jig_domain_host::LinkUp::Activity { run: Token::new(31), attempt: Token::new(932), activity }]
    );
}

#[test]
fn activity_past_the_queue_is_dropped_and_counted() {
    let mut world = live_world();
    world.lose_contact();
    for number in 1..=3 {
        world.activity(count(number));
    }
    assert_eq!(world.lost_facts(), 1);
    world.reconnect();
    assert_eq!(
        world.reports,
        [
            jig_domain_host::LinkUp::Activity { run: Token::new(31), attempt: Token::new(932), activity: count(1) },
            jig_domain_host::LinkUp::Activity { run: Token::new(31), attempt: Token::new(932), activity: count(2) },
        ]
    );
}

#[test]
fn a_newer_account_grant_reaches_a_started_agent() {
    use jig_host_world::fixtures::ANALYST;
    let mut world = World::new();
    world.assign_granted(Box::new([jig_domain_host::Grant {
        form: jig_domain_host::Form::Credential(jig_domain_host::Scheme::Key),
        account: ANALYST,
        generation: 1,
        valid: Duration::from_secs(30),
    }]));
    world.grant(jig_domain_host::Grant {
        form: jig_domain_host::Form::Credential(jig_domain_host::Scheme::Key),
        account: ANALYST,
        generation: 2,
        valid: Duration::from_secs(60),
    });
    assert_eq!(
        world.received_grant(),
        Some(jig_domain_host::Grant {
            form: jig_domain_host::Form::Credential(jig_domain_host::Scheme::Key),
            account: ANALYST,
            generation: 2,
            valid: Duration::from_secs(60),
        })
    );
}
