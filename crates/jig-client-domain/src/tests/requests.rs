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

fn seen_send(
    c: &mut Client,
    key: Key,
    name: Option<skein_lib::RequestKey>,
) -> (skein_lib::Token, skein_lib::RequestKey) {
    c.step(Event::Send { key, name, request: fixtures::seen() });
    seen_attempt(c)
}
fn seen_attempt(c: &mut Client) -> (skein_lib::Token, skein_lib::RequestKey) {
    let downs = c.downs();
    let [Down::Send { attempt, key, request, value: None }] = downs.as_ref() else {
        panic!("Seen sends without saving: {downs:?}");
    };
    assert_eq!(*request, fixtures::seen());
    (*attempt, *key)
}
fn retry_client() -> Client {
    let mut c = Client::live();
    c.step(Event::Up { up: me(PARTY, Duration::from_secs(60)) });
    c.downs();
    c.told();
    c
}
fn confirm(c: &mut Client, watch: skein_lib::Token, party: u64) {
    let mut up = me(party, Duration::from_secs(60));
    let Up::Watched { watch: token, .. } = &mut up else { panic!("frame fixture") };
    *token = watch;
    c.step(Event::Up { up });
}
fn final_seen(c: &mut Client, attempt: skein_lib::Token) {
    c.step(Event::Up {
        up: Up::Answer {
            attempt,
            envelope: RequestEnvelope::Final,
            answer: Some(Answered::Jig(contract::Answer::Done(contract::Outcome::Changed))),
            why: None,
        },
    });
}
#[test]
fn seen_stays_pending_without_saving_and_draws_fresh_engine_keys() {
    let mut c = Client::live();
    let (_, first) = seen_send(&mut c, KEY, None);
    assert_eq!(c.told().as_ref(), &[Told::Pending { key: KEY }]);
    assert_eq!(c.domain.pending(KEY), Some(Pending { progress: Progress::InFlight, party: PARTY, owner: None }));
    let (_, second) = seen_send(&mut c, OTHER_KEY, None);
    assert_ne!(first, second);
    assert_eq!(c.told().as_ref(), &[Told::Pending { key: OTHER_KEY }]);
}
#[test]
fn seen_duplicates_join_with_or_without_a_caller_name() {
    for name in [None, Some(NAME)] {
        let mut c = Client::live();
        let (_, engine) = seen_send(&mut c, KEY, name);
        if let Some(name) = name {
            assert_eq!(engine, name);
        }
        c.told();
        for duplicate in [name, None] {
            c.step(Event::Send { key: KEY, name: duplicate, request: fixtures::seen() });
            assert!(c.downs().is_empty());
            assert!(c.told().is_empty());
        }
    }
}
#[test]
fn seen_and_ordinary_requests_hold_each_others_names() {
    for (first, second) in
        [(fixtures::seen(), stop()), (stop(), fixtures::seen()), (fixtures::seen(), fixtures::seen())]
    {
        let mut c = Client::live();
        c.step(Event::Send { key: KEY, name: Some(NAME), request: first });
        c.downs();
        c.told();
        c.step(Event::Send { key: OTHER_KEY, name: Some(NAME), request: second });
        assert!(c.downs().is_empty());
        assert_eq!(c.told().as_ref(), &[Told::Answered { key: OTHER_KEY, outcome: Outcome::Held }]);
    }
}
#[test]
#[should_panic(expected = "request key conflict")]
fn seen_under_the_same_key_with_another_request_conflicts() {
    let mut c = Client::live();
    seen_send(&mut c, KEY, None);
    c.step(Event::Send { key: KEY, name: None, request: stop() });
}
#[test]
#[should_panic(expected = "request key conflict")]
fn seen_under_the_same_key_with_another_name_conflicts() {
    let mut c = Client::live();
    seen_send(&mut c, KEY, Some(NAME));
    c.step(Event::Send { key: KEY, name: Some(OTHER_NAME), request: fixtures::seen() });
}
#[test]
fn seen_retries_under_the_same_key_without_saving() {
    for (envelope, why) in [
        (RequestEnvelope::Again, Retry::Busy),
        (RequestEnvelope::Again, Retry::NotReady),
        (RequestEnvelope::Lost, Retry::Lost),
    ] {
        let mut c = retry_client();
        let (attempt, engine) = seen_send(&mut c, KEY, None);
        c.told();
        c.step(Event::Up { up: Up::Answer { attempt, envelope, answer: None, why: Some(why) } });
        assert_eq!(c.domain.pending(KEY).unwrap().progress, Progress::Retrying { why });
        assert_eq!(c.told().as_ref(), &[Told::Pending { key: KEY }]);
        assert!(c.downs().is_empty());
        c.step(Event::Up { up: Up::Answer { attempt, envelope, answer: None, why: Some(why) } });
        assert!(c.told().is_empty());
        c.at(next_deadline(&c.domain).unwrap());
        let (next, key) = seen_attempt(&mut c);
        assert_ne!(next, attempt);
        assert_eq!(key, engine);
        assert_eq!(c.told().as_ref(), &[Told::Pending { key: KEY }]);
    }
}
#[test]
fn seen_admitted_offline_is_not_saved_before_going_live() {
    let mut c = retry_client();
    c.end(fixtures::ME_WATCH, contract::WatchEnd::Dropped);
    c.downs();
    c.told();
    c.step(Event::Send { key: KEY, name: Some(NAME), request: fixtures::seen() });
    assert!(c.downs().is_empty());
    assert_eq!(c.domain.pending(KEY).unwrap().progress, Progress::Parked);
    c.told();
    c.at(next_deadline(&c.domain).unwrap());
    let watch = c.name(Watchable::Jig(contract::Subject::Me));
    c.downs();
    confirm(&mut c, watch, PARTY);
    assert_eq!(seen_attempt(&mut c).1, NAME);
    assert_eq!(c.domain.pending(KEY).unwrap().progress, Progress::InFlight);
}
#[test]
fn seen_parks_across_signout_until_its_party_returns() {
    let mut c = retry_client();
    let (_, engine) = seen_send(&mut c, KEY, None);
    c.told();
    c.step(Event::Up { up: Up::SignedOut });
    assert_eq!(c.domain.pending(KEY).unwrap().progress, Progress::Parked);
    assert!(c.downs().is_empty());
    c.told();
    c.step(Event::Up { up: Up::SignedIn });
    let watch = c.name(Watchable::Jig(contract::Subject::Me));
    c.downs();
    c.told();
    confirm(&mut c, watch, OTHER_PARTY);
    assert!(c.downs().is_empty());
    assert_eq!(c.domain.pending(KEY).unwrap().progress, Progress::Parked);
    c.told();
    confirm(&mut c, watch, PARTY);
    assert_eq!(seen_attempt(&mut c).1, engine);
}
#[test]
fn seen_final_answer_erases_and_releases_its_name_once() {
    let mut c = Client::live();
    let (attempt, _) = seen_send(&mut c, KEY, Some(NAME));
    c.told();
    final_seen(&mut c, attempt);
    assert_eq!(c.downs().as_ref(), &[Down::Erase { key: NAME }]);
    assert_eq!(
        c.told().as_ref(),
        &[Told::Answered {
            key: KEY,
            outcome: Outcome::Answered(Answered::Jig(contract::Answer::Done(contract::Outcome::Changed))),
        }]
    );
    assert_eq!(c.domain.pending(KEY), None);
    final_seen(&mut c, attempt);
    assert!(c.downs().is_empty());
    assert!(c.told().is_empty());
    assert_eq!(seen_send(&mut c, OTHER_KEY, Some(NAME)).1, NAME);
}
#[test]
fn seen_past_retention_erases_and_answers_unknown_once() {
    let mut c = Client::live();
    let (attempt, key) = seen_send(&mut c, KEY, None);
    c.told();
    c.at(Time::from_nanos(1));
    assert_eq!(c.downs().as_ref(), &[Down::Erase { key }]);
    assert_eq!(c.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Unknown }]);
    assert_eq!(c.domain.pending(KEY), None);
    final_seen(&mut c, attempt);
    assert!(c.told().is_empty());
    assert!(c.downs().is_empty());
}
#[test]
fn legacy_seen_restores_kept_without_resaving_then_erases() {
    for first_sent in [None, Some(skein_lib::Wall::EPOCH)] {
        let mut record = record(KEY, NAME, PARTY);
        record.first_sent = first_sent;
        record.request.request = fixtures::seen();
        let mut kept = List::with_capacity(1);
        kept.push(record).unwrap();
        let mut c = Client::new(fixtures::limits());
        c.step(Event::Start { kept });
        assert_eq!(
            c.told().as_ref(),
            &[Told::Kept { key: KEY, request: fixtures::seen() }, Told::Pending { key: KEY }]
        );
        assert!(c.downs().is_empty());
        c.step(Event::Up { up: me(PARTY, Duration::from_secs(60)) });
        let (attempt, key) = seen_attempt(&mut c);
        assert_eq!(key, NAME);
        c.told();
        final_seen(&mut c, attempt);
        assert_eq!(c.downs().as_ref(), &[Down::Erase { key: NAME }]);
        assert_eq!(c.told().len(), 1);
        assert_eq!(c.domain.pending(KEY), None);
    }
}
