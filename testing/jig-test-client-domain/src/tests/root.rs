#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::wildcard_enum_match_arm,
    reason = "test fixtures collect observed root outputs and locate their variants"
)]
use crate::{Address, Answered, Ask, Domain, Event, JigOut, Limits, Request, first, new, step, take};
use jig_client::{
    boundary::Offset,
    contract::{self, Answer, Delivery, Frame, Needs, Outcome, Party, PartyKind, VersionRange, WatchEnd, WatchEvent},
    limits::Backoff,
    link::Link,
};
use skein_lib::{Duration, Env, List, Queue, RequestEnvelope, RequestLimits, Time, Token, Wall};
const JIG: jig_client::Limits = jig_client::Limits {
    watches: 4,
    notices: 2,
    notice_life: Duration::from_secs(5),
    linger: Duration::from_secs(5),
    save_delay: Duration::from_secs(1),
    intents: 2,
    fields: 4,
    providers: 2,
    objects: 2,
    sources: 8,
    reads: 4,
    parts: 8,
    delegates: 2,
    document_bytes: 65536,
    records: 2,
    drafts: 1,
    projects: 2,
    roles: 2,
    charters: 2,
    text: 32,
    rows: 2,
    conversation_words: 2,
    navigation_entries: 8,
    updates_count: 99,
    backoff: Backoff { first: Duration::from_nanos(10), most: Duration::from_nanos(80) },
    heartbeat: Duration::from_nanos(100),
};
fn limits() -> Limits {
    Limits {
        jig: JIG,
        first: first::Limits { parts: 8, text: 32 },
        second_parts: 8,
        table: RequestLimits {
            requests: 4,
            bytes: 262_144,
            out: 16,
            first: Duration::from_nanos(10),
            most: Duration::from_nanos(80),
            margin: Duration::from_secs(1),
        },
    }
}
fn env() -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }
}
fn drive(root: &mut Domain, event: Event) -> Vec<Request> {
    let mut out = Queue::with_capacity(crate::max_out(&limits()));
    step(root, &env(), event, &mut out);
    out_iter(out)
}
fn out_iter(mut out: Queue<Request>) -> Vec<Request> {
    let mut rows = Vec::new();
    while let Some(row) = out.pop() {
        rows.push(row);
    }
    rows
}
fn taken(root: &mut Domain) -> Vec<Request> {
    let mut out = Queue::with_capacity(crate::max_out(&limits()));
    take(root, &env(), &mut out);
    out_iter(out)
}
fn start(root: &mut Domain, kept: List<crate::store::Record>) -> Token {
    let rows = drive(
        root,
        Event::Start {
            address: Address::Jig(jig_client::address::Address::Inbox { project: None }),
            kept,
            offset: Offset { minutes: 0 },
        },
    );
    rows.iter()
        .find_map(|row| match row {
            Request::Jig(JigOut::Open { watch, subject: contract::Subject::Frame }) => Some(*watch),
            _ => None,
        })
        .unwrap()
}
fn confirm(root: &mut Domain, watch: Token, party: u64) {
    let frame = Frame {
        party: Party {
            number: party,
            kind: PartyKind::Person,
            name: contract::Text::new(Box::from(*b"Pat"), 32).unwrap(),
        },
        projects: List::with_capacity(0),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        updates: contract::Count { value: 0, limited: false },
        versions: VersionRange { first: 1, last: 1 },
        retention: Duration::from_secs(1000),
        deployment_role: Some(1),
    };
    drive(
        root,
        Event::Jig(jig_client::Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Frame(Box::new(frame))) }),
    );
}
fn ask(root: &mut Domain, asked: Token) -> Vec<Request> {
    drive(
        root,
        Event::Jig(jig_client::Event::Ask {
            ask: jig_client::intents::ByValue {
                asked,
                scope: 7,
                ask: Box::new(contract::Ask::Stop { project: 1, task: 9 }),
            },
        }),
    )
}
#[test]
fn keyed_requests_leave_only_through_the_table_after_their_records() {
    let mut root = new(&limits(), 7);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    assert!(ask(&mut root, Token::new(42)).is_empty());
    let rows = taken(&mut root);
    assert_eq!(rows.len(), 2);
    let Request::Save { record: crate::store::Record::Table { record, .. } } = &rows[0] else {
        unreachable!("save precedes send");
    };
    assert!(record.first_sent.is_some());
    let Request::Send { ask: Ask::Jig(ask), .. } = &rows[1] else {
        unreachable!("one jig request was sent");
    };
    assert_eq!(**ask, contract::Ask::Stop { project: 1, task: 9 });
    assert!(taken(&mut root).is_empty());
}
#[test]
fn a_final_answer_reaches_the_child_that_asked_unread_and_once() {
    let mut root = new(&limits(), 8);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    ask(&mut root, Token::new(42));
    let rows = taken(&mut root);
    let attempt = rows
        .iter()
        .find_map(|r| match r {
            Request::Send { attempt, .. } => Some(*attempt),
            _ => None,
        })
        .unwrap();
    let answer = Answer::Done(Outcome::Changed);
    let rows = drive(
        &mut root,
        Event::Answer { attempt, envelope: RequestEnvelope::Final, answer: Some(Answered::Jig(answer.clone())) },
    );
    assert_eq!(rows, vec![Request::Jig(JigOut::Answered { asked: Token::new(42), answer: answer.clone() })]);
    assert!(
        drive(
            &mut root,
            Event::Answer { attempt, envelope: RequestEnvelope::Final, answer: Some(Answered::Jig(answer)) }
        )
        .is_empty()
    );
    let Request::Erase { key: crate::store::Key::Table(_) } = &taken(&mut root)[0] else {
        unreachable!("final erases its request record");
    };
}
#[test]
fn a_reload_parks_kept_requests_until_jig_confirms_their_party() {
    let mut old = new(&limits(), 9);
    let watch = start(&mut old, List::with_capacity(0));
    confirm(&mut old, watch, 7);
    ask(&mut old, Token::new(42));
    let rows = taken(&mut old);
    let record = rows
        .iter()
        .filter_map(|r| match r {
            Request::Save { record: crate::store::Record::Table { version, record } } => {
                Some(crate::store::Record::Table { version: *version, record: record.clone() })
            }
            _ => None,
        })
        .next_back()
        .unwrap();
    let mut kept = List::with_capacity(1);
    kept.push(record).unwrap();
    let mut root = new(&limits(), 10);
    let watch = start(&mut root, kept);
    assert!(taken(&mut root).is_empty());
    confirm(&mut root, watch, 8);
    assert!(taken(&mut root).is_empty());
    confirm(&mut root, watch, 7);
    let sends = taken(&mut root);
    let attempt = sends
        .iter()
        .find_map(|r| match r {
            Request::Send { attempt, .. } => Some(*attempt),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        drive(
            &mut root,
            Event::Answer {
                attempt,
                envelope: RequestEnvelope::Final,
                answer: Some(Answered::Jig(Answer::Done(Outcome::Changed)))
            }
        ),
        vec![Request::Fact { fact: crate::Fact::KeptAnswered }]
    );
}
#[test]
fn link_and_confirmation_reach_both_children_and_park_the_table() {
    let mut root = new(&limits(), 11);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    assert_eq!(root.first().link(), Link::Live);
    assert_eq!(root.second().link(), Link::Live);
    assert_eq!(root.first().party(), Some(7));
    assert_eq!(root.second().party(), Some(7));
    ask(&mut root, Token::new(42));
    drive(&mut root, Event::Jig(jig_client::Event::Ended { watch, end: WatchEnd::Dropped }));
    assert_eq!(root.first().link(), Link::Offline);
    assert_eq!(root.second().link(), Link::Offline);
    for row in taken(&mut root) {
        if let Request::Send { .. } = row {
            unreachable!("offline requests are parked");
        }
    }
    drive(&mut root, Event::SignedOut);
    assert_eq!(root.first().party(), None);
    assert_eq!(root.second().party(), None);
}

#[test]
fn a_current_signed_out_envelope_tells_jig_and_both_siblings() {
    let mut root = new(&limits(), 12);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    ask(&mut root, Token::new(42));
    let rows = taken(&mut root);
    let Request::Send { attempt, .. } = &rows[1] else {
        unreachable!("one request is in flight");
    };
    drive(&mut root, Event::Answer { attempt: *attempt, envelope: RequestEnvelope::SignedOut, answer: None });
    assert_eq!(root.first().party(), None);
    assert_eq!(root.second().party(), None);
    for row in taken(&mut root) {
        if let Request::Send { .. } = row {
            unreachable!("signed out requests remain parked");
        }
    }
}
