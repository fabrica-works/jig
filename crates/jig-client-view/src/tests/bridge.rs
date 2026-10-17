use super::fixtures::bridge::Fixture;
use crate::{Request, bridge};
use jig_client::{Asker, Handle, Told, contract};
use skein_lib::{Token, Wall};
#[test]
fn a_page_s_watch_is_the_client_s_under_jig_s_asker() {
    let request = Request::Open { watch: Token::new(17), subject: contract::Subject::Task { task: 9 } };
    let Some(jig_client::Event::Watch { handle, subject }) = bridge::asked(&request, 8, 4) else {
        panic!("the page asks the client to watch");
    };
    assert_eq!(handle, Handle { asker: Asker::JIG, number: 17 });
    assert_eq!(subject, jig_client::Watchable::Jig(contract::Subject::Task { task: 9 }));
}
#[test]
fn a_changed_handle_is_the_page_s_delivery() {
    let mut f = Fixture::new();
    f.tell(Told::Link { link: jig_client::Link::SignedOut });
    f.tell(Told::Link { link: jig_client::Link::Starting });
    f.watch = f.ui.frame_watch.expect("authentication takes a fresh frame handle").token();
    f.frame();
    assert_eq!(f.ui.frame().party(), Some(7));
}
#[test]
fn a_run_s_text_reaches_the_page_in_its_changed_document() {
    let mut f = Fixture::new();
    f.frame();
    f.ui.conversation.task = 9;
    let watch =
        f.ui.open(
            contract::Subject::Run { task: 9, turn: None, offset: 0 },
            crate::watches::Owner::Page { generation: 0 },
            &mut f.out,
        )
        .unwrap()
        .token();
    let handle = Handle { asker: Asker::JIG, number: watch.raw() };
    let text = contract::RunText {
        attempt: 1,
        began: Some(Wall::EPOCH),
        turn: contract::ProvisionalTurnName { attempt: 1, number: 1, retry: 0 },
        from: 0,
        through: 3,
        text: contract::Text::new(Box::from(*b"one"), 32).unwrap(),
        cut: false,
        call: None,
    };
    f.deliver(
        watch,
        contract::Subject::Run { task: 9, turn: None, offset: 0 },
        contract::Delivery::Run(Box::new(text)),
    );
    f.tell(Told::Changed { handle });
    f.tell(Told::Text { handle });
    assert_eq!(f.ui.conversation().live.get(0).unwrap().text.bytes(), b"one");
}
#[test]
fn out_of_date_shows_the_out_of_date_page() {
    let mut f = Fixture::new();
    f.tell(Told::Link { link: jig_client::Link::OutOfDate });
    assert_eq!(f.ui.page(), &crate::page::Page::OutOfDate);
}
#[test]
fn a_watch_the_client_s_limit_ends_retires() {
    let mut f = Fixture::new();
    f.tell(Told::Ended { handle: f.handle(), why: jig_client::Why::Limit });
    crate::reclaim(&mut f.ui);
    assert_eq!(f.ui.watch_ids.len(), 0);
}
#[test]
fn a_key_is_never_a_kept_key_after_a_reload() {
    let request =
        Request::Send { request: Token::new(11), scope: 7, ask: Box::new(contract::Ask::Stop { project: 1, task: 9 }) };
    let Some(jig_client::Event::Send { key: kept, .. }) = bridge::asked(&request, 8, 4) else { panic!("send") };
    let Some(jig_client::Event::Send { key: fresh, .. }) = bridge::asked(&request, 12, 4) else { panic!("send") };
    assert_eq!(kept, jig_client::Key { asker: Asker::JIG, number: 3 });
    assert_eq!(fresh, jig_client::Key { asker: Asker::JIG, number: 7 });
    assert_ne!(fresh, kept);
}

#[test]
fn a_lost_sign_out_reaches_the_ui() {
    let mut f = Fixture::new();
    f.frame();
    f.event(jig_client::Event::SignOut);
    f.event(jig_client::Event::SignOutLost);
    f.tell(Told::Link { link: jig_client::Link::Live });
    let mut notices = skein_lib::List::with_capacity(1);
    for (id, ()) in f.ui.notices().ids() {
        notices.push(&f.ui.notices().get(*id).unwrap().kind).unwrap();
    }
    assert_eq!(notices.as_slice(), &[&crate::notices::kept::Kind::SignOutLost]);
    assert_eq!(f.ui.frame().party(), Some(7));
}
