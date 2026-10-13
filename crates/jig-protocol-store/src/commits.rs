//! Root commits, numbered with skein-kv's (protocol/store.md, 6 and 11.2).
use crate::{Event, Phase, Store, Write, codec, format, keys};
use alloc::boxed::Box;
use jig_domain_core::{CoreRecord, Record};
use skein_lib::{List, Queue, Token};

pub(crate) fn commit(store: &mut Store, number: u64, writes: Box<[Write]>, above: &mut Queue<Event>) {
    let format_due = match store.phase {
        Phase::Ready { next, format_due } => {
            if number != next || store.stores.store.last().checked_add(1) != Some(number) {
                failed(store, number, above);
                return;
            }
            format_due
        }
        Phase::Closed => {
            failed(store, number, above);
            return;
        }
        Phase::Stopped => return,
    };
    let Some(ops) = operations(store, writes, format_due) else {
        failed(store, number, above);
        return;
    };
    let mut events = Queue::with_capacity(skein_kv::MAX_OUT.above);
    let mut files = Queue::with_capacity(skein_kv::MAX_OUT.below);
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::Commit { owner: Token::new(number), ops },
        &mut events,
        &mut files,
    );
    assert!(files.is_empty(), "memory commits issue no file operation");
    match events.pop().expect("a memory commit answers in its step") {
        skein_kv::Event::Committed { owner: _, number: committed } => {
            if committed != number {
                failed(store, number, above);
                return;
            }
            // At the integer's end, last() has no successor and the submission check rejects any repeat.
            let next = number.checked_add(1).unwrap_or(number);
            store.phase = Phase::Ready { next, format_due: false };
            above.push(Event::Committed { number, fill: store.stores.store.used() });
        }
        skein_kv::Event::Refused { .. }
        | skein_kv::Event::Failed { .. }
        | skein_kv::Event::Opened { .. }
        | skein_kv::Event::Got { .. }
        | skein_kv::Event::Loaded { .. }
        | skein_kv::Event::Closed { .. }
        | skein_kv::Event::OpenFailed { .. }
        | skein_kv::Event::Recovered { .. }
        | skein_kv::Event::RecoveryFailed { .. } => failed(store, number, above),
    }
}

pub(crate) fn operations(store: &Store, writes: Box<[Write]>, format_due: bool) -> Option<Box<[skein_kv::Op]>> {
    if writes.len() > usize::try_from(store.limits.journal.writes).ok()? {
        return None;
    }
    let mut ops = List::with_capacity(store.limits.journal.writes.checked_add(1)?);
    let mut bytes = 26;
    let mut deployment = None;
    for write in writes {
        if let Some(id) = header_id(&write) {
            deployment = Some(id);
        }
        let op = operation(store, write)?;
        append(&mut ops, op, &mut bytes, store.limits.store.commit)?;
    }
    if format_due {
        let id = deployment.expect("the first root commit saves its deployment header");
        let op = skein_kv::Op::Put {
            key: keys::encode_own(&keys::Own::Format, store.limits.store.key)?,
            value: format::current(store.application.version, id).encode().ok()?,
        };
        append(&mut ops, op, &mut bytes, store.limits.store.commit)?;
    }
    Some(ops.into_boxed())
}

fn operation(store: &Store, write: Write) -> Option<skein_kv::Op> {
    let limit = store.limits.store.key;
    match write {
        Write::Save(record) => Some(skein_kv::Op::Put {
            key: keys::encode(&record.key(), limit)?,
            value: codec::encode(&record, &store.codec)?,
        }),
        Write::Erase(key) => Some(skein_kv::Op::Erase { key: keys::encode(&key, limit)? }),
        Write::Put { key, value } => Some(skein_kv::Op::Put { key: keys::application(&key, limit)?, value }),
        Write::Remove { key } => Some(skein_kv::Op::Erase { key: keys::application(&key, limit)? }),
        // Jig 17 replaces this refusal with removal of the whole transcript.
        Write::DropTranscript { task: _ } => None,
    }
}

fn header_id(write: &Write) -> Option<[u8; 16]> {
    match write {
        Write::Save(Record::Core(CoreRecord::Deployment(header))) => Some(header.id),
        Write::Save(
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
            | Record::Notes(_),
        )
        | Write::Erase(_)
        | Write::Put { .. }
        | Write::Remove { .. }
        | Write::DropTranscript { .. } => None,
    }
}

fn append(ops: &mut List<skein_kv::Op>, op: skein_kv::Op, bytes: &mut u64, limit: u32) -> Option<()> {
    let key = u64::try_from(op.key().len()).ok()?;
    let size = match &op {
        skein_kv::Op::Put { key: _, value } => key.checked_add(u64::try_from(value.len()).ok()?)?.checked_add(9)?,
        skein_kv::Op::Erase { key: _ } => key.checked_add(5)?,
    };
    let total = bytes.checked_add(size)?;
    if total > u64::from(limit) {
        return None;
    }
    ops.push(op).ok()?;
    *bytes = total;
    Some(())
}

fn failed(store: &mut Store, number: u64, above: &mut Queue<Event>) {
    store.phase = Phase::Stopped;
    above.push(Event::Failed { number });
}
