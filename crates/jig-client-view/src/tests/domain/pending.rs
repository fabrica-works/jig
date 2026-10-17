use super::{confirmation, native, spine};
use crate::tests::fixtures::refusal_answer;
use crate::tests::{
    cards::{contains, tree},
    fixtures::{FireClient, Fixture, MOST, kept, one},
};
use crate::{Event, Request, action::Act, intents, notices::kept::Kind, objects::Card, page::Page};
use jig_client::{Asker, Down, Handle, Key, Progress, Retry, Up, contract::*};
use skein_lib::{Duration, Id, List, RequestEnvelope, Time, Token, Wall};

const ME: u64 = 7;
const OTHER: u64 = 8;
const PROJECT: u32 = 1;
const TASK: u64 = 8;
const FIRST: u8 = 1;
const SECOND: u8 = 2;
const THIRD: u8 = 3;
const OUTSIDE: Asker = Asker(u32::MAX);
const EXPIRES: Duration = Duration::from_secs(1000);

fn ready() -> (Fixture, Id<crate::objects::Object>, Handle) {
    let (mut fixture, handle) = native::ready(&spine::LIMITS);
    let id = card(&mut fixture);
    (fixture, id, handle)
}
fn card(fixture: &mut Fixture) -> Id<crate::objects::Object> {
    let mut task = confirmation::task();
    task.acts = one(Offered { effect: None, act: ActKind::Stop, admission: Admission::Allowed });
    confirmation::install(fixture, crate::objects::Body::Task(Box::new(task)))
}
fn press(fixture: &mut Fixture, id: Id<crate::objects::Object>) -> Box<[Request]> {
    fixture.drive(fixture.now, Event::Act { action: Act::Intend { object: id.token(), act: ActKind::Stop } });
    fixture.drive(fixture.now, Event::Act { action: Act::Confirm })
}
fn key(rows: &[Request]) -> Key {
    for row in rows {
        if let Request::Client { asked: jig_client::Event::Send { key, .. } } = row {
            return *key;
        }
    }
    panic!("one keyed request");
}
fn attempt(fixture: &Fixture) -> Token {
    for row in fixture.down.as_slice() {
        if let Down::Send { attempt, .. } = row {
            return *attempt;
        }
    }
    panic!("one transport attempt");
}
fn saved_record(fixture: &Fixture) -> &skein_lib::RequestRecord<jig_client::Held> {
    for row in fixture.down.as_slice() {
        if let Down::Save { record } = row {
            return record;
        }
    }
    panic!("one saved request");
}
fn answer(fixture: &mut Fixture, attempt: Token, answer: Answer) {
    fixture.drive(
        fixture.now,
        Up::Answer {
            attempt,
            envelope: RequestEnvelope::Final,
            answer: Some(jig_client::Answered::Jig(answer)),
            why: None,
        },
    );
}
fn inbox() -> Page {
    Page::Work { address: crate::address::Address::Inbox { project: None }, behind: false, excess: 0, refused: None }
}

#[test]
fn a_request_shows_the_progress_the_client_reports() {
    let (mut fixture, id, _) = ready();
    let key = key(&press(&mut fixture, id));
    let token = fixture.intent(key);
    assert_eq!(key.asker, Asker::JIG);
    assert_eq!(
        fixture.client.pending(key),
        Some(jig_client::Pending { party: ME, owner: None, progress: Progress::InFlight })
    );
    assert_eq!(
        fixture.objects().get(id).unwrap().card,
        Card::Deciding { request: Some(token), progress: intents::Progress::InFlight }
    );
    let first = attempt(&fixture);
    fixture.drive(
        Time::ZERO,
        Up::Answer { attempt: first, envelope: RequestEnvelope::Again, answer: None, why: Some(Retry::Busy) },
    );
    assert_eq!(
        fixture.objects().get(id).unwrap().card,
        Card::Deciding { request: Some(token), progress: intents::Progress::Retrying }
    );
    fixture.drive(Time::from_nanos(MOST.as_nanos()), FireClient { wall: Wall::EPOCH });
    let next = attempt(&fixture);
    answer(&mut fixture, next, Answer::Done(Outcome::Changed));
    assert_eq!(fixture.client.pending(key), None);
    assert_eq!(fixture.objects().get(id).unwrap().answer, Some(Answer::Done(Outcome::Changed)));
    answer(&mut fixture, next, refusal_answer(Refusal::NotOffered));
    assert_eq!(fixture.objects().get(id).unwrap().answer, Some(Answer::Done(Outcome::Changed)));
}

