//! Total, bounded moved-record decoding (protocol/llm.md, 5.6 and 8).
use jig_protocol_accounts::moved::{self, Moved};
use skein_lib::{Rng, Wall, bytes};
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

fn check(input: &[u8], limits: &Limits, seed: u64, case: u32) -> bool {
    if let Ok(decoded) = moved::decode(input, limits) {
        let encoded = moved::encode(&decoded, limits).expect("decoded parts fit");
        assert!(encoded.expose() == input, "round trip: seed {seed:#x}, case {case}");
        assert!(u64::try_from(encoded.len()).expect("size fits") <= moved::bound(limits));
        true
    } else {
        false
    }
}

#[test]
fn arbitrary_moved_files_never_panic_and_round_trip_when_they_decode() {
    let limits = limits();
    let mut buffer =
        bytes::zeroed(usize::try_from(moved::bound(&limits)).expect("small bound").checked_add(1).expect("one past"));
    for seed in [0x6d6f_7665_0001, 0x6d6f_7665_0002] {
        let mut rng = Rng::new(seed);
        for case in 0..2_000_u32 {
            let length = usize::try_from(
                rng.below(u64::try_from(buffer.len()).expect("small buffer").checked_add(1).expect("inclusive length")),
            )
            .expect("small length");
            for byte in buffer.get_mut(..length).expect("chosen length") {
                *byte = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
            }
            let _accepted = check(buffer.get(..length).expect("chosen length"), &limits, seed, case);
        }
    }
}

#[test]
fn single_byte_mutations_never_panic_and_round_trip_when_they_decode() {
    let limits = limits();
    let original = Moved {
        application: bytes::copy_of(b"app"),
        issuer: bytes::copy_of(b"https://issuer.example"),
        client_id: bytes::copy_of(b"client"),
        subject: bytes::copy_of(b"subject"),
        display: bytes::copy_of(b"Display"),
        record: SavedToken {
            key: 7,
            generation: 9,
            access_token: bytes::copy_of(b"access"),
            refresh_token: Some(bytes::copy_of(b"refresh")),
            metadata: Some(bytes::copy_of(b"metadata")),
            expires_at: Wall::from_nanos(123),
            issued_client_id: Some(bytes::copy_of(b"issued-client")),
            granted_scopes: Some(bytes::copy_of(b"scope one")),
            subject: Some(bytes::copy_of(b"subject")),
            id_token: Some(bytes::copy_of(b"id-token")),
            earliest_refresh_at: Some(Wall::from_nanos(100)),
        },
    };
    let valid = moved::encode(&original, &limits).expect("fixture");
    let seed = 0x6d6f_7665_0003;
    let mut rng = Rng::new(seed);
    assert!(check(valid.expose(), &limits, seed, 0));
    let mut accepted = 0_u32;
    let mut refused = 0_u32;
    for case in 0..10_000_u32 {
        let mut changed = bytes::copy_of(valid.expose());
        let index =
            usize::try_from(rng.below(u64::try_from(changed.len()).expect("small fixture"))).expect("index fits");
        let mask = u8::try_from(rng.below(255).checked_add(1).expect("nonzero byte")).expect("byte");
        *changed.get_mut(index).expect("chosen byte") ^= mask;
        if check(&changed, &limits, seed, case) {
            accepted = accepted.checked_add(1).expect("bounded cases");
        } else {
            refused = refused.checked_add(1).expect("bounded cases");
        }
    }
    assert!(accepted > 0 && refused > 0, "seed {seed:#x} exercises both outcomes");
}
