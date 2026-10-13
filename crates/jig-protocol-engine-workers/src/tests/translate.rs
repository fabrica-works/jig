use super::fixtures;
use crate::translate;
use skein_lib::{Duration, Token};
fn translated(record: translate::Up) -> (jig_domain_host::LinkUp, translate::Parts) {
    let mut parts = translate::Parts::new(4);
    let link = translate::up(record, &mut parts);
    (link, parts)
}
fn up(record: translate::Up) -> jig_domain_host::LinkUp {
    let (link, parts) = translated(record);
    assert!(parts.is_empty(), "this record has no application parts");
    link
}
fn answer(record: jig_protocol_worker_wire::Answer) -> (jig_domain_host::Answer, translate::Parts) {
    let (link, parts) = translated(translate::Up::Answer(record));
    let jig_domain_host::LinkUp::Answer { run, attempt, answer } = link else {
        panic!("an answer");
    };
    assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
    (answer, parts)
}
#[test]
fn a_hello_record_becomes_the_hubs_hello_with_its_bounds() {
    let hello = translate::hello(fixtures::hello_record());
    assert_eq!(hello.slots, 1);
    assert_eq!(hello.stop_bound, Duration::from_secs(20));
    assert_eq!(hello.workstreams.as_ref(), &[9]);
    assert_eq!(
        hello.hosting.as_ref(),
        &[jig_domain_host::Hosting {
            run: Token::new(7),
            attempt: Token::new(2),
            phase: jig_domain_host::Phase::Waiting
        }]
    );
    let model = hello.bounds.models.first().expect("one model");
    assert_eq!(model.name.as_ref(), b"model");
    assert_eq!(model.window, 8192);
    assert_eq!(model.output, 1024);
    assert_eq!(hello.bounds.turns, 8);
    assert_eq!(hello.bounds.spend, 100);
    assert_eq!(hello.bounds.time, Duration::from_secs(60));
    assert_eq!(hello.bounds.waiting, Duration::from_secs(10));
    assert_eq!(hello.bounds.checks, Duration::from_secs(2));
    assert_eq!(hello.bounds.message_bytes, 16);
    assert_eq!(hello.bounds.inbox, 1);
    assert_eq!(hello.bounds.unacknowledged, 1);
}

#[test]
fn a_turn_record_becomes_the_hubs_turn_with_its_refusals() {
    let link = up(translate::Up::Turn(fixtures::turn_record()));
    let jig_domain_host::LinkUp::Turn { run, attempt, turn } = link else {
        panic!("a turn");
    };
    assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
    assert!(turn.opens_window, "window marker kept");
    assert_eq!(
        turn.refused.as_ref(),
        &[jig_domain_host::Refused { name: Token::new(8), reason: jig_domain_host::MessageRefusal::Full }]
    );
    assert_eq!(turn.read, Some(Token::new(9)));
    assert_eq!(turn.body.as_ref(), b"opaque-turn");
}
#[test]
fn a_fact_record_becomes_the_hubs_activity() {
    assert_eq!(
        up(translate::Up::Fact(fixtures::fact_record())),
        jig_domain_host::LinkUp::Activity {
            run: Token::new(7),
            attempt: Token::new(2),
            activity: jig_domain_host::Activity::Count {
                kind: jig_domain_host::CountKind::LlmStarted,
                elapsed: Duration::from_secs(2),
                count: 3
            }
        }
    );
}
#[test]
fn rejected_and_exhausted_records_become_notices() {
    assert_eq!(
        up(translate::Up::Rejected(fixtures::rejected_record())),
        jig_domain_host::LinkUp::Rejected { run: Token::new(7), attempt: Token::new(2), account: 1, generation: 4 }
    );
    assert_eq!(
        up(translate::Up::Exhausted(fixtures::exhausted_record())),
        jig_domain_host::LinkUp::Exhausted {
            run: Token::new(7),
            attempt: Token::new(2),
            account: 1,
            retry_after: Duration::from_secs(5)
        }
    );
}

#[test]
fn an_answer_record_keeps_its_unread_fence() {
    let (answer, parts) = answer(fixtures::answer_record());
    drop(parts);
    assert_eq!(answer.read, Some(Token::new(9)));
    assert_eq!(
        answer.refused.as_ref(),
        &[jig_domain_host::Refused { name: Token::new(8), reason: jig_domain_host::MessageRefusal::Full }]
    );
}
#[test]
fn an_answer_hands_left_and_saved_work_up_as_parts() {
    let (answer, parts) = answer(fixtures::answer_record());
    let jig_domain_host::Ending::Ended { work, .. } = answer.ending else {
        panic!("ended");
    };
    assert_eq!(work.left, Some(Token::new(4)));
    assert_eq!(work.saved, Some(Token::new(5)));
    assert!(!parts.is_empty(), "application bytes extracted");
    assert_eq!(parts.next(), 6);
    let parts = parts.finish();
    assert_eq!(parts.first().expect("left part").bytes.as_ref(), b"left");
    assert_eq!(parts.get(1).expect("saved part").bytes.as_ref(), b"saved");
}
#[test]
fn a_call_record_keeps_its_name_bytes() {
    let link = up(translate::Up::Call(fixtures::call_record()));
    let jig_domain_host::LinkUp::Call { run, attempt, call, tool, writes, input, deadline } = link else {
        panic!("call");
    };
    assert_eq!(call.as_ref(), &[0, 2, 7]);
    assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
    assert_eq!(tool.as_ref(), b"read");
    assert!(!writes, "write bit kept");
    assert_eq!(input.as_ref(), b"input");
    assert_eq!(deadline, Duration::from_secs(4));
}

#[test]
fn a_failed_answers_resource_precedes_its_left_and_saved_parts() {
    let (answer, parts) = answer(fixtures::answer_with_resource());
    let jig_domain_host::Ending::Failed { failure, detail, work } = answer.ending else {
        panic!("failed");
    };
    assert_eq!(
        failure,
        jig_domain_host::Failure::Unprepared(jig_domain_host::Preparation::Permanent { resource: Some(Token::new(4)) })
    );
    assert_eq!(detail.as_ref(), b"detail");
    assert_eq!(work.left, Some(Token::new(5)));
    assert_eq!(work.saved, Some(Token::new(6)));
    assert_eq!(parts.next(), 7);
    let parts = parts.finish();
    assert_eq!(parts[0].bytes.as_ref(), b"resource");
    assert_eq!(parts[1].bytes.as_ref(), b"left");
    assert_eq!(parts[2].bytes.as_ref(), b"saved");
}
