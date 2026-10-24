use jig_protocol_store::{OpenRefusal, Owner};
use jig_store_world::{
    World,
    patches::{self, BASE, Records, SETS, Shelf},
};
use skein_lib::Rng;

// 64 seeds match this world's crash sweep: varied small records and one
// drawn mutation per seed, beside the focused exhaustive mutation sweep.
const SEEDS: std::ops::Range<u64> = 0..64;

fn drawn(random: &mut Rng) -> Records {
    let mut records = Records { labels: Vec::new(), shelves: Vec::new(), notes: Vec::new() };
    for n in 0..=random.below(4) {
        let caption = vec![
            b'a' + u8::try_from(random.below(26)).expect("letter");
            1 + usize::try_from(random.below(16)).expect("caption")
        ];
        records.labels.push((format!("l/{n}-{}", random.below(1000)).into_bytes(), caption));
    }
    for n in 0..=random.below(4) {
        let books = u32::try_from(random.below(8)).expect("book count");
        records.shelves.push(Shelf {
            key: format!("s/{n}-{}", random.below(1000)).into_bytes(),
            name: b"drawn shelf".to_vec(),
            books,
            caption: b"held label".to_vec(),
        });
    }
    for n in 0..=random.below(3) {
        records.notes.push((format!("n/{n}-{}", random.below(1000)).into_bytes(), b"drawn note".to_vec()));
    }
    records
}

fn refused(world: &mut World, records: &Records, seed: u64, cut: u32) {
    let empty =
        records.shelves.iter().filter(|shelf| shelf.books == 0).map(|shelf| &shelf.key).min().expect("empty shelf");
    let Some(OpenRefusal::Patch { owner: Owner::Application, patch: 2, key, .. }) = world.refusal() else {
        panic!("seed {seed}, cut {cut}, empty shelf: {:?}", world.refusal())
    };
    assert_eq!(key.as_ref(), empty, "seed {seed}, cut {cut}");
    assert_eq!(world.active(), b"1\n", "seed {seed}, cut {cut}");
}

#[test]
fn seeded_patch_cuts_keep_one_whole_modeled_generation() {
    let mut success = 0;
    let mut empty = 0;
    let mut cuts_reached = 0;
    for seed in SEEDS {
        let mut random = Rng::new(seed);
        let records = drawn(&mut random);
        // Discover the exact mutation range for this record shape. Empty
        // shelves use the same shape with a nonzero book count for discovery,
        // so cuts may land both before and after their refusal would occur.
        let mut valid = records.clone();
        for shelf in &mut valid.shelves {
            shelf.books = shelf.books.max(1);
        }
        let mut completed = valid.committed(seed, BASE);
        completed.patch_sets(SETS);
        completed.trace_writes();
        assert!(completed.open().is_some(), "seed {seed}: {:?}", completed.refusal());
        let cuts = patches::cuts(&completed);
        let cut = cuts[usize::try_from(random.below(u64::try_from(cuts.len()).expect("cuts"))).expect("cut")];
        let mut world = records.committed(seed, BASE);
        let original = patches::source(&mut world);
        world.patch_sets(SETS);
        world.trace_writes();
        world.cut_at(cut);
        assert_eq!(world.open(), None, "seed {seed}, cut {cut}");
        let has_empty = records.shelves.iter().any(|shelf| shelf.books == 0);
        if world.refusal().is_some() {
            refused(&mut world, &records, seed, cut);
        } else {
            cuts_reached += 1;
        }
        world.crash(random.next_u64());
        // Before the clean binary resumes patching, the active pointer names
        // a whole old image or the sealed new one. A v1 image is exactly the
        // committed bytes; the clean start's independent decoder checks v2.
        let active = world.active();
        assert!(active == b"1\n" || active == b"2\n", "seed {seed}, cut {cut}: {active:?}");
        if active == b"1\n" {
            assert_eq!(patches::source(&mut world), original, "v1 generation, seed {seed}, cut {cut}");
        }
        let opened = world.open();
        if has_empty {
            empty += 1;
            assert_eq!(opened, None, "seed {seed}, cut {cut}");
            refused(&mut world, &records, seed, cut);
            // Restart once more to prove the same typed record refusal, even
            // when the drawn power cut preceded the first attempt's site.
            world.crash(random.next_u64());
            assert_eq!(world.open(), None);
            refused(&mut world, &records, seed, cut);
        } else {
            success += 1;
            assert!(opened.is_some(), "seed {seed}, cut {cut}: {:?}", world.refusal());
            patches::whole(&mut world);
            records.check(&mut world, 2, &(seed, cut));
        }
    }
    assert!(
        success > 0 && empty > 0 && cuts_reached > 0,
        "both endings and power cuts: {success}, {empty}, {cuts_reached}"
    );
    eprintln!("seeds 0..64: {success} patched, {empty} refused, {cuts_reached} cuts reached");
}
