use crate::{Domain, Event, Limits, Request, fire, max_out, step};
use crate::{
    address::{Address, Place},
    boundary::Offset,
    contract::{Count, Delivery, Frame, Inbox, Needs, Party, PartyKind, VersionRange, WatchEnd, WatchEvent},
    limits::Backoff,
    link::Link,
    pages::Page,
    store::Record,
};
use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall};

pub(super) const LIMITS: Limits = Limits {
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
    navigation_entries: 8,
    updates_count: 99,
    backoff: Backoff { first: Duration::from_nanos(10), most: Duration::from_nanos(80) },
    heartbeat: Duration::from_nanos(100),
};
pub(super) fn env(now: Time) -> Env<Limits> {
    Env { now, wall: Wall::EPOCH, limits: LIMITS }
}
pub(super) fn drain(mut out: Queue<Request>) -> Box<[Request]> {
    let mut rows = List::with_capacity(max_out(&LIMITS));
    while let Some(request) = out.pop() {
        rows.push(request).expect("output fits declared bound");
    }
    rows.into_boxed()
}
pub(super) fn drive(domain: &mut Domain, now: Time, event: Event) -> Box<[Request]> {
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    step(domain, &env(now), event, &mut out);
    drain(out)
}
fn tick(domain: &mut Domain, now: Time) -> Box<[Request]> {
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    fire(domain, &env(now), &mut out);
    drain(out)
}
pub(super) fn open_token(requests: &[Request]) -> Token {
    for request in requests {
        if let Request::Open { watch, .. } = request {
            return *watch;
        }
    }
    unreachable!("a watch was opened")
}
fn needs() -> Needs {
    Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 }
}
pub(super) fn frame(party: u64) -> Frame {
    Frame {
        party: Party {
            number: party,
            kind: PartyKind::Person,
            name: crate::contract::Text::new(Box::from(*b"Pat"), 32).unwrap(),
        },
        projects: List::with_capacity(0),
        needs: needs(),
        updates: Count { value: 0, limited: false },
        versions: VersionRange { first: 1, last: 1 },
        retention: Duration::from_secs(1000),
        deployment_role: Some(1),
    }
}
pub(super) fn start(domain: &mut Domain, kept: List<Record>) -> Token {
    open_token(&drive(
        domain,
        Time::ZERO,
        Event::Start { address: Address::Inbox { project: None }, kept, offset: Offset { minutes: 60 } },
    ))
}
pub(super) fn delivered(domain: &mut Domain, now: Time, watch: Token, frame: Frame) -> Box<[Request]> {
    drive(domain, now, Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Frame(Box::new(frame))) })
}
fn inbox() -> Inbox {
    Inbox {
        entries: List::with_capacity(0),
        needs: needs(),
        updates: Count { value: 0, limited: false },
        results_seen: 0,
        replies_seen: List::with_capacity(0),
        more: false,
    }
}

#[test]
fn start_opens_only_the_frame_and_takes_nothing_before_it() {
    let mut domain = Domain::new(&LIMITS, 7);
    assert!(drive(&mut domain, Time::ZERO, Event::Frame { asked: Token::new(1) }).is_empty());
    let opened = drive(
        &mut domain,
        Time::ZERO,
        Event::Start { address: Address::Deployment, kept: List::with_capacity(0), offset: Offset { minutes: -60 } },
    );
    assert_eq!(opened.len(), 1);
    let Request::Open { subject, .. } = &opened[0] else {
        unreachable!("start opens a watch");
    };
    assert_eq!(*subject, crate::contract::Subject::Frame);
    assert_eq!(domain.page(), &Page::Starting);
    assert_eq!(domain.link(), Link::Starting);
    assert_eq!(domain.offset().minutes, -60_i32);
}

