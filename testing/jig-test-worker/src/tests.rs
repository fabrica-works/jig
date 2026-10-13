mod fixtures;
use crate::{Encodable, Encoded, Event, Invalid, Request};
use alloc::boxed::Box;
use fixtures::Worker;
use jig_domain_host as host;
use smith_host_domain as smith;

#[test]
fn a_start_waits_for_its_charter_to_come_back_decoded() {
    let mut worker = Worker::new(fixtures::limits());
    worker.assign();
    let (owner, source) = worker.take_decode();
    assert_eq!(owner, fixtures::AGENT);
    assert_eq!(source.charter, fixtures::charter_bytes());
    assert_eq!(worker.spawns(), 0);
    worker.step(Event::Decoded { agent: owner, start: Ok(Box::new(fixtures::start(source))) });
    assert_eq!(worker.spawns(), 1);
}

#[test]
fn an_undecodable_charter_fails_the_start_as_invalid() {
    let mut worker = Worker::new(fixtures::limits());
    worker.assign();
    let (owner, _) = worker.take_decode();
    worker.step(Event::Decoded { agent: owner, start: Err(Invalid::Charter) });
    assert_eq!(worker.spawns(), 0);
    assert_eq!(
        worker.take_answer().ending,
        host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start { why: Box::from(&b"charter"[..]) }))
    );
}

#[test]
fn a_turn_reaches_the_hub_only_after_it_is_encoded() {
    let mut worker = Worker::started();
    worker.receive(smith::Up::Turn { turn: fixtures::turn() });
    let (owner, value) = worker.take_encode();
    assert_eq!(value, Encodable::Turn(fixtures::turn()));
    let turn = fixtures::encoded_turn();
    worker.step(Event::Encoded { owner, encoded: Encoded::Turn(fixtures::encoded_turn()) });
    let mut actual = None;
    for _ in 0..worker.out.len() {
        if let Some(Request::Link(host::LinkUp::Turn { turn, .. })) = worker.out.pop() {
            actual = Some(turn);
        }
    }
    assert_eq!(actual, Some(turn));
}

#[test]
fn a_result_is_handed_out_once_and_reaches_the_hub_as_bytes() {
    let mut worker = Worker::started();
    worker.receive(smith::Up::Answer {
        answer: smith::Answer { read: None, turns: 0, spent: 0, result: fixtures::result() },
    });
    let (owner, value) = worker.take_encode();
    assert_eq!(value, Encodable::Result(fixtures::result()));
    assert_eq!(worker.hand_outs(), 0, "one hand-out");
    worker.step(Event::Encoded { owner, encoded: Encoded::Result(Box::from(&b"encoded result"[..])) });
    worker.gone();
    assert_eq!(
        worker.take_answer().ending,
        host::Ending::Ended {
            outcome: Box::from(&b"encoded result"[..]),
            work: host::Work { left: None, saved: None }
        }
    );
}

#[test]
fn a_channel_fact_reaches_the_hub_as_activity() {
    let mut worker = Worker::started();
    worker.receive(smith::Up::Fact { body: Box::from(&b"a fact"[..]) });
    assert_eq!(worker.out.pop(), Some(Request::Fact { owner: fixtures::AGENT, body: Box::from(&b"a fact"[..]) }));
    worker.clear();
    let activity =
        host::Activity::Count { kind: host::CountKind::TextArrived, elapsed: skein_lib::Duration::ZERO, count: 1 };
    worker.step(Event::Activity { owner: fixtures::AGENT, activity: Some(activity) });
    assert_eq!(
        worker.out.pop(),
        Some(Request::Link(host::LinkUp::Activity { run: fixtures::RUN, attempt: fixtures::ATTEMPT, activity }))
    );
}

#[test]
fn a_deliver_is_answered_with_its_title_and_body() {
    let mut worker = Worker::started();
    worker.receive(smith::Up::Call {
        call: fixtures::CALL,
        name: smith::CallName { activation: 7, completion: 1, position: 0 },
        deadline: skein_lib::Time::ZERO.saturating_add(skein_lib::Duration::from_secs(30)),
        ask: fixtures::delivery(),
    });
    let (owner, value) = worker.take_encode();
    assert_eq!(value, Encodable::CallName(smith::CallName { activation: 7, completion: 1, position: 0 }));
    worker.step(Event::Encoded { owner, encoded: Encoded::CallName(Box::from(&b"delivery call"[..])) });
    let content = worker.domain.delivery(fixtures::AGENT).expect("delivered workspace");
    assert_eq!(content.title.as_ref(), b"a title");
    assert_eq!(content.body.as_ref(), b"a body");
    let mut actual = None;
    for _ in 0..worker.out.len() {
        if let Some(Request::Process(request)) = worker.out.pop()
            && let smith::process::Request::Send { message: smith::Down::Answer { call, reply }, .. } = *request
        {
            actual = Some((call, reply));
        }
    }
    let receipt = smith::Receipt::new(0, Box::new(0_u64.to_be_bytes())).expect("fixture receipt");
    assert_eq!(
        actual,
        Some((
            fixtures::CALL,
            smith::Reply::Delivery(smith::Delivery::Delivered(
                smith::Delivered::new(Box::new([receipt])).expect("fixture delivered")
            ))
        ))
    );
}
