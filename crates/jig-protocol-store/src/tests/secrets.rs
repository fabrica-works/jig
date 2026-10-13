//! Secrets' durability, paging and restart sweep (store.md, section 9).
use super::fixtures;
use crate::{Event, Phase, Request, Restore, SecretOwner, Secrets, Store};
use alloc::boxed::Box;
use skein_lib::{Secret, Token, Wall};
const OWNER: Token = Token::new(9);
fn ask(store: &mut Store, request: Secrets) -> Box<[Event]> {
    fixtures::run(store, Request::Secrets { owner: OWNER, request })
}
fn kept() -> Store {
    let mut store = fixtures::opened();
    ask(&mut store, fixtures::sign_in_secret(4, 41));
    store
}
fn get(store: &mut Store, kind: SecretOwner, number: u64) -> Option<Secret> {
    let mut events = ask(store, Secrets::Get { owner: kind, number }).into_iter();
    let Some(Event::Secret { owner, value }) = events.next() else {
        panic!("one secret answer");
    };
    assert_eq!(owner, OWNER);
    assert!(events.next().is_none());
    value
}
fn restore_people(store: &mut Store) {
    let events = fixtures::run(
        store,
        Request::Restore {
            owner: OWNER,
            range: Restore::Jig(Box::new([jig_domain_core::RestoreRange::People])),
            after: None,
            rows: 20,
            bytes: u64::MAX,
        },
    );
    let [Event::Restored { next: None, .. }] = &*events else {
        panic!("people restored");
    };
}
#[test]
fn a_kept_secret_is_answered_once_committed() {
    let mut store = fixtures::opened();
    let events = ask(&mut store, fixtures::sign_in_secret(4, 41));
    let [Event::Kept { owner: OWNER }] = &*events else {
        panic!("kept once committed");
    };
    assert_eq!(store.stores.secrets.last(), 1);
    assert!(store.stores.store.is_empty());
}
#[test]
fn a_secret_is_got_by_owner_and_number() {
    let mut store = kept();
    assert_eq!(get(&mut store, SecretOwner::SignIn, 4), Some(Secret::new(Box::new([41; 32]))));
    assert_eq!(get(&mut store, SecretOwner::Service, 4), None);
}
#[test]
fn an_erased_secret_gets_none() {
    let mut store = kept();
    let events = ask(&mut store, Secrets::Erase { owner: SecretOwner::SignIn, number: 4 });
    let [Event::Erased { owner: OWNER }] = &*events else {
        panic!("erased terminal");
    };
    assert_eq!(get(&mut store, SecretOwner::SignIn, 4), None);
}
#[test]
fn a_load_pages_an_owners_secrets_in_number_order() {
    let mut store = kept();
    ask(&mut store, fixtures::sign_in_secret(9, 42));
    ask(&mut store, fixtures::sign_in_secret(6, 43));
    let first = ask(&mut store, Secrets::Load { owner: SecretOwner::SignIn, after: None });
    let [Event::SecretsLoaded { rows, next: Some(6), .. }] = &*first else {
        panic!("first page: {first:?}");
    };
    let [first, second] = &**rows else {
        panic!("two secret rows");
    };
    assert_eq!((first.number, second.number), (4, 6));
    let second = ask(&mut store, Secrets::Load { owner: SecretOwner::SignIn, after: Some(6) });
    let [Event::SecretsLoaded { rows, next: None, .. }] = &*second else {
        panic!("last page: {second:?}");
    };
    let [last] = &**rows else {
        panic!("one last row");
    };
    assert_eq!(last.number, 9);
}
#[test]
fn the_sweep_erases_a_digest_whose_sign_in_is_gone() {
    let mut store = kept();
    ask(&mut store, fixtures::sign_in_secret(5, 42));
    let row = jig_domain_people::Stored::SignIn { number: 4, person: 11, expires: Wall::EPOCH };
    fixtures::committed(
        &mut store,
        Box::new([fixtures::header(1), fixtures::save(jig_domain_core::Record::People(row))]),
    );
    restore_people(&mut store);
    assert_eq!(get(&mut store, SecretOwner::SignIn, 4), Some(Secret::new(Box::new([41; 32]))));
    assert_eq!(get(&mut store, SecretOwner::SignIn, 5), None);
}
#[test]
fn the_sweep_keeps_a_digest_whose_service_remains() {
    let mut store = fixtures::opened();
    let secret = Secret::new(Box::new([42; 32]));
    ask(&mut store, Secrets::Keep { owner: SecretOwner::Service, number: 11, value: secret.clone() });
    fixtures::committed(&mut store, Box::new([fixtures::header(1), fixtures::save(fixtures::person(11))]));
    restore_people(&mut store);
    assert_eq!(get(&mut store, SecretOwner::Service, 11), Some(secret));
}
#[test]
fn a_secrets_request_debug_shows_lengths_not_bytes() {
    let request = fixtures::sign_in_secret(4, 41);
    let debug = alloc::fmt::format(format_args!("{request:?}"));
    assert!(debug.contains("Secret { len: 32 }"));
    assert!(!debug.contains("41"));
}
#[test]
fn a_refused_secrets_commit_stops_without_answering() {
    let mut store = fixtures::opened();
    let request = Secrets::Keep { owner: SecretOwner::SignIn, number: 1, value: Secret::new(Box::new([])) };
    store.limits.secrets.ops = 0;
    assert!(ask(&mut store, request).is_empty());
    assert_eq!(store.phase, Phase::Stopped);
}
#[test]
fn a_secret_past_its_value_bound_stops_before_submission() {
    let mut store = fixtures::opened();
    store.limits.secrets.value = 31;
    assert!(ask(&mut store, fixtures::sign_in_secret(4, 41)).is_empty());
    assert_eq!(store.stores.secrets.last(), 0);
    assert_eq!(store.phase, Phase::Stopped);
}
#[test]
fn the_sweep_continues_one_bounded_page_per_fire() {
    let mut store = fixtures::opened();
    for number in 0_u64..6 {
        ask(&mut store, fixtures::sign_in_secret(number, 41));
    }
    restore_people(&mut store);
    assert_eq!(get(&mut store, SecretOwner::SignIn, 0), None);
    assert!(get(&mut store, SecretOwner::SignIn, 2).is_some());
    for _ in 0_u8..2 {
        let mut above = skein_lib::Queue::with_capacity(crate::MAX_OUT_DOWN);
        let mut below = fixtures::below();
        crate::fire(&mut store, &fixtures::env(), &mut above, &mut below);
        assert!(above.is_empty());
        assert!(below.secrets.is_empty());
    }
    for number in 0_u64..6 {
        assert_eq!(get(&mut store, SecretOwner::SignIn, number), None);
    }
}
