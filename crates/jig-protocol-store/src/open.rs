//! The format row and header checked before restart (protocol/store.md, 5.3 and 10).
use crate::{Event, Family, OpenRefusal, Phase, Store, codec, format, keys};
use alloc::boxed::Box;
use jig_domain_core::{CoreKey, CoreRecord, Key, Record};
use skein_lib::{Queue, Token};

pub(crate) fn open(store: &mut Store, owner: Token, above: &mut Queue<Event>, below: &mut crate::Below) {
    if store.phase != Phase::Closed {
        refuse(store, owner, OpenRefusal::Failed, above);
        return;
    }
    let format_key = keys::encode_own(&keys::Own::Format, store.limits.store.key).expect("format key fits");
    let header_key = keys::encode(&Key::Core(CoreKey::Deployment), store.limits.store.key).expect("header key fits");
    let Ok(found) = get(store, owner, format_key) else {
        refuse(store, owner, OpenRefusal::Failed, above);
        return;
    };
    let Ok(header) = get(store, owner, header_key) else {
        refuse(store, owner, OpenRefusal::Failed, above);
        return;
    };
    let last = store.stores.store.last();
    let Some(found) = found else {
        if header.is_some() {
            refuse(
                store,
                owner,
                OpenRefusal::Format {
                    family: Family::Component,
                    found: 0,
                    wanted: u32::from(format::current(store.application.version, [0; 16]).component),
                },
                above,
            );
        } else if last == 0 {
            ready(store, owner, last, true, above, below);
        } else {
            refuse(store, owner, OpenRefusal::Numbers { store: last, header: 0 }, above);
        }
        return;
    };
    let Ok(found) = codec::component::Format::decode(&found) else {
        corrupt(store, owner, above);
        return;
    };
    // Refuse another family's version before decoding its header.
    let wanted = format::current(store.application.version, found.deployment);
    if let Err(why) = format::check(&found, &wanted) {
        refuse(store, owner, why, above);
        return;
    }
    let Some(header) = header else {
        refuse(store, owner, OpenRefusal::Numbers { store: last, header: 0 }, above);
        return;
    };
    let Some(header) = codec::decode(&Key::Core(CoreKey::Deployment), &header, &store.codec) else {
        corrupt(store, owner, above);
        return;
    };
    match header {
        Record::Core(CoreRecord::Deployment(header)) => {
            let wanted = format::current(store.application.version, header.id);
            if let Err(why) = format::check(&found, &wanted) {
                refuse(store, owner, why, above);
            } else if header.commits != last {
                refuse(store, owner, OpenRefusal::Numbers { store: last, header: header.commits }, above);
            } else {
                ready(store, owner, last, false, above, below);
            }
        }
        Record::Core(
            CoreRecord::Projection(_)
            | CoreRecord::Call(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Turn(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::ProposalDecision(_)
            | CoreRecord::EscalationDecision(_),
        )
        | Record::Tasks(_)
        | Record::People(_)
        | Record::Notes(_) => corrupt(store, owner, above),
    }
}

pub(crate) fn get(store: &mut Store, owner: Token, key: Box<[u8]>) -> Result<Option<Box<[u8]>>, OpenRefusal> {
    let mut above = Queue::with_capacity(skein_kv::MAX_OUT.above);
    let mut below = Queue::with_capacity(skein_kv::MAX_OUT.below);
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::Get { owner, key },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty(), "a memory get issues no file operation");
    match above.pop().ok_or(OpenRefusal::Failed)? {
        skein_kv::Event::Got { owner: _, value } => Ok(value),
        skein_kv::Event::Opened { .. }
        | skein_kv::Event::Committed { .. }
        | skein_kv::Event::Refused { .. }
        | skein_kv::Event::Failed { .. }
        | skein_kv::Event::Loaded { .. }
        | skein_kv::Event::Closed { .. }
        | skein_kv::Event::OpenFailed { .. }
        | skein_kv::Event::Recovered { .. }
        | skein_kv::Event::RecoveryFailed { .. } => Err(OpenRefusal::Failed),
    }
}

fn ready(
    store: &mut Store,
    owner: Token,
    last: u64,
    format_due: bool,
    above: &mut Queue<Event>,
    below: &mut crate::Below,
) {
    let Some(next) = last.checked_add(1) else {
        refuse(store, owner, OpenRefusal::Failed, above);
        return;
    };
    store.phase = Phase::Ready { next, format_due };
    store.directory.owner = owner;
    crate::transcripts::cut::begin(store, above, below);
}

pub(crate) fn refuse(store: &mut Store, owner: Token, why: OpenRefusal, above: &mut Queue<Event>) {
    store.phase = Phase::Stopped;
    above.push(Event::OpenRefused { owner, why });
}

fn corrupt(store: &mut Store, owner: Token, above: &mut Queue<Event>) {
    store.phase = Phase::Stopped;
    above.push(Event::Corrupt { owner });
}
