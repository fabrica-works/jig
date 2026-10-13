use super::fixtures;
use crate::{Bounds, Hello, Reason};
use alloc::boxed::Box;
use skein_channel::{Direction, Role, Term};
use skein_lib::{Duration, List, Reader};

fn hello_schema() -> skein_channel::Schema {
    crate::schema(&fixtures::LIMITS).expect("small schema")
}

fn hello_terms(largest: u32) -> List<Term> {
    let mut terms = List::with_capacity(7);
    terms.push(Term { kind: 0x0100, largest }).expect("hello term slot");
    // Hand-counted bounds for Turn, Call, Answer, Fact and both notices.
    for (kind, largest) in [(0x0101, 96), (0x0102, 101), (0x0103, 145), (0x0104, 34), (0x0105, 28), (0x0106, 28)] {
        terms.push(Term { kind, largest }).expect("one term per up kind");
    }
    terms
}

fn down_terms() -> List<Term> {
    let mut terms = List::with_capacity(8);
    // Independently summed from field widths and the fixture limits.
    for (kind, largest) in [
        (0x0180, 600),
        (0x0181, 80),
        (0x0182, 74),
        (0x0183, 20),
        (0x0184, 20),
        (0x0185, 16),
        (0x0186, 16),
        (0x0187, 93),
    ] {
        terms.push(Term { kind, largest }).expect("one term per down kind");
    }
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
    let version = schema.version(2).expect("one version");
    assert_eq!(schema.magic, *b"jigw");
    assert_eq!(version.kinds.len(), 15);
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
    assert_eq!(crate::terms_gap(&hello_schema(), Role::Responder, &down_terms()), None);
}

fn answer_through_bytes(answer: &crate::Answer) -> crate::Answer {
    let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("small answer"));
    answer.encode(&mut writer).expect("measured room");
    crate::Answer::decode(&fixtures::LIMITS, &mut Reader::new(&writer.finish())).expect("answer within limits")
}

fn fact_through_bytes(fact: &crate::Fact) -> crate::Fact {
    let mut writer = skein_lib::Writer::new(usize::try_from(fact.measure()).expect("small fact"));
    fact.encode(&mut writer).expect("measured room");
    crate::Fact::decode(&fixtures::LIMITS, &mut Reader::new(&writer.finish())).expect("fact within limits")
}

#[test]
fn a_turn_keeps_its_refusals_and_window_flag_through_bytes() {
    let turn = fixtures::turn_with_refusals();
    let mut writer = skein_lib::Writer::new(usize::try_from(turn.measure()).expect("small turn"));
    turn.encode(&mut writer).expect("measured room");
    let decoded = crate::Turn::decode(&fixtures::LIMITS, &mut Reader::new(&writer.finish())).expect("turn");
    assert!(decoded.opens_window());
    assert_eq!(decoded.refused().len(), 2);
    let first = decoded.refused().get(0).expect("first refusal");
    let second = decoded.refused().get(1).expect("second refusal");
    assert_eq!(first.name(), fixtures::MESSAGE_FIRST);
    assert_eq!(first.reason(), &crate::MessageRefusal::TooLarge);
    assert_eq!(second.name(), fixtures::MESSAGE_SECOND);
    assert_eq!(second.reason(), &crate::MessageRefusal::Ending);
    assert_eq!(decoded, turn);
}

#[test]
fn a_turn_past_the_inbox_refusal_bound_is_refused() {
    let mut parts = fixtures::turn_with_refusals().into_parts();
    let mut refused = List::with_capacity(3);
    for name in [41, 42, 43] {
        refused.push(fixtures::refusal(name, crate::MessageRefusal::Full)).expect("three fixture slots");
    }
    parts.refused = refused;
    assert_eq!(
        crate::Turn::new(&fixtures::LIMITS, parts).expect_err("third refusal refused").reason,
        skein_codec::Reason::Bound
    );
}

#[test]
fn an_answer_keeps_its_unread_fence_through_bytes() {
    let answer = fixtures::answer_ended();
    let decoded = answer_through_bytes(&answer);
    assert_eq!(decoded.read(), &Some(fixtures::MESSAGE_FIRST));
    assert_eq!(decoded.refused().len(), 2);
    assert_eq!(decoded, answer);
}

