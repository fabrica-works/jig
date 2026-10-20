use jig_accounts_world::{ANALYST, World};
use jig_domain_accounts::{
    Event, Failure, FromComponent, Grant, GrantTable, Place, Request, State, ToComponent, Turned, Unread,
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
fn at(seconds: u64) -> Time {
    Time::ZERO.saturating_add(Duration::from_secs(seconds))
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
    assert_eq!(world.views()[0].state, State::NeedsSignIn);
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
fn the_first_unauthorized_of_a_generation_renews_and_the_second_needs_sign_in() {
    let mut world = granted();
    let before = world.views();
    assert_eq!(
        world.step(Event::Unauthorized { account: ANALYST, generation: 4 }),
        vec![Request::Component(ToComponent::Rejected { account: ANALYST, generation: 4 })]
    );
    assert_eq!(world.views(), before);
    assert_eq!(world.step(Event::Unauthorized { account: ANALYST, generation: 4 }), vec![]);
    assert_eq!(world.views()[0].state, State::NeedsSignIn);
    assert!(world.views()[0].attention);
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.domain.next_deadline(), None);
}
#[test]
fn an_unauthorized_generation_already_renewed_asks_nothing() {
    let mut world = granted();
    world.step(Event::Unauthorized { account: ANALYST, generation: 4 });
    world.step(Event::Component(FromComponent::Granted {
        account: ANALYST,
        generation: 5,
        valid: Duration::from_secs(100),
    }));
    assert_eq!(world.step(Event::Unauthorized { account: ANALYST, generation: 4 }), vec![]);
    assert!(world.domain.usable(ANALYST));
}
#[test]
fn a_limit_with_a_reset_ends_at_it_and_asks_nothing() {
    let mut world = granted();
    assert_eq!(world.step(Event::Limited { account: ANALYST, reset: Some(at(30)) }), vec![]);
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.views()[0].state, State::Limited { until: Some(at(30)) });
    assert!(world.views()[0].attention);
    assert_eq!(world.fire_at(29), vec![]);
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.fire_at(30), vec![]);
    assert!(world.domain.usable(ANALYST));
    assert_eq!(world.views()[0].state, State::Granted);
    assert!(!world.views()[0].attention);
}
#[test]
fn a_limit_without_a_reset_holds_until_resumed() {
    let mut world = granted();
    assert_eq!(world.step(Event::Limited { account: ANALYST, reset: None }), vec![]);
    assert_eq!(world.views()[0].state, State::Limited { until: None });
    assert!(world.views()[0].attention);
    assert_eq!(world.step(Event::Limited { account: ANALYST, reset: Some(at(10)) }), vec![]);
    assert_eq!(world.fire_at(10), vec![]);
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.step(Event::Resume { account: ANALYST }), vec![]);
    assert!(world.domain.usable(ANALYST));
    assert_eq!(world.views()[0].state, State::Granted);
}
#[test]
fn a_refused_account_holds_and_asks_nothing() {
    let mut world = granted();
    let refused = Event::Refused {
        account: ANALYST,
        code: b"permission_error".as_slice().into(),
        reason: b"no plan".as_slice().into(),
    };
    assert_eq!(world.step(refused), vec![]);
    assert_eq!(world.views()[0].state, State::Refused);
    assert!(world.views()[0].attention);
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.step(Event::Resume { account: ANALYST }), vec![]);
    assert_eq!(world.fire_at(1000), vec![]);
    assert_eq!(world.views()[0].state, State::Refused);
}
#[test]
fn a_failed_runs_limited_account_marks_it_limited() {
    let mut world = granted();
    let failed = Event::RunFailed { account: ANALYST, turned: Turned::Limited { reset: Some(at(12)) } };
    assert_eq!(world.step(failed), vec![]);
    assert_eq!(world.views()[0].state, State::Limited { until: Some(at(12)) });
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.fire_at(12), vec![]);
    assert!(world.domain.usable(ANALYST));
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

fn credential(generation: u64) -> Event {
    Event::Credential { number: 7, generation, valid: Duration::from_secs(100) }
}
fn credential_world() -> World {
    let mut world = World::default();
    world.step(credential(1));
    world
}
#[test]
fn a_credential_is_usable_while_its_generation_is_valid() {
    let mut world = credential_world();
    world.fire_at(99);
    assert!(world.domain.credential_usable(7));
    world.fire_at(100);
    assert!(!world.domain.credential_usable(7));
}
#[test]
fn a_credentials_new_generation_is_granted_on() {
    let mut world = credential_world();
    assert_eq!(
        world.step(credential(2)),
        vec![Request::Granted {
            grant: Grant { table: GrantTable::Application, account: 7, generation: 2, valid: Duration::from_secs(100) }
        }]
    );
}
