//! Account diagnostics carry kinds and room limits, independent of credential payloads.
use jig_domain_people::{
    AccountChange, AccountHolder, Ask, CapacityAction, CapacityCause, CapacityRefusal, LimitKind, Reply, RequestKey,
};
use jig_people_world::{Settings, World};
use skein_lib::{Duration, Wall};
const USER: u64 = 1;
const SIGN_IN: u64 = 10;
const KEY: u8 = 7;
const SEED: u64 = 73;
const PROVIDER: u32 = 987;
const SOURCE: u32 = 654;
fn asks() -> [(Ask, CapacityAction); 3] {
    [
        (Ask::AccountSignIn { holder: AccountHolder::Own, provider: PROVIDER }, CapacityAction::AccountSignIn),
        (Ask::AccountSignOut { holder: AccountHolder::Own, source: SOURCE }, CapacityAction::AccountSignOut),
        (
            Ask::AccountChange {
                holder: AccountHolder::Own,
                change: AccountChange::AddKey { provider: PROVIDER, join: None, digest: [255; 32] },
            },
            CapacityAction::AccountChange,
        ),
    ]
}
fn world(settings: Settings) -> (World, u64) {
    let mut world = World::new(settings);
    let signed_in = world.signin(SIGN_IN, USER);
    world.commit_all();
    let Some(Reply::SignedIn { person, .. }) = world.reply(signed_in) else { panic!("signed in fixture") };
    (world, person)
}
fn refused(which: LimitKind, ask: Ask) -> (World, u64) {
    let (mut world, party) = world(Settings { account_limit: Some(which), ..Settings::calm(SEED) });
    world.request(SIGN_IN, [KEY; 16], ask);
    world.commit_all();
    (world, party)
}
#[test]
fn account_room_diagnostics_exclude_payloads() {
    for which in [LimitKind::Accounts, LimitKind::HolderAccounts, LimitKind::Sources, LimitKind::SignIns] {
        for (ask, action) in asks() {
            let (world, party) = refused(which, ask);
            assert_eq!(
                world.capacity_refusals().as_slice(),
                &[CapacityRefusal {
                    key: RequestKey { person: party, key: [KEY; 16] },
                    project: None,
                    action,
                    at: Wall::EPOCH,
                    cause: CapacityCause::Limit { which },
                }]
            );
            world.assert_settled();
        }
    }
}
#[test]
fn account_room_replay_keeps_one_diagnostic() {
    for which in [LimitKind::Accounts, LimitKind::HolderAccounts, LimitKind::Sources, LimitKind::SignIns] {
        for (ask, _) in asks() {
            let (mut world, _) = refused(which, ask.clone());
            world.restart();
            world.request(SIGN_IN, [KEY; 16], ask);
            world.commit_all();
            assert_eq!((world.capacity_refusals().len(), world.stats().routes), (1, 1));
            world.assert_settled();
        }
    }
}
#[test]
fn account_room_diagnostics_expire() {
    for (ask, _) in asks() {
        let (mut world, _) = refused(LimitKind::Sources, ask);
        world.advance(Duration::from_secs(121));
        assert!(world.capacity_refusals().is_empty());
        world.assert_settled();
    }
}
fn entrances() -> [(u32, CapacityCause); 2] {
    [
        (jig_people_world::LIMITS.requests, CapacityCause::Pending { limit: 1 }),
        (1, CapacityCause::Requests { limit: 1 }),
    ]
}
fn entrance(requests: u32) -> World {
    let mut settings = Settings::calm(SEED);
    settings.limits.pending = 1;
    settings.limits.requests = requests;
    let (mut world, _) = world(settings);
    world.defer_routes(true);
    world.request(SIGN_IN, [KEY; 16], asks()[0].0.clone());
    for (at, (ask, _)) in asks().into_iter().enumerate() {
        world.request(SIGN_IN, [u8::try_from(at).expect("three asks"); 16], ask);
    }
    world.commit_all();
    world
}
fn settle(mut world: World) {
    world.finish_account_deferred();
    world.commit_all();
    world.assert_settled();
}
#[test]
fn account_entrance_history_is_bounded() {
    for (requests, _) in entrances() {
        let world = entrance(requests);
        assert_eq!(world.capacity_refusals().len(), jig_people_world::LIMITS.room_refusals);
        settle(world);
    }
}
#[test]
fn account_entrance_history_counts_forgotten() {
    for (requests, _) in entrances() {
        let world = entrance(requests);
        assert_eq!(world.capacity_forgotten(), 1);
        settle(world);
    }
}
#[test]
fn account_entrance_pressure_names_its_cause() {
    for (requests, cause) in entrances() {
        let world = entrance(requests);
        for refusal in &world.capacity_refusals() {
            assert_eq!(refusal.cause, cause);
        }
        settle(world);
    }
}
fn waiter() -> World {
    let (mut world, _) = world(Settings::calm(SEED));
    world.defer_routes(true);
    let ask = asks()[2].0.clone();
    for _ in 0..6 {
        world.request(SIGN_IN, [KEY; 16], ask.clone());
    }
    world.commit_all();
    world
}
#[test]
fn account_waiter_history_is_bounded() {
    let world = waiter();
    assert_eq!(world.capacity_refusals().len(), jig_people_world::LIMITS.room_refusals);
    settle(world);
}
#[test]
fn account_waiter_history_counts_forgotten() {
    let world = waiter();
    assert_eq!(world.capacity_forgotten(), 1);
    settle(world);
}
#[test]
fn account_waiter_diagnostics_exclude_payloads() {
    let world = waiter();
    for refusal in &world.capacity_refusals() {
        assert_eq!(refusal.action, CapacityAction::AccountChange);
    }
    settle(world);
}
#[test]
fn account_waiter_pressure_names_its_cause() {
    let world = waiter();
    for refusal in &world.capacity_refusals() {
        assert_eq!(refusal.cause, CapacityCause::Waiters { limit: 3 });
    }
    settle(world);
}

#[test]
fn program_requests_classify_as_account_changes() {
    for change in [
        AccountChange::DeclareProgram { provider: PROVIDER, join: None, name: b"personal".as_slice().into() },
        AccountChange::CheckProgram { source: SOURCE },
    ] {
        let (world, _) = refused(LimitKind::Sources, Ask::AccountChange { holder: AccountHolder::Own, change });
        assert_eq!(
            world.capacity_refusals().get(0).expect("one content-free room refusal").action,
            CapacityAction::AccountChange
        );
        world.assert_settled();
    }
}
