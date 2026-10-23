//! Checks before a memory restart (protocol/store.md, 5.3, 6 and 10).
use super::fixtures;
use crate::{Event, Family, OpenRefusal, Phase, Request, Store, Stores, codec, keys};
use alloc::boxed::Box;
use jig_domain_core::{CoreKey, Key};
use skein_lib::Token;

fn open(store: &mut Store) -> Box<[Event]> {
    open_drawing(store, fixtures::INSTALLATION)
}

fn open_drawing(store: &mut Store, installation: [u8; 16]) -> Box<[Event]> {
    fixtures::run(store, Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation })
}

fn planted(ops: &[skein_kv::Op]) -> Store {
    let mut stores = fixtures::stores();
    fixtures::plant(&mut stores, ops);
    fixtures::over(stores)
}

fn opened_rows() -> [skein_kv::Op; 3] {
    [
        fixtures::format_put(fixtures::DEPLOYMENT),
        fixtures::header_put(fixtures::DEPLOYMENT, 1),
        fixtures::installation_put(fixtures::INSTALLATION),
    ]
}

fn fixture_format() -> (Box<[u8]>, codec::component::Format) {
    let skein_kv::Op::Put { key, value } = fixtures::format_put(fixtures::DEPLOYMENT) else {
        panic!("format is a put");
    };
    (key, codec::component::Format::decode(&value).expect("format fixture"))
}

fn stores_with_header() -> Stores {
    let mut stores = fixtures::stores();
    fixtures::plant(&mut stores, &opened_rows());
    stores
}

fn header_without_format() -> Store {
    planted(&[fixtures::header_put(fixtures::DEPLOYMENT, 1)])
}

fn version_put(tasks: u16, people: u16, core: u16) -> skein_kv::Op {
    let (key, mut format) = fixture_format();
    format.tasks = tasks;
    format.people = people;
    format.core = core;
    skein_kv::Op::Put { key, value: format.encode().expect("format fits") }
}

fn accounts_put(accounts: u16) -> skein_kv::Op {
    let (key, mut format) = fixture_format();
    format.accounts = accounts;
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
    assert_opened_with(events, last, fixtures::INSTALLATION);
}

fn assert_opened_with(events: &[Event], last: u64, installation: [u8; 16]) {
    let [Event::Opened { owner, last: found, installation: answered }] = events else {
        panic!("one opened terminal: {events:?}");
    };
    assert_eq!(*owner, Token::new(1));
    assert_eq!(*found, last);
    assert_eq!(*answered, installation);
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
fn an_empty_store_answers_the_drawn_installation() {
    let mut store = fixtures::over(fixtures::stores());
    assert_opened_with(&open_drawing(&mut store, [6; 16]), 0, [6; 16]);
}

#[test]
fn a_store_answers_its_kept_installation_not_the_drawn_one() {
    let mut store = planted(&opened_rows());
    assert_opened_with(&open_drawing(&mut store, [6; 16]), 1, fixtures::INSTALLATION);
}

#[test]
fn a_checked_store_without_its_installation_is_corrupt() {
    let mut store =
        planted(&[fixtures::format_put(fixtures::DEPLOYMENT), fixtures::header_put(fixtures::DEPLOYMENT, 1)]);
    assert_corrupt(&open(&mut store));
    assert_eq!(store.phase, Phase::Stopped);
}

#[test]
fn an_undecodable_installation_is_corrupt() {
    let mut store = planted(&[
        fixtures::format_put(fixtures::DEPLOYMENT),
        fixtures::header_put(fixtures::DEPLOYMENT, 1),
        skein_kv::Op::Put { key: keys::installation(fixtures::KEY_LIMIT).expect("key fits"), value: Box::new([0, 1]) },
    ]);
    assert_corrupt(&open(&mut store));
}

#[test]
fn a_format_row_of_another_component_version_refuses_naming_it() {
    let skein_kv::Op::Put { key, mut value } = fixtures::format_put(fixtures::DEPLOYMENT) else {
        panic!("format is a put");
    };
    let [high, low, ..] = &mut *value else { panic!("a version prefix") };
    (*high, *low) = (0, 1);
    let mut store = planted(&[skein_kv::Op::Put { key, value }, fixtures::header_put(fixtures::DEPLOYMENT, 1)]);
    assert_refusal(&open(&mut store), OpenRefusal::Format { family: Family::Component, found: 1, wanted: 2 });
}

#[test]
fn a_store_of_another_accounts_version_refuses_naming_accounts() {
    let mut store = planted(&[accounts_put(2), fixtures::header_put(fixtures::DEPLOYMENT, 1)]);
    assert_refusal(&open(&mut store), OpenRefusal::Format { family: Family::Accounts, found: 2, wanted: 1 });
}

#[test]
fn a_store_of_another_deployment_refuses_to_open() {
    let mut store =
        planted(&[fixtures::format_put(fixtures::OTHER_DEPLOYMENT), fixtures::header_put(fixtures::DEPLOYMENT, 1)]);
    assert_refusal(&open(&mut store), OpenRefusal::Deployment);
}

#[test]
fn a_store_of_another_tasks_version_refuses_naming_tasks() {
    let mut store = planted(&[version_put(6, 8, 5), fixtures::header_put(fixtures::DEPLOYMENT, 1)]);
    assert_refusal(&open(&mut store), OpenRefusal::Format { family: Family::Tasks, found: 6, wanted: 9 });
}

#[test]
fn a_store_of_another_people_version_refuses_naming_people() {
    let mut store = planted(&[version_put(9, 9, 5), fixtures::header_put(fixtures::DEPLOYMENT, 1)]);
    assert_refusal(&open(&mut store), OpenRefusal::Format { family: Family::People, found: 9, wanted: 10 });
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
    assert_refusal(&open(&mut store), OpenRefusal::Format { family: Family::Component, found: 0, wanted: 2 });
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
    let mut store = planted(&[version_put(9, 8, 6), corrupt_header()]);
    assert_refusal(&open(&mut store), OpenRefusal::Format { family: Family::Core, found: 6, wanted: 5 });
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
    assert_refusal(&open(&mut store), OpenRefusal::Format { family: Family::Component, found: 0, wanted: 2 });
    assert!(open(&mut store).is_empty());
}

#[test]
fn opening_checks_no_row_beyond_the_format_header_and_installation() {
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
