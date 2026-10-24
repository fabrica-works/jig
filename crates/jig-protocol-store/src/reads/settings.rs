//! A bounded forward settings interval plus the newest's held point.
use super::Failure;
use crate::{Row, Store, intervals, pages};
use alloc::boxed::Box;
use jig_domain_core::reads;
use skein_lib::List;
pub(super) fn read(store: &mut Store, range: &reads::Range) -> Result<reads::Loaded, Failure> {
    if range.rows > store.limits.core.documents.rows.max(1)
        || range.bytes != reads::settings::load_bytes(&store.limits.core, range.rows).ok_or(Failure::Corrupt)?
    {
        return Ok(reads::Loaded::Busy);
    }
    let intervals = intervals::scope(range.party, &range.scope).ok_or(Failure::Corrupt)?;
    let mut found = List::with_capacity(range.rows);
    let mut after = None;
    // The selected dense interval has at most range.rows keys. Walk it forward
    // one physical page at a time, so the store's encoded-page cap cannot cut
    // off the newest end of this interval. The core reverses the result.
    for _ in 0..range.rows {
        let walked = pages::walk(store, &intervals, after.as_ref(), 1).ok_or(Failure::Corrupt)?;
        for item in walked.rows {
            let (value, _) = pages::decode(store, item.row).ok_or(Failure::Corrupt)?;
            let value = match value {
                Row::Jig(value) => value,
                Row::Application { .. } => return Err(Failure::Corrupt),
            };
            if !reads::admits(range.party, &range.scope, &value.key()) {
                return Err(Failure::Corrupt);
            }
            let Some(revision) = reads::settings::record_revision(&value) else { return Err(Failure::Corrupt) };
            if !reads::settings::valid_revision(revision, &store.limits.core) {
                return Err(Failure::Corrupt);
            }
            found
                .push(reads::Row {
                    value,
                    owner: None,
                    ancestors: Box::new([]),
                    related: None,
                    unjoined: None,
                    ended_read: None,
                    linked: Box::new([]),
                    inbox: None,
                })
                .expect("bounded revisions");
        }
        after = walked.next;
        if after.is_none() {
            break;
        }
    }
    Ok(reads::Loaded::Page { rows: found.into_boxed(), next: None })
}
