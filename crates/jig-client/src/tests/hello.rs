use super::spine::{LIMITS, drain, env, frame, open_token};
use crate::{
    Domain, Event, Request,
    address::Address,
    boundary::Offset,
    contract::{Delivery, VersionRange, WatchEvent},
    link::Link,
    pages::Page,
};
use skein_lib::{List, Queue, Time};
fn drive(domain: &mut Domain, event: Event) -> Box<[Request]> {
    let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
    crate::step(domain, &env(Time::ZERO), event, &mut out);
    drain(out)
}
fn start() -> (Domain, skein_lib::Token) {
    let mut domain = Domain::new(&LIMITS, 1);
    let out = drive(
        &mut domain,
        Event::Start {
            address: Address::Inbox { project: None },
            kept: List::with_capacity(0),
            offset: Offset { minutes: 0 },
        },
    );
    (domain, open_token(&out))
}
#[test]
fn frame_identity_waits_for_a_genuine_hello_before_confirmation_or_live_link() {
    let (mut domain, watch) = start();
    let frame = frame(7);
    assert!(
        drive(
            &mut domain,
            Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Frame(Box::new(frame.clone()))) }
        )
        .is_empty()
    );
    assert_eq!(domain.frame().document.as_ref(), Some(&frame));
    assert_eq!(domain.page(), &Page::Starting);
    assert_eq!(domain.link(), Link::Starting);
    let out = drive(&mut domain, Event::Hello { versions: VersionRange { first: 1, last: 1 } });
    assert_eq!(out[0], Request::Confirmed { party: Some(7) });
    assert_eq!(out[1], Request::Retention { retention: frame.retention });
    assert_eq!(out[2], Request::Link { link: Link::Live });
    assert_eq!(domain.link(), Link::Live);
}
#[test]
fn an_invalid_hello_closes_watches_without_fabricating_frame_and_a_later_range_cannot_unblock_it() {
    let (mut domain, watch) = start();
    let out = drive(&mut domain, Event::Hello { versions: VersionRange { first: 2, last: 1 } });
    assert_eq!(
        out.as_ref(),
        &[Request::Close { watch }, Request::Link { link: Link::Offline }, Request::Confirmed { party: None }]
    );
    assert_eq!(domain.frame().document, None);
    assert_eq!(domain.page(), &Page::OutOfDate);
    assert_eq!(domain.next_deadline(), None);
    assert!(drive(&mut domain, Event::Hello { versions: VersionRange { first: 1, last: 1 } }).is_empty());
    assert!(
        drive(&mut domain, Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Frame(Box::new(frame(7)))) })
            .is_empty()
    );
    assert_eq!(domain.page(), &Page::OutOfDate);
    assert_eq!(domain.link(), Link::Offline);
}
