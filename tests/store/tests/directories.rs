use jig_store_world::{World, referee::Referee, root};

#[test]
fn a_made_store_directory_keeps_its_first_acknowledged_commit() {
    let mut completed = World::new(18);
    assert_eq!(completed.commit(root::first()), Some(1));
    assert_eq!(completed.commit(root::plain_commit(2)), Some(2));
    let operations = completed.operation_count();
    for cut in 0..=operations {
        let mut world = World::new(u64::from(cut));
        assert_eq!(world.commit(root::first()), Some(1), "first commit acknowledged, cut {cut}");
        world.cut_at(cut);
        world.commit(root::plain_commit(2));
        world.crash(u64::from(cut));
        let restored = world.reopen();
        assert!(restored.last >= 1, "first acknowledged commit survives, cut {cut}");
        Referee::check(&world.model, &restored)
            .unwrap_or_else(|error| panic!("made directories keep a whole prefix, cut {cut}: {error:?}"));
    }
}
