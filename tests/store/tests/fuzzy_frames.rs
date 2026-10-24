//! Public decode against fixture bytes, using the fuzzy profile's binary filter.
#[path = "support/frame_reads.rs"]
mod frame_reads;
use frame_reads::{FRAME_KEY_BYTES, decode, turn};
use jig_test_patches_jig::frames;
use skein_lib::Rng;
const CASES: u32 = 512;
const SEED: u64 = 0x0003_5a17_2026;

#[test]
fn random_fixture_frames_match_patched_bytes_or_unread() {
    let (mut component, env) = frame_reads::closed(frame_reads::SETS, &jig_store_world::root::limits());
    frame_reads::open(&mut component, &env);
    let mut rng = Rng::new(SEED);
    for case in 0..CASES {
        let body = u16::try_from(4 + rng.below(4)).expect("body version");
        let version = u16::try_from(rng.below(4)).expect("turn version");
        let said = vec![b's'; usize::try_from(rng.below(65)).expect("said")];
        let replay = vec![b'r'; usize::try_from(rng.below(65)).expect("replay")];
        let no_site = rng.below(4) == 0;
        let corrupt = rng.below(8) == 0;
        let short = rng.below(8) == 0;
        let mut transcript = turn(version, &said, &replay, case);
        if short {
            transcript = transcript[..usize::try_from(rng.below(2)).expect("short")].into();
        }
        let mut raw = frames::frame(body, &transcript);
        if corrupt {
            let index = usize::try_from(rng.below(u64::try_from(raw.len()).expect("frame"))).expect("index");
            raw[index] ^= 1;
        }
        let valid = [5, 6].contains(&body)
            && [1, 2].contains(&version)
            && !short
            && !corrupt
            && (version == 2 || (!no_site && !replay.is_empty()));
        let expected = if valid {
            Some(turn(
                2,
                &said,
                &[],
                if version == 1 { u32::try_from(replay.len() + FRAME_KEY_BYTES).expect("tokens") } else { case },
            ))
        } else {
            None
        };
        assert_eq!(decode(&mut component, &env, &raw, no_site), expected, "case {case}, body {body}, turn {version}");
    }
}
