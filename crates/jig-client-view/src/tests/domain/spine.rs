use crate::{Domain, Event, Limits, Request, max_out, step};
use crate::{
    address::{Address, Place},
    boundary::Offset,
    link::Link,
    page::Page,
    store::Record,
};
use jig_client::contract::{
    Count, Delivery, DocumentText, Frame, FrameParty, HeldRole, Inbox, Needs, PartyKind, WatchEvent,
};
use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall};

pub(crate) const LIMITS: Limits = Limits {
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
    kept: 4,
    drafts: 1,
    projects: 2,
    roles: 2,
    charters: 2,
    text: 32,
    rows: 2,
    segments: 4,
    conversation_words: 2,
    navigation_entries: 8,
    updates_count: 99,
};
pub(crate) fn env(now: Time) -> Env<Limits> {
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
    step(domain, &env(now), event, &mut out);
    if starting {
        step(domain, &env(now), Event::Hello { served: true }, &mut out);
    }
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
pub(crate) fn frame(party: u64) -> Frame {
    Frame {
        party: FrameParty {
            number: party,
            kind: PartyKind::Person,
            login: DocumentText::complete(jig_client::contract::Text::new(Box::from(*b"fixture-login"), 32).unwrap()),
            name: DocumentText::complete(jig_client::contract::Text::new(Box::from(*b"Pat"), 32).unwrap()),
        },
        projects: List::with_capacity(0),
        needs: needs(),
        needs_total: 0,
        results_seen: 0,
        charters: List::with_capacity(0),
        cut: false,
        updates: Count { value: 0, limited: false },

        retention: Duration::from_secs(1000),
        deployment_role: Some(HeldRole::Policy { role: 1 }),
    }
}
pub(crate) fn start(domain: &mut Domain, kept: List<Record>) -> Token {
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
    let mut domain = Domain::new(&LIMITS);
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
    assert_eq!(*subject, jig_client::contract::Subject::Frame);
    assert_eq!(domain.page(), &Page::Starting);
    assert_eq!(domain.link(), Link::Starting);
    assert_eq!(domain.offset().minutes, -60_i32);
}

#[test]
fn a_reopened_page_is_behind_until_its_first_delivery_and_never_changes_the_link() {
    let mut domain = Domain::new(&LIMITS);
    let watch = start(&mut domain, List::with_capacity(0));
    let page = open_token(&delivered(&mut domain, Time::ZERO, watch, frame(7)));
    assert!(drive(&mut domain, Time::ZERO, Event::Watch { watch: page, event: WatchEvent::Behind }).is_empty());
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
    let retry = Time::ZERO;
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
    let mut domain = Domain::new(&LIMITS);
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
    let mut domain = Domain::new(&LIMITS);
    let watch = start(&mut domain, List::with_capacity(0));
    let refused = drive(&mut domain, Time::ZERO, Event::Hello { served: false });
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
    let mut domain = Domain::new(&LIMITS);
    let watch = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, watch, frame(7));
    assert_eq!(
        drive(&mut domain, Time::ZERO, Event::Frame { asked: Token::new(42) }).as_ref(),
        &[Request::FrameIs {
            asked: Token::new(42),
            frame: crate::frame::me::FrameShown {
                party: Some(7),
                address: Some(Address::Inbox { project: None }),
                link: Link::Live
            }
        }]
    );
}

#[test]
fn foreign_kept_requests_hide_work_but_confirm_only_the_frame_party() {
    let mut domain = Domain::new(&LIMITS);
    let watch = start(&mut domain, List::with_capacity(0));
    drive(&mut domain, Time::ZERO, Event::Kept { scope: 7, requests: 2 });
    let out = delivered(&mut domain, Time::ZERO, watch, frame(8));
    assert!(out.contains(&Request::Confirmed { party: Some(8) }));
    assert_eq!(domain.page(), &Page::SomeoneElse);
    drive(&mut domain, Time::ZERO, Event::KeptGone { scope: 7 });
    assert_eq!(domain.page(), &Page::SomeoneElse);
}

#[test]
fn the_last_foreign_kept_request_retiring_restores_the_work_page() {
    let mut domain = Domain::new(&LIMITS);
    let watch = start(&mut domain, List::with_capacity(0));
    drive(&mut domain, Time::ZERO, Event::Kept { scope: 7, requests: 2 });
    drive(&mut domain, Time::ZERO, Event::Kept { scope: 9, requests: 1 });
    delivered(&mut domain, Time::ZERO, watch, frame(8));
    drive(&mut domain, Time::ZERO, Event::KeptGone { scope: 7 });
    drive(&mut domain, Time::ZERO, Event::KeptGone { scope: 7 });
    assert_eq!(domain.page(), &Page::SomeoneElse);
    drive(&mut domain, Time::ZERO, Event::KeptGone { scope: 9 });
    assert!(work_page(&domain));
    assert!(drive(&mut domain, Time::ZERO, Event::KeptGone { scope: 9 }).is_empty());
}

#[test]
fn own_kept_requests_do_not_hide_work_or_override_kept_draft_guard() {
    let mut domain = Domain::new(&LIMITS);
    let watch = start(&mut domain, List::with_capacity(0));
    drive(&mut domain, Time::ZERO, Event::Kept { scope: 7, requests: 2 });
    delivered(&mut domain, Time::ZERO, watch, frame(7));
    assert!(work_page(&domain));
    drive(&mut domain, Time::ZERO, Event::KeptGone { scope: 7 });
    assert!(work_page(&domain));

    let mut guarded = Domain::new(&LIMITS);
    let mut kept = List::with_capacity(1);
    kept.push(Record::Draft {
        version: crate::store::STORE_VERSION,
        party: 8,
        task: 9,
        words: jig_client::contract::Text::new(Box::from(*b"old"), LIMITS.text).unwrap(),
        touched: 1,
    })
    .unwrap();
    let watch = start(&mut guarded, kept);
    drive(&mut guarded, Time::ZERO, Event::Kept { scope: 7, requests: 1 });
    delivered(&mut guarded, Time::ZERO, watch, frame(7));
    drive(&mut guarded, Time::ZERO, Event::KeptGone { scope: 7 });
    assert_eq!(guarded.page(), &Page::SomeoneElse);
}

fn work_page(domain: &Domain) -> bool {
    match domain.page() {
        Page::Work { .. } => true,
        Page::Starting | Page::Elsewhere | Page::SigningIn | Page::Missing | Page::OutOfDate | Page::SomeoneElse => {
            false
        }
    }
}

pub(super) fn party(number: u64) -> jig_client::contract::Party {
    jig_client::contract::Party {
        number,
        kind: PartyKind::Person,
        name: jig_client::contract::Text::new(Box::from(*b"Pat"), 32).unwrap(),
    }
}
