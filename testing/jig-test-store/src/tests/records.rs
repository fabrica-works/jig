use super::fixtures;
use crate::{Undecodable, records};
#[test]
fn each_record_kind_decodes_to_itself() {
    for record in fixtures::records() {
        assert_eq!(records::decode(&records::encode(&record), &fixtures::LIMITS), Ok(record));
    }
}
#[test]
fn a_truncated_record_fails_to_decode() {
    for record in fixtures::records() {
        let bytes = records::encode(&record);
        for end in 0..bytes.len() {
            assert_eq!(records::decode(bytes.get(..end).expect("prefix"), &fixtures::LIMITS), Err(Undecodable));
        }
    }
}
#[test]
fn a_path_past_its_limit_fails_to_decode() {
    let record = jig_test_connector::Record::Pool { path: fixtures::path(), slots: 1 };
    let mut limits = fixtures::LIMITS;
    limits.path_segments = 1;
    assert_eq!(records::decode(&records::encode(&record), &limits), Err(Undecodable));
}
