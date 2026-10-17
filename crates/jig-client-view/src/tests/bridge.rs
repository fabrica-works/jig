use super::{
    cards,
    fixtures::{Fixture, Watched},
    turns,
};
use crate::{
    Event, Request,
    address::{Address, Place},
};
use jig_client::{Asker, Key, Up, contract};
use skein_lib::{List, Time, Wall};

#[test]
fn a_page_s_watch_is_the_client_s_under_jig_s_asker() {
    let (fixture, handle) = cards::start();
    assert_eq!(handle.asker, Asker::JIG);
    assert!(fixture.client.document(handle).is_some());
}
#[test]
fn a_changed_handle_is_the_page_s_delivery() {
    let (mut fixture, watch) = cards::start();
    cards::drive(
        &mut fixture,
        Watched { watch, event: contract::WatchEvent::Delivered(contract::Delivery::Me(Box::new(frame()))) },
    );
    assert_eq!(fixture.frame().party(), Some(7));
}
#[test]
fn a_run_s_text_reaches_the_page_in_its_changed_document() {
    let (mut fixture, _, watch, _) = turns::ready();
    let text = contract::RunText {
        attempt: 2,
        began: Some(Wall::EPOCH),
        turn: contract::ProvisionalTurnName { attempt: 2, number: 1, retry: 0 },
        from: 0,
        through: 3,
        text: cards::text(b"one"),
        cut: false,
        call: None,
    };
    cards::drive(
        &mut fixture,
        Watched { watch, event: contract::WatchEvent::Delivered(contract::Delivery::Run(Box::new(text))) },
    );
    assert_eq!(fixture.conversation().live.get(0).unwrap().text.bytes(), b"one");
}
#[test]
fn out_of_date_shows_the_out_of_date_page() {
    let (mut fixture, _) = cards::start();
    cards::drive(&mut fixture, Up::Served { served: false, versions: None });
    assert_eq!(fixture.page(), &crate::page::Page::OutOfDate);
}
#[test]
fn a_watch_the_client_s_limit_ends_retires() {
    let (mut fixture, handle) = cards::start();
    cards::drive(&mut fixture, Event::Client { told: jig_client::Told::Ended { handle, why: jig_client::Why::Limit } });
    assert!(!fixture.handles.watches.contains_key(&handle));
}
fn sent(seed: u64) -> Key {
    let mut fixture = Fixture::new(&cards::CLIENT);
    cards::drive(
        &mut fixture,
        Event::Start { seed, place: Place::Jig(Address::Inbox { project: None }), records: List::with_capacity(0) },
    );
    let out =
        cards::drive(&mut fixture, crate::tests::fixtures::intent(7, 1, contract::Ask::Stop { project: 1, task: 9 }));
    let Request::Client { asked: jig_client::Event::Send { key, .. } } = out[0] else { panic!("send") };
    key
}
#[test]
fn a_key_is_never_a_kept_key_after_a_reload() {
    let kept = sent(8);
    let fresh = sent(12);
    assert_eq!(kept.asker, Asker::JIG);
    assert_eq!(fresh.asker, Asker::JIG);
    assert_ne!(fresh, kept);
}
#[test]
fn a_lost_sign_out_reaches_the_ui() {
    let (mut fixture, watch) = cards::start();
    cards::drive(
        &mut fixture,
        Watched { watch, event: contract::WatchEvent::Delivered(contract::Delivery::Me(Box::new(frame()))) },
    );
    fixture.drive(Time::ZERO, jig_client::Event::SignOut);
    fixture.drive(Time::ZERO, jig_client::Event::SignOutLost);
    let mut notices = List::with_capacity(1);
    for (id, ()) in fixture.notices().ids() {
        notices.push(&fixture.notices().get(*id).unwrap().kind).unwrap();
    }
    assert_eq!(notices.as_slice(), &[&crate::notices::kept::Kind::SignOutLost]);
    assert_eq!(fixture.frame().party(), Some(7));
}

fn frame() -> contract::Me {
    let mut frame = cards::document();
    frame.party.number = 7;
    frame
}
