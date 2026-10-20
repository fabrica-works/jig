use super::*;
use jig_client::contract as c;
use jig_client_world::harness::Receipt;
use skein_lib::{List, Overflow, Writer};
const ATTEMPT: Token = Token::new(7);
const TASK: u64 = 8;
const PROJECT: u32 = 1;
const FIRST_MESSAGE: u64 = 19;
const KEY: [u8; 16] = [3; 16];
fn encoded<T>(value: &T, measure: fn(&T) -> u32, encode: fn(&T, &mut Writer) -> Result<(), Overflow>) -> Box<[u8]> {
    let mut out = Writer::new(usize::try_from(measure(value)).expect("bounded wire fixture"));
    encode(value, &mut out).expect("bounded wire fixture");
    out.finish()
}
fn request() -> Request {
    let mut operations = List::with_capacity(2);
    operations
        .push(c::Operation::Message {
            project: PROJECT,
            task: TASK,
            words: c::Text::new(b"continue".as_slice().into(), 64).expect("bounded wire fixture"),
        })
        .expect("bounded wire fixture");
    operations.push(c::Operation::Stop { project: PROJECT, task: TASK }).expect("bounded wire fixture");
    let ask = translate::asks::ask(&c::Ask::Compound { project: PROJECT, operations }, &w::CEILINGS)
        .expect("bounded wire fixture");
    let keyed = w::Keyed::new(
        &w::CEILINGS,
        w::KeyedParts {
            key: KEY,
            owner: w::Owner::Jig,
            version: jig_client_protocol_wire::VERSION,
            request: encoded(&ask, w::Ask::measure, w::Ask::encode),
            value: None,
        },
    )
    .expect("bounded wire fixture");
    Request::Exchange {
        exchange: ATTEMPT,
        credential: None,
        message: Message::Request(encoded(&keyed, w::Keyed::measure, w::Keyed::encode)),
    }
}
fn reply() -> Event {
    let value = w::OwnerOutcome::Compound(
        w::OwnerOutcomeCompound::new(
            &w::CEILINGS,
            w::OwnerOutcomeCompoundParts { project: PROJECT, operations: 2, first_message: FIRST_MESSAGE },
        )
        .expect("bounded wire fixture"),
    );
    let outcome = w::Outcome::Owner(
        w::OutcomeOwner::new(&w::CEILINGS, w::OutcomeOwnerParts { value }).expect("bounded wire fixture"),
    );
    let answer = w::Final::Done(w::Done::new(&w::CEILINGS, w::DoneParts { outcome }).expect("bounded wire fixture"));
    let value = w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts {
            owner: w::Owner::Jig,
            version: jig_client_protocol_wire::VERSION,
            record: encoded(&answer, w::Final::measure, w::Final::encode),
        },
    )
    .expect("bounded wire fixture");
    let answer = w::KeyedAnswer::FinalAnswer(value);
    Event::Reply { exchange: ATTEMPT, message: encoded(&answer, w::KeyedAnswer::measure, w::KeyedAnswer::encode) }
}
#[test]
fn a_compound_word_receipt_joins_its_observed_target_and_answered_message() {
    let mut wire = Wire::default();
    wire.sent(&request());
    let receipts = wire.additional_receipts(&reply());
    let [Receipt::Answer { attempt, final_answer, words }] = receipts.as_slice() else {
        panic!("compound words receipt: {receipts:?}");
    };
    assert_eq!((*attempt, *final_answer, *words), (ATTEMPT, true, Some((TASK, FIRST_MESSAGE))));
}
