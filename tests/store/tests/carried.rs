//! A value a client's request carried, across crashes (store.md 9: kept
//! durably before the core sees the request, numbered by jig and never
//! reused, erased when its request goes unused).
use jig_protocol_store::{CARRIED, Event, SecretOwner, Secrets};
use jig_store_world::{World, root};
use skein_lib::Secret;

fn value(byte: u8) -> Secret {
    Secret::new(vec![byte; 300].into_boxed_slice())
}

fn opened(seed: u64) -> World {
    let mut world = World::new(seed);
    assert_eq!(world.commit(root::first()), Some(1));
    world
}

/// The number a carry was kept under, none when it was not answered.
fn carry(world: &mut World, byte: u8) -> Option<u64> {
    match world.secret(Secrets::Carry { value: value(byte) }).as_slice() {
        [Event::Carried { number, .. }] => Some(*number),
        [] => None,
        events => panic!("a carry's terminal: {events:?}"),
    }
}

fn found(world: &mut World, number: u64) -> Option<Secret> {
    let events = world.secret(Secrets::Get { owner: SecretOwner::Application, number });
    let [Event::Secret { value, .. }] = events.as_slice() else { panic!("secret terminal: {events:?}") };
    value.clone()
}

#[test]
fn a_carried_value_is_durable_once_answered_and_its_number_moves_with_it() {
    let count = {
        let mut world = opened(18);
        carry(&mut world, 1);
        world.operation_count()
    };
    assert!(count > 0, "a carry writes");
    for cut in 0..=count {
        let mut world = opened(u64::from(cut));
        world.cut_at(cut);
        let carried = carry(&mut world, 1);
        world.crash(u64::from(cut));
        world.reopen();
        let kept = found(&mut world, CARRIED);
        if let Some(number) = carried {
            assert_eq!(number, CARRIED);
            assert_eq!(kept, Some(value(1)), "kept only once durable, cut {cut}");
        } else {
            assert!(kept.is_none() || kept == Some(value(1)), "a whole value, cut {cut}");
        }
        assert!(cut < count || carried.is_some(), "carried once every operation is done");
        let next = if kept.is_some() { CARRIED + 1 } else { CARRIED };
        assert_eq!(carry(&mut world, 2), Some(next), "the counter moved with the value, cut {cut}");
        assert_eq!(found(&mut world, next), Some(value(2)));
        if kept.is_some() {
            assert_eq!(found(&mut world, CARRIED), Some(value(1)), "never overwritten, cut {cut}");
        }
    }
}

#[test]
fn an_erased_carried_value_stays_erased_and_its_number_is_never_reused() {
    let mut world = opened(18);
    assert_eq!(carry(&mut world, 1), Some(CARRIED));
    assert_eq!(carry(&mut world, 2), Some(CARRIED + 1));
    let events = world.secret(Secrets::Erase { owner: SecretOwner::Application, number: CARRIED + 1 });
    assert!(matches!(events.as_slice(), [Event::Erased { .. }]), "erased: {events:?}");
    world.crash(18);
    world.reopen();
    assert_eq!(found(&mut world, CARRIED), Some(value(1)));
    assert_eq!(found(&mut world, CARRIED + 1), None, "erased for good");
    assert_eq!(carry(&mut world, 3), Some(CARRIED + 2), "the highest number erased is not drawn again");
}
