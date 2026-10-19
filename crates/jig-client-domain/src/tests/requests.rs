use super::fixtures::{
    self, Client, ENGINE_KEY, KEY, NAME, OTHER_ENGINE_KEY, OTHER_KEY, OTHER_NAME, OTHER_PARTY, PARTY, me, record, stop,
};
use crate::*;
use skein_lib::{Duration, List, RequestEnvelope, Time};
#[test]
fn a_request_is_saved_before_it_is_sent_under_its_key() {
    let mut c = Client::live();
    c.step(Event::Send { name: None, key: KEY, request: stop() });
    let downs = c.downs();
    let [Down::Save { record }, Down::Send { key, request, .. }] = downs.as_ref() else {
        panic!("save then send");
    };
    assert_eq!(record.request.key, KEY);
    assert_eq!(record.scope, PARTY);
    assert_eq!(*key, record.key);
    assert_eq!(*request, stop());
    assert_eq!(c.told().as_ref(), &[Told::Pending { key: KEY }]);
    assert_eq!(c.domain.pending(KEY), Some(Pending { progress: Progress::InFlight, party: PARTY, owner: None }));
}
#[test]
fn a_second_send_of_the_same_request_joins_the_first() {
    let mut c = Client::live();
    c.send();
    c.step(Event::Send { name: None, key: KEY, request: stop() });
    assert!(c.downs().is_empty());
    assert!(c.told().is_empty());
}
#[test]
fn a_final_answer_is_told_once_with_its_outcome() {
    let mut c = Client::live();
    let attempt = c.send();
    let answer = Answered::Jig(contract::Answer::Done(contract::Outcome::Changed));
    c.step(Event::Up {
        up: Up::Answer { attempt, envelope: RequestEnvelope::Final, answer: Some(answer.clone()), why: None },
    });
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Answered(answer.clone()) }]);
    c.step(Event::Up { up: Up::Answer { attempt, envelope: RequestEnvelope::Final, answer: Some(answer), why: None } });
    assert!(c.told().is_empty());
    assert_eq!(c.domain.pending(KEY), None);
}
#[test]
fn a_send_past_the_table_s_limit_is_answered_at_once() {
    let mut c = Client::live();
    c.env.limits.requests.requests = 4;
    for number in 0..c.env.limits.requests.requests {
        c.step(Event::Send { name: None, key: Key { asker: KEY.asker, number: u64::from(number) }, request: stop() });
    }
    c.downs();
    c.told();
    c.step(Event::Send { name: None, key: OTHER_KEY, request: stop() });
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: OTHER_KEY, outcome: Outcome::Limit }]);
    assert!(c.downs().is_empty());
}
#[test]
fn a_send_before_me_names_a_party_is_unconfirmed() {
    let mut c = Client::started();
    c.step(Event::Send { name: None, key: KEY, request: stop() });
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Unconfirmed }]);
    assert!(c.downs().is_empty());
}
#[test]
fn another_party_s_kept_request_stays_parked() {
    let mut kept = List::with_capacity(1);
    kept.push(record(KEY, ENGINE_KEY, OTHER_PARTY)).unwrap();
    let mut c = Client::new(fixtures::limits());
    c.step(Event::Start { kept });
    c.downs();
    c.told();
    c.step(Event::Up { up: me(PARTY, Duration::from_secs(1)) });
    assert!(c.downs().is_empty());
    assert_eq!(c.domain.pending(KEY), Some(Pending { progress: Progress::Parked, party: OTHER_PARTY, owner: None }));
}
#[test]
fn a_kept_record_is_restored_and_told_kept_by_its_key() {
    let mut kept = List::with_capacity(1);
    kept.push(record(KEY, ENGINE_KEY, PARTY)).unwrap();
    let mut c = Client::new(fixtures::limits());
    c.step(Event::Start { kept });
    assert_eq!(c.told().as_ref(), &[Told::Kept { key: KEY, request: stop() }, Told::Pending { key: KEY }]);
    c.downs();
    c.step(Event::Up { up: me(PARTY, Duration::from_secs(1)) });
    let downs = c.downs();
    let [Down::Send { key, request, .. }] = downs.as_ref() else {
        panic!("restored send");
    };
    assert_eq!(*key, ENGINE_KEY);
    assert_eq!(*request, stop());
}
#[test]
fn a_kept_record_past_the_table_s_room_is_dropped_and_told() {
    let mut limits = fixtures::limits();
    limits.requests.requests = 1;
    let mut kept = List::with_capacity(2);
    kept.push(record(KEY, ENGINE_KEY, PARTY)).unwrap();
    kept.push(record(OTHER_KEY, OTHER_ENGINE_KEY, PARTY)).unwrap();
    let mut c = Client::new(limits);
    c.step(Event::Start { kept });
    assert_eq!(
        c.told().as_ref(),
        &[Told::Kept { key: KEY, request: stop() }, Told::Dropped { count: 1 }, Told::Pending { key: KEY }]
    );
    assert_eq!(c.domain.pending(OTHER_KEY), None);
}
#[test]
fn a_kept_record_whose_engine_key_is_held_is_dropped_and_told() {
    // Two records share one engine key; only the first can be restored.
    const RECORDS: u32 = 2;
    let mut kept = List::with_capacity(RECORDS);
    kept.push(record(KEY, ENGINE_KEY, PARTY)).unwrap();
    kept.push(record(OTHER_KEY, ENGINE_KEY, PARTY)).unwrap();
    let mut c = Client::new(fixtures::limits());
    c.step(Event::Start { kept });
    assert_eq!(
        c.told().as_ref(),
        &[Told::Kept { key: KEY, request: stop() }, Told::Dropped { count: 1 }, Told::Pending { key: KEY }]
    );
    assert_eq!(c.domain.pending(OTHER_KEY), None);
}
#[test]
fn a_retrying_request_says_why_once_per_answer() {
    const ROWS: [(RequestEnvelope, Retry); 4] = [
        (RequestEnvelope::Again, Retry::Busy),
        (RequestEnvelope::Again, Retry::NotReady),
        (RequestEnvelope::Lost, Retry::Lost),
        (RequestEnvelope::Final, Retry::Undecodable),
    ];
    for (envelope, why) in ROWS {
        let mut c = Client::live();
        let attempt = c.send();
        c.step(Event::Up { up: Up::Answer { attempt, envelope, answer: None, why: Some(why) } });
        assert_eq!(c.told().as_ref(), &[Told::Pending { key: KEY }], "{why:?}");
        assert_eq!(
            c.domain.pending(KEY),
            Some(Pending { progress: Progress::Retrying { why }, party: PARTY, owner: None }),
            "{why:?}"
        );
        c.step(Event::Up { up: Up::Answer { attempt, envelope, answer: None, why: Some(why) } });
        assert!(c.told().is_empty(), "{why:?}");
    }
}
#[test]
fn a_request_past_the_retention_is_answered_unknown() {
    let mut c = Client::live();
    c.send();
    c.at(Time::from_nanos(1));
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Unknown }]);
    assert_eq!(c.domain.pending(KEY), None);
}
#[test]
#[should_panic(expected = "request key conflict")]
fn a_second_send_with_another_request_is_a_loud_key_conflict() {
    let mut c = Client::live();
    c.send();
    c.step(Event::Send {
        name: None,
        key: KEY,
        request: Ask::Jig(Box::new(contract::Ask::Stop { project: 1, task: fixtures::OTHER_SUBJECT })),
    });
}

