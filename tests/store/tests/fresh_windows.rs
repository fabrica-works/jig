//! A fresh mark closes the copy, and leaves the operator's way back whole.
use jig_domain_core::{CoreKey, CoreRecord, Key, Record};
use jig_protocol_store::{Counts, Owner, TranscriptRead, codec};
use jig_store_world::{
    World,
    patches::{
        self, FRESH_AFTER, FRESH_BEFORE, FRESH_REPLAY, FRESH_SAID, FRESH_WINDOWS, fixture_turn, fresh_committed,
        fresh_proof,
    },
    referee::Referee,
    root,
    world::Change,
};

const SEED: u64 = 940;
// The frame's task key (10 bytes) followed by its offset (8).
const FRAME_KEY_BYTES: u32 = 18;
const TOKENS: u32 = 6 + FRAME_KEY_BYTES; // six replay bytes and the frame name.

type Files = std::collections::BTreeMap<Vec<u8>, Option<Vec<u8>>>;
fn transcripts(world: &mut World) -> Files {
    world.contents().into_iter().filter(|(path, _)| path.starts_with(b"transcripts/")).collect()
}
fn check(world: &mut World, version: u16, closed: bool) {
    world.crash(SEED);
    let restored = world.reopen();
    if closed {
        Referee::fresh_windows(&restored).expect("all live windows closed");
    }
    let expected = fixture_turn(version, FRESH_SAID, FRESH_REPLAY, TOKENS);
    for (index, window) in FRESH_WINDOWS.iter().enumerate() {
        let task = root::LIVE + u64::try_from(index).expect("task index");
        let proof = fresh_proof(task, *window, closed);
        assert_eq!(restored.rows.get(&proof.key()), Some(&proof));
        if *window {
            let turns = restored.turns(task);
            assert_eq!(turns.len(), 2);
            for turn in turns {
                assert_eq!(turn.transcript, expected);
            }
        }
        for read in [
            TranscriptRead::Page { task, backwards: false, after: None },
            TranscriptRead::Turn { task, attempt: 1, turn: 2 },
        ] {
            let turns = world.transcript(read);
            assert_eq!(turns.len(), if matches!(read, TranscriptRead::Turn { .. }) { 1 } else { 2 });
            for turn in turns {
                assert_eq!(turn.transcript, expected);
            }
        }
    }
}

// Parse SKVC puts rather than searching the bytes: a copied log also holds
// proofs, but their windows are open. Every selected write is a closing
// commit's actual frame, or the seal's format put.
type Put = (Box<[u8]>, Box<[u8]>);
fn commit_keys(raw: &[u8]) -> Option<Vec<Put>> {
    let mut input = skein_lib::Reader::new(raw);
    if input.bytes(4)? != b"SKVC" {
        return None;
    }
    input.bytes(10)?; // format version and commit number
    let count = u32::from_be_bytes(input.bytes(4)?.try_into().ok()?);
    input.bytes(4)?; // body length
    let mut puts = Vec::new();
    for _ in 0..count {
        if input.bytes(1)? != [1] {
            return None;
        }
        let length = input.u32()?;
        let key = input.bytes(length)?.into();
        let length = input.u32()?;
        let value = input.bytes(length)?.into();
        puts.push((key, value));
    }
    input.bytes(4)?; // checksum
    if input.remaining() != 0 {
        return None;
    }
    Some(puts)
}
fn closing_cuts(world: &World) -> Vec<u32> {
    let limits = codec::limits(&root::CORE_LIMITS).expect("codec limits");
    let mut cuts = Vec::new();
    let mut commits = 0;
    let mut seals = 0;
    for (operation, raw) in world.written() {
        let Some(puts) = commit_keys(raw) else { continue };
        let closing = !puts.is_empty() && puts.iter().all(|(key, value)| {
            let Some(key @ Key::Core(CoreKey::RunProof(_))) = jig_protocol_store::keys::decode(key) else { return false };
            matches!(codec::decode(&key, value, &limits, &[]), Some(Record::Core(CoreRecord::RunProof(proof))) if proof.window.is_some_and(|window| window.closed))
        });
        let sealing = puts.iter().any(|(key, value)| {
            key.as_ref()
                == jig_protocol_store::keys::encode_own(&jig_protocol_store::keys::Own::Format, 64)
                    .expect("format key")
                    .as_ref()
                && jig_protocol_store::codec::component::Format::decode(value)
                    .is_ok_and(|format| format.levels.smith == Some(1))
        });
        if closing {
            commits += 1;
            cuts.push(*operation);
            let &(sync, change) = world.changes().iter().find(|(next, _)| next > operation).expect("commit sync");
            assert_eq!(change, Change::Sync);
            cuts.push(sync);
        }
        if sealing {
            seals += 1;
            cuts.push(*operation);
        }
    }
    assert_eq!(commits, 2, "five windows in four-put pages");
    assert_eq!(seals, 1);
    cuts
}

#[test]
fn fresh_windows_close_in_the_copy_and_rollback_opens_them() {
    let mut world = fresh_committed(SEED, FRESH_WINDOWS);
    let original = patches::source(&mut world);
    let frames = transcripts(&mut world);
    world.patch_sets(FRESH_AFTER);
    world.trace_writes();
    let restored = world.reopen();
    Referee::fresh_windows(&restored).expect("fresh switch closes windows");
    assert_eq!(world.patched(), [(Owner::Smith, 1, Counts { records: 0, commits: 0 })]);
    let cuts = closing_cuts(&world);
    eprintln!("fresh windows: {} cuts (two closing writes/syncs and the seal write)", cuts.len());
    check(&mut world, 2, true);
    assert_eq!(transcripts(&mut world), frames);
    patches::kept(&original, &patches::source(&mut world), &"fresh story");
    for cut in cuts {
        let mut cut_world = fresh_committed(SEED, FRESH_WINDOWS);
        cut_world.patch_sets(FRESH_AFTER);
        cut_world.trace_writes();
        cut_world.cut_at(cut);
        assert_eq!(cut_world.open(), None, "cut {cut}");
        cut_world.crash(u64::from(cut));
        assert_eq!(cut_world.active(), b"1\n");
        patches::kept(&original, &patches::source(&mut cut_world), &cut);
        check(&mut cut_world, 2, true);
        assert_eq!(transcripts(&mut cut_world), frames);
    }
    world.crash(SEED);
    world.rewrite(b"active", b"1\n");
    world.patch_sets(FRESH_BEFORE);
    check(&mut world, 1, false);
    patches::kept(&original, &patches::source(&mut world), &"fresh story");
    assert_eq!(transcripts(&mut world), frames);
}
