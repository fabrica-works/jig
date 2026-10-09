use jig_core_views::Event;
use jig_views_world::World;
use skein_lib::Token;
#[test]
fn random_changes_keep_one_delivery_per_watcher() {
    for seed in 1..=64_u64 {
        let mut world = World::default();
        for watcher in 0..4 {
            world.watch(watcher, 7);
        }
        let mut state = seed;
        for _ in 0..80 {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            world.changed(7);
            for watcher in 0..4 {
                if state.rotate_left(u32::try_from(watcher).expect("small")).is_multiple_of(3) {
                    world.delivered(watcher);
                }
            }
        }
        for watcher in 0..4 {
            world.step(Event::Unwatch { watcher: Token::new(watcher) });
            world.delivered(watcher);
        }
        assert_eq!(world.referee.live(), 0, "seed {seed}");
    }
}
