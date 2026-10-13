//! Deployment directories and the two independent stores (store.md, 2, 6 and 10).
use crate::{Arrived, Below, Event, Io, Limits, OpenRefusal, Phase, Root, Store, open, secrets};
use alloc::boxed::Box;
use skein_io::file;
use skein_lib::{Env, Queue, Token};

pub(crate) const OWNER: Token = Token::new(0x004a_4947);
#[derive(Debug)]
pub(crate) enum Opening {
    Idle,
    StoreMade,
    StoreOpen,
    TranscriptsMade,
    TranscriptsOpen,
    SecretsOpen,
    Stores,
}
#[derive(Debug)]
pub(crate) struct Directory {
    pub disk: bool,
    pub transcripts: Option<Root>,
    pub root: Option<Root>,
    pub store_root: Option<Root>,
    pub owner: Token,
    pub opening: Opening,
    pub opened: u8,
}
impl Directory {
    pub fn new() -> Directory {
        Directory {
            disk: false,
            transcripts: None,
            root: None,
            store_root: None,
            owner: OWNER,
            opening: Opening::Idle,
            opened: 0,
        }
    }
}
pub(crate) fn env(store: &Store, env: &Env<Limits>, secret: bool) -> Env<skein_kv::Limits> {
    Env { now: env.now, wall: env.wall, limits: if secret { store.limits.secrets } else { store.limits.store } }
}
pub(crate) fn open(
    store: &mut Store,
    _env: &Env<Limits>,
    owner: Token,
    root: Root,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    if !store.directory.disk {
        store.directory.transcripts = Some(root);
        open::open(store, owner, above, below);
        return;
    }
    store.directory.owner = owner;
    if store.phase != Phase::Closed || store.directory.root.is_some() {
        refuse(store, above);
        return;
    }
    store.directory.root = Some(root);
    store.directory.opening = Opening::StoreMade;
    below.store.push(file::Request::MakeDirectory {
        owner: OWNER,
        root: root.token,
        path: Box::from(b"store".as_slice()),
        mode: 0o700,
    });
}
fn refuse(store: &mut Store, above: &mut Queue<Event>) {
    store.phase = Phase::Stopped;
    above.push(Event::OpenRefused { owner: store.directory.owner, why: OpenRefusal::Failed });
}
fn made(event: file::Event) -> bool {
    match event {
        file::Event::Made { .. } => true,
        file::Event::Failed { error, .. } => error == skein_io::kernel::Error::Exists,
        file::Event::Opened { .. }
        | file::Event::Refused { .. }
        | file::Event::Loaded { .. }
        | file::Event::Scanned { .. }
        | file::Event::Stored { .. }
        | file::Event::Conflict { .. }
        | file::Event::TooLarge { .. }
        | file::Event::Stated { .. }
        | file::Event::Written { .. }
        | file::Event::Read { .. }
        | file::Event::Cut { .. }
        | file::Event::Synced { .. }
        | file::Event::Closed { .. }
        | file::Event::Renamed { .. }
        | file::Event::Removed { .. }
        | file::Event::Listed { .. }
        | file::Event::Cancelled { .. } => false,
    }
}
fn opened(arrived: Arrived) -> Option<Root> {
    match arrived.event {
        file::Event::Opened { file, .. } => Some(Root { token: file, fd: arrived.fd? }),
        file::Event::Made { .. }
        | file::Event::Failed { .. }
        | file::Event::Refused { .. }
        | file::Event::Loaded { .. }
        | file::Event::Scanned { .. }
        | file::Event::Stored { .. }
        | file::Event::Conflict { .. }
        | file::Event::TooLarge { .. }
        | file::Event::Stated { .. }
        | file::Event::Written { .. }
        | file::Event::Read { .. }
        | file::Event::Cut { .. }
        | file::Event::Synced { .. }
        | file::Event::Closed { .. }
        | file::Event::Renamed { .. }
        | file::Event::Removed { .. }
        | file::Event::Listed { .. }
        | file::Event::Cancelled { .. } => None,
    }
}
fn directory(store: &mut Store, env: &Env<Limits>, arrived: Arrived, above: &mut Queue<Event>, below: &mut Below) {
    let root = store.directory.root.expect("opening keeps the deployment root");
    match core::mem::replace(&mut store.directory.opening, Opening::Idle) {
        Opening::StoreMade => {
            if !made(arrived.event) {
                refuse(store, above);
                return;
            }
            store.directory.opening = Opening::StoreOpen;
            below.store.push(file::Request::OpenDirectory {
                owner: OWNER,
                root: root.fd,
                name: Box::from(b"store".as_slice()),
                no_follow: true,
            });
        }
        Opening::StoreOpen => {
            let Some(opened) = opened(arrived) else {
                refuse(store, above);
                return;
            };
            store.directory.store_root = Some(opened);
            store.directory.opening = Opening::TranscriptsMade;
            below.transcripts.push(file::Request::MakeDirectory {
                owner: OWNER,
                root: root.token,
                path: Box::from(b"transcripts".as_slice()),
                mode: 0o700,
            });
        }
        Opening::TranscriptsMade => {
            if !made(arrived.event) {
                refuse(store, above);
                return;
            }
            store.directory.opening = Opening::TranscriptsOpen;
            below.transcripts.push(file::Request::OpenDirectory {
                owner: OWNER,
                root: root.fd,
                name: Box::from(b"transcripts".as_slice()),
                no_follow: true,
            });
        }
        Opening::TranscriptsOpen => {
            let Some(opened) = opened(arrived) else {
                refuse(store, above);
                return;
            };
            store.directory.transcripts = Some(opened);
            store.directory.opening = Opening::SecretsOpen;
            below.secrets.push(file::Request::OpenPrivate {
                owner: OWNER,
                root: root.token,
                path: Box::from(b"secrets".as_slice()),
            });
        }
        Opening::SecretsOpen => {
            let Some(opened) = opened(arrived) else {
                refuse(store, above);
                return;
            };
            store.directory.opening = Opening::Stores;
            let kv_env = self::env(store, env, false);
            let mut events = Queue::with_capacity(skein_kv::MAX_OUT.above);
            let root = store.directory.store_root.expect("the store directory is open");
            skein_kv::down(
                &mut store.stores.store,
                &kv_env,
                skein_kv::Request::Open { owner: OWNER, root: root.fd },
                &mut events,
                &mut below.store,
            );
            events_up(store, false, &mut events, above, below);
            let kv_env = self::env(store, env, true);
            let mut events = Queue::with_capacity(skein_kv::MAX_OUT.above);
            skein_kv::down(
                &mut store.stores.secrets,
                &kv_env,
                skein_kv::Request::Open { owner: OWNER, root: opened.fd },
                &mut events,
                &mut below.secrets,
            );
            events_up(store, true, &mut events, above, below);
        }
        Opening::Idle | Opening::Stores => refuse(store, above),
    }
}
pub(crate) fn up(store: &mut Store, env: &Env<Limits>, io: Io, above: &mut Queue<Event>, below: &mut Below) {
    if store.phase == Phase::Stopped {
        return;
    }
    let (arrived, secret) = match io {
        Io::Store(arrived) => (arrived, false),
        Io::Secrets(arrived) => (arrived, true),
        Io::Transcripts(arrived) => {
            match store.phase {
                Phase::Ready { .. } => {
                    if arrived.event.owner() == crate::transcripts::cut::OWNER {
                        crate::transcripts::cut::arrived(store, arrived, above, below);
                    } else if arrived.event.owner() == crate::transcripts::drop::OWNER {
                        crate::transcripts::drop::arrived(store, env, arrived, above, below);
                    } else if arrived.event.owner() == crate::transcripts::read::OWNER {
                        crate::transcripts::read::arrived(store, arrived, above, below);
                    } else {
                        crate::transcripts::write::arrived(store, env, arrived, above, below);
                    }
                }
                Phase::Closed => directory(store, env, arrived, above, below),
                Phase::Stopped => {}
            }
            return;
        }
    };
    if arrived.event.owner() == OWNER {
        directory(store, env, arrived, above, below);
        return;
    }
    let kv_env = self::env(store, env, secret);
    let mut events = Queue::with_capacity(skein_kv::MAX_OUT.above);
    if secret {
        skein_kv::up(&mut store.stores.secrets, &kv_env, arrived.event, &mut events, &mut below.secrets);
    } else {
        skein_kv::up(&mut store.stores.store, &kv_env, arrived.event, &mut events, &mut below.store);
    }
    events_up(store, secret, &mut events, above, below);
}
pub(crate) fn events_up(
    store: &mut Store,
    secret: bool,
    events: &mut Queue<skein_kv::Event>,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    for _ in 0..skein_kv::MAX_OUT.above {
        let Some(event) = events.pop() else {
            break;
        };
        match event {
            skein_kv::Event::Opened { .. } => {
                store.directory.opened |= if secret { 2 } else { 1 };
                if store.directory.opened == 3 {
                    open::open(store, store.directory.owner, above, below);
                }
            }
            skein_kv::Event::Committed { owner, number } => {
                if secret {
                    secrets::committed(store, owner, above);
                } else if owner == Token::new(number) {
                    crate::transcripts::committed(store, number);
                    above.push(Event::Committed { number, fill: store.stores.store.used() });
                } else {
                    store.phase = Phase::Stopped;
                    above.push(Event::Failed { number });
                }
            }
            skein_kv::Event::Refused { owner, .. } => {
                store.phase = Phase::Stopped;
                if !secret {
                    crate::transcripts::failed(store, Some(owner.raw()), above);
                }
            }
            skein_kv::Event::Failed { number, .. } => {
                store.phase = Phase::Stopped;
                if !secret {
                    crate::transcripts::failed(store, Some(number), above);
                }
            }
            skein_kv::Event::OpenFailed { .. } => refuse(store, above),
            skein_kv::Event::Recovered { .. } | skein_kv::Event::RecoveryFailed { .. } => store.phase = Phase::Stopped,
            skein_kv::Event::Got { .. } | skein_kv::Event::Loaded { .. } | skein_kv::Event::Closed { .. } => {
                store.phase = Phase::Stopped;
            }
        }
    }
}
pub(crate) fn fire(store: &mut Store, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Below) {
    for secret in [false, true] {
        let kv_env = self::env(store, env, secret);
        let mut events = Queue::with_capacity(skein_kv::MAX_OUT.above);
        if secret {
            if skein_kv::is_ready(&store.stores.secrets, &kv_env.limits) {
                skein_kv::fire(&mut store.stores.secrets, &kv_env, &mut events, &mut below.secrets);
            }
        } else if skein_kv::is_ready(&store.stores.store, &kv_env.limits) {
            skein_kv::fire(&mut store.stores.store, &kv_env, &mut events, &mut below.store);
        }
        events_up(store, secret, &mut events, above, below);
    }
}
