use super::fixtures;
use crate::{keys, live};
use jig_test_connector::RecordKey;
use skein_lib::Rng;
#[test]
fn encoded_keys_sort_as_the_derived_order() {
    let mut random = Rng::new(18);
    let mut previous = fixtures::record_key(&mut random);
    for _ in 0_u32..500 {
        let next = fixtures::record_key(&mut random);
        assert_eq!(
            keys::encode(previous.0, &previous.1).cmp(&keys::encode(next.0, &next.1)),
            previous.cmp(&next),
            "{previous:?} against {next:?}"
        );
        previous = next;
    }
}
#[test]
fn a_decoded_key_is_the_key_encoded() {
    let mut random = Rng::new(18);
    for _ in 0_u32..500 {
        let key = fixtures::record_key(&mut random);
        assert_eq!(keys::decode(&keys::encode(key.0, &key.1), &fixtures::LIMITS), Some(key));
    }
}
#[test]
fn a_shorter_path_sorts_before_its_extensions() {
    let shorter = jig_test_connector_world::path(1, 0);
    let longer = fixtures::path();
    assert!(
        keys::encode(fixtures::FIRST, &RecordKey::Pool(shorter))
            < keys::encode(fixtures::FIRST, &RecordKey::Pool(longer))
    );
}
#[test]
fn each_record_lies_within_its_connectors_live_interval() {
    for connector in [fixtures::FIRST, fixtures::SECOND] {
        let interval = live(connector);
        for record in fixtures::records() {
            let bytes = keys::encode(connector, &record.key());
            assert!(bytes >= interval.start);
            assert!(bytes < *interval.end.as_ref().expect("finite interval"));
        }
    }
}
