use jig_core as core;
use jig_core_people as people;
use jig_core_world::{Store, client_peer::ClientPeer, effects::World, store_peer::StorePeer};
use jig_fake_store::{Fault, Write};
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Time, Token, Wall};

fn sign_in(number: u8) -> root::Event {
    root::Event::Core(core::Event::SignIn {
        reply_to: ReplyTo::new(Token::new(7000 + u64::from(number))),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: Box::from([number, 255]) },
            login: Box::from([number]),
            name: Box::from([number, 0, 255]),
        },
    })
}

fn observed_world(seed: u64) -> World {
    let mut world = World::new(seed, false);
    assert!(world.store_peer.is_none(), "default bootstrap has no capturing peer");
    let mut peer = StorePeer::after(world.store.applied);
    peer.capture(Time::ZERO, &world.store);
    assert!(peer.take().is_empty(), "installing after bootstrap does not reclassify it");
    world.store_peer = Some(peer);
    world.client_peer = Some(ClientPeer::default());
    world
}

fn opaque_turn(bytes: &[u8]) -> root::Record {
    root::Record::Core(core::Record::Core(core::CoreRecord::Turn(core::TurnRecord {
        task: u64::MAX - 1,
        attempt: u64::MAX - 2,
        turn: u32::MAX,
        spent: u64::MAX - 3,
        read: Some(u64::MAX - 4),
        at: Wall::from_nanos(999_999),
        transcript: bytes.into(),
    })))
}

#[test]
fn an_applied_world_transaction_is_captured_before_its_held_client_answer() {
    let mut world = observed_world(641);
    let before = world.store.applied;
    world.store.fault(Fault::Hold { commits: 1 });
    world.send(sign_in(11));
    assert_eq!(world.store.applied, before + 1);
    assert!(world.client_peer.as_ref().expect("client").delivered.is_empty());
    assert_eq!(world.store.held.front(), Some(&world.store.applied));
    let snapshots = world.store_peer.as_mut().expect("store peer").take();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].commit, world.store.applied, "the actual numbered durable transaction");
    assert_eq!(snapshots[0].rows, world.store.rows, "the complete externally applied store");
    assert!(snapshots[0].at > Time::ZERO);
    assert!(snapshots[0].at.as_nanos() <= world.wall_time().as_nanos());
}

#[test]
fn repeated_drains_and_released_acknowledgements_do_not_duplicate_a_snapshot() {
    let mut world = observed_world(642);
    world.store.fault(Fault::Hold { commits: 1 });
    world.send(sign_in(12));
    let applied = world.store.applied;
    assert_eq!(world.store_peer.as_mut().expect("store peer").take().len(), 1);
    for _ in 0..3 {
        world.drain();
    }
    assert_eq!(world.store.applied, applied);
    assert!(world.store_peer.as_mut().expect("store peer").take().is_empty());
    world.release_commits();
    assert_eq!(world.store.applied, applied);
    assert_eq!(world.client_peer.as_ref().expect("client").delivered.len(), 1);
    assert!(world.store_peer.as_mut().expect("store peer").take().is_empty());
    let mut empty = StorePeer::new();
    empty.capture(Time::from_nanos(1), &Store::new());
    assert!(empty.take().is_empty(), "no invented Durable0");
}

#[test]
fn a_single_drain_preserves_every_intermediate_applied_number_and_whole_snapshot() {
    let mut world = observed_world(643);
    let before = world.store.applied;
    world.store.fault(Fault::Hold { commits: 3 });
    world.send_all(vec![sign_in(13), sign_in(14), sign_in(15)]);
    let snapshots = world.store_peer.as_mut().expect("store peer").take();
    assert_eq!(snapshots.len(), 3, "three real sign-in transactions in one drain");
    let actual: Vec<_> = world.store.held.iter().copied().collect();
    assert_eq!(snapshots.iter().map(|row| row.commit).collect::<Vec<_>>(), actual);
    assert_eq!(actual, vec![before + 1, before + 2, before + 3]);
    assert!(snapshots.windows(2).all(|pair| pair[0].at < pair[1].at));
    assert!(snapshots.windows(2).all(|pair| pair[0].rows != pair[1].rows));
    assert_eq!(snapshots.last().expect("last").rows, world.store.rows);
    assert!(world.client_peer.as_ref().expect("client").delivered.is_empty());
}

