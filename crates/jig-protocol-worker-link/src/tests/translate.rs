use crate::{Request, tests::fixtures, translate};
use jig_domain_host as host;
use jig_protocol_worker_wire::{self as wire, Part};
use skein_lib::{List, Queue, Reader, Time, Token};
fn translated(kind: u16, bytes: &[u8], limits: &wire::v2::Limits) -> (host::LinkDown, Box<[Part]>, Queue<Request>) {
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
#[test]
fn a_granted_record_hands_its_value_out_and_the_hub_hears_the_name() {
    let record = wire::Granted::new(
        &fixtures::codec(),
        wire::GrantedParts { task: 7, attempt: 2, grant: fixtures::grant_record(wire::GrantTable::Accounts) },
    )
    .expect("grant");
    let bytes = fixtures::granted_bytes(&record);
    let (link, _, mut values) = translated(0x0187, &bytes, &fixtures::codec());
    let host::LinkDown::Grant { run, attempt, grant } = link else { panic!("grant: {link:?}") };
    assert_eq!((run, attempt, grant.generation), (Token::new(7), Token::new(2), 3));
    let Request::Value { value, account_id, expires, .. } = values.pop().expect("one value") else { panic!("value") };
    assert_eq!(value.expose(), fixtures::TOKEN_ONE);
    assert_eq!(account_id.expose(), fixtures::ACCOUNT_ONE);
    assert_eq!(expires, Time::ZERO.saturating_add(skein_lib::Duration::from_secs(60)));
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
