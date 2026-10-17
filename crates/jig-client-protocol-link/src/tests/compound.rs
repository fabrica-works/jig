use super::*;
const PROJECT: u32 = 3;
const FIRST_MESSAGE: u64 = 19;
#[test]
fn a_compound_answer_names_its_first_message_and_operations() {
    let limits = fixtures::limits();
    let value = w::OwnerOutcome::Compound(
        w::OwnerOutcomeCompound::new(
            &limits.wire,
            w::OwnerOutcomeCompoundParts { project: PROJECT, operations: 2, first_message: FIRST_MESSAGE },
        )
        .unwrap(),
    );
    let outcome = w::Outcome::Owner(w::OutcomeOwner::new(&limits.wire, w::OutcomeOwnerParts { value }).unwrap());
    let answer = w::Final::Done(w::Done::new(&limits.wire, w::DoneParts { outcome }).unwrap());
    let mut bytes = skein_lib::Writer::new(usize::try_from(answer.measure()).unwrap());
    answer.encode(&mut bytes).unwrap();
    let wire = w::KeyedAnswer::FinalAnswer(
        w::Owned::new(
            &limits.wire,
            w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: bytes.finish() },
        )
        .unwrap(),
    );
    assert_eq!(
        translate::answers::answer(wire, &limits),
        Ok(c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Compound {
            project: PROJECT,
            operations: 2,
            first_message: FIRST_MESSAGE
        })))
    );
}
