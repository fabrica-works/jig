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
    c.step(Event::Carry { key, name: NAME, request: ask(), value: Some(value(bytes)) });
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
    c.step(Event::Carry {
        key: KEY,
        name: NAME,
        request: ask(),
        value: Some(Secret::new(skein_lib::bytes::zeroed(VALUE_BYTES + 1))),
    });
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Limit }]);
    for number in 0..4 {
        let name = [u8::try_from(number).expect("small"); 16];
        c.step(Event::Carry {
            key: Key { asker: KEY.asker, number },
            name,
            request: ask(),
            value: Some(value(b"key")),
        });
    }
    c.downs();
    c.told();
    c.step(Event::Carry { key: OTHER_KEY, name: [9; 16], request: ask(), value: Some(value(b"key")) });
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: OTHER_KEY, outcome: Outcome::Limit }]);
    assert!(c.downs().is_empty());
}

const ACCOUNT: u32 = 7;
const MOVE: u32 = 9;
fn change(change: contract::AccountChange) -> Ask {
    Ask::Jig(Box::new(contract::Ask::AccountChange { holder: contract::AccountHolder::Own, change }))
}
fn move_out() -> Ask {
    change(contract::AccountChange::MoveOut { account: ACCOUNT })
}
fn moving_out(record: &[u8]) -> Answered {
    Answered::Jig(contract::Answer::Done(contract::Outcome::Made {
        what: contract::Made::MovingOut { account: ACCOUNT, number: MOVE, record: value(record) },
    }))
}
fn moving() -> (Client, Token) {
    let mut c = Client::live();
    c.step(Event::Carry { key: KEY, name: NAME, request: move_out(), value: None });
    let downs = c.downs();
    let [Down::Send { attempt, key: NAME, request, value: None }] = downs.as_ref() else {
        panic!("one move-out exchange without Save: {downs:?}");
    };
    assert_eq!(*request, move_out());
    c.told();
    (c, *attempt)
}
#[test]
fn a_move_out_is_sent_once_without_a_save() {
    let (mut c, _) = moving();
    never_sent_again(&mut c);
}
#[test]
fn a_move_out_answer_hands_its_record_to_its_caller() {
    let (mut c, attempt) = moving();
    let done = moving_out(b"moved record");
    answer(&mut c, attempt, RequestEnvelope::Final, Some(done.clone()));
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Answered(done) }]);
    assert!(c.downs().is_empty());
    never_sent_again(&mut c);
}
#[test]
fn a_move_out_answer_with_no_record_is_lost() {
    let (mut c, attempt) = moving();
    answer(&mut c, attempt, RequestEnvelope::Final, Some(moving_out(b"")));
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Lost }]);
    never_sent_again(&mut c);
}
#[test]
fn a_busy_move_out_is_unsent_without_a_retry() {
    let (mut c, attempt) = moving();
    answer(&mut c, attempt, RequestEnvelope::Again, None);
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Unsent }]);
    never_sent_again(&mut c);
}
#[test]
fn a_move_out_without_a_whole_answer_is_lost() {
    for envelope in [RequestEnvelope::Lost, RequestEnvelope::Final] {
        let (mut c, attempt) = moving();
        answer(&mut c, attempt, envelope, None);
        assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Lost }]);
        never_sent_again(&mut c);
    }
}
#[test]
fn a_move_in_goes_once_with_its_value_without_a_save() {
    let mut c = Client::live();
    let file = value(b"moved record");
    let request = change(contract::AccountChange::MoveIn { digest: skein_lib::sha256::digest(file.expose()) });
    c.step(Event::Carry { key: KEY, name: NAME, request: request.clone(), value: Some(file.clone()) });
    let downs = c.downs();
    let [Down::Send { key: NAME, request: sent, value: Some(sent_file), .. }] = downs.as_ref() else {
        panic!("one move-in exchange without Save: {downs:?}");
    };
    assert_eq!((sent, sent_file), (&request, &file));
    never_sent_again(&mut c);
}
#[test]
fn written_is_saved_before_send_and_retried_after_busy() {
    let mut c = Client::live();
    // Keep the retry inside retention, past the fixture's one-second margin.
    c.step(Event::Up { up: me(super::fixtures::PARTY, Duration::from_secs(30)) });
    c.downs();
    c.told();
    let request = change(contract::AccountChange::Written { number: MOVE });
    c.step(Event::Send { key: KEY, name: Some(NAME), request: request.clone() });
    let downs = c.downs();
    let [Down::Save { record }, Down::Send { attempt, key: NAME, request: sent, value: None }] = downs.as_ref() else {
        panic!("written saved before send: {downs:?}");
    };
    assert_eq!((&record.request.request, sent), (&request, &request));
    answer(&mut c, *attempt, RequestEnvelope::Again, None);
    c.downs();
    c.at(next_deadline(&c.domain).unwrap());
    let downs = c.downs();
    let [Down::Send { key: NAME, request: sent, value: None, .. }] = downs.as_ref() else {
        panic!("written retried under its name: {downs:?}");
    };
    assert_eq!(*sent, request);
}
#[test]
fn an_empty_carry_joins_without_becoming_a_value() {
    let (mut c, _) = moving();
    c.step(Event::Carry { key: KEY, name: NAME, request: move_out(), value: None });
    assert!(c.downs().is_empty());
    assert!(c.told().is_empty());
}
#[test]
#[should_panic(expected = "request key conflict")]
fn a_missing_value_differs_from_an_empty_value() {
    let (mut c, _) = moving();
    c.step(Event::Carry { key: KEY, name: NAME, request: move_out(), value: Some(value(b"")) });
}

#[test]
fn a_finish_is_carried_once_never_saved_and_answered_once() {
    const SOURCE: u32 = 29;
    let address = b"http://127.0.0.1:1455/auth/callback?code=c&state=s";
    let request =
        change(contract::AccountChange::FinishSignIn { source: SOURCE, digest: skein_lib::sha256::digest(address) });
    let mut c = Client::live();
    c.step(Event::Carry { key: KEY, name: NAME, request: request.clone(), value: Some(value(address)) });
    let downs = c.downs();
    let [Down::Send { attempt, key: NAME, request: sent, value: Some(bytes) }] = downs.as_ref() else {
        panic!("one send, no save")
    };
    assert_eq!(sent, &request);
    assert_eq!(bytes.expose(), address);
    c.told();
    never_sent_again(&mut c);
    let done =
        Answered::Jig(contract::Answer::Done(contract::Outcome::Owner(contract::OwnerOutcome::AccountFinishing {
            source: SOURCE,
        })));
    answer(&mut c, *attempt, RequestEnvelope::Final, Some(done.clone()));
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Answered(done.clone()) }]);
    answer(&mut c, *attempt, RequestEnvelope::Final, Some(done));
    assert!(c.told().is_empty());
    assert!(c.downs().is_empty());
}