#[test]
fn frame_confirms_its_party_and_link_changes_once_each_way() {
    let mut domain = Domain::new(&LIMITS, 8);
    let watch = start(&mut domain, List::with_capacity(0));
    let first = delivered(&mut domain, Time::ZERO, watch, frame(7));
    assert_eq!(first[0], Request::Confirmed { party: Some(7) });
    assert_eq!(first[1], Request::Retention { retention: Duration::from_secs(1000) });
    assert_eq!(first[2], Request::Link { link: Link::Live });
    assert_eq!(domain.link(), Link::Live);
    assert_eq!(
        delivered(&mut domain, Time::ZERO, watch, frame(7)).as_ref(),
        &[Request::Retention { retention: Duration::from_secs(1000) }]
    );
    assert_eq!(
        drive(&mut domain, Time::ZERO, Event::Ended { watch, end: WatchEnd::Dropped }).as_ref(),
        &[Request::Link { link: Link::Offline }]
    );
    assert_eq!(domain.notices().ids().len(), 1);
    let retry = domain.next_deadline().unwrap();
    assert_eq!(tick(&mut domain, retry).as_ref(), &[Request::Open { watch, subject: crate::contract::Subject::Frame }]);
    assert_eq!(
        delivered(&mut domain, retry, watch, frame(7)).as_ref(),
        &[Request::Retention { retention: Duration::from_secs(1000) }, Request::Link { link: Link::Live }]
    );
    assert_eq!(domain.notices().ids().len(), 2);
}

#[test]
fn silence_waits_for_the_old_terminal_before_it_reopens() {
    let mut domain = Domain::new(&LIMITS, 9);
    let watch = start(&mut domain, List::with_capacity(0));
    assert_eq!(
        tick(&mut domain, Time::from_nanos(100)).as_ref(),
        &[Request::Close { watch }, Request::Link { link: Link::Offline }]
    );
    assert_eq!(domain.next_deadline(), None);
    assert!(tick(&mut domain, Time::from_nanos(1000)).is_empty());
    assert!(drive(&mut domain, Time::from_nanos(1000), Event::Ended { watch, end: WatchEnd::Closed }).is_empty());
    let retry = domain.next_deadline().unwrap();
    assert!(retry >= Time::from_nanos(1010) && retry <= Time::from_nanos(1012));
    assert_eq!(tick(&mut domain, retry).as_ref(), &[Request::Open { watch, subject: crate::contract::Subject::Frame }]);
}

#[test]
fn an_alive_frame_rearms_its_heartbeat_without_changing_the_link() {
    let mut domain = Domain::new(&LIMITS, 2);
    let watch = start(&mut domain, List::with_capacity(0));
    assert!(drive(&mut domain, Time::from_nanos(50), Event::Watch { watch, event: WatchEvent::Alive }).is_empty());
    assert!(tick(&mut domain, Time::from_nanos(100)).is_empty());
    assert_eq!(domain.next_deadline(), Some(Time::from_nanos(150)));
}

#[test]
fn a_reopened_page_is_behind_until_its_first_delivery_and_never_changes_the_link() {
    let mut domain = Domain::new(&LIMITS, 3);
    let watch = start(&mut domain, List::with_capacity(0));
    let page = open_token(&delivered(&mut domain, Time::ZERO, watch, frame(7)));
    assert!(drive(&mut domain, Time::ZERO, Event::Ended { watch: page, end: WatchEnd::Dropped }).is_empty());
    assert_eq!(domain.link(), Link::Live);
    assert_eq!(
        domain.page(),
        &Page::Work {
            address: Address::Inbox { project: None },
            generation: 0,
            behind: true,
            excess: 0,
            refused: None
        }
    );
    let retry = domain.next_deadline().unwrap();
    let reopened = tick(&mut domain, retry);
    assert_eq!(open_token(&reopened), page);
    assert!(drive(&mut domain, retry, Event::Watch { watch: page, event: WatchEvent::Opened }).is_empty());
    assert_eq!(
        domain.page(),
        &Page::Work {
            address: Address::Inbox { project: None },
            generation: 0,
            behind: true,
            excess: 0,
            refused: None
        }
    );
    drive(
        &mut domain,
        retry,
        Event::Watch { watch: page, event: WatchEvent::Delivered(Delivery::Inbox(Box::new(inbox()))) },
    );
    assert_eq!(
        domain.page(),
        &Page::Work {
            address: Address::Inbox { project: None },
            generation: 0,
            behind: false,
            excess: 0,
            refused: None
        }
    );
}

