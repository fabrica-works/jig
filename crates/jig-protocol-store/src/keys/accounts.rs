//! The accounts child's keys (protocol/store.md, 4.4 item 9): the counter's one
//! row, an account by its number, a source by its account's and its own,
//! and a lifetime name reservation by its original source number.
//! Tag 3 is the installation's identifier, which no domain key names.
use jig_domain_accounts::Key;
use skein_kv::{KeyReader, KeyWriter};
pub(super) fn write(writer: &mut KeyWriter, key: &Key) {
    match key {
        Key::Issued { number } => {
            writer.tag(4);
            writer.u64(u64::from(*number));
        }
        Key::Next => {
            writer.tag(0);
        }
        Key::Account { number } => {
            writer.tag(1);
            writer.u64(u64::from(*number));
        }
        Key::Source { account, number } => {
            writer.tag(2);
            writer.u64(u64::from(*account));
            writer.u64(u64::from(*number));
        }
    }
}
pub(super) fn read(reader: &mut KeyReader<'_>) -> Option<Key> {
    Some(match reader.tag()? {
        0 => Key::Next,
        4 => Key::Issued { number: u32::try_from(reader.u64()?).ok()? },
        1 => Key::Account { number: u32::try_from(reader.u64()?).ok()? },
        2 => Key::Source { account: u32::try_from(reader.u64()?).ok()?, number: u32::try_from(reader.u64()?).ok()? },
        _ => return None,
    })
}
