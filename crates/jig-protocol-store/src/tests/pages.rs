//! Child pages and exclusive continuation (protocol/store.md, 7.1).
use super::fixtures::{self, PROJECT};
use crate::{Cursor, Event, Load, Phase, Request, Row, Store, Write, keys};
use jig_domain_core::Record;
use jig_domain_notes as notes;
use skein_lib::{List, Token};

const OWNER: Token = Token::new(6);

fn scope() -> notes::Scope {
    notes::Scope::Project { project: PROJECT }
}

fn notes_store() -> Store {
    let mut store = fixtures::opened();
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(1),
            fixtures::save(fixtures::notes_line(&scope(), 1)),
            fixtures::save(fixtures::notes_line(&scope(), 2)),
            fixtures::save(fixtures::notes_line(&scope(), 3)),
        ]),
    );
    store
}

fn load(store: &mut Store, range: Load, after: Option<Cursor>, rows: u32, bytes: u64) -> Box<[Event]> {
    fixtures::run(store, Request::Load { owner: OWNER, range, after, rows, bytes })
}

fn notes_page(store: &mut Store, after: Option<Cursor>, rows: u32, bytes: u64) -> Box<[Event]> {
    load(store, Load::Notes(notes::Range::Lines { scope: scope(), after: None }), after, rows, bytes)
}

fn page(events: Box<[Event]>) -> (Box<[Row]>, Option<Cursor>) {
    let mut events = events.into_iter();
    let Some(Event::Loaded { owner, rows, next }) = events.next() else { panic!("one loaded page") };
    assert_eq!(owner, OWNER);
    assert!(events.next().is_none());
    (rows, next)
}

fn jig_rows(rows: Box<[Row]>) -> Box<[Record]> {
    let mut found = List::with_capacity(u32::try_from(rows.len()).expect("bounded fixture page"));
    for row in rows {
        let Row::Jig(record) = row else { panic!("typed jig row") };
        found.push(record).expect("counted fixture rows");
    }
    found.into_boxed()
}

fn line_cursor(name: u64) -> Cursor {
    Cursor {
        interval: 0,
        after: keys::encode(&fixtures::notes_line(&scope(), name).key(), fixtures::KEY_LIMIT).expect("line key fits"),
    }
}

fn assert_corrupt(store: &Store, events: &[Event]) {
    let [Event::Corrupt { owner }] = events else { panic!("corruption terminal: {events:?}") };
    assert_eq!(*owner, OWNER);
    assert_eq!(store.phase, Phase::Stopped);
}

#[test]
fn a_notes_page_stops_at_the_askers_rows_with_a_cursor() {
    let mut store = notes_store();
    let (rows, next) = page(notes_page(&mut store, None, 2, 4096));
    assert_eq!(next, Some(line_cursor(2)));
    assert_eq!(jig_rows(rows).as_ref(), &[fixtures::notes_line(&scope(), 1), fixtures::notes_line(&scope(), 2)]);
}

#[test]
fn a_page_keeps_the_prefix_within_the_decoded_bytes() {
    let mut store = notes_store();
    // These project lines measure 34 bytes each; two exceed 50 bytes.
    let (rows, next) = page(notes_page(&mut store, None, 2, 50));
    assert_eq!(next, Some(line_cursor(1)));
    assert_eq!(jig_rows(rows).as_ref(), &[fixtures::notes_line(&scope(), 1)]);
}

#[test]
fn a_page_keeps_the_prefix_within_the_encoded_bytes() {
    let mut store = notes_store();
    // The project line's key and value fit 80 bytes, while two exceed it.
    store.limits.store.page.bytes = 80;
    let (rows, next) = page(notes_page(&mut store, None, 2, 4096));
    assert_eq!(next, Some(line_cursor(1)));
    assert_eq!(jig_rows(rows).as_ref(), &[fixtures::notes_line(&scope(), 1)]);
}

#[test]
fn lines_after_a_name_start_just_past_it() {
    let mut store = notes_store();
    let range = Load::Notes(notes::Range::Lines { scope: scope(), after: Some(1) });
    let (rows, next) = page(load(&mut store, range, None, 2, 4096));
    assert_eq!(next, None);
    assert_eq!(jig_rows(rows).as_ref(), &[fixtures::notes_line(&scope(), 2), fixtures::notes_line(&scope(), 3)]);
}

