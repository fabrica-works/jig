use crate::moved::{self, Moved, Unread};
use jig_domain_accounts::{DISPLAY, SUBJECT};
use skein_lib::{Reader, Wall, Writer, bytes};
use skein_oauth::{Limits, SavedToken};

fn limits() -> Limits {
    Limits {
        document_bytes: 512,
        string_bytes: 128,
        token_bytes: 128,
        client_bytes: 64,
        issued_client_bytes: 64,
        granted_bytes: 64,
        subject_bytes: 64,
        id_token_bytes: 64,
        detail_bytes: 64,
        record_bytes: 512,
        depth: 4,
        tokens: 32,
    }
}

fn fixture(full: bool) -> Moved {
    let mut record = SavedToken {
        key: 7,
        generation: 9,
        access_token: bytes::copy_of(b"a"),
        refresh_token: None,
        metadata: None,
        expires_at: Wall::from_nanos(123),
        issued_client_id: None,
        granted_scopes: None,
        subject: None,
        id_token: None,
        earliest_refresh_at: None,
    };
    if full {
        record.refresh_token = Some(bytes::copy_of(b"refresh"));
        record.metadata = Some(bytes::copy_of(b"metadata"));
        record.issued_client_id = Some(bytes::copy_of(b"issued-client"));
        record.granted_scopes = Some(bytes::copy_of(b"scope one"));
        record.subject = Some(bytes::copy_of(b"subject"));
        record.id_token = Some(bytes::copy_of(b"id-token"));
        record.earliest_refresh_at = Some(Wall::from_nanos(100));
    }
    Moved {
        application: bytes::copy_of(b"a"),
        issuer: bytes::copy_of(b"i"),
        client_id: bytes::copy_of(b"c"),
        subject: bytes::copy_of(b"s"),
        display: bytes::copy_of(if full { b"Display" } else { b"" }),
        record,
    }
}

fn assert_same(expected: &Moved, actual: &Moved) {
    assert_eq!(expected.application, actual.application);
    assert_eq!(expected.issuer, actual.issuer);
    assert_eq!(expected.client_id, actual.client_id);
    assert_eq!(expected.subject, actual.subject);
    assert_eq!(expected.display, actual.display);
    assert!(expected.record == actual.record, "all saved-token fields retained");
}

#[test]
fn full_and_smallest_moved_records_round_trip() {
    for full in [true, false] {
        let original = fixture(full);
        let encoded = moved::encode(&original, &limits()).expect("valid record");
        assert_eq!(&encoded.expose()[..6], b"JIGM\0\x01");
        let decoded = moved::decode(encoded.expose(), &limits()).expect("valid file");
        assert_same(&original, &decoded);
        assert_eq!(moved::encode(&decoded, &limits()).expect("decoded record fits"), encoded);
    }
}

// Walk the wire's lengths to edit a named part's prefix, not a magic offset.
fn prefix(input: &[u8], part: usize) -> usize {
    let mut reader = Reader::new(input);
    reader.skip(6).expect("header");
    for _ in 0..part {
        let len = reader.u32().expect("part length");
        reader.skip(len).expect("part bytes");
    }
    input.len().checked_sub(usize::try_from(reader.remaining()).expect("remaining fits")).expect("cursor")
}

fn length(input: &mut [u8], part: usize, len: u32) {
    let offset = prefix(input, part);
    for (index, byte) in len.to_be_bytes().into_iter().enumerate() {
        input[offset.checked_add(index).expect("prefix byte")] = byte;
    }
}

fn refused(input: &[u8], limits: &Limits, expected: Unread, name: &str) {
    assert_eq!(moved::decode(input, limits).err(), Some(expected), "{name}");
}

#[test]
fn decode_refuses_each_named_bad_form() {
    let valid = moved::encode(&fixture(true), &limits()).expect("fixture");
    let caps = [moved::APPLICATION, moved::ISSUER, moved::CLIENT_ID, SUBJECT, DISPLAY, limits().record_bytes];
    for (name, part, cap) in [
        ("application", 0, caps[0]),
        ("issuer", 1, caps[1]),
        ("client id", 2, caps[2]),
        ("subject", 3, caps[3]),
        ("display", 4, caps[4]),
        ("record", 5, caps[5]),
    ] {
        let mut changed = bytes::copy_of(valid.expose());
        length(&mut changed, part, cap.checked_add(1).expect("small cap"));
        refused(&changed, &limits(), Unread::TooLarge, name);
        // Only the prefix remains: oversize wins before trying to read bytes.
        let end = prefix(&changed, part).checked_add(4).expect("prefix end");
        refused(&changed[..end], &limits(), Unread::TooLarge, name);
    }
    for (name, offset, byte, expected) in [
        ("magic", 0, b'X', Unread::Magic),
        ("version 2", 5, 2, Unread::Version),
        ("skein record", prefix(valid.expose(), 5).checked_add(4).expect("record start"), b'X', Unread::Record),
    ] {
        let mut changed = bytes::copy_of(valid.expose());
        changed[offset] = byte;
        refused(&changed, &limits(), expected, name);
    }
    let mut trailing = Writer::new(valid.len().checked_add(1).expect("one extra byte"));
    trailing.put(valid.expose()).expect("room");
    trailing.put(b"x").expect("room");
    refused(&trailing.finish(), &limits(), Unread::Trailing, "trailing byte");
    let oversized =
        bytes::zeroed(usize::try_from(moved::bound(&limits()).checked_add(1).expect("bound plus one")).expect("fits"));
    refused(&oversized, &limits(), Unread::TooLarge, "whole-file bound before magic");
}

