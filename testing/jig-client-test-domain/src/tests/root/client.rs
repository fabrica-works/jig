use super::fixtures::{RECORD_ITEM, RECORD_PROJECT, ROUTED_TOKEN, UNOWNED_KEY_NUMBER};
use super::{ask, confirm, drive, env, limits, out_iter, ready, start};
use crate::{Address, Event, JigOut, Request, first, max_out, new, route};
use jig_client::Link;
use jig_client::{Asker, Key, Told, contract};
use skein_lib::{List, Queue};

#[test]
fn the_link_reaches_each_child() {
    let root = ready(7);
    assert_eq!(root.first().link(), Link::Live);
    assert_eq!(root.second().link(), Link::Live);
    assert_eq!(root.jig().link(), Link::Live);
}

#[test]
fn passes_the_client_s_transport_requests_out_in_order() {
    let mut root = new(&limits(), 7);
    let mut child = Queue::with_capacity(2);
    child.push(jig_client::Request::OpenStream { credential: None });
    child.push(jig_client::Request::CloseStream);
    let mut out = Queue::with_capacity(max_out(&limits()));
    route::route_client(&mut root, &env(), &mut child, &mut out);
    assert_eq!(
        out_iter(out),
        vec![
            Request::Client { request: jig_client::Request::OpenStream { credential: None } },
            Request::Client { request: jig_client::Request::CloseStream },
        ]
    );
}

#[test]
fn a_kept_request_of_an_asker_no_owner_names_reaches_no_child() {
    let mut root = new(&limits(), 7);
    let before = crate::shown(&root);
    let mut out = Queue::with_capacity(max_out(&limits()));
    route::told(
        &mut root,
        &env(),
        Told::Kept {
            key: Key { asker: Asker(u32::MAX), number: UNOWNED_KEY_NUMBER },
            request: jig_client::Ask::Jig(Box::new(contract::Ask::Stop { project: 1, task: 9 })),
        },
        &mut out,
    );
    assert_eq!(out_iter(out), vec![]);
    assert_eq!(crate::shown(&root), before);
}

#[test]
fn routes_a_tell_to_the_child_its_asker_names() {
    let mut root = crate::unlinked(&limits(), 7);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    let rows = ask(&mut root, ROUTED_TOKEN);
    let key = rows
        .iter()
        .find_map(|row| match row {
            Request::Save { record: crate::store::Record::Unlinked(record) } => Some(record.request.key),
            _ => None,
        })
        .expect("the client keeps the original asker's key");
    let mut out = Queue::with_capacity(max_out(&limits()));
    let answer = contract::Answer::Done(contract::Outcome::Changed);
    route::told(
        &mut root,
        &env(),
        Told::Answered { key, outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(answer.clone())) },
        &mut out,
    );
    assert_eq!(out_iter(out), vec![Request::Jig(JigOut::Answered { asked: ROUTED_TOKEN, answer })]);
}

#[test]
fn the_reference_ui_s_records_leave_before_the_step_s_exchanges() {
    let mut root = ready(7);
    let rows = drive(
        &mut root,
        Event::Went {
            place: crate::Place::Address(Address::First(first::Address { project: RECORD_PROJECT, item: RECORD_ITEM })),
        },
    );
    let mut exchange = false;
    let mut stored = false;
    for row in rows {
        match row {
            Request::Client { .. } => exchange = true,
            Request::Save { .. } | Request::Erase { .. } => {
                assert!(!exchange, "child records precede transport work");
                stored = true;
            }
            Request::Show { .. }
            | Request::SignIn { .. }
            | Request::SignOut
            | Request::Jig(_)
            | Request::First(_)
            | Request::Fact { .. } => {}
        }
    }
    assert!(stored);
    assert!(exchange);
}
