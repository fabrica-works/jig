use super::fixtures;
use crate::{Bounds, Hello, Reason};
use skein_channel::{Direction, Role, Term};
use skein_lib::{Duration, List, Reader};

fn hello_schema() -> skein_channel::Schema {
    crate::schema(&fixtures::LIMITS).expect("small schema")
}

fn hello_terms(largest: u32) -> List<Term> {
    let mut terms = List::with_capacity(1);
    terms.push(Term { kind: 0x0100, largest }).expect("hello term slot");
    terms
}

fn no_terms() -> List<Term> {
    List::with_capacity(0)
}

fn hello_through_bytes(hello: &Hello) -> Hello {
    let bytes = fixtures::encode(hello);
    Hello::decode(&fixtures::LIMITS, &mut Reader::new(&bytes)).expect("a hello within limits")
}

#[test]
fn a_hello_keeps_its_agent_bounds_through_bytes() {
    let hello = fixtures::hello_fullest();
    let decoded = hello_through_bytes(&hello);
    let bounds = decoded.bounds();
    let small = bounds.models().get(0).expect("the first model");
    assert_eq!(bounds.models().len(), 2);
    assert_eq!(small.name(), b"small");
    assert_eq!(small.window(), 8192);
    assert_eq!(small.output(), 1024);
    assert_eq!(bounds.turns(), 40);
    assert_eq!(bounds.spend(), 100);
    assert_eq!(bounds.time(), Duration::from_secs(300));
    assert_eq!(bounds.waiting(), Duration::from_secs(60));
    assert_eq!(bounds.checks(), Duration::from_secs(20));
    assert_eq!(bounds.message_bytes(), 32);
    assert_eq!(bounds.inbox(), 2);
    assert_eq!(bounds.unacknowledged(), 2);
    assert_eq!(decoded.stop_bound(), Duration::from_secs(95));
    assert_eq!(decoded, hello);
}

#[test]
fn a_smallest_hello_crosses_bytes() {
    let hello = fixtures::hello_smallest();
    assert_eq!(hello_through_bytes(&hello), hello);
}

#[test]
fn a_credential_debug_shows_the_secret_length_only() {
    let credential = fixtures::credential(fixtures::WORKER_A, b"hidden-key");
    assert_eq!(alloc::fmt::format(format_args!("{:?}", credential.secret())), "Secret { len: 10 }");
    assert!(!alloc::fmt::format(format_args!("{credential:?}")).contains("hidden-key"));
}

#[test]
fn a_credential_keeps_its_secret_through_bytes() {
    let credential = fixtures::credential(fixtures::WORKER_B, b"private");
    let mut writer = skein_lib::Writer::new(usize::try_from(credential.measure()).expect("small credential"));
    credential.encode(&mut writer).expect("measured room");
    let bytes = writer.finish();
    let decoded = crate::Credential::decode(&fixtures::LIMITS, &mut Reader::new(&bytes)).expect("credential");
    assert_eq!(decoded, credential);
}

#[test]
fn a_hello_one_model_past_its_list_bound_is_refused() {
    let mut parts = fixtures::hello_fullest().into_parts();
    let mut bounds = parts.bounds.into_parts();
    let mut models = List::with_capacity(3);
    for _ in 0_u32..3 {
        models.push(fixtures::model()).expect("three fixture slots");
    }
    bounds.models = models;
    parts.bounds = Bounds::new(&crate::CEILINGS, bounds).expect("three models fit the ceilings");
    assert_eq!(
        Hello::new(&fixtures::LIMITS, parts).expect_err("third model refused").reason,
        skein_codec::Reason::Bound
    );
}

#[test]
fn every_reason_reads_back_from_its_code() {
    for (reason, code) in [
        (Reason::Unauthorized, 256),
        (Reason::Application, 257),
        (Reason::Busy, 258),
        (Reason::Replaced, 259),
        (Reason::NotAdmitted, 260),
        (Reason::Stopping, 261),
    ] {
        assert_eq!(reason.code(), code);
        assert_eq!(Reason::from_code(code), Some(reason));
    }
}

#[test]
fn a_reason_below_256_reads_as_none() {
    for code in 0..256 {
        assert_eq!(Reason::from_code(code), None);
    }
}

#[test]
fn the_schema_lists_the_hello_from_the_initiator() {
    let schema = hello_schema();
    let version = schema.version(1).expect("one version");
    assert_eq!(schema.magic, *b"jigw");
    assert_eq!(version.kinds.len(), 1);
    assert_eq!(
        version.kind(0x0100),
        Some(skein_channel::Kind { kind: 0x0100, direction: Direction::FromInitiator, largest: 178 })
    );
}

#[test]
fn terms_that_take_every_kind_leave_no_gap() {
    assert_eq!(crate::terms_gap(&hello_schema(), Role::Initiator, &hello_terms(178)), None);
}

#[test]
fn terms_below_the_hello_bound_name_the_hello_kind() {
    assert_eq!(crate::terms_gap(&hello_schema(), Role::Initiator, &hello_terms(177)), Some(0x0100));
}

#[test]
fn terms_without_the_hello_name_the_hello_kind() {
    assert_eq!(crate::terms_gap(&hello_schema(), Role::Initiator, &no_terms()), Some(0x0100));
}

#[test]
fn the_responder_needs_no_sending_hello_term() {
    assert_eq!(crate::terms_gap(&hello_schema(), Role::Responder, &no_terms()), None);
}