#[test]
fn every_cut_is_malformed_or_an_unread_record() {
    let valid = moved::encode(&fixture(true), &limits()).expect("fixture");
    for end in 0..valid.len() {
        let error = moved::decode(&valid.expose()[..end], &limits()).err().expect("cut refused");
        assert!(error == Unread::Malformed || error == Unread::Record, "cut {end}: {error:?}");
    }
}

fn filled(len: u32) -> Box<[u8]> {
    let mut field = bytes::zeroed(usize::try_from(len).expect("small bound"));
    field.fill(b'x');
    field
}

#[test]
fn each_part_accepts_its_bound_and_encode_refuses_one_past() {
    for (name, part, cap) in [
        ("application", 0_usize, moved::APPLICATION),
        ("issuer", 1, moved::ISSUER),
        ("client id", 2, moved::CLIENT_ID),
        ("subject", 3, SUBJECT),
        ("display", 4, DISPLAY),
    ] {
        let mut original = fixture(false);
        for len in [cap, cap.checked_add(1).expect("small cap")] {
            let field = match part {
                0 => &mut original.application,
                1 => &mut original.issuer,
                2 => &mut original.client_id,
                3 => &mut original.subject,
                4 => &mut original.display,
                _ => unreachable!("five fields"),
            };
            *field = filled(len);
            let encoded = moved::encode(&original, &limits());
            if len == cap {
                let encoded = encoded.expect("field at bound");
                let decoded = moved::decode(encoded.expose(), &limits()).expect("field at bound decodes");
                assert_same(&original, &decoded);
            } else {
                assert_eq!(encoded.err(), Some(Unread::TooLarge), "{name}");
            }
        }
    }
    let original = fixture(true);
    let mut limits = limits();
    limits.record_bytes =
        u32::try_from(skein_oauth::encode_record(&original.record, &limits).expect("record").len()).expect("size");
    let encoded = moved::encode(&original, &limits).expect("record at bound");
    assert_same(&original, &moved::decode(encoded.expose(), &limits).expect("record at bound decodes"));
    limits.record_bytes = limits.record_bytes.checked_sub(1).expect("nonzero record");
    assert_eq!(moved::encode(&original, &limits).err(), Some(Unread::Record));
    refused(encoded.expose(), &limits, Unread::TooLarge, "record one past");
}

#[test]
fn required_parts_refuse_empty_but_display_accepts_it() {
    let valid = moved::encode(&fixture(false), &limits()).expect("empty display allowed");
    for part in [0, 1, 2, 3, 5] {
        let mut changed = bytes::copy_of(valid.expose());
        length(&mut changed, part, 0);
        refused(&changed, &limits(), Unread::Malformed, "empty required part");
    }
    for part in 0_usize..4 {
        let mut original = fixture(false);
        match part {
            0 => original.application = bytes::copy_of(b""),
            1 => original.issuer = bytes::copy_of(b""),
            2 => original.client_id = bytes::copy_of(b""),
            3 => original.subject = bytes::copy_of(b""),
            _ => unreachable!("required outer fields"),
        }
        assert_eq!(moved::encode(&original, &limits()).err(), Some(Unread::Malformed));
    }
}

#[test]
fn encode_refuses_a_record_skein_will_not_encode() {
    let mut original = fixture(true);
    original.record.access_token = bytes::copy_of(b"invalid bearer!");
    assert_eq!(moved::encode(&original, &limits()).err(), Some(Unread::Record));
}

#[test]
fn bound_is_the_encoded_size_with_every_part_at_its_bound() {
    let mut original = fixture(true);
    original.application = filled(moved::APPLICATION);
    original.issuer = filled(moved::ISSUER);
    original.client_id = filled(moved::CLIENT_ID);
    original.subject = filled(SUBJECT);
    original.display = filled(DISPLAY);
    let mut limits = limits();
    limits.record_bytes = u32::try_from(skein_oauth::encode_record(&original.record, &limits).expect("record").len())
        .expect("record length");
    let encoded = moved::encode(&original, &limits).expect("all parts at bounds");
    assert_eq!(moved::bound(&limits), u64::try_from(encoded.len()).expect("size"));
    assert_same(&original, &moved::decode(encoded.expose(), &limits).expect("largest file decodes"));
    limits.record_bytes = u32::MAX;
    assert_eq!(
        moved::bound(&limits),
        30_u64
            .checked_add(u64::from(moved::APPLICATION))
            .expect("sum")
            .checked_add(u64::from(moved::ISSUER))
            .expect("sum")
            .checked_add(u64::from(moved::CLIENT_ID))
            .expect("sum")
            .checked_add(u64::from(SUBJECT))
            .expect("sum")
            .checked_add(u64::from(DISPLAY))
            .expect("sum")
            .checked_add(u64::from(u32::MAX))
            .expect("sum")
    );
}