#[test]
fn a_named_send_is_saved_and_sent_under_its_name() {
    let mut c = Client::live();
    c.step(Event::Send { key: KEY, name: Some(NAME), request: stop() });
    let downs = c.downs();
    let [Down::Save { record }, Down::Send { key, .. }] = downs.as_ref() else {
        panic!("save then send");
    };
    assert_eq!((record.key, record.request.key, *key), (NAME, KEY, NAME));
}
#[test]
fn a_name_another_key_holds_is_answered_held_at_once() {
    let mut c = Client::live();
    c.send_as(KEY, Some(NAME));
    c.step(Event::Send { key: OTHER_KEY, name: Some(NAME), request: stop() });
    assert!(c.downs().is_empty());
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: OTHER_KEY, outcome: Outcome::Held }]);
}
fn restored_named() -> Client {
    // One restored request holds the name before a party is confirmed.
    const RECORDS: u32 = 1;
    let mut kept = List::with_capacity(RECORDS);
    kept.push(record(KEY, NAME, PARTY)).unwrap();
    let mut c = Client::new(fixtures::limits());
    c.step(Event::Start { kept });
    c.downs();
    c.told();
    c
}
#[test]
fn a_name_a_restored_record_holds_is_answered_held() {
    let mut c = restored_named();
    c.step(Event::Up { up: me(PARTY, Duration::from_secs(1)) });
    c.downs();
    c.told();
    c.step(Event::Send { key: OTHER_KEY, name: Some(NAME), request: stop() });
    assert!(c.downs().is_empty());
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: OTHER_KEY, outcome: Outcome::Held }]);
}
#[test]
fn a_held_name_sent_before_me_names_a_party_is_unconfirmed() {
    let mut c = restored_named();
    c.step(Event::Send { key: OTHER_KEY, name: Some(NAME), request: stop() });
    assert!(c.downs().is_empty());
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: OTHER_KEY, outcome: Outcome::Unconfirmed }]);
}
#[test]
fn the_same_key_and_name_sent_again_joins_the_first() {
    let mut c = Client::live();
    c.send_as(KEY, Some(NAME));
    c.step(Event::Send { key: KEY, name: Some(NAME), request: stop() });
    assert!(c.downs().is_empty());
    assert!(c.told().is_empty());
}
#[test]
#[should_panic(expected = "request key conflict")]
fn the_same_key_under_another_name_is_a_loud_key_conflict() {
    let mut c = Client::live();
    c.send_as(KEY, Some(NAME));
    c.step(Event::Send { key: KEY, name: Some(OTHER_NAME), request: stop() });
}
#[test]
fn a_name_is_admitted_again_once_its_holder_is_answered() {
    let mut c = Client::live();
    let attempt = c.send_as(KEY, Some(NAME));
    let answer = Answered::Jig(contract::Answer::Done(contract::Outcome::Changed));
    c.step(Event::Up { up: Up::Answer { attempt, envelope: RequestEnvelope::Final, answer: Some(answer), why: None } });
    c.downs();
    c.told();
    c.step(Event::Send { key: OTHER_KEY, name: Some(NAME), request: stop() });
    let downs = c.downs();
    let [Down::Save { record }, Down::Send { key, .. }] = downs.as_ref() else {
        panic!("save then send");
    };
    assert_eq!((record.key, record.request.key, *key), (NAME, OTHER_KEY, NAME));
}
#[test]
fn a_name_admitted_again_is_retired_by_its_own_not_kept() {
    let mut c = Client::live();
    c.send_as(KEY, Some(NAME));
    c.step(Event::Up { up: Up::NotKept { key: NAME } });
    c.send_as(OTHER_KEY, Some(NAME));
    c.step(Event::Up { up: Up::NotKept { key: NAME } });
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: OTHER_KEY, outcome: Outcome::NotKept }]);
}
