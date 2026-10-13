//! Single task scopes use exact keys, live before ended (store.md, 7.3).
use super::{Failure, Spent, empty, joins};
use crate::{Get, Row, Store, gets};
use alloc::boxed::Box;
use jig_domain_core::{self as core, reads};
use skein_lib::Token;

pub(super) fn record(store: &mut Store, key: &core::Key) -> Result<Option<core::Record>, Failure> {
    let row = gets::row(store, Token::new(0), Get::Jig(key.clone())).or(Err(Failure::Corrupt))?;
    let Some(row) = row else { return Ok(None) };
    match row {
        Row::Jig(record) => Ok(Some(record)),
        Row::Application { .. } => Err(Failure::Corrupt),
    }
}

pub(super) fn task(
    store: &mut Store,
    range: &reads::Range,
    task: u64,
    spent: &mut Spent,
) -> Result<reads::Loaded, Failure> {
    if spent.rows == 0 {
        return Ok(empty());
    }
    for key in [jig_domain_tasks::Key::Live(task), jig_domain_tasks::Key::Ended(task)] {
        let key = core::Key::Tasks(key);
        let Some(value) = record(store, &key)? else { continue };
        let position = reads::Position::Owner(key);
        if let Some(after) = &range.after
            && position <= *after
        {
            return Ok(empty());
        }
        let Some(owner) = joins::as_task(&value) else { return Err(Failure::Corrupt) };
        let owner = owner.clone();
        let related = core::Key::Core(core::CoreKey::Terminal { task, attempt: owner.attempt });
        let row = joins::row(store, value, Some(&owner), Some(related), spent)?;
        return Ok(reads::Loaded::Page { rows: Box::new([row]), next: None });
    }
    Ok(empty())
}
