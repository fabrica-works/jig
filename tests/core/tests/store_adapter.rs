//! The real component's durability behind the world's store face.
use jig_domain_core as core;
use jig_domain_fleet::HostKind;
use jig_domain_people as people;
use jig_test_domain as root;
use jig_test_store::{StoreOut, Up};
use jig_test_store_files::{Fault, Store};
use skein_lib::Wall;
const LIVE: u64 = 21;
fn fresh() -> Store {
    Store::new(&jig_core_world::world::limits(), &[])
}
fn save(row: core::CoreRecord) -> root::Write {
    root::Write::Save(root::Record::Core(core::Record::Core(row)))
}
fn header(commits: u64) -> root::Write {
    save(core::CoreRecord::Deployment(core::Deployment {
        id: [18; 16],
        tasks: LIVE,
        people: 0,
        sign_ins: 0,
        messages: 0,
        runs: 1,
        calls: 0,
        connector_rows: 0,
        projects: 0,
        applications: 0,
        commits,
    }))
}
fn first_commit() -> Box<[root::Write]> {
    Box::new([
        header(1),
        root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Roles {
            project: 1,
            holdings: Box::new([]),
        }))),
    ])
}
fn committed() -> Store {
    let mut store = fresh();
    store.submit(1, first_commit(), 0);
    assert_eq!(store.tick(false), Ok(Some(1)));
    store
}
fn turn_commit(task: u64, turn: u32, bytes: &[u8]) -> Box<[root::Write]> {
    Box::new([
        header(u64::from(turn) + 1),
        save(core::CoreRecord::Turn(core::TurnRecord {
            task,
            attempt: 1,
            turn,
            opens_window: turn == 1,
            spent: 0,
            read: None,
            at: Wall::EPOCH,
            transcript: Box::from(bytes),
        })),
        save(core::CoreRecord::RunProof(core::RunProof {
            accounts: Box::new([]),
            placement: None,
            account_failure: None,
            account_terminal: false,
            task,
            attempt: 1,
            conversation: 1,
            credentials: Box::new([]),
            window: Some(core::CurrentWindow { attempt: 1, turn: 1, closed: false }),
            host: HostKind::Worker,
            offered: None,
            outstanding: Box::new([]),
            turn: Some(core::TurnProof { turn, cumulative: 0, read: None }),
            terminal: None,
        })),
    ])
}
fn window_turns(store: &mut Store, task: u64) -> Vec<Box<[u8]>> {
    store.ask(StoreOut::Now(core::Now::StartPreparation { task, transcript_waiter: None, from: Some((1, 1)) }));
    let mut found = Vec::new();
    for answer in store.answers() {
        if let Up::Root(root::Event::TranscriptLoaded { rows, .. }) = answer {
            found.extend(rows.into_vec().into_iter().map(|(row, _)| row.transcript));
        }
    }
    found
}
#[test]
fn a_crash_keeps_a_durable_turn_and_loses_a_held_one() {
    let mut store = committed();
    store.submit(2, turn_commit(LIVE, 1, b"kept"), 0);
    assert_eq!(store.tick(false), Ok(Some(2)));
    store.submit(3, turn_commit(LIVE, 2, b"held by the world"), 5);
    store.crash(7);
    assert_eq!(store.applied, 2);
    assert_eq!(window_turns(&mut store, LIVE), [Box::from(b"kept".as_slice())]);
}
#[test]
fn a_held_commit_is_durable_but_unacknowledged() {
    let mut store = fresh();
    store.fault(Fault::Hold { commits: 1 });
    store.submit(1, first_commit(), 0);
    assert_eq!(store.tick(false), Ok(None));
    assert_eq!(store.applied, 1);
    store.submit(2, Box::new([header(2)]), 0);
    assert_eq!(store.tick(false), Ok(None));
    assert_eq!(store.release_held(), Some(1));
    assert_eq!(store.release_held(), Some(2));
}
#[test]
fn a_failed_commit_never_reaches_the_component() {
    let mut store = fresh();
    store.fault(Fault::Fail { commit: 1 });
    store.submit(1, first_commit(), 0);
    assert_eq!(store.tick(false), Err(1));
    assert_eq!(store.applied, 0);
    assert!(store.rows.is_empty());
}
#[test]
fn slow_pages_hold_a_restore_answer_for_their_ticks() {
    let mut store = committed();
    store.fault(Fault::SlowPages { by: 2 });
    store.ask(StoreOut::Restart { step: core::RestartStep::LoadCore, census: None });
    assert!(store.answers().is_empty());
    store.tick(false).expect("page tick");
    assert!(store.answers().is_empty());
    store.tick(false).expect("page tick");
    assert!(store.answers().iter().any(|answer| matches!(answer, Up::Restore(_))));
}
#[test]
fn a_fork_replays_to_the_same_rows() {
    let mut store = committed();
    store.submit(2, turn_commit(LIVE, 1, b"forked"), 0);
    assert_eq!(store.tick(false), Ok(Some(2)));
    let mut fork = store.fork();
    assert_eq!(fork.rows, store.rows);
    assert_eq!(window_turns(&mut fork, LIVE), [Box::from(b"forked".as_slice())]);
}
#[test]
fn a_seeded_commit_keeps_the_header_in_step() {
    let mut store = committed();
    store.seed(Box::new([root::Record::Connector {
        number: 1,
        record: jig_test_connector::Record::Pool { path: jig_test_connector_world::path(1, 1), slots: 2 },
    }]));
    let root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header))) =
        store.rows.get(&root::Key::Core(core::Key::Core(core::CoreKey::Deployment))).expect("header")
    else {
        panic!("header type");
    };
    assert_eq!(store.applied, 2);
    assert_eq!(header.commits, 2);
}

