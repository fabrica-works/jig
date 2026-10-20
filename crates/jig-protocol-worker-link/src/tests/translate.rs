use crate::{Request, tests::fixtures, translate};
use jig_domain_host as host;
use jig_protocol_worker_wire::{self as wire, Part};
use skein_lib::{List, Queue, Reader, Time, Token};
fn translated(kind: u16, bytes: &[u8], limits: &wire::v3::Limits) -> (host::LinkDown, Box<[Part]>, Queue<Request>) {
    let mut parts = List::with_capacity(1);
    let mut values = Queue::with_capacity(4);
    let down =
        translate::down(kind, bytes, limits, Time::ZERO, &mut parts, &mut 1, &mut values).expect("translated record");
    (down, parts.into_boxed(), values)
}
fn assigned() -> (host::Assignment, Box<[Part]>, Queue<Request>) {
    let record = fixtures::assignment_record(fixtures::window());
    let bytes = fixtures::assignment_bytes(&record);
    let (down, parts, values) = translated(0x0180, &bytes, &fixtures::codec());
    let host::LinkDown::Assign { assignment } = down else { panic!("assignment: {down:?}") };
    (assignment, parts, values)
}
fn body(frame: &skein_channel::Frame) -> Reader<'_> {
    Reader::new(frame.bytes().get(8..).expect("body after header"))
}
fn granted(value: wire::GrantValue) -> (host::Grant, crate::GrantValue) {
    let record = wire::Granted::new(
        &fixtures::codec(),
        wire::GrantedParts { task: 7, attempt: 2, grant: fixtures::grant_record(value) },
    )
    .expect("grant");
    let bytes = fixtures::granted_bytes(&record);
    let (link, _, mut values) = translated(0x0187, &bytes, &fixtures::codec());
    let host::LinkDown::Grant { run, attempt, grant } = link else { panic!("grant: {link:?}") };
    assert_eq!((run, attempt, grant.account, grant.generation), (Token::new(7), Token::new(2), 1, 3));
    let Request::Value { account, generation, value, expires } = values.pop().expect("one value") else {
        panic!("value")
    };
    assert_eq!((account, generation), (1, 3));
    assert_eq!(expires, Time::ZERO.saturating_add(skein_lib::Duration::from_secs(60)));
    (grant, value)
}
#[test]
fn a_granted_credential_hands_its_scheme_and_token_out_and_the_hub_hears_the_account() {
    for (scheme, expected, form) in [
        (wire::Scheme::Bearer, crate::Scheme::Bearer, host::Scheme::Bearer),
        (wire::Scheme::Key, crate::Scheme::Key, host::Scheme::Key),
    ] {
        let (grant, value) = granted(fixtures::credential_value(scheme));
        assert_eq!(grant.form, host::Form::Credential(form));
        let crate::GrantValue::Credential { scheme, token } = value else { panic!("credential: {value:?}") };
        assert_eq!((scheme, token.expose()), (expected, fixtures::TOKEN_ONE));
    }
}
#[test]
fn a_granted_withdrawn_value_keeps_its_account_id_apart_from_its_bearer() {
    let (grant, value) = granted(fixtures::withdrawn_value());
    assert_eq!(grant.form, host::Form::Withdrawn);
    let crate::GrantValue::Withdrawn { id, bearer } = value else { panic!("withdrawn: {value:?}") };
    assert_eq!((id.expose(), bearer.expose()), (fixtures::ACCOUNT_ONE, fixtures::TOKEN_ONE));
}
#[test]
fn a_granted_application_secret_names_the_application_table() {
    let (grant, value) = granted(fixtures::application_value());
    assert_eq!(grant.form, host::Form::Application);
    let crate::GrantValue::Application { secret } = value else { panic!("application: {value:?}") };
    assert_eq!(secret.expose(), fixtures::TOKEN_ONE);
}
#[test]
fn a_hub_turn_becomes_a_turn_record_with_its_refusals() {
    let frame = translate::up(fixtures::turn(), &[], &fixtures::codec()).expect("turn");
    let record = wire::Turn::decode(&fixtures::codec(), &mut body(&frame)).expect("turn");
    assert_eq!(record.body(), b"turn");
    assert!(record.opens_window());
    assert_eq!(record.refused().get(0).expect("refusal").reason(), &wire::MessageRefusal::Full);
}
#[test]
fn a_hub_answer_writes_left_and_saved_work_from_their_parts() {
    let frame = translate::up(fixtures::answer(), &fixtures::work_parts(), &fixtures::codec()).expect("answer");
    let record = wire::Answer::decode(&fixtures::codec(), &mut body(&frame)).expect("answer");
    let wire::Ending::Ended(ended) = record.ending() else { panic!("ended") };
    assert_eq!(ended.work().left().as_deref(), Some(b"left".as_slice()));
    assert_eq!(ended.work().saved().as_deref(), Some(b"saved".as_slice()));
}
#[test]
fn activity_becomes_a_fact_record() {
    let up = host::LinkUp::Activity {
        run: Token::new(7),
        attempt: Token::new(2),
        activity: host::Activity::Count {
            kind: host::CountKind::LlmFinished,
            elapsed: skein_lib::Duration::from_secs(1),
            count: 2,
        },
    };
    let frame = translate::up(up, &[], &fixtures::codec()).expect("fact");
    let record = wire::Fact::decode(&fixtures::codec(), &mut body(&frame)).expect("fact");
    let wire::Activity::Count(count) = record.activity() else { panic!("count") };
    assert_eq!((count.kind(), count.value()), (&wire::CountKind::LlmFinished, 2));
}
#[test]
fn an_assignment_record_hands_its_items_up_as_a_part() {
    let (assignment, parts, _) = assigned();
    let workspace = assignment.assignment.workspace.expect("workspace");
    assert_eq!(parts.as_ref(), [Part { token: workspace.items, bytes: b"items".as_slice().into() }]);
    assert_eq!(workspace.workstream, 3);
}
#[test]
fn an_assignment_record_keeps_its_start_messages_in_order() {
    let limits = fixtures::codec();
    let mut record = fixtures::assignment_record(fixtures::window()).into_parts();
    let mut messages = List::with_capacity(2);
    for name in [5, 6] {
        messages
            .push(
                wire::Message::new(
                    &limits,
                    wire::MessageParts { name, sender: b"sender".as_slice().into(), words: b"words".as_slice().into() },
                )
                .expect("message"),
            )
            .expect("room");
    }
    record.messages = messages;
    let mut limits = limits;
    limits.assignment_messages = 2;
    let record = wire::Assignment::new(&limits, record).expect("two messages");
    let bytes = fixtures::assignment_bytes(&record);
    let (link, _, _) = translated(0x0180, &bytes, &limits);
    let host::LinkDown::Assign { assignment } = link else { panic!("assignment") };
    let mut names = List::with_capacity(2);
    for message in assignment.messages {
        names.push(message.name).expect("two names");
    }
    assert_eq!(names.into_boxed().as_ref(), [Token::new(5), Token::new(6)]);
}
#[test]
fn a_message_record_becomes_the_hubs_inbound_message() {
    let record = wire::Inbound::new(
        &fixtures::codec(),
        wire::InboundParts {
            task: 7,
            attempt: 2,
            name: 5,
            sender: b"sender".as_slice().into(),
            words: b"words".as_slice().into(),
        },
    )
    .expect("message");
    let bytes = fixtures::inbound_bytes(&record);
    let (link, _, _) = translated(0x0181, &bytes, &fixtures::codec());
    assert_eq!(
        link,
        host::LinkDown::Message {
            run: Token::new(7),
            attempt: Token::new(2),
            name: Token::new(5),
            sender: b"sender".as_slice().into(),
            words: b"words".as_slice().into()
        }
    );
}
#[test]
fn a_named_part_missing_from_the_root_is_dropped() {
    assert!(translate::up(fixtures::answer(), &[], &fixtures::codec()).is_none());
}

