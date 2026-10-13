//! Single memory records, typed or application bytes (protocol/store.md, 3 and 12).
use crate::{Event, Get, Phase, Store, keys, loads, open, pages};
use skein_lib::{Queue, Token};

pub(crate) fn get(store: &mut Store, owner: Token, key: Get, above: &mut Queue<Event>) {
    let Ok(row) = row(store, owner, key) else {
        loads::corrupt(store, owner, above);
        return;
    };
    above.push(Event::Got { owner, row });
}

pub(crate) fn row(store: &mut Store, owner: Token, key: Get) -> Result<Option<crate::Row>, ()> {
    match store.phase {
        Phase::Ready { .. } => {}
        Phase::Closed | Phase::Stopped => return Err(()),
    }
    let key = match key {
        Get::Jig(key) => keys::encode(&key, store.limits.store.key).ok_or(())?,
        Get::Application(key) => keys::application(&key, store.limits.store.key).ok_or(())?,
    };
    let Ok(value) = open::get(store, owner, key.clone()) else { return Err(()) };
    let Some(value) = value else { return Ok(None) };
    let (row, _bytes) = pages::decode(store, skein_kv::Row { key, value }).ok_or(())?;
    Ok(Some(row))
}