#[test]
fn a_failed_answer_keeps_its_failure_kind_and_detail() {
    let answer = fixtures::answer_failed();
    let decoded = answer_through_bytes(&answer);
    match decoded.ending() {
        crate::Ending::Failed(failed) => {
            assert_eq!(failed.detail(), b"bounded-detail");
            match failed.failure() {
                crate::Failure::Run(run) => assert_eq!(run.value(), &crate::RunFailure::Model),
                other @ (crate::Failure::Unprepared(_) | crate::Failure::Agent(_) | crate::Failure::Cancelled(_)) => {
                    panic!("unexpected failure {other:?}")
                }
            }
        }
        other @ (crate::Ending::Refused(_) | crate::Ending::Ended(_) | crate::Ending::Parked(_)) => {
            panic!("unexpected ending {other:?}")
        }
    }
    assert_eq!(decoded, answer);
}

#[test]
fn a_count_fact_carries_kind_time_and_count() {
    let fact = fixtures::fact_count();
    let decoded = fact_through_bytes(&fact);
    match decoded.activity() {
        crate::Activity::Count(count) => {
            assert_eq!(count.kind(), &crate::CountKind::LlmRetried);
            assert_eq!(count.elapsed(), Duration::from_secs(2));
            assert_eq!(count.value(), 3);
        }
        other @ (crate::Activity::Phase(_) | crate::Activity::Long(_) | crate::Activity::LongDone) => {
            panic!("unexpected activity {other:?}")
        }
    }
    assert_eq!(decoded, fact);
}

#[test]
fn a_long_operation_fact_keeps_its_span() {
    let fact = fixtures::fact_long();
    let decoded = fact_through_bytes(&fact);
    match decoded.activity() {
        crate::Activity::Long(long) => {
            assert_eq!(long.operation(), &crate::Operation::Checks);
            assert_eq!(long.span(), Duration::from_secs(6));
        }
        other @ (crate::Activity::Phase(_) | crate::Activity::Count(_) | crate::Activity::LongDone) => {
            panic!("unexpected activity {other:?}")
        }
    }
    assert_eq!(decoded, fact);
}

#[test]
fn the_schema_lists_every_up_kind_from_the_initiator() {
    let schema = hello_schema();
    assert_eq!(schema.versions.len(), 1);
    for version in &schema.versions {
        assert_eq!(version.kinds.len(), 15);
        for number in [0x0100, 0x0101, 0x0102, 0x0103, 0x0104, 0x0105, 0x0106] {
            assert_eq!(version.kind(number).expect("every up kind").direction, Direction::FromInitiator);
        }
    }
}

fn assignment_through_bytes(assignment: &crate::Assignment) -> crate::Assignment {
    let mut writer = skein_lib::Writer::new(usize::try_from(assignment.measure()).expect("small assignment"));
    assignment.encode(&mut writer).expect("measured room");
    crate::Assignment::decode(&fixtures::LIMITS, &mut Reader::new(&writer.finish())).expect("assignment within limits")
}

fn grant_through_bytes(grant: &crate::Grant) -> crate::Grant {
    let mut writer = skein_lib::Writer::new(usize::try_from(grant.measure()).expect("small grant"));
    grant.encode(&mut writer).expect("measured room");
    crate::Grant::decode(&fixtures::LIMITS, &mut Reader::new(&writer.finish())).expect("grant within limits")
}

#[test]
fn an_assignment_keeps_its_start_messages_in_order() {
    let assignment = fixtures::assignment_resumed();
    let decoded = assignment_through_bytes(&assignment);
    assert_eq!(decoded.messages().len(), 2);
    assert_eq!(decoded.messages().get(0).expect("first start message").name(), fixtures::MESSAGE_FIRST);
    assert_eq!(decoded.messages().get(1).expect("second start message").name(), fixtures::MESSAGE_SECOND);
    assert_eq!(decoded, assignment);
}

#[test]
fn an_assignment_keeps_its_sixteen_byte_identity() {
    let decoded = assignment_through_bytes(&fixtures::assignment_resumed());
    assert_eq!(decoded.identity(), &fixtures::IDENTITY_FIRST);
}

#[test]
fn a_grant_names_the_table_its_value_comes_from() {
    for (table, value) in
        [(crate::GrantTable::Accounts, fixtures::TOKEN_ONE), (crate::GrantTable::Application, fixtures::CREDENTIAL_ONE)]
    {
        let grant = fixtures::grant(table.clone(), 5, 9);
        let decoded = grant_through_bytes(&grant);
        assert_eq!(decoded.table(), &table);
        assert_eq!(decoded.value().expose(), value);
        assert_eq!(decoded, grant);
    }
}

