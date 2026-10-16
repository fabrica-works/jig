//! Checks before a memory restart (protocol/store.md, 5.3, 6 and 10).
use super::fixtures;
use crate::{Event, Family, OpenRefusal, Phase, Request, Store, Stores, codec, keys};
use alloc::boxed::Box;
use jig_domain_core::{CoreKey, Key};
use skein_lib::Token;

fn open(store: &mut Store) -> Box<[Event]> {
    fixtures::run(store, Request::Open { owner: Token::new(1), root: fixtures::ROOT })
}

fn planted(ops: &[skein_kv::Op]) -> Store {
    let mut stores = fixtures::stores();
    fixtures::plant(&mut stores, ops);
    fixtures::over(stores)
}

fn opened_rows() -> [skein_kv::Op; 2] {
    [fixtures::format_put(fixtures::DEPLOYMENT), fixtures::header_put(fixtures::DEPLOYMENT, 1)]
}

fn stores_with_header() -> Stores {
    let mut stores = fixtures::stores();
    fixtures::plant(&mut stores, &opened_rows());
    stores
}

fn header_without_format() -> Store {
    planted(&[fixtures::header_put(fixtures::DEPLOYMENT, 1)])
}

fn version_put(tasks: u16, core: u16) -> skein_kv::Op {
    let skein_kv::Op::Put { key, value } = fixtures::format_put(fixtures::DEPLOYMENT) else {
        panic!("format is a put");
    };
    let mut format = codec::component::Format::decode(&value).expect("format fixture");
    format.tasks = tasks;
    format.core = core;
    skein_kv::Op::Put { key, value: format.encode().expect("format fits") }
}

fn assert_refusal(events: &[Event], why: OpenRefusal) {
    let [Event::OpenRefused { owner, why: found }] = events else {
        panic!("one refusal: {events:?}");
    };
    assert_eq!(*owner, Token::new(1));
    assert_eq!(*found, why);
}

fn assert_opened(events: &[Event], last: u64) {
    let [Event::Opened { owner, last: found }] = events else {
        panic!("one opened terminal: {events:?}");
    };
    assert_eq!(*owner, Token::new(1));
    assert_eq!(*found, last);
}

fn assert_corrupt(events: &[Event]) {
    let [Event::Corrupt { owner }] = events else {
        panic!("one corrupt terminal: {events:?}");
    };
    assert_eq!(*owner, Token::new(1));
}

fn corrupt_header() -> skein_kv::Op {
    skein_kv::Op::Put {
        key: keys::encode(&Key::Core(CoreKey::Deployment), fixtures::KEY_LIMIT).expect("header key fits"),
        value: Box::new([]),
    }
}

#[test]
fn an_empty_store_opens_with_its_format_row_due() {
    let store = fixtures::opened();
    assert_eq!(store.phase, Phase::Ready { next: 1, format_due: true });
    let stores = store.into_stores();
    assert_eq!(stores.store.last(), 0);
    assert!(stores.store.is_empty());
}

#[test]
fn a_store_of_another_deployment_refuses_to_open() {
    let mut store =
        planted(&[fixtures::format_put(fixtures::OTHER_DEPLOYMENT), fixtures::header_put(fixtures::DEPLOYMENT, 1)]);
    assert_refusal(&open(&mut store), OpenRefusal::Deployment);
}

#[test]
fn a_store_of_another_tasks_version_refuses_naming_tasks() {
    let mut store = planted(&[version_put(3, 3), fixtures::header_put(fixtures::DEPLOYMENT, 1)]);
    assert_refusal(&open(&mut store), OpenRefusal::Format { family: Family::Tasks, found: 3, wanted: 2 });
}

#[test]
fn a_header_behind_the_last_commit_refuses_naming_both_numbers() {
    let mut stores = stores_with_header();
    fixtures::plant(&mut stores, &[]);
    let mut store = fixtures::over(stores);
    assert_refusal(&open(&mut store), OpenRefusal::Numbers { store: 2, header: 1 });
}

#[test]
fn a_header_without_its_format_row_refuses_as_format() {
    let mut store = header_without_format();
    assert_refusal(&open(&mut store), OpenRefusal::Format { family: Family::Component, found: 0, wanted: 1 });
}

#[test]
fn a_header_equal_to_the_last_commit_opens_with_it() {
    let mut store = planted(&opened_rows());
    assert_opened(&open(&mut store), 1);
    assert_eq!(store.phase, Phase::Ready { next: 2, format_due: false });
}

#[test]
fn a_format_row_without_its_header_refuses_as_numbers() {
    let mut store = planted(&[fixtures::format_put(fixtures::DEPLOYMENT)]);
    assert_refusal(&open(&mut store), OpenRefusal::Numbers { store: 1, header: 0 });
}

#[test]
fn an_unknown_core_version_is_refused_before_decoding_its_header() {
    let mut store = planted(&[version_put(2, 4), corrupt_header()]);
    assert_refusal(&open(&mut store), OpenRefusal::Format { family: Family::Core, found: 4, wanted: 3 });
}

#[test]
fn a_corrupt_header_stops_the_open() {
    let mut store = planted(&[fixtures::format_put(fixtures::DEPLOYMENT), corrupt_header()]);
    assert_corrupt(&open(&mut store));
    assert_eq!(store.phase, Phase::Stopped);
}

#[test]
fn a_corrupt_format_stops_the_open() {
    let mut store = planted(&[skein_kv::Op::Put {
        key: keys::encode_own(&keys::Own::Format, fixtures::KEY_LIMIT).expect("format key fits"),
        value: Box::new([]),
    }]);
    assert_corrupt(&open(&mut store));
    assert_eq!(store.phase, Phase::Stopped);
}

#[test]
fn a_stopped_open_answers_nothing_further() {
    let mut store = header_without_format();
    assert_refusal(&open(&mut store), OpenRefusal::Format { family: Family::Component, found: 0, wanted: 1 });
    assert!(open(&mut store).is_empty());
}

#[test]
fn opening_checks_no_row_beyond_the_format_and_header() {
    let mut stores = stores_with_header();
    fixtures::plant(&mut stores, &[skein_kv::Op::Put { key: Box::new([1, 0, 255]), value: Box::new([]) }]);
    fixtures::plant(&mut stores, &[fixtures::header_put(fixtures::DEPLOYMENT, 3)]);
    let mut store = fixtures::over(stores);
    assert_opened(&open(&mut store), 3);
}

#[test]
fn the_callers_two_stores_are_handed_back_without_changes() {
    let stores = stores_with_header();
    let expected = stores.store.used();
    let stores: Stores = fixtures::over(stores).into_stores();
    assert_eq!(stores.store.last(), 1);
    assert_eq!(stores.store.used(), expected);
    assert_eq!(stores.secrets.last(), 0);
    assert!(stores.secrets.is_empty());
}
