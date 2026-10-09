use jig_views_world::World;
#[test]
fn a_restart_keeps_no_watch_state() {
    let mut before = World::default();
    before.watch(1, 7);
    assert_eq!(before.referee.live(), 1);
    let mut after = World::default();
    assert_eq!(after.referee.live(), 0);
    assert_eq!(after.watch(1, 7).len(), 3);
}
