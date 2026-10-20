use jig_protocol_store::{TranscriptRead, Write};
use jig_store_world::{
    World,
    referee::Referee,
    root::{self, LIVE},
};

fn committed(seed: u64) -> World {
    let mut world = World::new(seed);
    assert_eq!(world.commit(root::first()), Some(1));
    world
}

fn checked_cut(cut: u32, writes: Box<[Write]>, label: &str) -> (jig_store_world::Restored, Option<u64>) {
    let mut world = committed(u64::from(cut));
    world.cut_at(cut);
    let acknowledged = world.commit(writes);
    world.crash(u64::from(cut));
    let restored = world.reopen();
    Referee::check(&world.model, &restored)
        .unwrap_or_else(|error| panic!("{label}, cut {cut}: {error:?}, {restored:?}"));
    (restored, acknowledged)
}

fn operations(writes: Box<[Write]>) -> u32 {
    let mut world = committed(18);
    assert_eq!(world.commit(writes), Some(2));
    world.operation_count()
}

#[test]
fn a_crash_at_each_file_operation_of_a_turn_keeps_a_whole_prefix() {
    let count = operations(root::turn_commit(2, LIVE, 1, b"one turn"));
    assert!(count <= 12, "turn commit's {count} file operations fit the prescribed cuts");
    for cut in 0..=12 {
        let label = "whole turn prefix";
        let (restored, acknowledged) = checked_cut(cut, root::turn_commit(2, LIVE, 1, b"one turn"), label);
        assert!(restored.last == 1 || restored.last == 2, "{label}, cut {cut}");
        if acknowledged.is_some() {
            assert_eq!(restored.last, 2, "acknowledged turn, cut {cut}");
        }
        let expected = if restored.last == 2 { vec![root::turn(LIVE, 1, 1, b"one turn")] } else { Vec::new() };
        assert_eq!(restored.turns(LIVE), expected, "{label}, cut {cut}");
    }
}

#[test]
fn a_crash_at_each_operation_of_a_plain_commit_keeps_a_whole_prefix() {
    let count = operations(root::plain_commit(2));
    for cut in 0..=count {
        let (restored, acknowledged) = checked_cut(cut, root::plain_commit(2), "whole plain prefix");
        assert!(restored.last == 1 || restored.last == 2, "whole plain prefix, cut {cut}");
        if acknowledged.is_some() {
            assert_eq!(restored.last, 2, "acknowledged plain commit, cut {cut}");
        }
        let expected = (restored.last == 2).then(|| root::live_task(LIVE + 1));
        assert_eq!(
            restored.rows.get(&jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Live(LIVE + 1))),
            expected.as_ref(),
            "whole task row, cut {cut}"
        );
    }
}

fn account_claim() -> Box<[Write]> {
    use jig_domain_core::{CoreRecord, Record};
    let Record::Tasks(jig_domain_tasks::Stored::Live(mut task)) = root::live_task(LIVE + 1) else {
        unreachable!("live task fixture")
    };
    task.kept_account = Some(7);
    let Record::Core(CoreRecord::RunProof(mut proof)) = root::run_proof(LIVE + 1) else {
        unreachable!("claim fixture")
    };
    proof.accounts = Box::new([jig_domain_authority::AccountBinding { provider: 2, account: 7, source: 9 }]);
    proof.placement = Some(skein_lib::Token::new(3));
    Box::new([
        Write::Save(root::header(2)),
        Write::Save(Record::Tasks(jig_domain_tasks::Stored::Live(task))),
        Write::Save(Record::Core(CoreRecord::RunProof(proof))),
    ])
}

#[test]
fn a_crash_keeps_the_task_pin_and_claim_bindings_together() {
    use jig_domain_core::{CoreKey, Key};
    let count = operations(account_claim());
    let writes = account_claim();
    let [_, Write::Save(task), Write::Save(proof)] = writes.as_ref() else {
        unreachable!("claim writes")
    };
    for cut in 0..=count {
        let (restored, acknowledged) = checked_cut(cut, account_claim(), "account claim");
        assert_eq!(
            restored.rows.get(&Key::Tasks(jig_domain_tasks::Key::Live(LIVE + 1))),
            (restored.last == 2).then_some(task),
            "task pin, cut {cut}"
        );
        assert_eq!(
            restored.rows.get(&Key::Core(CoreKey::RunProof(LIVE + 1))),
            (restored.last == 2).then_some(proof),
            "claim bindings, cut {cut}"
        );
        assert!(acknowledged.is_none() || restored.last == 2, "acknowledged claim, cut {cut}");
    }
}

#[test]
fn a_frame_past_its_commit_is_cut_at_open() {
    let mut complete = committed(18);
    assert_eq!(complete.commit(root::turn_commit(2, LIVE, 1, b"uncommitted frame")), Some(2));
    let synced = complete.transcript_sync().expect("frame sync before map commit");
    let mut world = committed(18);
    let writes = root::turn_commit(2, LIVE, 1, b"uncommitted frame");
    world.cut_at(synced);
    assert_eq!(world.commit(writes), None);
    world.crash(18);
    let restored = world.reopen();
    assert_eq!(restored.last, 1);
    assert!(restored.turns(LIVE).is_empty());
    assert_eq!(world.commit(root::turn_commit(2, LIVE, 1, b"replacement frame")), Some(2));
    world.crash(19);
    let restored = world.reopen();
    assert_eq!(restored.turns(LIVE), [root::turn(LIVE, 1, 1, b"replacement frame")]);
}

#[test]
fn the_header_matches_the_store_after_each_crash() {
    let count = operations(root::plain_commit(2));
    for cut in 0..=count {
        let (restored, _) = checked_cut(cut, root::plain_commit(2), "header number matches whole prefix");
        assert_eq!(
            restored.rows.get(&jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment)),
            Some(&root::header(restored.last)),
            "header number, cut {cut}"
        );
    }
}

#[test]
fn a_dropped_transcript_stays_dropped_after_a_crash() {
    let mut world = committed(18);
    assert_eq!(world.commit(root::turn_commit(2, LIVE, 1, b"removed turn")), Some(2));
    assert_eq!(world.commit(Box::new([Write::Save(root::header(3)), Write::DropTranscript { task: LIVE }])), Some(3));
    world.crash(18);
    let restored = world.reopen();
    Referee::check(&world.model, &restored).expect("dropped transcript prefix");
    assert!(restored.turns(LIVE).is_empty());
    assert!(world.transcript(TranscriptRead::Page { task: LIVE, backwards: false, after: None }).is_empty());
}
