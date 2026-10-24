//! The layout vocabulary and bounded tree removal (store.md 2 and 10.1).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "disk step tests use ordinary Rust (skein's programming-model.md, 10.2)"
)]
use super::fixtures::{self, disk::Disk};
use crate::{
    Io,
    generations::{self, Progress},
};
use skein_fake_machine::Item;

#[test]
fn active_parses_only_a_number_and_a_newline() {
    for (bytes, expected) in [
        (b"1\n".as_slice(), Some(1)),
        (b"7\n", Some(7)),
        (b"18446744073709551615\n", Some(u64::MAX)),
        (b"", None),
        (b"0\n", None),
        (b"07\n", None),
        (b"7", None),
        (b"7\n\n", None),
        (b" 7\n", None),
        (b"x\n", None),
        (b"18446744073709551616\n", None),
    ] {
        assert_eq!(generations::parse(bytes), expected, "{bytes:?}");
    }
    for number in [1, 9, 10, u64::MAX] {
        assert_eq!(generations::parse(&generations::encode(number)), Some(number));
    }
}
#[test]
fn only_ios_temporaries_beside_active_are_temporaries() {
    for (name, expected) in [
        (b"active.skein-0123456789abcdef".as_slice(), true),
        (b"active.skein-fedcba9876543210", true),
        (b"active.skein-0123456789abcde", false),
        (b"active.skein-0123456789abcdef0", false),
        (b"active.skein-0123456789abcdeF", false),
        (b"active.skein-0123456789abcdeg", false),
        (b"manifest.skein-0123456789abcdef", false),
        (b"active", false),
    ] {
        assert_eq!(generations::is_temporary(name), expected, "{name:?}");
    }
}
fn tree() -> Vec<Item> {
    let mut items = vec![
        Item::directory(b"target"),
        Item::directory(b"target/store"),
        Item::directory(b"target/secrets").mode(0o700),
        Item::file(b"target/secrets/key", b"private"),
        Item::directory(b"keep"),
        Item::file(b"keep/row", b"kept"),
    ];
    // One beyond a page, so taking one page cannot finish the map.
    for n in 0..=generations::PAGE {
        items.push(Item::file(format!("target/store/row-{n:02}").as_bytes(), b"row"));
    }
    items
}
fn remove(disk: &mut Disk) -> Progress {
    let mut below = fixtures::below();
    let mut removing = generations::remove(disk.root(), Box::from(b"target".as_slice()), &mut below.transcripts);
    let mut progress = Progress::Pending;
    disk.drive(&mut below, |io, below| {
        let Io::Transcripts(arrived) = io else { panic!("removal driver") };
        progress = generations::removed(&mut removing, arrived, &mut below.transcripts);
    });
    progress
}
#[test]
fn removing_a_tree_takes_files_and_directories_deepest_first() {
    let mut disk = Disk::new(&tree());
    assert_eq!(remove(&mut disk), Progress::Done);
    assert_eq!(
        disk.tree().into_iter().map(|(p, _, _)| p).collect::<Vec<_>>(),
        [Box::from(b"keep".as_slice()), Box::from(b"keep/row".as_slice())]
    );
    assert_eq!(disk.bytes(b"keep/row").as_ref(), b"kept");
    let names: Vec<_> = disk
        .requests()
        .iter()
        .filter_map(|(_, r)| {
            if let fixtures::disk::Seen::Remove { name, directory } = r {
                Some((name.as_ref(), *directory))
            } else {
                None
            }
        })
        .collect();
    let target = names.iter().position(|(name, dir)| *name == b"target" && *dir).expect("target removed");
    let store = names.iter().position(|(name, dir)| *name == b"store" && *dir).expect("store removed");
    let row = names.iter().rposition(|(name, _)| name.starts_with(b"row-")).expect("rows removed");
    assert!(row < store && store < target);
}
#[test]
fn removing_deeper_than_a_generation_fails() {
    let mut disk = Disk::new(&[
        Item::directory(b"target"),
        Item::directory(b"target/a"),
        Item::directory(b"target/a/b"),
        Item::directory(b"target/a/b/c"),
    ]);
    assert_eq!(remove(&mut disk), Progress::Failed);
    assert!(disk.tree().iter().any(|(p, _, _)| p.as_ref() == b"target/a/b/c"));
}
#[test]
fn a_removal_cut_at_each_operation_is_finished_by_the_next() {
    let mut completed = Disk::new(&tree());
    assert_eq!(remove(&mut completed), Progress::Done);
    let operations = completed.operations();
    for cut in 0..=operations {
        let mut disk = Disk::new(&tree());
        disk.cut_at(cut);
        remove(&mut disk);
        disk.crash(u64::from(cut));
        assert_eq!(remove(&mut disk), Progress::Done, "cut {cut}");
        assert!(disk.tree().iter().all(|(p, _, _)| !p.starts_with(b"target")), "cut {cut}");
    }
}

#[test]
fn a_manifest_encodes_levels_counts_map_commits_and_write_time_without_growth() {
    // Version 2 + generation 8 + from 9 + two full Levels (20 each) +
    // counts 4 + two (name length 4 + 13 name bytes + two u64s 16),
    // store 8 + secrets 9 + written 8. Writer allocates exactly the measure.
    const FULL: usize = 154;
    let before = crate::patching::plan::Levels { skein: Some(1), smith: Some(2), jig: Some(3), application: Some(4) };
    let after = crate::patching::plan::Levels { skein: Some(2), smith: Some(3), jig: Some(4), application: Some(5) };
    let counts = fixtures::manifest_counts(&[("fixture-label", 3, 3), ("fixture-shelf", 7, 9)]);
    let bytes = generations::manifest(
        8,
        Some(7),
        before,
        after,
        counts.as_slice(),
        17,
        Some(11),
        skein_lib::Wall::from_nanos(73),
    );
    assert_eq!(bytes.len(), FULL);
    assert_eq!(
        crate::codec::component::Manifest::decode(&bytes),
        Ok(crate::codec::component::Manifest {
            generation: 8,
            from: Some(7),
            before: crate::format::recorded(before),
            after: crate::format::recorded(after),
            counts,
            store: 17,
            secrets: Some(11),
            written: 73,
        })
    );
}
