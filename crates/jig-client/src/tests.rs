//! Vocabulary invariants; behaviour step tests land with the spine.
use crate::contract::{Text, VersionRange};
use skein_lib::List;
#[test]
fn text_refuses_bytes_past_the_owner_bound() {
    assert_eq!(Text::new(Box::from(*b"abcd"), 3).unwrap_err().as_ref(), b"abcd");
    assert_eq!(Text::new(Box::from(*b"abc"), 3).unwrap().bytes(), b"abc");
    assert_eq!(Text::new(Box::from([]), 0).unwrap().bytes(), b"");
}
#[test]
fn lists_refuse_past_their_capacity() {
    let mut rows = List::with_capacity(1);
    assert_eq!(rows.push(7_u64), Ok(()));
    assert_eq!(rows.push(8_u64), Err(8));
    assert_eq!(rows.as_slice(), &[7]);
}
#[test]
fn version_ranges_are_inclusive_and_invalid_ranges_match_nothing() {
    let range = VersionRange { first: 2, last: 4 };
    assert!(!range.contains(1));
    assert!(range.contains(2));
    assert!(range.contains(4));
    assert!(!range.contains(5));
    assert!(!VersionRange { first: 4, last: 2 }.contains(3));
}

mod spine;

mod storage;

mod lifecycle;

mod time;

mod objects;

mod confirmation;
mod pages;

mod answers;
mod conversations;
mod drafts;

mod entrance;
mod policy;

mod frame_facts;
mod hello;

mod chip_facts;

mod history_facts;

mod owner_inbox;
mod task_ending;