#[test]
fn a_request_not_kept_is_offered_again() {
    let (mut fixture, id, _) = ready();
    let original = key(&press(&mut fixture, id));
    let saved = saved_record(&fixture).key;
    fixture.drive(Time::ZERO, Up::NotKept { key: saved });
    assert_eq!(fixture.client.pending(original), None);
    assert_eq!(fixture.objects().get(id).unwrap().card, Card::NotSent { why: intents::NotSent::NotKept });
    assert_eq!(fixture.objects().get(id).unwrap().answer, None);
    assert!(contains(&tree(&fixture, None, Some(id)), b"Request not kept; it was not sent. Try again."));
    fixture.reclaim();
    let fresh = key(&press(&mut fixture, id));
    assert_ne!(fresh, original);
    assert_eq!(fixture.client.pending(fresh).unwrap().party, ME);
}

#[test]
fn a_kept_request_refused_shows_a_notice() {
    let (mut fixture, handle) = spine::restored(&[(FIRST, ME)], List::with_capacity(0), &spine::LIMITS);
    spine::delivered(&mut fixture, Time::ZERO, handle, spine::frame(ME));
    let sent = attempt(&fixture);
    answer(&mut fixture, sent, refusal_answer(Refusal::NotOffered));
    let (id, ()) = fixture.notices().ids().first().unwrap();
    assert_eq!(
        fixture.notices().get(*id).unwrap().kind,
        Kind::PageAnswer { answer: refusal_answer(Refusal::NotOffered) }
    );
    assert!(fixture.intents().ids().is_empty());
    assert!(fixture.kept_requests.is_empty());
}

#[test]
fn another_party_s_kept_work_shows_someone_else_s_work() {
    let (mut fixture, handle) = spine::restored(&[(FIRST, OTHER)], List::with_capacity(0), &spine::LIMITS);
    spine::delivered(&mut fixture, Time::ZERO, handle, spine::frame(ME));
    assert_eq!(fixture.client.pending(Key { asker: Asker::JIG, number: u64::from(FIRST) }).unwrap().party, OTHER);
    assert_eq!(fixture.page(), &Page::SomeoneElse);
    assert!(contains(&tree(&fixture, None, None), b"Someone else"));
    assert!(fixture.intents().ids().is_empty());
}

#[test]
fn an_act_past_the_client_s_limit_is_refused_in_place() {
    let (mut fixture, id, _) = ready();
    for number in [FIRST, SECOND] {
        fixture.drive(
            Time::ZERO,
            jig_client::Event::Send {
                key: Key { asker: OUTSIDE, number: u64::from(number) },
                request: jig_client::Ask::Jig(Box::new(Ask::Stop { project: PROJECT, task: TASK })),
            },
        );
    }
    let refused = key(&press(&mut fixture, id));
    assert_eq!(fixture.client.pending(refused), None);
    assert_eq!(fixture.objects().get(id).unwrap().card, Card::NotSent { why: intents::NotSent::Limit });
    assert_eq!(fixture.objects().get(id).unwrap().answer, None);
    assert!(fixture.down.as_slice().is_empty());
    assert!(contains(&tree(&fixture, None, Some(id)), b"Too many requests in flight"));
    fixture.drive(Time::from_nanos(MOST.as_nanos()), FireClient { wall: Wall::EPOCH });
    assert!(fixture.down.as_slice().is_empty());
}