#[test]
fn the_hello_carries_the_hubs_agent_bounds_and_hosting() {
    let frame = translate::hello(fixtures::hello(), &fixtures::codec()).expect("hello");
    let record = wire::Hello::decode(&fixtures::codec(), &mut body(&frame)).expect("hello");
    assert_eq!((record.slots(), record.stop_bound()), (1, skein_lib::Duration::from_secs(1)));
    assert_eq!(record.workstreams().get(0), Some(&3));
    let run = record.hosting().get(0).expect("hosting");
    assert_eq!((run.task(), run.attempt(), run.phase()), (7, 2, &wire::Phase::Active));
    let bounds = record.bounds();
    let model = bounds.models().get(0).expect("model");
    assert_eq!((model.name(), model.window(), model.output()), (b"model".as_slice(), 16, 8));
    assert_eq!(
        (bounds.turns(), bounds.spend(), bounds.inbox(), bounds.message_bytes(), bounds.unacknowledged()),
        (1, 10, 1, 16, 1)
    );
}

#[test]
fn a_hub_invalid_start_keeps_its_words_in_the_answer_record() {
    let host::LinkUp::Answer { run, attempt, mut answer } = fixtures::answer() else { panic!("answer") };
    answer.ending = host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start {
        why: b"budget too small".as_slice().into(),
    }));
    let frame = translate::up(host::LinkUp::Answer { run, attempt, answer }, &[], &fixtures::codec())
        .expect("invalid start answer");
    let record = wire::Answer::decode(&fixtures::codec(), &mut body(&frame)).expect("answer");
    let wire::Ending::Refused(refusal) = record.ending() else { panic!("refusal") };
    let wire::Refusal::Invalid(invalid) = refusal.value() else { panic!("invalid") };
    let wire::Invalid::Start(start) = invalid.value() else { panic!("start") };
    assert_eq!(start.why(), b"budget too small");
}

