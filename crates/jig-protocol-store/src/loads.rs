//! Same-step child pages under both bounds (protocol/store.md, 7.1 and 12).
use crate::{Cursor, Event, Load, Phase, Store, intervals, pages};
use alloc::boxed::Box;
use skein_lib::{List, Queue, Token};

pub(crate) fn load(
    store: &mut Store,
    owner: Token,
    range: Load,
    after: Option<Cursor>,
    rows: u32,
    bytes: u64,
    above: &mut Queue<Event>,
) {
    let Some(page) = page(store, range, after.as_ref(), rows, bytes) else {
        corrupt(store, owner, above);
        return;
    };
    above.push(Event::Loaded { owner, rows: page.rows, next: page.next });
}

fn page(store: &mut Store, range: Load, after: Option<&Cursor>, rows: u32, bytes: u64) -> Option<pages::Decoded> {
    match store.phase {
        Phase::Ready { .. } => {}
        Phase::Closed | Phase::Stopped => return None,
    }
    let ranges = match range {
        Load::Notes(range) => intervals::notes(&range),
        Load::Application(ranges) => pages::application(&ranges, store.limits.store.key)?,
    };
    let walked = pages::walk(store, &ranges, after, rows)?;
    let mut found = List::with_capacity(u32::try_from(walked.rows.len()).ok()?);
    let mut used = 0_u64;
    let mut cursor = after.cloned().unwrap_or(Cursor { interval: 0, after: Box::new([]) });
    for item in walked.rows {
        let (row, size) = pages::decode(store, item.row)?;
        let total = used.checked_add(size)?;
        if total > bytes {
            return Some(pages::Decoded { rows: found.into_boxed(), next: Some(cursor) });
        }
        used = total;
        cursor = item.cursor;
        found.push(row).expect("bounded by walked rows");
    }
    Some(pages::Decoded { rows: found.into_boxed(), next: walked.next })
}

pub(crate) fn corrupt(store: &mut Store, owner: Token, above: &mut Queue<Event>) {
    store.phase = Phase::Stopped;
    above.push(Event::Corrupt { owner });
}
