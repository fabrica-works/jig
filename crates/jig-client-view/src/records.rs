//! The reference UI's own small-store records (client/domain/view.md, 18.8).
use crate::address::Address;
use alloc::boxed::Box;
use jig_client::contract::Text;
use skein_lib::{Reader, Writer, bytes};
mod address;
use crate::records_wire as w;
/// The version of the reference UI's records.
pub const RECORDS_VERSION: u16 = 1;
/// The reference UI's own record, kept beside the client's records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Place { version: u16, party: u64, address: Address },
    Draft { version: u16, party: u64, task: u64, words: Text, touched: u64 },
}
/// A key in the reference UI's own namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordKey {
    Place,
    Draft { task: u64 },
}
impl RecordKey {
    /// The reference UI's storage key, distinct from the client's 16-byte keys.
    #[must_use]
    pub fn encode(self) -> Box<[u8]> {
        match self {
            Self::Place => Box::new([0]),
            Self::Draft { task } => {
                let mut writer = Writer::new(9);
                writer.put(&[1]).expect("reserved key tag");
                writer.put(&task.to_be_bytes()).expect("reserved task key");
                writer.finish()
            }
        }
    }
}
/// The key and bytes the application keeps.
#[must_use]
pub fn encode(record: &Record) -> (Box<[u8]>, Box<[u8]>) {
    let (version, key, record) = match record {
        Record::Place { version, party, address: at } => (
            *version,
            RecordKey::Place,
            w::JigRecord::Place(
                w::Place::new(
                    &w::CEILINGS,
                    w::PlaceParts {
                        party: *party,
                        address: address::encode(*at, &w::CEILINGS).expect("bounded place"),
                    },
                )
                .expect("bounded place"),
            ),
        ),
        Record::Draft { version, party, task, words, touched } => (
            *version,
            RecordKey::Draft { task: *task },
            w::JigRecord::Draft(
                w::Draft::new(
                    &w::CEILINGS,
                    w::DraftParts {
                        party: *party,
                        task: *task,
                        words: bytes::copy_of(words.bytes()),
                        touched: *touched,
                    },
                )
                .expect("bounded draft"),
            ),
        ),
    };
    let mut payload = Writer::new(usize::try_from(record.measure()).expect("record fits usize"));
    record.encode(&mut payload).expect("measured page record");
    let record = w::Record::new(&w::CEILINGS, w::RecordParts { store_version: version, record: payload.finish() })
        .expect("bounded record");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("record fits usize"));
    record.encode(&mut writer).expect("measured record");
    (key.encode(), writer.finish())
}
/// Drops unreadable records, other versions and records under a different key.
#[must_use]
pub fn decode(key: &[u8], bytes: &[u8]) -> Option<Record> {
    let mut reader = Reader::new(bytes);
    let record = w::Record::decode(&w::CEILINGS, &mut reader).ok()?;
    let version = record.store_version();
    if reader.remaining() != 0 || version != RECORDS_VERSION {
        return None;
    }
    let mut payload = Reader::new(record.record());
    let page = w::JigRecord::decode(&w::CEILINGS, &mut payload).ok()?;
    if payload.remaining() != 0 {
        return None;
    }
    let (record_key, record) = match page {
        w::JigRecord::Place(value) => (
            RecordKey::Place,
            Record::Place { version, party: value.party(), address: address::decode(value.address()) },
        ),
        w::JigRecord::Draft(value) => (
            RecordKey::Draft { task: value.task() },
            Record::Draft {
                version,
                party: value.party(),
                task: value.task(),
                words: Text::new(bytes::copy_of(value.words()), w::CEILINGS.draft_words).ok()?,
                touched: value.touched(),
            },
        ),
    };
    if key != record_key.encode().as_ref() {
        return None;
    }
    Some(record)
}
