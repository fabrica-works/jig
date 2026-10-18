//! Each owner keeps its own bytes under the shell's small-store prefix.
use jig_client_test_domain::{
    first,
    store::{Key, Record},
};
use jig_client_view::{records as wire, store as ui};
type Bytes = (Box<[u8]>, Box<[u8]>);
const FIRST: &[u8] = b"first.place";
pub fn encode(record: Record) -> Bytes {
    match record {
        Record::Jig(record) => wire::encode(&match record {
            ui::Record::Place { version, party, address } => wire::Record::Place { version, party, address },
            ui::Record::Draft { version, party, task, words, touched } => {
                wire::Record::Draft { version, party, task, words, touched }
            }
        }),
        Record::Client(record) => (record.key, record.bytes),
        Record::First(first::Record::Place { version, party, address }) => {
            let mut bytes = Vec::new();
            bytes.extend(version.to_le_bytes());
            bytes.extend(party.to_le_bytes());
            bytes.extend(address.project.to_le_bytes());
            bytes.extend(address.item.to_le_bytes());
            (FIRST.into(), bytes.into())
        }
        Record::Unlinked(_) => unreachable!("browser client owns a byte link"),
    }
}
pub fn decode(record: skein_ui_web::store::Record) -> Option<Record> {
    if record.key.len() == 16 {
        return Some(Record::Client(jig_client::Record { key: record.key, bytes: record.value }));
    }
    if record.key.as_ref() == FIRST {
        let bytes: [u8; 22] = record.value.as_ref().try_into().ok()?;
        return Some(Record::First(first::Record::Place {
            version: u16::from_le_bytes(bytes[0..2].try_into().ok()?),
            party: u64::from_le_bytes(bytes[2..10].try_into().ok()?),
            address: first::Address {
                project: u32::from_le_bytes(bytes[10..14].try_into().ok()?),
                item: u64::from_le_bytes(bytes[14..22].try_into().ok()?),
            },
        }));
    }
    Some(Record::Jig(match wire::decode(&record.key, &record.value)? {
        wire::Record::Place { version, party, address } => ui::Record::Place { version, party, address },
        wire::Record::Draft { version, party, task, words, touched } => {
            ui::Record::Draft { version, party, task, words, touched }
        }
    }))
}
pub fn key(key: Key) -> Box<[u8]> {
    match key {
        Key::Jig(ui::StoreKey::Place) => wire::RecordKey::Place.encode(),
        Key::Jig(ui::StoreKey::Draft { task }) => wire::RecordKey::Draft { task }.encode(),
        Key::First(_) => FIRST.into(),
        Key::Client(key) => key,
        Key::Unlinked(_) => unreachable!("browser client owns byte keys"),
    }
}
