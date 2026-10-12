//! Client wire refusals and round trips (protocol/client.md, section 3).
mod fixtures;
use crate::*;
use skein_lib::Reader;

#[test]
fn a_keyed_request_past_its_bound_is_refused_naming_it() {
    let limits = fixtures::limits();
    let request = fixtures::bytes(limits.keyed_request.checked_add(1).expect("small bound"));
    let refused = Keyed::new(&limits, KeyedParts { key: [7; 16], owner: Owner::Jig, version: 1, request });
    assert_eq!(refused, Err(Problem { path: Path::KeyedRequest, reason: skein_codec::Reason::Bound }));
}

#[test]
fn an_owned_record_keeps_its_owners_version() {
    let limits = fixtures::limits();
    let owned = fixtures::owned();
    let bytes = fixtures::encoded(&owned, Owned::measure, Owned::encode);
    assert_eq!(Owned::decode(&limits, &mut Reader::new(&bytes)), Ok(owned));
}

#[test]
fn a_refusal_with_its_facts_round_trips_unchanged() {
    let limits = fixtures::limits();
    let refused = fixtures::refused_moved();
    let bytes = fixtures::encoded(&refused, Refused::measure, Refused::encode);
    assert_eq!(Refused::decode(&limits, &mut Reader::new(&bytes)), Ok(refused));
}

#[test]
fn each_answer_kind_round_trips_unchanged() {
    let limits = fixtures::limits();
    for answer in [Answer::FinalAnswer(fixtures::owned()), Answer::Busy, Answer::NotReady, Answer::SignedOut] {
        let bytes = fixtures::encoded(&answer, Answer::measure, Answer::encode);
        assert_eq!(Answer::decode(&limits, &mut Reader::new(&bytes)), Ok(answer));
    }
}

#[test]
fn a_bearer_token_past_its_bound_is_refused() {
    let limits = fixtures::limits();
    let token = fixtures::bytes(limits.signed_in_token.checked_add(1).expect("small bound"));
    assert_eq!(
        SignedIn::new(&limits, SignedInParts { token, party: 17, expires: 29 }),
        Err(Problem { path: Path::SignedInToken, reason: skein_codec::Reason::Bound })
    );
}