#[test]
fn whole_opaque_rows_remain_immutable_after_later_save_erase_and_store_mutation() {
    let mut store = Store::new();
    let mut peer = StorePeer::new();
    let original = opaque_turn(&[0, 255, 128, 1, 0, 254]);
    let key = original.key();
    store.submit(1, Box::new([Write::Save { key: key.clone(), row: original.clone() }]), 0);
    assert_eq!(store.tick(false), Ok(Some(1)));
    peer.capture(Time::from_nanos(17), &store);
    let first = peer.take().pop().expect("first durable snapshot");
    assert_eq!(first.rows.get(&key), Some(&original));
    let replacement = opaque_turn(b"replacement allocation");
    store.submit(2, Box::new([Write::Save { key: key.clone(), row: replacement.clone() }]), 0);
    assert_eq!(store.tick(false), Ok(Some(2)));
    peer.capture(Time::from_nanos(18), &store);
    store.submit(3, Box::new([Write::Erase(key.clone())]), 0);
    assert_eq!(store.tick(false), Ok(Some(3)));
    peer.capture(Time::from_nanos(19), &store);
    let later = peer.take();
    assert_eq!(later[0].rows.get(&key), Some(&replacement));
    assert!(later[1].rows.is_empty());
    store.rows.insert(key.clone(), opaque_turn(b"external mutation"));
    assert_eq!(first.rows.get(&key), Some(&original), "all allocated bytes and scalar facts survived");
    assert_eq!(later[0].rows.get(&key), Some(&replacement));
    assert!(later[1].rows.is_empty());
}

#[test]
fn pending_failed_and_abandoned_transactions_emit_nothing_and_crash_preserves_captured_rows() {
    let mut store = Store::new();
    let mut peer = StorePeer::new();
    let durable = opaque_turn(b"actually applied");
    let key = durable.key();
    store.submit(1, Box::new([Write::Save { key: key.clone(), row: durable.clone() }]), 0);
    assert_eq!(store.tick(true), Ok(None));
    peer.capture(Time::from_nanos(20), &store);
    store.submit(2, Box::new([Write::Erase(key.clone())]), 2);
    assert_eq!(store.tick(false), Ok(None));
    peer.capture(Time::from_nanos(21), &store);
    store.crash();
    peer.capture(Time::from_nanos(22), &store);
    let captured = peer.take();
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].rows.get(&key), Some(&durable));
    store.fault(Fault::Fail { commit: 2 });
    store.submit(2, Box::new([Write::Erase(key.clone())]), 0);
    store.submit(3, Box::new([Write::Save { key: key.clone(), row: opaque_turn(b"abandoned after failure") }]), 0);
    assert_eq!(store.tick(false), Err(2));
    peer.capture(Time::from_nanos(23), &store);
    assert!(peer.take().is_empty());
    assert_eq!(store.applied, 1);
    assert_eq!(store.rows.get(&key), Some(&durable));
    assert!(store.pending.is_empty());
    store.crash();
    peer.capture(Time::from_nanos(24), &store);
    assert!(peer.take().is_empty());
    assert_eq!(captured[0].rows.get(&key), Some(&durable));
}

#[test]
fn supplied_monotonic_time_is_exact_and_independent_of_wall_corrections() {
    let mut store = Store::new();
    let mut peer = StorePeer::new();
    let row = opaque_turn(b"independent wall timestamp");
    store.submit(1, Box::new([Write::Save { key: row.key(), row }]), 0);
    assert_eq!(store.tick(false), Ok(Some(1)));
    peer.capture(Time::from_nanos(7), &store);
    assert_eq!(peer.take()[0].at, Time::from_nanos(7), "no conversion from the row's unrelated Wall");
    let mut ordinary = observed_world(644);
    let mut corrected = observed_world(644);
    corrected.correct_wall_backwards(Duration::from_secs(60));
    ordinary.send(sign_in(16));
    corrected.send(sign_in(16));
    let first = ordinary.store_peer.as_mut().expect("store peer").take();
    let second = corrected.store_peer.as_mut().expect("store peer").take();
    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_eq!(first[0].at, second[0].at, "Wall corrections never alter the supplied monotonic tick");
    assert_eq!(first[0].commit, second[0].commit);
}
