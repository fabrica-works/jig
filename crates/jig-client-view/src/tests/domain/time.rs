use super::spine::{LIMITS, drain, frame, open_token};
use crate::{
    Domain, Event, Limits, Request,
    action::Act,
    address::{Address, Place},
    boundary::Offset,
    fields::Purpose,
    intents::{ByValue, send_card},
    notices::kept::{self, Kind},
    objects::Source,
    store::{self, Record, STORE_VERSION},
};
use jig_client::contract::*;
use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall};
fn limits() -> Limits {
    Limits { ..LIMITS }
}
fn drive(domain: &mut Domain, limits: &Limits, now: Time, event: Event) -> Box<[Request]> {
    let mut out = Queue::with_capacity(crate::max_out(limits));
    let starting = match &event {
        Event::Start { .. } => true,
        Event::Hello { .. }
        | Event::Kept { .. }
        | Event::KeptGone { .. }
        | Event::Went { .. }
        | Event::Act { .. }
        | Event::Ask { .. }
        | Event::Progress { .. }
        | Event::Answered { .. }
        | Event::Read { .. }
        | Event::Watch { .. }
        | Event::Ended { .. }
        | Event::ShowObjects { .. }
        | Event::ShowSections { .. }
        | Event::ObjectsLeft
        | Event::SignOutLost
        | Event::SignedOut
        | Event::Frame { .. } => false,
    };
    crate::step(domain, &Env { now, wall: Wall::EPOCH, limits: *limits }, event, &mut out);
    if starting {
        crate::step(domain, &Env { now, wall: Wall::EPOCH, limits: *limits }, Event::Hello { served: true }, &mut out);
    }
    drain(out)
}
fn tick(domain: &mut Domain, limits: &Limits, now: Time) -> Box<[Request]> {
    let mut out = Queue::with_capacity(crate::max_out(limits));
    crate::fire(domain, &Env { now, wall: Wall::EPOCH, limits: *limits }, &mut out);
    drain(out)
}
fn start(domain: &mut Domain, limits: &Limits, kept: List<Record>) -> (Token, Token) {
    let f = open_token(&drive(
        domain,
        limits,
        Time::ZERO,
        Event::Start { address: Address::Inbox { project: None }, kept, offset: Offset { minutes: 0 } },
    ));
    let mut source = frame(7);
    source.party.login = DocumentText::complete(text(b"L"));
    let initial = drive(
        domain,
        limits,
        Time::ZERO,
        Event::Watch { watch: f, event: WatchEvent::Delivered(Delivery::Frame(Box::new(source))) },
    );
    let page = open_token(&initial);
    for request in &initial {
        if let Request::Read { read, what: Read::Inbox { .. } } = request {
            drive(
                domain,
                limits,
                Time::ZERO,
                Event::Read {
                    read: *read,
                    result: ReadResult::Value(ReadValue::Inbox(Box::new(InboxPage {
                        cut: false,
                        rows: List::with_capacity(0),
                        before: None,
                        after: None,
                    }))),
                },
            );
        }
    }
    domain.reclaim();
    (f, page)
}
fn text(bytes: &[u8]) -> Text {
    Text::new(Box::from(bytes), 32).unwrap()
}

#[test]
fn a_minute_counts_a_change_only_while_the_page_shows_wall_times() {
    let limits = limits();
    let mut domain = Domain::new(&limits);
    start(&mut domain, &limits, List::with_capacity(0));
    assert_eq!(domain.next_deadline(), None);
    domain.timed_page(true);
    assert_eq!(domain.next_deadline(), Some(Time::from_nanos(Duration::from_secs(60).as_nanos())));
    let shown = domain.shown();
    assert!(tick(&mut domain, &limits, Time::from_nanos(Duration::from_secs(59).as_nanos())).is_empty());
    assert_eq!(domain.shown(), shown);
    assert!(tick(&mut domain, &limits, Time::from_nanos(Duration::from_secs(60).as_nanos())).is_empty());
    assert_eq!(domain.shown(), shown.checked_add(1).unwrap());
    drive(
        &mut domain,
        &limits,
        Time::from_nanos(Duration::from_secs(60).as_nanos()),
        Event::Went { place: Place::Elsewhere },
    );
    let shown = domain.shown();
    assert!(tick(&mut domain, &limits, Time::from_nanos(Duration::from_secs(120).as_nanos())).is_empty());
    assert_eq!(domain.shown(), shown);
}

