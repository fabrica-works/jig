//! People-owned address fields (protocol/store.md, section 4.1).
use jig_domain_people::{Key, RequestKey};
use skein_kv::{KeyReader, KeyWriter};
pub(super) fn write(writer: &mut KeyWriter, key: &Key) {
    match key {
        Key::Person(number) => {
            writer.tag(0);
            writer.u64(*number);
        }
        Key::ReadResults { party } => {
            writer.tag(1);
            writer.u64(*party);
        }
        Key::ReadChat { party, task } => {
            writer.tag(2);
            writer.u64(*party);
            writer.u64(*task);
        }
        Key::SignIn(number) => {
            writer.tag(3);
            writer.u64(*number);
        }
        Key::Roles(project) => {
            writer.tag(4);
            writer.u64(u64::from(*project));
        }
        Key::Policy(project) => {
            writer.tag(5);
            writer.u64(u64::from(*project));
        }
        Key::Answer(key) => {
            writer.tag(6);
            writer.u64(key.person);
            for &byte in &key.key {
                writer.tag(byte);
            }
        }
    }
}
pub(super) fn read(reader: &mut KeyReader<'_>) -> Option<Key> {
    Some(match reader.tag()? {
        0 => Key::Person(reader.u64()?),
        1 => Key::ReadResults { party: reader.u64()? },
        2 => Key::ReadChat { party: reader.u64()?, task: reader.u64()? },
        3 => Key::SignIn(reader.u64()?),
        4 => Key::Roles(u32::try_from(reader.u64()?).ok()?),
        5 => Key::Policy(u32::try_from(reader.u64()?).ok()?),
        6 => {
            let person = reader.u64()?;
            let mut key = [0; 16];
            for byte in &mut key {
                *byte = reader.tag()?;
            }
            Key::Answer(RequestKey { person, key })
        }
        _ => return None,
    })
}
