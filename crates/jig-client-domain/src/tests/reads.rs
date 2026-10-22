use super::fixtures::{Client, OTHER, PAGE, PROJECT, project};
use crate::contract::{Scope, Window};
use crate::*;
use alloc::boxed::Box;
use skein_lib::List;

const FAMILY: u16 = 19;
const OTHER_ASKER: Asker = Asker(7);
const FAMILY_PAGE: Handle = Handle { asker: OTHER_ASKER, number: PAGE.number };
// More read terminals than the fixture's output bound without reads, with no watch handles.
const INDEPENDENT_READS: u32 = 64;

#[test]
fn a_read_is_told_its_one_page_by_its_handle() {
    for what in [project(PROJECT), Readable::Jig(contract::Read::Turn { task: 7, attempt: 44, turn: 3 })] {
        let mut c = Client::started();
        c.step(Event::Read { handle: PAGE, read: what.clone() });
        let downs = c.downs();
        let [Down::Read { read, what }] = downs.as_ref() else {
            panic!("one read");
        };
        assert_eq!(*what, what.clone());
        let page = Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unknown));
        c.step(Event::Up { up: Up::ReadDone { read: *read, result: page.clone() } });
        assert_eq!(c.told().as_ref(), &[Told::Page { handle: PAGE, page: page.clone() }]);
        c.step(Event::Up { up: Up::ReadDone { read: *read, result: page } });
        assert!(c.told().is_empty());
    }
}

#[test]
fn a_second_read_on_a_handle_drops_the_first_s_page() {
    for what in [project(PROJECT), Readable::Jig(contract::Read::Turn { task: 7, attempt: 44, turn: 3 })] {
        let mut limits = super::fixtures::limits();
        limits.reads = 1;
        let mut c = Client::new(limits);
        let first = c.read(PAGE, what.clone());
        let second = c.read(PAGE, what.clone());
        assert_ne!(first, second);
        let page = Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unknown));
        c.step(Event::Up { up: Up::ReadDone { read: first, result: page.clone() } });
        assert!(c.told().is_empty());
        c.step(Event::Up { up: Up::ReadDone { read: second, result: page.clone() } });
        assert_eq!(c.told().as_ref(), &[Told::Page { handle: PAGE, page }]);
    }
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
    for what in [project(PROJECT), Readable::Jig(contract::Read::Turn { task: 7, attempt: 44, turn: 3 })] {
        let mut c = Client::started();
        let read = c.read(PAGE, what.clone());
        c.step(Event::Unwatch { handle: PAGE });
        c.step(Event::Up {
            up: Up::ReadDone { read, result: Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unknown)) },
        });
        assert!(c.told().is_empty());
        assert!(c.downs().is_empty());
    }
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

#[test]
fn credential_boundaries_end_all_independently_bounded_reads_once_and_free_room() {
    // No watch handles: read room and cancellation output cannot borrow their bound.
    let limits = Limits { handles: 0, watches: 1, reads: INDEPENDENT_READS, ..super::fixtures::limits() };
    for start in [Client::unsigned, Client::new] {
        for boundary in [Up::SignedOut, Up::SignedIn] {
            let mut c = start(limits);
            let mut pending = List::with_capacity(limits.reads);
            let mut expected = List::with_capacity(limits.reads);
            for number in 0..u64::from(limits.reads) {
                let handle = Handle { asker: Asker::JIG, number };
                let (what, page) = match number % 3 {
                    0 => (project(PROJECT), Readout::Jig(contract::ReadResult::None(contract::ReadNone::SignedOut))),
                    choice => {
                        let family = if choice == 1 { FAMILY } else { 1 };
                        (
                            Readable::Family {
                                scope: Scope::Project(PROJECT),
                                window: Window { rows: 1, after: None },
                                record: Family { family, version: 1, bytes: Box::from([1]) },
                            },
                            Readout::Family { family, record: None },
                        )
                    }
                };
                pending.push((handle, c.read(handle, what.clone()), what)).unwrap();
                expected.push(Told::Page { handle, page }).unwrap();
            }
            let extra = Handle { asker: OTHER_ASKER, number: u64::from(limits.reads) };
            c.step(Event::Read { handle: extra, read: project(PROJECT) });
            assert_eq!(c.told().as_ref(), &[Told::Page { handle: extra, page: Readout::Limit }]);
            c.downs();
            c.step(Event::Up { up: boundary });
            let told = c.told();
            assert_eq!(pages(&told).as_slice(), expected.as_slice());
            assert!(u32::try_from(told.len() + c.downs().len()).unwrap() <= max_out(&limits));
            c.step(Event::Up { up: Up::SignedOut });
            assert!(pages(&c.told()).is_empty());
            c.downs();
            for (handle, old, what) in pending.into_boxed() {
                let new = c.read(handle, what);
                assert_ne!(old, new);
                let page = Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unknown));
                c.step(Event::Up { up: Up::ReadDone { read: old, result: page.clone() } });
                assert!(c.told().is_empty());
            }
            assert_eq!(c.domain.reads.len(), limits.reads);
        }
    }
}

fn pages(told: &[Told]) -> List<Told> {
    let mut pages = List::with_capacity(INDEPENDENT_READS);
    for told in told {
        if let Told::Page { .. } = told {
            pages.push(told.clone()).unwrap();
        }
    }
    pages
}
