//! Directory startup and independent file routes (store.md, 2, 6 and 13).
use super::fixtures;
use crate::{Arrived, Below, Event, Io, Phase, Request, Store};
use alloc::boxed::Box;
use skein_io::{
    file,
    kernel::{Error, Fd},
};
use skein_lib::{Queue, Token};
fn made(store: &mut Store, secret: bool, owner: Token) -> (Box<[Event]>, Below) {
    let arrived = Arrived { event: file::Event::Made { owner }, fd: None };
    fixtures::answer(store, if secret { Io::Secrets(arrived) } else { Io::Store(arrived) })
}
fn opened(store: &mut Store, io: u8, owner: Token) -> (Box<[Event]>, Below) {
    let arrived = Arrived { event: file::Event::Opened { owner, file: Token::new(100), len: 0 }, fd: Some(Fd::new(4)) };
    fixtures::answer(store, driver(io, arrived))
}
fn driver(io: u8, arrived: Arrived) -> Io {
    match io {
        0 => Io::Store(arrived),
        1 => Io::Transcripts(arrived),
        _ => Io::Secrets(arrived),
    }
}
fn parent_request(below: &mut Below, io: u8, label: &str) -> Token {
    let request = match io {
        0 => below.store.pop(),
        1 => below.transcripts.pop(),
        _ => below.secrets.pop(),
    };
    let Some(file::Request::SyncDirectory { owner, root }) = request else { panic!("{label}: parent sync first") };
    assert_eq!(root, fixtures::ROOT.fd, "{label}");
    owner
}
fn parent_synced(store: &mut Store, mut below: Below, io: u8) -> Below {
    let owner = parent_request(&mut below, io, "fixture parent synced");
    let arrived = Arrived { event: file::Event::Synced { owner }, fd: None };
    let (events, below) = fixtures::answer(store, driver(io, arrived));
    assert!(events.is_empty());
    below
}
fn at_private() -> (Store, Below) {
    let (mut store, below) = at_parent_sync(true);
    let mut below = parent_synced(&mut store, below, 1);
    let Some(file::Request::OpenDirectory { owner, .. }) = below.transcripts.pop() else {
        panic!("open transcripts");
    };
    let (_, below) = opened(&mut store, 1, owner);
    (store, below)
}
#[test]
fn opening_on_a_directory_makes_the_stores_directory_first() {
    let (mut store, mut below) = fixtures::directory();
    let Some(file::Request::MakeDirectory { owner, root, path, mode }) = below.store.pop() else {
        panic!("the store's directory is made first");
    };
    assert_eq!((root, path.as_ref(), mode), (Token::new(900), b"store".as_slice(), 0o700));
    let (events, below) = made(&mut store, false, owner);
    assert!(events.is_empty());
    let mut below = parent_synced(&mut store, below, 0);
    let Some(file::Request::OpenDirectory { name, no_follow: true, .. }) = below.store.pop() else {
        panic!("the store's directory is opened next");
    };
    assert_eq!(name.as_ref(), b"store");
}
#[test]
fn the_secrets_directory_is_opened_as_private() {
    let (_, mut below) = at_private();
    let Some(file::Request::OpenPrivate { root, path, .. }) = below.secrets.pop() else {
        panic!("private secrets");
    };
    assert_eq!((root, path.as_ref()), (fixtures::ROOT.token, b"secrets".as_slice()));
}
#[test]
fn skein_requests_go_below_on_their_stores_driver() {
    let (mut store, mut below) = at_private();
    let Some(file::Request::OpenPrivate { owner, .. }) = below.secrets.pop() else {
        panic!("private secrets");
    };
    let (events, below) = opened(&mut store, 2, owner);
    assert!(events.is_empty());
    let mut below = parent_synced(&mut store, below, 2);
    let Some(file::Request::List { owner, .. }) = below.secrets.pop() else {
        panic!("secrets recovery list");
    };
    assert_ne!(owner, crate::directory::OWNER);
    assert!(below.transcripts.is_empty());
}
#[test]
fn a_failed_skein_commit_on_disk_answers_failed() {
    let mut store = disk();
    let mut below = submit(&mut store, 1);
    let Some(file::Request::WriteAt { owner, .. }) = below.store.pop() else {
        panic!("disk write");
    };
    let arrived = Arrived {
        event: file::Event::Failed { owner, error: Error::Permission, committed: false, residue: None },
        fd: None,
    };
    let (events, _) = fixtures::answer(&mut store, Io::Store(arrived));
    let [Event::Failed { number: 1 }] = &*events else {
        panic!("failed root commit");
    };
    assert_eq!(store.phase, Phase::Stopped);
}
#[test]
fn a_recovery_beneath_the_store_stops_it() {
    let mut store = fixtures::opened();
    let mut kv = Queue::with_capacity(skein_kv::MAX_OUT.above);
    kv.push(skein_kv::Event::Recovered { last: 0 });
    let mut above = Queue::with_capacity(crate::MAX_OUT_DOWN);
    crate::directory::events_up(&mut store, false, &mut kv, &mut above, &mut fixtures::below());
    assert_eq!(store.phase, Phase::Stopped);
}
#[test]
fn a_directory_failure_refuses_open() {
    let (mut store, mut below) = fixtures::directory();
    let Some(file::Request::MakeDirectory { owner, .. }) = below.store.pop() else {
        panic!("make store");
    };
    let arrived = Arrived {
        event: file::Event::Failed { owner, error: Error::Permission, committed: false, residue: None },
        fd: None,
    };
    let (events, _) = fixtures::answer(&mut store, Io::Store(arrived));
    let [Event::OpenRefused { .. }] = &*events else {
        panic!("open refused");
    };
}

