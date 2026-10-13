//! Root commits, numbered with skein-kv's (protocol/store.md, 6 and 11.2).
use crate::{Event, Store, Write, codec, format, keys};
use alloc::boxed::Box;
use jig_domain_core::{CoreRecord, Record};
use skein_lib::{List, Queue, Token};

pub(crate) fn submit(
    store: &mut Store,
    env: &skein_lib::Env<crate::Limits>,
    number: u64,
    ops: Box<[skein_kv::Op]>,
    above: &mut Queue<Event>,
    below: &mut crate::Below,
) {
    let kv_env = crate::directory::env(store, env, false);
    let mut events = Queue::with_capacity(skein_kv::MAX_OUT.above);
    skein_kv::down(
        &mut store.stores.store,
        &kv_env,
        skein_kv::Request::Commit { owner: Token::new(number), ops },
        &mut events,
        &mut below.store,
    );
    crate::directory::events_up(store, false, &mut events, above);
}
pub(crate) fn operations(
    store: &mut Store,
    number: u64,
    writes: Box<[Write]>,
    format_due: bool,
) -> Option<crate::transcripts::Pending> {
    if writes.len() > usize::try_from(store.limits.journal.writes).ok()? {
        return None;
    }
    let mut ops = List::with_capacity(store.limits.journal.writes.checked_mul(2)?.checked_add(1)?);
    let mut frames = Queue::with_capacity(store.limits.journal.writes);
    let mut bytes = 26;
    let mut deployment = None;
    for write in writes {
        match write {
            Write::Save(Record::Core(CoreRecord::Turn(turn))) => {
                let body = crate::transcripts::frame::encode(&turn, &store.codec)?;
                let offset =
                    crate::transcripts::place(store, number, &turn, u64::try_from(body.len()).ok()?, &mut ops)?;
                frames.push(crate::transcripts::Frame { task: turn.task, offset, bytes: body });
            }
            write @ (Write::Save(
                Record::Core(
                    CoreRecord::Deployment(_)
                    | CoreRecord::Projection(_)
                    | CoreRecord::Call(_)
                    | CoreRecord::RunProof(_)
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
            | Write::DropTranscript { .. }) => {
                if let Some(id) = header_id(&write) {
                    deployment = Some(id);
                }
                ops.push(operation(store, write)?).ok()?;
            }
        }
    }
    if format_due {
        let id = deployment.expect("the first root commit saves its deployment header");
        ops.push(skein_kv::Op::Put {
            key: keys::encode_own(&keys::Own::Format, store.limits.store.key)?,
            value: format::current(store.application.version, id).encode().ok()?,
        })
        .ok()?;
    }
    let mut bounded = List::with_capacity(store.limits.journal.writes.checked_mul(2)?.checked_add(1)?);
    for op in ops.into_boxed() {
        append(&mut bounded, op, &mut bytes, store.limits.store.commit)?;
    }
    Some(crate::transcripts::Pending { number, frames, ops: bounded.into_boxed() })
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
        // Whole transcript removal joins in jig 17.4.
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
