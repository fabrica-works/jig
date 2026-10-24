//! A refused restart preserves the operator's way back (store.md 2, 10.2, 16).
//!
//! The check before the switch runs neither the children's rules nor the
//! rows the core's census chooses. A record that breaks one stops the start
//! after the switch: a child's restore refuses it, and the store's own
//! census join stops on a result row whose ended task it does not hold
//! (`restore.rs`: `ended` answers none at :116, or a mismatched key at
//! :118; `decode` passes it up at :95 and `restore_page` at :73; `restore`
//! stops with `Event::Corrupt` at :26-28). This test models the first.
use jig_store_world::patches::{self, BASE, Records, SETS};
use jig_test_patches::patches::kept::fixture_shelf;

/// A referee rule standing in for a child's restore rule (engine.md 6);
/// the store world has no children, and jig's store has no such rule.
/// This child accepts shelves weighing at most 30.
fn a_child_refuses(key: &[u8], bytes: &[u8]) -> bool {
    if !key.starts_with(b"s/") {
        return false;
    }
    let schema = skein_codec::schema::parse(fixture_shelf::TEXTS[1].as_bytes()).expect("kept shelf schema");
    let root = schema.find(b"Shelf").expect("shelf root");
    let mut value = skein_codec::value::Value::with_room(16);
    let shelf = value.decode(&schema, root, bytes.into()).expect("patched shelf decodes");
    let weight = value.field(shelf, fixture_shelf::v2::shelf::WEIGHT).expect("shelf weight field");
    value.uint(weight).expect("weight is a u32") > 30
}

#[test]
fn a_record_a_child_refuses_after_the_switch_keeps_the_generation_before() {
    const SEED: u64 = 911;
    let records = Records::story();
    let mut world = records.committed(SEED, BASE);
    let original = patches::source(&mut world);
    world.patch_sets(SETS);
    let last = world.open().expect("patched open");
    patches::whole(&mut world);
    records.check(&mut world, 2, &"after switch");

    let key = b"s/2".as_slice();
    let expected = records.rows(2).get(key).expect("story's heaviest shelf").clone();
    // The store opened after the switch; the modelled child's restore now
    // stops this start before it can make any commit of its own.
    let refused: Vec<_> =
        world.application(false).into_iter().filter(|(key, bytes)| a_child_refuses(key, bytes)).collect();
    assert_eq!(refused, [(key.into(), expected.clone())]);
    world.crash(SEED);

    for start in ["first further start", "second further start"] {
        assert_eq!(world.open(), Some(last), "{start}");
        assert!(world.patched().is_empty(), "{start}");
        assert_eq!(world.active(), b"2\n", "{start}");
        let refused: Vec<_> =
            world.application(false).into_iter().filter(|(key, bytes)| a_child_refuses(key, bytes)).collect();
        assert_eq!(refused, [(key.into(), expected.clone())], "{start}");
        patches::kept(&original, &patches::source(&mut world), &start);
        world.crash(SEED);
    }

    // Before generation 2's first commit of its own, the operator names
    // generation 1 again and starts with its matching patch sets.
    world.rewrite(b"active", b"1\n");
    world.patch_sets(BASE);
    assert_eq!(world.open(), Some(1));
    assert!(world.patched().is_empty());
    assert_eq!(world.active(), b"1\n");
    records.check(&mut world, 1, &"operator's way back");
    patches::kept(&original, &patches::source(&mut world), &"operator's way back");
    assert!(!world.paths().iter().any(|path| path == b"generations/2" || path.starts_with(b"generations/2/")));
}