#[test]
fn an_act_answered_unconfirmed_is_offered_again_once_live() {
    let (mut fixture, id, handle) = ready();
    // The borrowed client's identity disappeared while the Ui still recalled the offer.
    fixture.client = Fixture::new(&spine::LIMITS).client;
    let refused = key(&press(&mut fixture, id));
    assert_eq!(fixture.client.pending(refused), None);
    assert_eq!(fixture.objects().get(id).unwrap().card, Card::NotSent { why: intents::NotSent::Unconfirmed });
    assert_eq!(fixture.objects().get(id).unwrap().answer, None);
    fixture.drive(Time::ZERO, jig_client::Event::Watch { handle, subject: jig_client::Watchable::Jig(Subject::Me) });
    spine::delivered(&mut fixture, Time::ZERO, handle, spine::frame(ME));
    fixture.reclaim();
    let fresh = key(&press(&mut fixture, id));
    assert_ne!(fresh, refused);
    assert_eq!(fixture.client.pending(fresh).unwrap().progress, Progress::InFlight);
}

#[test]
fn someone_else_s_work_leaves_when_its_party_signs_in() {
    let (mut fixture, handle) = spine::restored(&[(FIRST, OTHER)], List::with_capacity(0), &spine::LIMITS);
    spine::delivered(&mut fixture, Time::ZERO, handle, spine::frame(ME));
    assert_eq!(fixture.page(), &Page::SomeoneElse);
    spine::delivered(&mut fixture, Time::ZERO, handle, spine::frame(OTHER));
    assert_eq!(fixture.page(), &inbox());
    assert_eq!(fixture.client.pending(Key { asker: Asker::JIG, number: u64::from(FIRST) }).unwrap().party, OTHER);
}

#[test]
fn someone_else_s_work_leaves_when_its_requests_are_unknown() {
    let (mut fixture, handle) =
        spine::restored(&[(FIRST, OTHER), (SECOND, OTHER)], List::with_capacity(0), &spine::LIMITS);
    spine::delivered(&mut fixture, Time::ZERO, handle, spine::frame(ME));
    assert_eq!(fixture.page(), &Page::SomeoneElse);
    fixture.drive(Time::from_nanos(EXPIRES.as_nanos()), FireClient { wall: Wall::from_nanos(EXPIRES.as_nanos()) });
    assert!(fixture.kept_requests.is_empty());
    assert_eq!(fixture.page(), &inbox());
    for number in [FIRST, SECOND] {
        assert_eq!(fixture.client.pending(Key { asker: Asker::JIG, number: u64::from(number) }), None);
    }
}

#[test]
fn kept_work_dropped_at_start_shows_a_notice() {
    let mut records = List::with_capacity(3);
    for number in [FIRST, SECOND, THIRD] {
        records.push(kept(number, ME)).unwrap();
    }
    let fixture = Fixture::restored(&spine::LIMITS, List::with_capacity(0), records);
    let (id, ()) = fixture.notices().ids().first().unwrap();
    assert_eq!(fixture.notices().get(*id).unwrap().kind, Kind::Dropped { count: 1 });
    assert!(contains(&tree(&fixture, None, None), b"1 requests kept from before were not sent"));
    assert!(fixture.intents().ids().is_empty());
}

#[test]
fn a_repeated_explicit_act_consumes_the_client_s_joined_kept_answer() {
    let (mut original, id, _) = ready();
    let first = key(&press(&mut original, id));
    let record = saved_record(&original).clone();
    let mut fixture = Fixture::restored(&spine::LIMITS, List::with_capacity(0), one(record));
    let handle = fixture.frame_watch.unwrap();
    spine::delivered(&mut fixture, Time::ZERO, handle, spine::frame(ME));
    let attempt = attempt(&fixture);
    let id = card(&mut fixture);
    assert_eq!(key(&press(&mut fixture, id)), first);
    assert_eq!(fixture.intents().ids().len(), 1);
    assert_eq!(fixture.kept_requests.len(), 1);
    answer(&mut fixture, attempt, Answer::Done(Outcome::Changed));
    assert_eq!(fixture.objects().get(id).unwrap().card, Card::Open);
    assert_eq!(fixture.objects().get(id).unwrap().answer, Some(Answer::Done(Outcome::Changed)));
    assert!(fixture.intents().ids().is_empty());
    assert!(fixture.kept_requests.is_empty());
}
