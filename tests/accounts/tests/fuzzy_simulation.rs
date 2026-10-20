use jig_accounts_world::{ANALYST, World};
use jig_domain_accounts::{Event, Failure, Form, FromComponent, Place, Request, Scheme, ToComponent, Unread};
use skein_lib::{Duration, Rng};
fn respond(world: &mut World, rng: &mut Rng, requests: Vec<Request>, trace: &mut Vec<Request>) {
    for request in &requests {
        let response = match *request {
            Request::Component(ToComponent::Want { account }) => Some(match rng.below(7) {
                0 => FromComponent::Failed { account, failure: Failure::Later { after: None } },
                1 => FromComponent::Failed { account, failure: Failure::SignInAgain },
                2 => FromComponent::Failed { account, failure: Failure::Misconfigured },
                3 => FromComponent::Failed {
                    account,
                    failure: Failure::Unloaded { at: Place::File, why: Unread::Stalled },
                },
                4 => FromComponent::Failed { account, failure: Failure::Cancelled },
                _ => FromComponent::Granted {
                    form: Form::Credential(Scheme::Bearer),
                    account,
                    generation: 4,
                    valid: Duration::from_secs(40),
                },
            }),
            Request::Component(ToComponent::Rejected { account, generation }) => {
                Some(FromComponent::Expiring { account, generation })
            }
            Request::Component(ToComponent::Close) => Some(FromComponent::Closed),
            Request::Component(ToComponent::Release { .. })
            | Request::Signing(_)
            | Request::Answer { .. }
            | Request::Granted { .. }
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Closed => None,
        };
        if let Some(event) = response {
            trace.extend(world.step(Event::Component(event)));
        }
    }
    trace.extend(requests);
}
fn run(seed: u64) -> Vec<Request> {
    let mut world = World::default();
    let mut rng = Rng::new(seed);
    let mut trace = Vec::new();
    let wanted = world.step(Event::Want);
    respond(&mut world, &mut rng, wanted, &mut trace);
    for seconds in 1..160 {
        if rng.chance(40) {
            let requests = world.step(Event::Unauthorized { account: ANALYST, generation: 4 });
            respond(&mut world, &mut rng, requests, &mut trace);
        }
        if rng.chance(30) {
            let reset = match rng.below(4) {
                0 => None,
                _ => Some(world.now.saturating_add(Duration::from_secs(5))),
            };
            trace.extend(world.step(Event::Limited { account: ANALYST, reset }));
        }
        if rng.chance(20) {
            trace.extend(world.step(Event::Resume { account: ANALYST }));
        }
        let due = world.fire_at(seconds);
        respond(&mut world, &mut rng, due, &mut trace);
        if world.domain.usable(ANALYST) {
            assert!(world.domain.grant(ANALYST, world.now).is_some());
        }
    }
    let closing = world.step(Event::Close);
    respond(&mut world, &mut rng, closing, &mut trace);
    assert_eq!(world.domain.accounts(), 0);
    assert_eq!(world.domain.next_deadline(), None);
    assert!(world.step(Event::Want).is_empty());
    trace
}
#[test]
fn seeded_account_timers_and_faults_replay() {
    for seed in 0..64 {
        assert_eq!(run(seed), run(seed), "seed {seed}");
    }
}
