use jig_accounts_world::{ANALYST, World};
use jig_domain_accounts::{
    Event, Failure, FromComponent, Grant, GrantTable, Place, Request, State, ToComponent, Unread,
};
use skein_lib::{Duration, Time};
fn granted() -> World {
    let mut world = World::wanting(&[ANALYST]);
    assert_eq!(
        world.step(Event::Component(FromComponent::Granted {
            account: ANALYST,
            generation: 4,
            valid: Duration::from_secs(100)
        })),
        vec![Request::Granted {
            grant: Grant {
                table: GrantTable::Accounts,
                account: ANALYST,
                generation: 4,
                valid: Duration::from_secs(100)
            }
        }]
    );
    world
}
fn failed(world: &mut World, failure: Failure) -> Vec<Request> {
    world.step(Event::Component(FromComponent::Failed { account: ANALYST, failure }))
}
fn want() -> Request {
    Request::Component(ToComponent::Want { account: ANALYST })
}
#[test]
fn every_configured_account_is_wanted_at_the_start() {
    let mut world = World::configured(&[ANALYST, 8]);
    assert_eq!(world.step(Event::Want), vec![want(), Request::Component(ToComponent::Want { account: 8 })]);
    assert_eq!(world.step(Event::Want), vec![]);
}
#[test]
fn a_grant_is_usable_until_its_token_lapses() {
    let mut world = granted();
    assert!(world.domain.usable(ANALYST));
    assert_eq!(world.fire_at(99), vec![]);
    assert!(world.domain.grant(ANALYST, world.now).is_some());
    assert_eq!(world.fire_at(100), vec![]);
    assert!(!world.domain.usable(ANALYST));
    assert!(world.domain.grant(ANALYST, world.now).is_none());
}
#[test]
fn later_asks_again_after_the_drivers_delay() {
    let mut world = granted();
    assert_eq!(failed(&mut world, Failure::Later { after: Some(Duration::from_secs(12)) }), vec![]);
    assert_eq!(world.views()[0].state, State::Later { until: Time::ZERO.saturating_add(Duration::from_secs(12)) });
    assert_eq!(world.fire_at(11), vec![]);
    assert_eq!(world.fire_at(12), vec![want()]);
}
#[test]
fn later_without_a_delay_waits_the_childs_backoff() {
    let mut world = granted();
    failed(&mut world, Failure::Later { after: None });
    assert_eq!(world.fire_at(1), vec![]);
    assert_eq!(world.fire_at(2), vec![want()]);
    failed(&mut world, Failure::Later { after: None });
    assert_eq!(world.fire_at(5), vec![]);
    assert_eq!(world.fire_at(6), vec![want()]);
}
#[test]
fn sign_in_again_is_unusable_and_asked_again_at_the_retry_interval() {
    let mut world = granted();
    assert_eq!(failed(&mut world, Failure::SignInAgain), vec![]);
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.views()[0].state, State::SignInAgain);
    assert_eq!(world.fire_at(29), vec![]);
    assert_eq!(world.fire_at(30), vec![want()]);
}
#[test]
fn misconfigured_is_never_asked_again() {
    let mut world = granted();
    failed(&mut world, Failure::Misconfigured);
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.views()[0].state, State::Misconfigured);
    assert_eq!(world.fire_at(1000), vec![]);
    assert_eq!(world.domain.next_deadline(), None);
}
#[test]
fn unloaded_says_where_and_why_and_is_asked_again() {
    let mut world = granted();
    failed(&mut world, Failure::Unloaded { at: Place::File, why: Unread::Refused });
    assert_eq!(world.views()[0].state, State::Unloaded { at: Place::File, why: Unread::Refused });
    assert!(world.views()[0].attention);
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.fire_at(29), vec![]);
    assert_eq!(world.fire_at(30), vec![want()]);
}
#[test]
fn expiring_stays_usable_and_says_a_newer_record_is_needed() {
    let mut world = granted();
    assert_eq!(world.step(Event::Component(FromComponent::Expiring { account: ANALYST, generation: 4 })), vec![]);
    assert!(world.domain.usable(ANALYST));
    assert!(world.views()[0].expiring && world.views()[0].attention);
    world.fire_at(100);
    assert!(!world.domain.usable(ANALYST));
}
#[test]
fn a_rejection_goes_to_the_component_and_changes_nothing_else() {
    let mut world = granted();
    let before = world.views();
    assert_eq!(
        world.step(Event::Rejected { account: ANALYST, generation: 4 }),
        vec![Request::Component(ToComponent::Rejected { account: ANALYST, generation: 4 })]
    );
    assert_eq!(world.views(), before);
}
#[test]
fn exhausted_is_spent_until_its_time_and_asks_nothing() {
    let mut world = granted();
    assert_eq!(world.step(Event::Exhausted { account: ANALYST, retry_after: Duration::from_secs(30) }), vec![]);
    assert!(!world.domain.usable(ANALYST));
    assert!(world.views()[0].attention);
    assert_eq!(world.fire_at(30), vec![]);
    assert!(world.domain.usable(ANALYST));
    assert!(!world.views()[0].attention);
}
#[test]
fn close_releases_every_account_and_ends_on_closed() {
    let mut world = World::wanting(&[ANALYST, 8]);
    assert_eq!(
        world.step(Event::Close),
        vec![
            Request::Component(ToComponent::Release { account: ANALYST }),
            Request::Component(ToComponent::Release { account: 8 }),
            Request::Component(ToComponent::Close)
        ]
    );
    assert!(world.domain.waiting());
    assert_eq!(world.domain.next_deadline(), None);
    assert_eq!(world.step(Event::Component(FromComponent::Closed)), vec![Request::Closed]);
    assert!(!world.domain.waiting());
    assert_eq!(world.domain.accounts(), 0);
    assert_eq!(world.fire_at(100), vec![]);
}
