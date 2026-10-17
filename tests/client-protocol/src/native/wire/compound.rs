//! Join a compound's independently decoded target and answered message position.
use super::{Reader, Token, w};
use jig_client_world::harness::Receipt;
pub(super) fn message(bytes: &[u8]) -> Option<(u64, u32)> {
    let keyed = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("keyed request");
    if keyed.owner() != &w::Owner::Jig {
        return None;
    }
    let w::Ask::Compound(value) = w::Ask::decode(&w::CEILINGS, &mut Reader::new(keyed.request())).expect("jig request")
    else {
        return None;
    };
    let mut reader = Reader::new(value.operations());
    let mut at = 0;
    while !reader.is_empty() {
        let length = reader.u32().expect("piece length");
        let bytes = reader.bytes(length).expect("operation piece");
        if let w::Operation::Message(value) =
            w::Operation::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("operation")
        {
            return Some((value.task(), at));
        }
        at += 1;
    }
    None
}
pub(super) fn receipt(attempt: Token, message: Option<(u64, u32)>, bytes: &[u8]) -> Vec<Receipt> {
    let Some((task, at)) = message else {
        return Vec::new();
    };
    let w::Final::Done(done) = w::Final::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("final answer") else {
        return Vec::new();
    };
    let w::Outcome::Owner(owner) = done.outcome() else {
        return Vec::new();
    };
    let w::OwnerOutcome::Compound(value) = owner.value() else {
        return Vec::new();
    };
    let message = value.first_message().checked_add(u64::from(at)).expect("bounded message positions");
    vec![Receipt::Answer { attempt, final_answer: true, words: Some((task, message)) }]
}
