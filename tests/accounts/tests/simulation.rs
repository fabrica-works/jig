use jig_accounts_world::{ANALYST, World};
use jig_domain_accounts::{
    Event, Failure, Form, FromComponent, Grant, Held, Kind, Place, Request, Scheme, State, Stored, ToComponent, Turned,
    Unread,
};
use skein_lib::{Duration, Time, Wall};
fn granted() -> World {
    let mut world = World::wanting(&[ANALYST]);
    assert_eq!(
        world.step(Event::Component(FromComponent::Granted {
            form: Form::Credential(Scheme::Bearer),
            account: ANALYST,
            generation: 4,
            valid: Duration::from_secs(100)
        })),
        vec![Request::Granted {
            grant: Grant {
                form: Form::Credential(Scheme::Bearer),
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
/// The analyst's source's record, saved in the step that changes its hold.
fn kept(held: Option<Held>) -> Request {
    let record =
        Stored::Source { account: ANALYST, number: ANALYST, place: 0, kind: Kind::SignIn, name: Box::new([]), held };
    Request::Save { record }
}
/// Limited until a reset, on the world's wall clock, which starts at its epoch.
fn until(seconds: u64) -> Held {
    Held::Limited { reset: Some(Wall::from_nanos(seconds * 1_000_000_000)) }
}
#[test]
fn every_configured_account_is_wanted_at_the_start() {
    let mut world = World::configured(&[ANALYST, 8]);
    assert_eq!(
        world.step(Event::Want),
        vec![
            Request::Save { record: Stored::Next { number: 9 } },
            Request::Signing(jig_domain_accounts::Signing::Restored),
            want(),
            Request::Component(ToComponent::Want { account: 8 })
        ],
        "the counter has moved past the configured accounts"
    );
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
        form: Form::Credential(Scheme::Bearer),
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
    assert_eq!(world.step(Event::Limited { account: ANALYST, reset: Some(at(30)) }), vec![kept(Some(until(30)))]);
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.views()[0].state, State::Limited { until: Some(at(30)) });
    assert!(world.views()[0].attention);
    assert_eq!(world.fire_at(29), vec![]);
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.fire_at(30), vec![kept(None)], "the lifted limit is saved");
    assert!(world.domain.usable(ANALYST));
    assert_eq!(world.views()[0].state, State::Granted);
    assert!(!world.views()[0].attention);
}
#[test]
fn a_limit_without_a_reset_holds_until_resumed() {
    let mut world = granted();
    let held = Some(Held::Limited { reset: None });
    assert_eq!(world.step(Event::Limited { account: ANALYST, reset: None }), vec![kept(held)]);
    assert_eq!(world.views()[0].state, State::Limited { until: None });
    assert!(world.views()[0].attention);
    assert_eq!(world.step(Event::Limited { account: ANALYST, reset: Some(at(10)) }), vec![], "a hold unchanged");
    assert_eq!(world.fire_at(10), vec![]);
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(
        world.step(Event::Change {
            request: skein_lib::Token::new(0),
            holder: jig_domain_accounts::Holder::Deployment,
            change: jig_domain_accounts::Change::Resume { source: ANALYST }
        }),
        vec![
            kept(None),
            Request::Answer { request: skein_lib::Token::new(0), answer: jig_domain_accounts::Answer::Done }
        ]
    );
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
    assert_eq!(world.step(refused), vec![kept(Some(Held::Refused))]);
    assert_eq!(world.views()[0].state, State::Refused);
    assert!(world.views()[0].attention);
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(
        world.step(Event::Change {
            request: skein_lib::Token::new(0),
            holder: jig_domain_accounts::Holder::Deployment,
            change: jig_domain_accounts::Change::Resume { source: ANALYST }
        }),
        vec![Request::Answer {
            request: skein_lib::Token::new(0),
            answer: jig_domain_accounts::Answer::Refused { refusal: jig_domain_accounts::Refusal::NotLimited }
        }]
    );
    assert_eq!(world.fire_at(1000), vec![]);
    assert_eq!(world.views()[0].state, State::Refused);
}
#[test]
fn a_failed_runs_limited_account_marks_it_limited() {
    let mut world = granted();
    let failed = Event::RunFailed { account: ANALYST, turned: Turned::Limited { reset: Some(at(12)) } };
    assert_eq!(world.step(failed), vec![kept(Some(until(12)))]);
    assert_eq!(world.views()[0].state, State::Limited { until: Some(at(12)) });
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.fire_at(12), vec![kept(None)]);
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
            grant: Grant { form: Form::Application, account: 7, generation: 2, valid: Duration::from_secs(100) }
        }]
    );
}

#[test]
fn a_sign_in_sources_grant_carries_the_form_its_component_announced() {
    for form in [Form::Credential(Scheme::Bearer), Form::Credential(Scheme::Key), Form::Withdrawn] {
        let mut world = World::wanting(&[ANALYST]);
        for generation in [4, 5] {
            let grant = Grant { form, account: ANALYST, generation, valid: Duration::from_secs(100) };
            assert_eq!(
                world.step(Event::Component(FromComponent::Granted {
                    account: ANALYST,
                    generation,
                    valid: grant.valid,
                    form,
                })),
                vec![Request::Granted { grant }]
            );
            assert_eq!(world.domain.grant(ANALYST, world.now), Some(grant));
        }
    }
}

#[test]
fn a_component_grant_in_the_application_form_is_ignored() {
    let mut world = World::wanting(&[ANALYST]);
    assert!(
        world
            .step(Event::Component(FromComponent::Granted {
                account: ANALYST,
                generation: 4,
                valid: Duration::from_secs(100),
                form: Form::Application,
            }))
            .is_empty()
    );
    assert!(!world.domain.usable(ANALYST));
    assert_eq!(world.domain.grant(ANALYST, world.now), None);
    assert_eq!(world.views()[0].state, State::Asking);
}
