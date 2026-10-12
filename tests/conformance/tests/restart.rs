use jig_conformance::Cut;
use jig_conformance_world::actions;
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

fn story(cut: u64) -> actions::World {
    let mut world = actions::world(23, false);
    world.cut(Cut::AfterCommit(cut));
    world.drain().expect("cold startup conforms");
    if world.peers.assignments.is_empty() {
        world.advance(6_000_000_000);
        world.drain().expect("unseen claim passes its adoption grace");
        world.advance(1_000_000_000);
        world.drain().expect("replacement passes its retry backoff");
    }
    assert!(
        !world.peers.assignments.is_empty(),
        "cut {cut}, final trace {:?}",
        &world.trace[world.trace.len().saturating_sub(15)..]
    );
    actions::effect(&mut world, 1);
    world.drain().expect("effect after cut conforms");
    if !world.systems[0].observed().iter().any(|effect| effect.applied) {
        world.advance(2_000_000_000);
        world.drain().expect("effect retry deadline passes");
    }
    world
}

#[test]
fn cuts_around_the_party_answer_claim_and_effect_restart_from_durable_rows() {
    // Two surrounds the party's answer, four the first claim, five through
    // seven the effect admission, attempted write and its settled answer.
    for cut in [2, 4, 5, 6, 7] {
        let world = story(cut);
        assert_eq!(world.outcome().crashes, vec![cut], "chosen cut {cut}");
        assert_eq!(world.systems[0].observed().iter().filter(|effect| effect.applied).count(), 1);
        assert!(!world.outcome().stopped);
    }
}

#[test]
fn a_drained_shutdown_ends_with_nothing_waiting_for_a_commit() {
    let mut world = actions::world(32, false);
    world.drain().expect("startup conforms");
    let answers = world.peers.answers;
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    actions::say(&mut world, 2);
    world.drain().expect("party decision waits for its commit completion");
    assert_eq!(world.peers.answers, answers);
    actions::stop(&mut world);
    world.drain().expect("stop waits for the outstanding commit");
    assert!(world.peers.drained.is_empty());
    world.release_commits();
    world.drain().expect("held answers precede the drain barrier");
    assert_eq!(world.peers.answers, answers + 1);
    assert_eq!(world.peers.drained, [world.store.applied]);
    assert!(world.store.pending.is_empty() && world.store.held.is_empty());
    world.advance(1);
    world.drain().expect("timers after the drain cannot make another commit");
    assert_eq!(world.peers.drained, [world.store.applied]);
    assert_eq!(world.peers.busy_events.len(), 2, "each connector retains its timer");
    let rows = world.store.rows.clone();
    world.crash().expect("every drained receipt survives the cold store");
    assert_eq!(world.store.rows, rows);
    world.drain().expect("work restarts from the drained durable rows");
    actions::duplicate(&mut world);
    world.drain().expect("the party's durable answer replays after restart");
    assert_eq!(world.peers.answers, answers + 2);
}
