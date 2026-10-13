use jig_store_world::{
    World,
    referee::Referee,
    root::{self, LIVE},
};
use skein_lib::Rng;

#[test]
fn random_commits_and_crashes_keep_the_referees_promises() {
    for seed in 0..64 {
        let mut random = Rng::new(seed);
        let mut world = World::new(seed);
        assert_eq!(world.commit(root::first()), Some(1), "initial prefix, seed {seed}");
        let mut next = 2;
        let mut turn = 1;
        for _ in 0..8 {
            let has_turn = random.below(2) == 0;
            let writes =
                if has_turn { root::turn_commit(next, LIVE, turn, b"seeded turn") } else { root::plain_commit(next) };
            let cut = u32::try_from(random.below(13)).expect("twelve file operation cuts");
            world.cut_at(cut);
            let acknowledged = world.commit(writes);
            world.crash(random.next_u64());
            let restored = world.reopen();
            Referee::check(&world.model, &restored)
                .unwrap_or_else(|error| panic!("whole random prefix, seed {seed}, cut {cut}: {error:?}, {restored:?}"));
            if acknowledged.is_some() {
                assert_eq!(restored.last, next, "acknowledged prefix, seed {seed}, cut {cut}");
            }
            if has_turn && restored.last == next {
                turn += 1;
            }
            next = restored.last + 1;
            world.accept_recovery(restored.last);
        }
    }
}
