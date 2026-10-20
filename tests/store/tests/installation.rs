//! The installation's identifier across crashes (store.md 4.4 item 9;
//! configuration.md 8): an empty store's first commit keeps the identifier
//! the shell drew, and every later start restores it instead of its own draw.
use jig_store_world::{World, root};

const FIRST: [u8; 16] = [1; 16];
const SECOND: [u8; 16] = [2; 16];
const THIRD: [u8; 16] = [3; 16];

/// A world over an empty store, opened with `FIRST` drawn.
fn empty(seed: u64) -> World {
    let world = World::new(seed);
    assert_eq!(world.installation, world.drawn, "an empty store answers this start's draw");
    assert_eq!(world.installation, FIRST);
    world
}

#[test]
fn the_identifier_survives_a_crash_at_the_first_commit_and_is_never_redrawn_once_kept() {
    let count = {
        let mut world = empty(18);
        assert_eq!(world.commit(root::first()), Some(1));
        world.operation_count()
    };
    assert!(count > 0, "the first commit writes");
    for cut in 0..=count {
        let mut world = empty(u64::from(cut));
        world.cut_at(cut);
        let acknowledged = world.commit(root::first());
        world.crash(u64::from(cut));
        world.drawn = SECOND;
        let restored = world.reopen();
        let expected = if restored.last == 1 { FIRST } else { SECOND };
        assert_eq!(world.installation, expected, "kept with the first commit or not at all, cut {cut}");
        assert!(acknowledged.is_none() || restored.last == 1, "an acknowledged first commit is durable, cut {cut}");
        assert!(cut > 0 || restored.last == 0, "nothing kept before the first operation");
        assert!(cut < count || restored.last == 1, "kept once every operation is done, cut {cut}");
        if restored.last == 1 {
            world.accept_recovery(1);
        } else {
            assert_eq!(world.commit(root::first()), Some(1), "this start keeps its own draw, cut {cut}");
        }
        let kept = world.installation;
        world.crash(u64::from(cut).wrapping_add(100));
        world.drawn = THIRD;
        let restored = world.reopen();
        assert_eq!(restored.last, 1);
        assert_eq!(world.installation, kept, "a later start restores the kept identifier, cut {cut}");
    }
}
