use super::fixtures;
use crate::records::{self, Record, RecordKey};
fn round_trip(record: Record) {
    let (key, bytes) = records::encode(&record);
    assert_eq!(records::decode(&key, &bytes), Some(record));
}
#[test]
fn a_place_record_round_trips_through_its_bytes() {
    round_trip(fixtures::place());
}
#[test]
fn a_draft_record_round_trips_through_its_bytes() {
    round_trip(fixtures::draft());
}
#[test]
fn a_draft_round_trips_through_the_small_store() {
    round_trip(fixtures::draft());
}
#[test]
fn a_record_of_another_version_is_dropped() {
    let record =
        Record::Place { version: u16::MAX, party: fixtures::PARTY, address: crate::address::Address::Deployment };
    let (key, bytes) = records::encode(&record);
    assert_eq!(records::decode(&key, &bytes), None);
}
#[test]
fn a_malformed_record_is_dropped() {
    let (key, bytes) = records::encode(&fixtures::draft());
    for length in 0..bytes.len() {
        assert_eq!(records::decode(&key, &bytes[..length]), None, "cut at {length}");
    }
    assert_eq!(records::decode(&RecordKey::Place.encode(), &bytes), None);
    let mut trailing = skein_lib::Writer::new(bytes.len() + 1);
    trailing.put(&bytes).unwrap();
    trailing.put(&[0]).unwrap();
    assert_eq!(records::decode(&key, &trailing.finish()), None);
}
#[test]
fn a_draft_at_its_ceiling_round_trips_unchanged() {
    let record = Record::Draft {
        version: records::RECORDS_VERSION,
        party: fixtures::PARTY,
        task: fixtures::TASK,
        words: jig_client::contract::Text::new(skein_lib::bytes::zeroed(65536), 65536).unwrap(),
        touched: u64::MAX,
    };
    round_trip(record);
}
#[test]
fn a_place_round_trips_every_jig_address() {
    for record in fixtures::places().into_boxed() {
        round_trip(record);
    }
}
