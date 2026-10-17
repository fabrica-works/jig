use super::fixtures as f;
use crate::translate::{answers, asks, operations};
use jig_client_protocol_wire::v8 as w;
use jig_domain_people as p;
use skein_lib::{List, Reader};
const PROJECT: u32 = 3;
const OTHER_PROJECT: u32 = 4;
const TASK: u64 = 8;
const FIRST_MESSAGE: u64 = 19;
fn compound(project: u32, stop_project: u32) -> w::Ask {
    let limits = f::wire();
    let values = [
        w::Operation::Message(
            w::Message::new(&limits, w::MessageParts { project, task: TASK, words: b"words".as_slice().into() })
                .unwrap(),
        ),
        w::Operation::Stop(w::Stop::new(&limits, w::StopParts { project: stop_project, task: TASK }).unwrap()),
    ];
    f::compound(project, &values)
}
#[test]
fn a_compound_reaches_the_people_child_in_order() {
    let mut expected = List::with_capacity(2);
    expected.push(p::Operation::Message { task: TASK, words: b"words".as_slice().into(), release: false }).unwrap();
    expected.push(p::Operation::Stop { task: TASK }).unwrap();
    assert_eq!(
        asks::ask(compound(PROJECT, PROJECT), &f::wire()),
        Ok(p::Ask::Compound { project: PROJECT, operations: expected })
    );
}
#[test]
fn a_compound_operation_naming_another_project_is_refused_at_its_place() {
    let limits = f::wire();
    let answer = operations::refused(&compound(PROJECT, OTHER_PROJECT), &limits).unwrap().unwrap();
    let w::KeyedAnswer::FinalAnswer(value) = answer else {
        panic!("final answer");
    };
    let w::Final::Refused(value) = w::Final::decode(&limits, &mut Reader::new(value.record())).unwrap() else {
        panic!("refusal");
    };
    assert_eq!(*value.at(), Some(1));
    assert_eq!(
        value.refusal(),
        &w::Refusal::Malformed(
            w::RefusalMalformed::new(&limits, w::RefusalMalformedParts { at: 1, why: w::Malformed::ProjectMismatch })
                .unwrap()
        )
    );
}
#[test]
fn a_compound_refusal_names_its_first_failing_place() {
    let limits = f::wire();
    let answer =
        answers::answer(&p::Reply::Outcome(p::Outcome::CompoundRefused { at: 1, why: p::Refusal::Unknown }), &limits)
            .unwrap();
    let w::KeyedAnswer::FinalAnswer(value) = answer else {
        panic!("final answer");
    };
    let w::Final::Refused(value) = w::Final::decode(&limits, &mut Reader::new(value.record())).unwrap() else {
        panic!("refusal");
    };
    assert_eq!(*value.at(), Some(1));
    assert_eq!(value.refusal(), &w::Refusal::Unknown);
}
#[test]
fn a_compound_answer_keeps_its_message_and_operation_count() {
    let limits = f::wire();
    let answer = answers::answer(
        &p::Reply::Outcome(p::Outcome::Compound { project: PROJECT, operations: 2, first_message: FIRST_MESSAGE }),
        &limits,
    )
    .unwrap();
    let w::KeyedAnswer::FinalAnswer(value) = answer else {
        panic!("final answer");
    };
    let expected = w::OwnerOutcome::Compound(
        w::OwnerOutcomeCompound::new(
            &limits,
            w::OwnerOutcomeCompoundParts { project: PROJECT, operations: 2, first_message: FIRST_MESSAGE },
        )
        .unwrap(),
    );
    let w::Final::Done(value) = w::Final::decode(&limits, &mut Reader::new(value.record())).unwrap() else {
        panic!("done");
    };
    assert_eq!(
        value.outcome(),
        &w::Outcome::Owner(w::OutcomeOwner::new(&limits, w::OutcomeOwnerParts { value: expected }).unwrap())
    );
}