fn application_store() -> Store {
    let mut store = fixtures::opened();
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(1),
            Write::Put { key: Box::from(b"a1".as_slice()), value: Box::new([7]) },
            Write::Put { key: Box::from(b"a2".as_slice()), value: Box::new([9]) },
        ]),
    );
    store
}

fn application_page(store: &mut Store, after: Option<Cursor>, rows: u32, bytes: u64) -> Box<[Event]> {
    load(store, Load::Application(Box::new([skein_kv::Range::prefix(b"a")])), after, rows, bytes)
}

fn assert_application_row(rows: &[Row], key: &[u8], value: &[u8]) {
    let [Row::Application { key: found_key, value: found_value }] = rows else { panic!("one application row") };
    assert_eq!(found_key.as_ref(), key);
    assert_eq!(found_value.as_ref(), value);
}

#[test]
fn an_application_page_carries_its_cursor_and_no_tag() {
    let mut store = application_store();
    let (rows, next) = page(application_page(&mut store, None, 1, 4096));
    assert_application_row(&rows, b"a1", &[7]);
    assert_eq!(next, Some(Cursor { interval: 0, after: Box::new([2, b'a', b'1']) }));
    let (rows, next) = page(application_page(&mut store, next, 1, 4096));
    assert_eq!(next, None);
    assert_application_row(&rows, b"a2", &[9]);
}

fn corrupt_store() -> Store {
    let mut stores = fixtures::stores();
    fixtures::plant(
        &mut stores,
        &[
            fixtures::format_put(fixtures::DEPLOYMENT),
            fixtures::header_put(fixtures::DEPLOYMENT, 1),
            skein_kv::Op::Put {
                key: keys::encode(&fixtures::notes_line(&scope(), 1).key(), fixtures::KEY_LIMIT)
                    .expect("line key fits"),
                value: Box::new([255]),
            },
        ],
    );
    let mut store = fixtures::over(stores);
    let events = fixtures::run(&mut store, Request::Open { owner: OWNER, root: fixtures::ROOT_FD });
    let [Event::Opened { .. }] = &*events else { panic!("fixture opens: {events:?}") };
    store
}

#[test]
fn an_undecodable_page_row_is_corrupt() {
    let mut store = corrupt_store();
    let events = notes_page(&mut store, None, 2, 4096);
    assert_corrupt(&store, &events);
}

#[test]
fn a_notes_page_cursor_resumes_after_its_last_row() {
    let mut store = notes_store();
    let (_, next) = page(notes_page(&mut store, None, 2, 4096));
    let (rows, next) = page(notes_page(&mut store, next, 2, 4096));
    assert_eq!(next, None);
    assert_eq!(jig_rows(rows).as_ref(), &[fixtures::notes_line(&scope(), 3)]);
}

#[test]
fn an_application_page_counts_the_untagged_key_and_value() {
    let mut store = application_store();
    let (rows, next) = page(application_page(&mut store, None, 2, 3));
    assert_application_row(&rows, b"a1", &[7]);
    assert_eq!(next, Some(Cursor { interval: 0, after: Box::new([2, b'a', b'1']) }));
}

#[test]
fn a_notes_entry_load_answers_the_named_entry() {
    let mut store = fixtures::opened();
    fixtures::committed(&mut store, Box::new([fixtures::header(1), fixtures::save(fixtures::notes_entry(17))]));
    let (rows, next) = page(load(&mut store, Load::Notes(notes::Range::Entry { name: 17 }), None, 2, 4096));
    assert_eq!(next, None);
    assert_eq!(jig_rows(rows).as_ref(), &[fixtures::notes_entry(17)]);
}

#[test]
fn a_notes_entry_load_of_a_missing_name_is_empty() {
    let mut store = notes_store();
    let (rows, next) = page(load(&mut store, Load::Notes(notes::Range::Entry { name: 19 }), None, 2, 4096));
    assert_eq!(next, None);
    assert!(rows.is_empty());
}

#[test]
fn nothing_is_answered_after_a_corrupt_load() {
    let mut store = corrupt_store();
    let events = notes_page(&mut store, None, 2, 4096);
    assert_corrupt(&store, &events);
    assert!(notes_page(&mut store, None, 2, 4096).is_empty());
}