#[path = "support/fresh_windows.rs"]
mod fresh_windows;

#[test]
fn the_walks_closed_proof_restores_until_the_next_attempt() {
    use fresh_windows as f;
    let mut world = f::parked();
    let original = f::proof(&world).clone();
    let tries = f::tries(&world);
    f::restart(&mut world, f::AFTER);
    assert_eq!(f::tries(&world), tries);
    assert!(world.domain.ready(), "the real root restored the closed proof");
    let closed = f::proof(&world).clone();
    let mut expected = original;
    expected.window.as_mut().expect("parked window").closed = true;
    assert_eq!(closed, expected, "the mirror reads the component's own bytes");
    let limits = jig_core_world::world::limits().core;
    let mut engine =
        core::Core::new(jig_core_world::effects::fixture(f::SEED, false).0.core, &limits).expect("core configuration");
    engine
        .restoring_proofs
        .insert(
            closed.task,
            core::RestoringProof {
                attempt: closed.attempt,
                turn: closed.turn.expect("committed turn").turn,
                run_spent: 2,
                last_answer: closed.terminal.as_ref().map(|terminal| terminal.attempt),
            },
        )
        .expect("one proof");
    assert!(engine.restore_proof(closed.clone(), limits.tasks.result_bytes));
    assert_eq!(engine.proofs.get(&closed.task), Some(&closed));
    world.restart_with_patch_sets(f::SEED, false, f::AFTER);
    assert_eq!(f::proof(&world), &closed, "a later open keeps it closed");
    f::hello(&mut world);
    world.grant_account(1);
    world.say(87);
    f::fresh(&world);
    assert_eq!(f::tail(&world), Some(f::TAIL));
    world.window_turn(1, 1, None, &f::turn(b"new window"));
    let attempt = f::proof(&world).attempt;
    assert_eq!(f::proof(&world).window, Some(core::CurrentWindow { attempt, turn: 1, closed: false }));
}

#[test]
fn a_fork_replays_the_fresh_mark_and_its_commit_numbers() {
    use fresh_windows as f;
    let mut world = f::parked();
    f::restart(&mut world, f::AFTER);
    let mut fork = world.store.fork();
    assert_eq!(fork.rows, world.store.rows);
    assert_eq!(fork.applied, world.store.applied);
    fork.crash(f::SEED);
    assert_eq!(fork.rows, world.store.rows);
    assert_eq!(fork.applied, world.store.applied);
}
