//! Requests carrying a value (client/domain/client.md, 6): sent once beside
//! the table, never saved nor sent again; one answer each.
use super::fixtures::{Client, KEY, NAME, OTHER_KEY, me, stop};
use crate::contract::Scope;
use crate::*;
use alloc::boxed::Box;
use skein_lib::{Duration, RequestEnvelope, Secret, Token};
fn ask() -> Ask {
    Ask::Family { scope: Scope::Project(1), record: Family { family: 19, version: 1, bytes: Box::new([1, 2, 3]) } }
}
fn value(bytes: &[u8]) -> Secret {
    Secret::new(Box::from(bytes))
}
fn carry(c: &mut Client, key: Key, bytes: &[u8]) {
    c.step(Event::Carry { key, name: NAME, request: ask(), value: value(bytes) });
}
/// A live client with one carried request in flight; its attempt.
fn carried() -> (Client, Token) {
    let mut c = Client::live();
    carry(&mut c, KEY, b"key");
    let downs = c.downs();
    let [Down::Send { attempt, key, request, value: Some(sent) }] = downs.as_ref() else {
        panic!("one send carrying the value, no save: {downs:?}");
    };
    assert_eq!((*key, request, sent.expose()), (NAME, &ask(), b"key".as_slice()));
    assert_eq!(c.told().as_ref(), &[Told::Pending { key: KEY }]);
    (c, *attempt)
}
fn answer(c: &mut Client, attempt: Token, envelope: RequestEnvelope, answer: Option<Answered>) {
    c.step(Event::Up { up: Up::Answer { attempt, envelope, answer, why: None } });
}
fn never_sent_again(c: &mut Client) {
    let later = c.env.now.saturating_add(Duration::from_secs(1));
    c.at(later);
    assert!(c.downs().is_empty(), "never sent again");
}

#[test]
fn a_carried_request_is_sent_once_and_never_saved() {
    let (mut c, _) = carried();
    assert_eq!(c.domain.pending(KEY), Some(Pending { progress: Progress::InFlight, party: 1, owner: Some(19) }));
    never_sent_again(&mut c);
    assert!(c.told().is_empty());
}
#[test]
fn its_one_answer_is_the_engine_s() {
    let (mut c, attempt) = carried();
    let done = Answered::Family(Family { family: 19, version: 1, bytes: Box::new([7]) });
    answer(&mut c, attempt, RequestEnvelope::Final, Some(done.clone()));
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Answered(done.clone()) }]);
    answer(&mut c, attempt, RequestEnvelope::Final, Some(done));
    assert!(c.told().is_empty(), "answered once");
    assert_eq!(c.domain.pending(KEY), None);
}
#[test]
fn an_attempt_the_engine_did_not_make_is_unsent_and_not_sent_again() {
    for envelope in [RequestEnvelope::Again, RequestEnvelope::SignedOut] {
        let (mut c, attempt) = carried();
        answer(&mut c, attempt, envelope, None);
        assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Unsent }]);
        never_sent_again(&mut c);
        assert_eq!(c.domain.pending(KEY), None);
    }
}
#[test]
fn an_exchange_with_no_whole_answer_is_lost() {
    for envelope in [RequestEnvelope::Lost, RequestEnvelope::Final] {
        let (mut c, attempt) = carried();
        answer(&mut c, attempt, envelope, None);
        assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Lost }]);
        never_sent_again(&mut c);
    }
}
#[test]
fn a_carried_request_the_link_cannot_send_is_unsent_at_once() {
    let mut unconfirmed = Client::started();
    let mut signed_out = Client::live();
    signed_out.step(Event::Up { up: Up::SignedOut });
    let mut not_served = Client::live();
    not_served.step(Event::Up { up: Up::FamilyServed { family: 19, versions: None } });
    for mut c in [unconfirmed, signed_out, not_served] {
        c.downs();
        c.told();
        carry(&mut c, KEY, b"key");
        assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Unsent }]);
        assert!(c.downs().is_empty());
        assert_eq!(c.domain.pending(KEY), None);
    }
    unconfirmed = Client::started();
    unconfirmed.step(Event::Up { up: me(1, Duration::from_secs(1)) });
    unconfirmed.told();
    carry(&mut unconfirmed, KEY, b"key");
    assert_eq!(unconfirmed.told().as_ref(), &[Told::Pending { key: KEY }], "once me names its party");
}
#[test]
fn a_name_held_either_side_of_the_table_is_answered_held() {
    let mut c = Client::live();
    c.send_as(OTHER_KEY, Some(NAME));
    carry(&mut c, KEY, b"key");
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Held }]);
    assert!(c.downs().is_empty());
    let (mut c, _) = carried();
    c.step(Event::Send { key: OTHER_KEY, name: Some(NAME), request: stop() });
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: OTHER_KEY, outcome: Outcome::Held }]);
    carry(&mut c, OTHER_KEY, b"other");
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: OTHER_KEY, outcome: Outcome::Held }]);
    assert!(c.downs().is_empty());
}
#[test]
fn the_same_carried_request_again_joins_the_first() {
    let (mut c, _) = carried();
    carry(&mut c, KEY, b"key");
    assert!(c.downs().is_empty());
    assert!(c.told().is_empty());
}
#[test]
#[should_panic(expected = "request key conflict")]
fn the_same_key_with_another_value_is_a_loud_key_conflict() {
    let (mut c, _) = carried();
    carry(&mut c, KEY, b"other");
}
#[test]
#[should_panic(expected = "request key conflict")]
fn a_carried_key_sent_to_the_table_is_a_loud_key_conflict() {
    let (mut c, _) = carried();
    c.step(Event::Send { key: KEY, name: None, request: ask() });
}
#[test]
fn a_carried_request_past_the_client_s_bounds_is_answered_at_once() {
    let mut c = Client::live();
    c.step(Event::Carry { key: KEY, name: NAME, request: ask(), value: Secret::new(Box::new([0; VALUE_BYTES + 1])) });
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Limit }]);
    for number in 0..4 {
        let name = [u8::try_from(number).expect("small"); 16];
        c.step(Event::Carry { key: Key { asker: KEY.asker, number }, name, request: ask(), value: value(b"key") });
    }
    c.downs();
    c.told();
    c.step(Event::Carry { key: OTHER_KEY, name: [9; 16], request: ask(), value: value(b"key") });
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: OTHER_KEY, outcome: Outcome::Limit }]);
    assert!(c.downs().is_empty());
}
