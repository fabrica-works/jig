use super::fixtures::{Client, OTHER, PAGE, PROJECT, project};
use crate::contract::{Scope, Window};
use crate::*;
use alloc::boxed::Box;

const FAMILY: u16 = 19;
const OTHER_ASKER: Asker = Asker(7);
const FAMILY_PAGE: Handle = Handle { asker: OTHER_ASKER, number: PAGE.number };

#[test]
fn a_read_is_told_its_one_page_by_its_handle() {
    let mut c = Client::started();
    c.step(Event::Read { handle: PAGE, read: project(PROJECT) });
    let downs = c.downs();
    let [Down::Read { read, what }] = downs.as_ref() else {
        panic!("one read");
    };
    assert_eq!(*what, project(PROJECT));
    let page = Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unknown));
    c.step(Event::Up { up: Up::ReadDone { read: *read, result: page.clone() } });
    assert_eq!(c.told().as_ref(), &[Told::Page { handle: PAGE, page: page.clone() }]);
    c.step(Event::Up { up: Up::ReadDone { read: *read, result: page } });
    assert!(c.told().is_empty());
}

#[test]
fn a_second_read_on_a_handle_drops_the_first_s_page() {
    let mut limits = super::fixtures::limits();
    limits.reads = 1;
    let mut c = Client::new(limits);
    let first = c.read(PAGE, project(PROJECT));
    let second = c.read(PAGE, project(PROJECT));
    assert_ne!(first, second);
    let page = Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unknown));
    c.step(Event::Up { up: Up::ReadDone { read: first, result: page.clone() } });
    assert!(c.told().is_empty());
    c.step(Event::Up { up: Up::ReadDone { read: second, result: page.clone() } });
    assert_eq!(c.told().as_ref(), &[Told::Page { handle: PAGE, page }]);
}

#[test]
fn a_read_past_the_limit_is_refused_by_the_client_s_limit() {
    let mut limits = super::fixtures::limits();
    limits.reads = 1;
    let mut c = Client::new(limits);
    let read = c.read(PAGE, project(PROJECT));
    c.step(Event::Read { handle: OTHER, read: project(PROJECT) });
    assert!(c.downs().is_empty());
    assert_eq!(c.told().as_ref(), &[Told::Page { handle: OTHER, page: Readout::Limit }]);
    let page = Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unknown));
    c.step(Event::Up { up: Up::ReadDone { read, result: page.clone() } });
    assert_eq!(c.told().as_ref(), &[Told::Page { handle: PAGE, page }]);
    c.read(OTHER, project(PROJECT));
}

#[test]
fn unwatching_a_handle_drops_its_read_s_late_page() {
    let mut c = Client::started();
    let read = c.read(PAGE, project(PROJECT));
    c.step(Event::Unwatch { handle: PAGE });
    c.step(Event::Up {
        up: Up::ReadDone { read, result: Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unknown)) },
    });
    assert!(c.told().is_empty());
    assert!(c.downs().is_empty());
}

#[test]
fn a_family_read_s_page_is_its_bytes_under_its_owner() {
    let mut c = Client::started();
    let what = Readable::Family {
        scope: Scope::Project(1),
        window: Window { rows: 3, after: None },
        record: Family { family: FAMILY, version: 1, bytes: Box::from([1, 2]) },
    };
    c.step(Event::Read { handle: FAMILY_PAGE, read: what.clone() });
    let downs = c.downs();
    let [Down::Read { read, what: sent }] = downs.as_ref() else {
        panic!("one family read");
    };
    assert_eq!(*sent, what);
    let page = Readout::Family { family: FAMILY, record: Some(Box::from([3, 4])) };
    c.step(Event::Up { up: Up::ReadDone { read: *read, result: page.clone() } });
    assert_eq!(c.told().as_ref(), &[Told::Page { handle: FAMILY_PAGE, page }]);
}