fn disk() -> Store {
    let (mut store, mut below) = at_private();
    let Some(file::Request::OpenPrivate { owner, .. }) = below.secrets.pop() else {
        panic!("private secrets");
    };
    let (_, below) = opened(&mut store, 2, owner);
    let mut below = parent_synced(&mut store, below, 2);
    for secret in [false, true] {
        let mut queue = if secret {
            core::mem::replace(&mut below.secrets, Queue::with_capacity(skein_kv::MAX_OUT.below))
        } else {
            core::mem::replace(&mut below.store, Queue::with_capacity(skein_kv::MAX_OUT.below))
        };
        for operation in 0_u8..3 {
            let request = queue.pop().expect("store open operation");
            let event = match operation {
                0 => {
                    let file::Request::List { owner, .. } = request else {
                        panic!("list store files");
                    };
                    file::Event::Listed { owner, entries: Box::new([]) }
                }
                1 => {
                    let file::Request::Create { owner, mode: 0o600, .. } = request else {
                        panic!("create private store log");
                    };
                    file::Event::Opened { owner, file: Token::new(if secret { 201 } else { 200 }), len: 0 }
                }
                _ => {
                    let file::Request::SyncDirectory { owner, .. } = request else {
                        panic!("sync store directory");
                    };
                    file::Event::Synced { owner }
                }
            };
            let arrived = Arrived { event, fd: None };
            let (events, next) =
                fixtures::answer(&mut store, if secret { Io::Secrets(arrived) } else { Io::Store(arrived) });
            queue = if secret { next.secrets } else { next.store };
            if !events.is_empty() {
                let [Event::Opened { last: 0, .. }] = &*events else {
                    panic!("both stores opened");
                };
            }
        }
        if secret {
            below.secrets = queue;
        } else {
            below.store = queue;
        }
    }
    assert_eq!(store.phase, Phase::Ready { next: 1, format_due: true });
    store
}
fn submit(store: &mut Store, number: u64) -> Below {
    let mut above = Queue::with_capacity(crate::MAX_OUT_DOWN);
    let mut below = fixtures::below();
    crate::down(
        store,
        &fixtures::env(),
        fixtures::commit(number, Box::new([fixtures::header(number)])),
        &mut above,
        &mut below,
    );
    assert!(above.is_empty());
    below
}
#[test]
fn submitted_commits_advance_before_the_durable_number() {
    let mut store = disk();
    let mut below = submit(&mut store, 1);
    assert_eq!(store.stores.store.last(), 0);
    let Some(file::Request::WriteAt { owner, .. }) = below.store.pop() else {
        panic!("write the first commit");
    };
    let queued = submit(&mut store, 2);
    assert!(queued.store.is_empty());
    assert_eq!(store.phase, Phase::Ready { next: 3, format_due: false });
    let (_, mut below) =
        fixtures::answer(&mut store, Io::Store(Arrived { event: file::Event::Written { owner }, fd: None }));
    let Some(file::Request::Sync { .. }) = below.store.pop() else {
        panic!("sync root commit");
    };
    let (events, _) =
        fixtures::answer(&mut store, Io::Store(Arrived { event: file::Event::Synced { owner }, fd: None }));
    let [Event::Committed { number: 1, .. }] = &*events else {
        panic!("first commit durable");
    };
    assert_eq!(store.stores.store.last(), 1);
    assert_eq!(store.phase, Phase::Ready { next: 3, format_due: false });
}
#[test]
fn a_disk_secret_waits_for_its_own_sync() {
    let mut store = disk();
    let mut above = Queue::with_capacity(crate::MAX_OUT_DOWN);
    let mut below = fixtures::below();
    crate::down(
        &mut store,
        &fixtures::env(),
        Request::Secrets { owner: Token::new(9), request: fixtures::sign_in_secret(4, 41) },
        &mut above,
        &mut below,
    );
    assert!(above.is_empty());
    let Some(file::Request::WriteAt { owner, .. }) = below.secrets.pop() else {
        panic!("write the secret");
    };
    let (events, mut below) =
        fixtures::answer(&mut store, Io::Secrets(Arrived { event: file::Event::Written { owner }, fd: None }));
    assert!(events.is_empty());
    let Some(file::Request::Sync { .. }) = below.secrets.pop() else {
        panic!("sync secret");
    };
    let (events, _) =
        fixtures::answer(&mut store, Io::Secrets(Arrived { event: file::Event::Synced { owner }, fd: None }));
    let [Event::Kept { owner }] = &*events else {
        panic!("secret durable");
    };
    assert_eq!(*owner, Token::new(9));
    assert_eq!(store.stores.secrets.last(), 1);
    assert_eq!(store.stores.store.last(), 0);
}

