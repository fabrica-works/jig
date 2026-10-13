//! Restart pages, including the inbox census joins (protocol/store.md, 7.2).
use crate::pages::Decoded as Restored;
use crate::{Cursor, Event, Phase, Restore, Row, Store, codec, intervals, keys, pages};
use alloc::boxed::Box;
use jig_domain_core::{Key, Record};
use jig_domain_tasks as tasks;
use skein_lib::{List, Queue, Token};

pub(crate) fn restore(
    store: &mut Store,
    owner: Token,
    range: Restore,
    after: Option<Cursor>,
    rows: u32,
    bytes: u64,
    above: &mut Queue<Event>,
) {
    let Some(walked) = restore_page(store, range, after.as_ref(), rows, bytes) else {
        store.phase = Phase::Stopped;
        above.push(Event::Corrupt { owner });
        return;
    };
    above.push(Event::Restored { owner, rows: walked.rows, next: walked.next });
}

fn restore_page(store: &mut Store, range: Restore, after: Option<&Cursor>, rows: u32, bytes: u64) -> Option<Restored> {
    match store.phase {
        Phase::Ready { .. } => {}
        Phase::Closed | Phase::Stopped => return None,
    }
    let ranges = match range {
        Restore::Application(ranges) => pages::application(&ranges, store.limits.store.key)?,
        Restore::Jig(ranges) => {
            let mut count = 0_u32;
            for range in &ranges {
                count = count.checked_add(u32::try_from(intervals::restore(range).len()).ok()?)?;
            }
            let mut all = List::with_capacity(count);
            for range in &ranges {
                for interval in intervals::restore(range) {
                    all.push(interval).expect("counted restart intervals");
                }
            }
            all.into_boxed()
        }
    };
    let walked = pages::walk(store, &ranges, after, rows)?;
    let mut found = List::with_capacity(u32::try_from(walked.rows.len()).ok()?);
    let mut used = 0_u64;
    let mut cursor = after.cloned().unwrap_or(Cursor { interval: 0, after: Box::new([]) });
    for item in walked.rows {
        let (row, size) = decode(store, item.row)?;
        let total = used.checked_add(size)?;
        if total > bytes {
            return Some(Restored { rows: found.into_boxed(), next: Some(cursor) });
        }
        used = total;
        cursor = item.cursor;
        found.push(row).expect("bounded by walked rows");
    }
    Some(Restored { rows: found.into_boxed(), next: walked.next })
}

fn decode(store: &mut Store, row: skein_kv::Row) -> Option<(Row, u64)> {
    let (row, size) = pages::decode(store, row)?;
    let task = match &row {
        Row::Jig(Record::Tasks(tasks::Stored::Result(row))) => row.task,
        Row::Jig(Record::Tasks(tasks::Stored::ConversationOldest(row))) => row.task,
        Row::Jig(_) | Row::Application { .. } => return Some((row, size)),
    };
    let key = Key::Tasks(tasks::Key::Ended(task));
    let mut above = Queue::with_capacity(skein_kv::MAX_OUT.above);
    let mut below = Queue::with_capacity(skein_kv::MAX_OUT.below);
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::Get { owner: Token::new(0), key: keys::encode(&key, store.limits.store.key)? },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty(), "memory census joins issue no files");
    let Some(skein_kv::Event::Got { value: Some(value), .. }) = above.pop() else { return None };
    let record = codec::decode(&key, &value, &store.codec)?;
    if record.key() != key {
        return None;
    }
    Some((Row::Jig(record), u64::try_from(value.len()).ok()?))
}