#[test]
fn each_account_notice_becomes_its_record() {
    let codec = fixtures::codec();
    let (run, attempt) = (fixtures::RUN_SEVEN, fixtures::ATTEMPT_TWO);
    let frame = translate::up(host::LinkUp::Unauthorized { run, attempt, account: 1, generation: 3 }, &[], &codec)
        .expect("unauthorized");
    let record = wire::Unauthorized::decode(&codec, &mut body(&frame)).expect("unauthorized");
    assert_eq!(
        (frame.kind(), record.task(), record.attempt(), record.account(), record.generation()),
        (0x0105, 7, 2, 1, 3)
    );
    for reset in [Some(skein_lib::Duration::from_secs(60)), None] {
        let frame =
            translate::up(host::LinkUp::Limited { run, attempt, account: 1, reset }, &[], &codec).expect("limited");
        let record = wire::Limited::decode(&codec, &mut body(&frame)).expect("limited");
        assert_eq!(
            (frame.kind(), record.task(), record.attempt(), record.account(), *record.reset()),
            (0x0106, 7, 2, 1, reset)
        );
    }
    let refused = host::LinkUp::AccountRefused {
        run,
        attempt,
        account: 1,
        code: b"forbidden".as_slice().into(),
        reason: b"suspended".as_slice().into(),
    };
    let frame = translate::up(refused, &[], &codec).expect("account refused");
    let record = wire::AccountRefused::decode(&codec, &mut body(&frame)).expect("account refused");
    assert_eq!(
        (frame.kind(), record.task(), record.account(), record.code(), record.reason()),
        (0x0107, 7, 1, b"forbidden".as_slice(), b"suspended".as_slice())
    );
}

#[test]
fn an_account_refused_reason_past_its_bound_is_not_sent() {
    let refused = host::LinkUp::AccountRefused {
        run: fixtures::RUN_SEVEN,
        attempt: fixtures::ATTEMPT_TWO,
        account: 1,
        code: b"forbidden".as_slice().into(),
        reason: [b'r'; 17].as_slice().into(),
    };
    assert!(translate::up(refused, &[], &fixtures::codec()).is_none());
}

#[test]
fn a_failed_answer_keeps_the_limited_or_refused_account_in_its_record() {
    let reset = Some(skein_lib::Duration::from_secs(60));
    for failure in [
        host::RunFailure::Limited { account: 1, reset },
        host::RunFailure::Limited { account: 1, reset: None },
        host::RunFailure::Refused { account: 1 },
    ] {
        let host::LinkUp::Answer { run, attempt, mut answer } = fixtures::answer() else { panic!("answer") };
        answer.ending = host::Ending::Failed {
            failure: host::Failure::Run(failure),
            detail: Box::new([]),
            work: host::Work { left: None, saved: None },
        };
        let frame =
            translate::up(host::LinkUp::Answer { run, attempt, answer }, &[], &fixtures::codec()).expect("answer");
        let record = wire::Answer::decode(&fixtures::codec(), &mut body(&frame)).expect("answer");
        let wire::Ending::Failed(failed) = record.ending() else { panic!("failed") };
        let wire::Failure::Run(run_failure) = failed.failure() else { panic!("run failure") };
        let kept = if let wire::RunFailure::Limited(limited) = run_failure.value() {
            host::RunFailure::Limited { account: limited.account(), reset: *limited.reset() }
        } else if let wire::RunFailure::Refused(refused) = run_failure.value() {
            host::RunFailure::Refused { account: refused.account() }
        } else {
            panic!("not an account: {:?}", run_failure.value())
        };
        assert_eq!(kept, failure);
    }
}