#[test]
fn a_different_party_keeps_the_previous_partys_work_hidden_until_they_return() {
    let mut kept = List::with_capacity(1);
    kept.push(Record::Place { version: crate::store::STORE_VERSION, party: 7, address: Address::Deployment }).unwrap();
    let mut domain = Domain::new(&LIMITS, 4);
    let watch = start(&mut domain, kept);
    let refused = delivered(&mut domain, Time::ZERO, watch, frame(8));
    assert_eq!(refused[0], Request::Confirmed { party: None });
    assert_eq!(refused.len(), 3);
    assert_eq!(domain.page(), &Page::SomeoneElse);
    let returned = delivered(&mut domain, Time::ZERO, watch, frame(7));
    assert_eq!(returned[0], Request::Confirmed { party: Some(7) });
    assert_eq!(domain.frame().party(), Some(7));
    assert_eq!(
        domain.page(),
        &Page::Work {
            address: Address::Inbox { project: None },
            generation: 0,
            behind: false,
            excess: 0,
            refused: None
        }
    );
}

#[test]
fn an_unsupported_version_parks_work_and_never_sends_a_new_engine_request() {
    let mut domain = Domain::new(&LIMITS, 5);
    let watch = start(&mut domain, List::with_capacity(0));
    let mut old = frame(7);
    old.versions = VersionRange { first: 2, last: 3 };
    let refused = delivered(&mut domain, Time::ZERO, watch, old);
    assert_eq!(
        refused.as_ref(),
        &[Request::Close { watch }, Request::Link { link: Link::Offline }, Request::Confirmed { party: None }]
    );
    assert_eq!(domain.page(), &Page::OutOfDate);
    assert_eq!(domain.next_deadline(), None);
    assert!(drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Deployment) }).is_empty());
    assert!(delivered(&mut domain, Time::ZERO, watch, frame(7)).is_empty());
}

#[test]
fn siblings_get_the_current_party_and_address_in_the_same_step() {
    let mut domain = Domain::new(&LIMITS, 6);
    let watch = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, watch, frame(7));
    assert_eq!(
        drive(&mut domain, Time::ZERO, Event::Frame { asked: Token::new(42) }).as_ref(),
        &[Request::FrameIs {
            asked: Token::new(42),
            frame: crate::frame::FrameShown {
                party: Some(7),
                address: Some(Address::Inbox { project: None }),
                link: Link::Live
            }
        }]
    );
}

#[test]
fn retry_spans_double_to_the_cap_and_keep_the_same_seeded_jitter() {
    let mut first = Domain::new(&LIMITS, 7);
    let watch = start(&mut first, List::with_capacity(0));
    let mut second = Domain::new(&LIMITS, 7);
    let other = start(&mut second, List::with_capacity(0));
    let mut now = Time::ZERO;
    for floor in [10_u64, 20, 40, 80, 80] {
        drive(&mut first, now, Event::Ended { watch, end: WatchEnd::Dropped });
        drive(&mut second, now, Event::Ended { watch: other, end: WatchEnd::Dropped });
        let retry = first.next_deadline().unwrap();
        assert_eq!(second.next_deadline(), Some(retry));
        let span = retry.saturating_since(now).as_nanos();
        assert!(span >= floor && span <= floor.saturating_add(floor / 4).min(80));
        tick(&mut first, retry);
        tick(&mut second, retry);
        now = retry;
    }
}

#[test]
fn a_watch_closed_during_backoff_needs_no_second_terminal_and_reclaims_its_slot() {
    let mut domain = Domain::new(&LIMITS, 7);
    let watch = start(&mut domain, List::with_capacity(0));
    let page = open_token(&delivered(&mut domain, Time::ZERO, watch, frame(7)));
    drive(&mut domain, Time::ZERO, Event::Ended { watch: page, end: WatchEnd::Dropped });
    let left = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert_eq!(left.as_ref(), &[Request::PartsLeft { generation: 0 }]);
    domain.reclaim();
    let opened = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Deployment) });
    assert_ne!(open_token(&opened), page);
}