#[test]
fn an_accounts_grant_keeps_its_account_id_separate_from_its_token() {
    let decoded = grant_through_bytes(&fixtures::grant(crate::GrantTable::Accounts, 5, 9));
    assert_eq!(decoded.value().expose(), fixtures::TOKEN_ONE);
    assert_eq!(decoded.account_id().expose(), fixtures::ACCOUNT_ONE);
}

#[test]
fn an_applications_grant_carries_an_empty_account_id() {
    let decoded = grant_through_bytes(&fixtures::grant(crate::GrantTable::Application, 5, 9));
    assert_eq!(decoded.account_id().expose(), b"");
}

fn granted_debug() -> Box<[u8]> {
    let record = crate::Granted::new(
        &fixtures::LIMITS,
        crate::GrantedParts { task: 3, attempt: 4, grant: fixtures::grant(crate::GrantTable::Accounts, 5, 9) },
    )
    .expect("granted record");
    alloc::fmt::format(format_args!("{record:?}")).into_bytes().into_boxed_slice()
}

#[test]
fn a_granted_record_debug_shows_the_value_length_only() {
    let debug = granted_debug();
    assert!(skein_lib::bytes::find(&debug, b"len: 9").is_some());
    assert_eq!(skein_lib::bytes::find(&debug, fixtures::TOKEN_ONE), None);
    assert_eq!(skein_lib::bytes::find(&debug, b"116, 111, 107, 101, 110, 45, 111, 110, 101"), None);
}

#[test]
fn a_granted_record_debug_shows_the_account_id_length_only() {
    let debug = granted_debug();
    assert!(skein_lib::bytes::find(&debug, b"len: 11").is_some());
    assert_eq!(skein_lib::bytes::find(&debug, fixtures::ACCOUNT_ONE), None);
    assert_eq!(skein_lib::bytes::find(&debug, b"97, 99, 99, 111, 117, 110, 116, 45, 111, 110, 101"), None);
}

#[test]
fn a_relayed_busy_answer_carries_no_text() {
    let record = crate::Relayed::new(
        &fixtures::LIMITS,
        crate::RelayedParts {
            task: 3,
            attempt: 4,
            name: b"call-one".as_slice().into(),
            answer: crate::RelayAnswer::Busy,
        },
    )
    .expect("busy answer");
    let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("small relayed"));
    record.encode(&mut writer).expect("measured room");
    let bytes = writer.finish();
    assert_eq!(bytes.len(), 29);
    let decoded = crate::Relayed::decode(&fixtures::LIMITS, &mut Reader::new(&bytes)).expect("relayed within limits");
    assert_eq!(decoded.answer(), &crate::RelayAnswer::Busy);
}

#[test]
fn an_assignment_past_its_grant_bound_is_refused() {
    let mut parts = fixtures::assignment_resumed().into_parts();
    let mut grants = List::with_capacity(3);
    for account in [5, 6, 7] {
        grants.push(fixtures::grant(crate::GrantTable::Accounts, account, 9)).expect("three fixture slots");
    }
    parts.grants = grants;
    assert_eq!(
        crate::Assignment::new(&fixtures::LIMITS, parts).expect_err("third grant refused").reason,
        skein_codec::Reason::Bound
    );
}

#[test]
fn the_schema_lists_every_down_kind_from_the_responder() {
    let schema = hello_schema();
    let version = schema.version(2).expect("one version");
    assert_eq!(version.kinds.len(), 15);
    for term in &down_terms() {
        assert_eq!(
            version.kind(term.kind),
            Some(skein_channel::Kind { kind: term.kind, direction: Direction::FromResponder, largest: term.largest })
        );
    }
}

#[test]
fn an_accounts_grant_without_a_provider_id_carries_an_empty_account_id() {
    let mut parts = fixtures::grant(crate::GrantTable::Accounts, 5, 9).into_parts();
    parts.account_id = skein_lib::Secret::new(b"".as_slice().into());
    let grant = crate::Grant::new(&fixtures::LIMITS, parts).expect("provider without an account id");
    let decoded = grant_through_bytes(&grant);
    assert_eq!(decoded.account_id().expose(), b"");
    assert_eq!(decoded.value().expose(), fixtures::TOKEN_ONE);
}

#[test]
fn an_assignment_keeps_its_transcript_piece_opaque() {
    let decoded = assignment_through_bytes(&fixtures::assignment_resumed());
    assert_eq!(decoded.transcript(), b"\0\0\0\x08turn-one\0\0\0\x08turn-two");
}