fn at_parent_sync(transcripts: bool) -> (Store, Below) {
    let (mut store, mut below) = fixtures::directory();
    let Some(file::Request::MakeDirectory { owner, .. }) = below.store.pop() else { panic!("make store") };
    let (_, below) = made(&mut store, false, owner);
    if !transcripts {
        return (store, below);
    }
    let mut below = parent_synced(&mut store, below, 0);
    let Some(file::Request::OpenDirectory { owner, .. }) = below.store.pop() else { panic!("open store") };
    let (_, mut below) = opened(&mut store, 0, owner);
    let Some(file::Request::MakeDirectory { owner, .. }) = below.transcripts.pop() else { panic!("make transcripts") };
    let arrived = Arrived { event: file::Event::Made { owner }, fd: None };
    let (_, below) = fixtures::answer(&mut store, Io::Transcripts(arrived));
    (store, below)
}

#[test]
fn a_made_directory_opens_only_after_its_parent_syncs() {
    for (label, transcripts, private, fails) in [
        ("store parent synced", false, false, false),
        ("transcripts parent synced", true, false, false),
        ("store parent sync refused", false, false, true),
        ("transcripts parent sync refused", true, false, true),
        ("private secrets parent synced", false, true, false),
        ("private secrets parent sync refused", false, true, true),
    ] {
        check_parent_sync(label, transcripts, private, fails);
    }
}

fn check_parent_sync(label: &str, transcripts: bool, private: bool, fails: bool) {
    let (mut store, mut below) = if private { at_private_parent_sync() } else { at_parent_sync(transcripts) };
    let io = if private { 2 } else { u8::from(transcripts) };
    let owner = parent_request(&mut below, io, label);
    let event = if fails {
        file::Event::Failed { owner, error: Error::Permission, committed: false, residue: None }
    } else {
        file::Event::Synced { owner }
    };
    let arrived = Arrived { event, fd: None };
    let (events, mut below) = fixtures::answer(&mut store, driver(io, arrived));
    if fails {
        let [Event::OpenRefused { .. }] = &*events else { panic!("{label}: open refused") };
        assert_eq!(store.phase, Phase::Stopped, "{label}");
        assert!(
            below.store.is_empty() && below.transcripts.is_empty() && below.secrets.is_empty(),
            "{label}: no child opens"
        );
    } else if private {
        assert!(events.is_empty(), "{label}");
        let Some(file::Request::List { root, .. }) = below.secrets.pop() else {
            panic!("{label}: secrets recovery after sync")
        };
        assert_eq!(root, PRIVATE_FD, "{label}: private root retained");
    } else {
        assert!(events.is_empty(), "{label}");
        let request = if transcripts { below.transcripts.pop() } else { below.store.pop() };
        let Some(file::Request::OpenDirectory { root, name, no_follow: true, .. }) = request else {
            panic!("{label}: child opens after sync")
        };
        assert_eq!(root, fixtures::ROOT.fd, "{label}");
        assert_eq!(name.as_ref(), if transcripts { b"transcripts".as_slice() } else { b"store".as_slice() }, "{label}");
    }
}

const PRIVATE_FD: Fd = Fd::new(7);
fn at_private_parent_sync() -> (Store, Below) {
    let (mut store, mut below) = at_private();
    let Some(file::Request::OpenPrivate { owner, .. }) = below.secrets.pop() else { panic!("private open") };
    let arrived = Arrived { event: file::Event::Opened { owner, file: Token::new(300), len: 0 }, fd: Some(PRIVATE_FD) };
    let (events, below) = fixtures::answer(&mut store, Io::Secrets(arrived));
    assert!(events.is_empty());
    (store, below)
}