#[test]
fn hidden_answers_have_notices_and_lingers_and_coalesced_saves_end_on_deadlines() {
    let limits = limits();
    let mut domain = Domain::new(&limits);
    start(&mut domain, &limits, List::with_capacity(0));
    let key = ObjectKey::Task(8);
    domain.objects.name(key, Source::Window { generation: 0, number: 1 }).unwrap();
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    let request =
        send_card(&mut domain, key, ActKind::Stop, Box::new(Ask::Stop { project: 1, task: 8 }), &mut out).unwrap();
    drive(&mut domain, &limits, Time::ZERO, Event::Went { place: Place::Elsewhere });
    drive(&mut domain, &limits, Time::ZERO, Event::Answered { request, answer: Answer::Refused(Refusal::NotOffered) });
    let notice = *domain.notices().ids().first().unwrap().0;
    let Kind::Answer { object, .. } = &domain.notices().get(notice).unwrap().kind else {
        panic!("hidden card outcome");
    };
    assert_eq!(*object, key);
    assert_eq!(domain.notices().get(notice).unwrap().until, Time::from_nanos(Duration::from_secs(5).as_nanos()));
    let id = domain.objects.name(ObjectKey::Task(9), Source::Window { generation: 1, number: 1 }).unwrap();
    assert!(domain.leave(id, Ending::Cancelled { by: Actor::Party(7), reason: Some(text(b"done")), at: None }));
    out = Queue::with_capacity(crate::max_out(&limits));
    assert!(store::coalesce(
        &mut domain,
        Record::Draft { version: STORE_VERSION, party: 7, task: 8, words: text(b"first"), touched: 1 },
        &mut out
    ));
    drive(&mut domain, &limits, Time::from_nanos(500_000_000), Event::Frame { asked: Token::new(90) });
    let newest = Record::Draft { version: STORE_VERSION, party: 7, task: 8, words: text(b"newest"), touched: 1 };
    assert!(store::coalesce(&mut domain, newest.clone(), &mut out));
    assert!(out.is_empty());
    assert!(tick(&mut domain, &limits, Time::from_nanos(1_499_999_999)).is_empty());
    assert_eq!(
        tick(&mut domain, &limits, Time::from_nanos(1_500_000_000)).as_ref(),
        &[Request::Save { record: newest }]
    );
    tick(&mut domain, &limits, Time::from_nanos(5_000_000_000));
    tick(&mut domain, &limits, Time::from_nanos(5_000_000_000));
    assert!(domain.notices().ids().is_empty());
    assert!(domain.objects.find(ObjectKey::Task(9)).is_none());
}

#[test]
fn checked_worst_case_and_full_containers_keep_steps_within_max_out() {
    let limits = Limits {
        watches: 2,
        objects: 1,
        sources: 1,
        reads: 1,
        intents: 1,
        fields: 1,
        providers: 1,
        notices: 1,
        records: 1,
        projects: 1,
        roles: 1,
        charters: 1,
        parts: 1,
        text: 4,
        ..limits()
    };
    check_worst_case(limits);
    let mut domain = Domain::new(&limits);
    domain.provider(crate::signin::Provider { number: 1, name: text(b"P") }).unwrap();
    let mut kept = List::with_capacity(1);
    kept.push(Record::Draft { version: STORE_VERSION, party: 7, task: 8, words: text(b"kept"), touched: 1 }).unwrap();
    let (f, page) = start(&mut domain, &limits, kept);
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    domain.reads.ask(0, Source::Window { generation: 0, number: 1 }, Read::Deployment, &mut out).unwrap();
    let mut entries = List::with_capacity(2);
    for task in 1..=2 {
        entries
            .push(InboxEntry {
                reply: None,
                lineage: List::with_capacity(0),
                goal: None,
                recipient: None,
                stall: None,
                object: ObjectKey::Result(task),
                project: 1,
                group: InboxGroup::Results,
                position: task,
                at: Wall::EPOCH,
            })
            .unwrap();
    }
    let inbox = Inbox {
        entries,
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        updates: Count { value: 0, limited: false },
        results_seen: 0,
        replies_seen: List::with_capacity(0),
        more: true,
    };
    let effects = drive(
        &mut domain,
        &limits,
        Time::ZERO,
        Event::Watch { watch: page, event: WatchEvent::Delivered(Delivery::Inbox(Box::new(inbox))) },
    );
    assert_eq!(effects.len(), 2);
    let crate::page::Page::Work { excess: 1, refused: Some(Limit::Objects), .. } = domain.page() else {
        panic!("full kept the first and counted the rest");
    };
    let ask = ByValue { asked: Token::new(50), scope: 7, ask: Box::new(Ask::Stop { project: 1, task: 8 }) };
    assert_eq!(drive(&mut domain, &limits, Time::ZERO, Event::Ask { ask: ask.clone() }).len(), 1);
    let mut other = ask;
    other.asked = Token::new(51);
    assert_eq!(drive(&mut domain, &limits, Time::ZERO, Event::Ask { ask: other }).len(), 2);
    drive(
        &mut domain,
        &limits,
        Time::ZERO,
        Event::Act { act: Act::Edit { field: Purpose::NewChat, text: text(b"one") } },
    );
    assert_eq!(
        drive(
            &mut domain,
            &limits,
            Time::ZERO,
            Event::Act { act: Act::Edit { field: Purpose::Reason, text: text(b"two") } }
        )
        .len(),
        1
    );
    kept::show(&mut domain, Kind::Refused(Limit::Requests));
    kept::show(&mut domain, Kind::Refused(Limit::Fields));
    assert_eq!(domain.notices().ids().len(), 1);
    let mut oversized = frame(7);
    oversized.projects = List::with_capacity(2);
    assert_eq!(
        drive(
            &mut domain,
            &limits,
            Time::ZERO,
            Event::Watch { watch: f, event: WatchEvent::Delivered(Delivery::Frame(Box::new(oversized))) }
        )
        .len(),
        1
    );
    let effects = drive(&mut domain, &limits, Time::ZERO, Event::Went { place: Place::Jig(Address::Deployment) });
    assert!(u64::try_from(effects.len()).unwrap() <= u64::from(crate::max_out(&limits)));
}

fn check_worst_case(limits: Limits) {
    let bound = crate::worst_case(&limits).unwrap();
    assert!(crate::worst_case(&Limits { notices: 2, ..limits }).unwrap() > bound);
    for invalid in [
        Limits { watches: 0, ..limits },
        Limits { objects: u32::MAX, ..limits },
        Limits { notices: u32::MAX, ..limits },
        Limits { document_bytes: u64::MAX, ..limits },
        Limits { notice_life: Duration::ZERO, ..limits },
        Limits { save_delay: Duration::ZERO, ..limits },
    ] {
        assert!(crate::worst_case(&invalid).is_none());
    }
}
