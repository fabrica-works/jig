//! Ordered keys and owner tags (protocol/store.md, sections 4.1 and 4.2).
use alloc::boxed::Box;
use jig_domain_core as domain;
use skein_kv::{KeyReader, KeyWriter};
mod accounts;
mod core;
mod notes;
mod own;
mod people;
mod tasks;
pub use own::Own;
pub const COMPONENT: u8 = 0;
pub const JIG: u8 = 1;
pub const APPLICATION: u8 = 2;
/// The accounts child's tag under jig's, after the core's own and the tasks',
/// people's and notes' children (4.2).
pub const ACCOUNTS: u8 = 4;
/// The installation's identifier's key (4.4): the accounts' one row without a
/// key of its own, after the tags of the domain's accounts keys (the counter,
/// an account, a source: 0 to 2).
#[must_use]
pub fn installation(limit: u32) -> Option<Box<[u8]>> {
    let mut writer = KeyWriter::new(limit);
    writer.tag(JIG);
    writer.tag(ACCOUNTS);
    writer.tag(3);
    writer.finish()
}
#[must_use]
pub fn encode_own(key: &Own, limit: u32) -> Option<Box<[u8]>> {
    let mut writer = KeyWriter::new(limit);
    writer.tag(COMPONENT);
    own::write(&mut writer, key);
    writer.finish()
}
#[must_use]
pub fn decode_own(bytes: &[u8]) -> Option<Own> {
    let mut reader = KeyReader::new(bytes);
    if reader.tag()? != COMPONENT {
        return None;
    }
    let key = own::read(&mut reader)?;
    if !reader.is_empty() {
        return None;
    }
    Some(key)
}
#[must_use]
pub fn encode(key: &domain::Key, limit: u32) -> Option<Box<[u8]>> {
    let mut writer = KeyWriter::new(limit);
    writer.tag(JIG);
    match key {
        domain::Key::Core(key) => {
            writer.tag(0);
            core::write(&mut writer, key);
        }
        domain::Key::People(key) => {
            writer.tag(2);
            people::write(&mut writer, key);
        }
        domain::Key::Tasks(key) => {
            writer.tag(1);
            tasks::write(&mut writer, key);
        }
        domain::Key::Notes(key) => {
            writer.tag(3);
            notes::write(&mut writer, key);
        }
        domain::Key::Accounts(key) => {
            writer.tag(ACCOUNTS);
            accounts::write(&mut writer, key);
        }
    }
    writer.finish()
}
#[must_use]
pub fn decode(bytes: &[u8]) -> Option<domain::Key> {
    let mut reader = KeyReader::new(bytes);
    if reader.tag()? != JIG {
        return None;
    }
    let key = match reader.tag()? {
        0 => domain::Key::Core(core::read(&mut reader)?),
        1 => domain::Key::Tasks(tasks::read(&mut reader)?),
        2 => domain::Key::People(people::read(&mut reader)?),
        3 => domain::Key::Notes(notes::read(&mut reader, bytes.len())?),
        ACCOUNTS => domain::Key::Accounts(accounts::read(&mut reader)?),
        _ => return None,
    };
    if !reader.is_empty() {
        return None;
    }
    Some(key)
}
#[must_use]
pub fn application(key: &[u8], limit: u32) -> Option<Box<[u8]>> {
    let mut writer = KeyWriter::new(limit);
    writer.tag(APPLICATION);
    for &byte in key {
        writer.tag(byte);
    }
    writer.finish()
}
#[must_use]
pub fn of_application(bytes: &[u8]) -> Option<&[u8]> {
    let (&tag, key) = bytes.split_first()?;
    if tag != APPLICATION {
        return None;
    }
    Some(key)
}
