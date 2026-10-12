//! Keys' bytes against the order Rust derives (protocol/store.md, 4.1, 14).
use super::fixtures;
use crate::keys;
use jig_domain_core::{CoreKey, Key};
use skein_lib::Rng;

#[test]
fn core_and_people_keys_sort_as_bytes_as_they_derive() {
    let mut rng = Rng::new(0x5701_e001);
    for _ in 0_u32..2_000 {
        let left = fixtures::key(&mut rng);
        let right = fixtures::key(&mut rng);
        let left_bytes = keys::encode(&left, fixtures::KEY_LIMIT).expect("within the key limit");
        let right_bytes = keys::encode(&right, fixtures::KEY_LIMIT).expect("within the key limit");
        assert_eq!(left.cmp(&right), left_bytes.cmp(&right_bytes), "{left:?} against {right:?}");
    }
}
#[test]
fn every_generated_key_decodes_to_itself() {
    let mut rng = Rng::new(0x5701_e002);
    for _ in 0_u32..2_000 {
        let key = fixtures::key(&mut rng);
        let bytes = keys::encode(&key, fixtures::KEY_LIMIT).expect("within the key limit");
        assert_eq!(keys::decode(&bytes), Some(key));
    }
}
#[test]
fn own_keys_sort_as_bytes_as_they_derive() {
    let mut rng = Rng::new(0x5701_e003);
    for _ in 0_u32..2_000 {
        let left = fixtures::own(&mut rng);
        let right = fixtures::own(&mut rng);
        let left_bytes = keys::encode_own(&left, fixtures::KEY_LIMIT).expect("within the key limit");
        let right_bytes = keys::encode_own(&right, fixtures::KEY_LIMIT).expect("within the key limit");
        assert_eq!(left.cmp(&right), left_bytes.cmp(&right_bytes), "{left:?} against {right:?}");
    }
}
#[test]
fn every_generated_own_key_decodes_to_itself() {
    let mut rng = Rng::new(0x5701_e013);
    for _ in 0_u32..2_000 {
        let key = fixtures::own(&mut rng);
        let bytes = keys::encode_own(&key, fixtures::KEY_LIMIT).expect("within the key limit");
        assert_eq!(keys::decode_own(&bytes), Some(key));
    }
}
#[test]
fn a_key_with_bytes_left_over_does_not_decode() {
    assert_eq!(keys::decode(&[1, 0, 0, 0]), None);
    assert_eq!(keys::decode_own(&[0, 0, 0]), None);
}
#[test]
fn a_key_past_its_limit_is_none() {
    assert_eq!(keys::encode(&Key::Core(CoreKey::Deployment), 2), None);
    assert_eq!(keys::encode_own(&keys::Own::Format, 1), None);
}
#[test]
fn an_application_key_carries_tag_two_and_strips_back() {
    let bytes = keys::application(&[0, 1, 255], 4).expect("at the key limit");
    assert_eq!(&*bytes, &[2, 0, 1, 255]);
    assert_eq!(keys::of_application(&bytes), Some(&[0, 1, 255][..]));
}
#[test]
fn an_application_key_past_its_limit_is_none() {
    assert_eq!(keys::application(&[0, 1, 255], 3), None);
}
#[test]
fn an_unknown_or_short_key_does_not_decode() {
    for bytes in [&[][..], &[1, 0, 255], &[1, 0, 2, 0], &[1, 2, 6], &[2, 0, 0]] {
        assert_eq!(keys::decode(bytes), None, "{bytes:?}");
    }
    for bytes in [&[][..], &[0, 255], &[0, 1, 0], &[1, 0]] {
        assert_eq!(keys::decode_own(bytes), None, "{bytes:?}");
    }
}
#[test]
fn an_application_key_must_have_its_owners_tag() {
    assert_eq!(keys::of_application(&[]), None);
    assert_eq!(keys::of_application(&[1, 0]), None);
}
