//! Page policies exercised through the client boundary.
use super::spine::{LIMITS, delivered, drive, frame, open_token, start};
use crate::{
    Domain, Event, Request,
    address::{Address, Place, ProjectSection},
    contract::*,
    link::Link,
    pages::{
        Page,
        navigation::{Destination, Navigation},
    },
};
use skein_lib::{List, Time};
fn text(value: &[u8]) -> Text {
    Text::new(Box::from(value), LIMITS.text).unwrap()
}
fn project(number: u32, chats: bool) -> ProjectRole {
    let mut charters = List::with_capacity(1);
    if chats {
        charters.push(Charter { number: 1, name: text(b"guide") }).unwrap();
    }
    ProjectRole {
        project: number,
        name: text(b"shop"),
        roles: List::with_capacity(0),
        chats: charters,
        goals: List::with_capacity(0),
        pool: Budget { total: 20, own: 0, ended: 0, reserved: 0, left: 20, funder: Actor::Party(7) },
    }
}
#[test]
fn the_frame_counts_needs_exactly_and_caps_updates_on_every_page() {
    let mut domain = Domain::new(&LIMITS, 1);
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    value.needs = Needs { questions: u32::MAX, proposals: 2, escalations: 3, person_tasks: 4 };
    value.updates = Count { value: 100, limited: false };
    delivered(&mut domain, Time::ZERO, f, value.clone());
    let summary = domain.frame_summary();
    assert_eq!(summary.needs, u64::from(u32::MAX) + 9);
    assert_eq!(summary.updates, Count { value: 99, limited: true });
    assert_eq!(summary.party, Some(7));
    assert_eq!(summary.link, Link::Live);
    drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert_eq!(domain.frame_summary().needs, summary.needs);
    value.updates = Count { value: 99, limited: false };
    delivered(&mut domain, Time::ZERO, f, value.clone());
    assert_eq!(domain.frame_summary().updates, Count { value: 99, limited: false });
    value.updates.limited = true;
    delivered(&mut domain, Time::ZERO, f, value);
    assert!(domain.frame_summary().updates.limited);
    drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Project { project: 1, section: ProjectSection::Overview }) },
    );
    assert_eq!(domain.frame_summary().project, Some(1));
    assert_eq!(Address::task(42), Address::Task { task: 42, section: crate::address::TaskSection::Overview });
}
#[test]
fn navigation_keeps_the_application_order_and_leaves_out_inapplicable_pages() {
    let mut domain = Domain::new(&LIMITS, 2);
    let mut navigation = Navigation::new(8);
    assert_eq!(navigation.home, Destination::Jig(Address::Inbox { project: None }));
    let own = Destination::Application { owner: 1, number: 3 };
    let chat = Destination::Jig(Address::Conversations { project: None });
    let deployment = Destination::Jig(Address::Deployment);
    navigation.home = own;
    for destination in [own, chat, deployment] {
        navigation.entries.push(destination).unwrap();
    }
    domain.configure_navigation(navigation).unwrap();
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    value.deployment_role = None;
    delivered(&mut domain, Time::ZERO, f, value.clone());
    assert_eq!(domain.navigation().entries.as_slice(), &[own, chat, deployment]);
    assert!(domain.navigation().offered(own, domain.frame().document.as_ref()));
    assert!(!domain.navigation().offered(chat, domain.frame().document.as_ref()));
    assert!(!domain.navigation().offered(deployment, domain.frame().document.as_ref()));
    value.projects = List::with_capacity(2);
    value.projects.push(project(1, false)).unwrap();
    value.projects.push(project(2, true)).unwrap();
    value.deployment_role = Some(1);
    delivered(&mut domain, Time::ZERO, f, value);
    assert!(domain.navigation().offered(chat, domain.frame().document.as_ref()));
    assert!(
        !domain
            .navigation()
            .offered(Destination::Jig(Address::Conversations { project: Some(1) }), domain.frame().document.as_ref())
    );
    assert!(domain.navigation().offered(deployment, domain.frame().document.as_ref()));
    assert!(domain.configure_navigation(Navigation::new(0)).is_err());
    let mut fresh = Domain::new(&LIMITS, 3);
    assert!(fresh.configure_navigation(Navigation::new(9)).is_err());
    assert_eq!(own.place(), Place::Elsewhere);
}
#[test]
fn non_work_pages_follow_their_causes_and_missing_tasks_do_not_reveal_visibility() {
    let mut domain = Domain::new(&LIMITS, 4);
    let f = start(&mut domain, List::with_capacity(0));
    assert_eq!(domain.page(), &Page::Starting);
    delivered(&mut domain, Time::ZERO, f, frame(7));
    drive(&mut domain, Time::ZERO, Event::Went { place: Place::Missing });
    assert_eq!(domain.page(), &Page::Missing);
    drive(&mut domain, Time::ZERO, Event::SignedOut);
    assert_eq!(domain.page(), &Page::SigningIn);
    for end in [WatchEnd::Unknown, WatchEnd::NotAllowed] {
        let mut domain = Domain::new(&LIMITS, 5);
        let f = start(&mut domain, List::with_capacity(0));
        delivered(&mut domain, Time::ZERO, f, frame(7));
        let opened = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(42)) });
        drive(&mut domain, Time::ZERO, Event::Ended { watch: open_token(&opened), end });
        assert_eq!(domain.page(), &Page::Missing);
    }
    let mut old = Domain::new(&LIMITS, 6);
    let f = start(&mut old, List::with_capacity(0));
    let mut value = frame(7);
    value.versions = VersionRange { first: 2, last: 2 };
    delivered(&mut old, Time::ZERO, f, value);
    assert_eq!(old.page(), &Page::OutOfDate);
    let mut other = Domain::new(&LIMITS, 7);
    let mut kept = List::with_capacity(1);
    kept.push(crate::store::Record::Draft { version: 1, party: 8, task: 42, words: text(b"secret") }).unwrap();
    let f = start(&mut other, kept);
    delivered(&mut other, Time::ZERO, f, frame(7));
    assert_eq!(other.page(), &Page::SomeoneElse);
}
#[test]
fn deployment_without_a_role_is_missing_and_never_watched() {
    let mut domain = Domain::new(&LIMITS, 8);
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    value.deployment_role = None;
    delivered(&mut domain, Time::ZERO, f, value);
    let out = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Deployment) });
    assert_eq!(domain.page(), &Page::Missing);
    for request in &out {
        if let Request::Open { subject, .. } = request {
            assert_ne!(*subject, Subject::Deployment);
        }
    }
}
