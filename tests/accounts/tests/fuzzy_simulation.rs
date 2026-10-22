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
            Request::Programs { .. }
            | Request::CheckProgram { .. }
            | Request::SignOutProgram { .. }
            | Request::Component(ToComponent::Release { .. })
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
            trace.extend(world.step(Event::Change {
                request: skein_lib::Token::new(0),
                holder: jig_domain_accounts::Holder::Deployment,
                change: jig_domain_accounts::Change::Resume { source: ANALYST },
            }));
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

/// Every holder change, interleaved with component answers and source holds.
fn changes(seed: u64) -> Vec<Request> {
    use jig_accounts_world::{provider, providers};
    use jig_domain_accounts::{Answer, Change, Holder, Signing, Stored, Use};
    use skein_lib::Token;
    let mut world = World::with(providers(vec![provider(1, true, true)]), &[]);
    world.step(Event::Want);
    let mut rng = Rng::new(seed);
    let mut trace = Vec::new();
    for request in 0..160_u64 {
        let views = world.views();
        let chosen = views.get(
            usize::try_from(rng.below(u64::try_from(views.len().max(1)).expect("tiny views"))).expect("tiny index"),
        );
        let account = chosen.map_or(u32::MAX, |view| view.account);
        let source = chosen.map_or(u32::MAX, |view| view.source);
        let change = match rng.below(8) {
            0 => Change::AddKey {
                provider: 1,
                join: if rng.chance(40) { Some(account) } else { None },
                carried: 1 << 63,
            },
            1 => Change::ReplaceKey { source, carried: 1 << 63 },
            2 => Change::RemoveKey { source },
            3 => Change::Resume { source },
            4 => Change::Remove { account },
            5 => Change::MoveAccount { account, place: u32::try_from(rng.below(4)).expect("tiny place") },
            6 => Change::MoveSource { account, source, place: u32::try_from(rng.below(4)).expect("tiny place") },
            _ => Change::SetUses { account, uses: if rng.chance(50) { Some(Box::new([Use::Party(7)])) } else { None } },
        };
        let outputs = world.step(Event::Change { request: Token::new(request), holder: Holder::Party(7), change });
        let mut pending = std::collections::VecDeque::from(outputs);
        while let Some(output) = pending.pop_front() {
            let event = match output {
                Request::Signing(Signing::Keep { source, .. }) => {
                    if rng.chance(25) {
                        Some(Event::Component(FromComponent::KeepFailed {
                            account: source,
                            why: jig_domain_accounts::KeyFailure::NotStored,
                        }))
                    } else {
                        let generation = world.domain.source(source).and_then(|s| s.generation).unwrap_or(0) + 1;
                        Some(Event::Component(FromComponent::Granted {
                            form: jig_domain_accounts::Form::Credential(jig_domain_accounts::Scheme::Key),
                            account: source,
                            generation,
                            valid: Duration::from_secs(100),
                        }))
                    }
                }
                Request::Signing(Signing::SignOut { source }) => {
                    Some(Event::SignedOut { source, confirmed: rng.chance(70) })
                }
                Request::Signing(
                    Signing::CancelSignIn { .. }
                    | Signing::SignIn { .. }
                    | Signing::Replaced { .. }
                    | Signing::Forget { .. }
                    | Signing::Restore { .. }
                    | Signing::Restored,
                )
                | Request::Programs { .. }
                | Request::CheckProgram { .. }
                | Request::SignOutProgram { .. }
                | Request::Component(_)
                | Request::Granted { .. }
                | Request::Save { .. }
                | Request::Erase { .. }
                | Request::Closed => None,
                Request::Answer { ref answer, .. } => {
                    assert!(!matches!(answer, Answer::Started { .. }));
                    None
                }
            };
            trace.push(output);
            if let Some(event) = event {
                pending.extend(world.step(event));
            }
        }
        if source != u32::MAX && rng.chance(20) {
            trace.extend(world.step(Event::Limited { account: source, reset: None }));
        }
        dense_and_bounded(&world);
        // Counter records carry only numbers; LLM-key values never enter the trace.
        for output in &trace {
            if let Request::Save { record: Stored::Next { number } } = output {
                assert!(*number <= world.domain.next_number());
            }
        }
    }
    trace.extend(world.step(Event::Close));
    trace.extend(world.step(Event::Component(FromComponent::Closed)));
    assert_eq!(world.domain.accounts(), 0);
    trace
}
fn dense_and_bounded(world: &World) {
    let views = world.views();
    let mut places = Vec::new();
    for view in &views {
        let account = world.domain.account(view.account).expect("account view");
        places.push(account.place());
        assert_eq!(
            account.sources().iter().map(|(p, _)| *p).collect::<Vec<_>>(),
            (0..account.sources().len()).collect::<Vec<_>>()
        );
        if let Some(uses) = account.uses() {
            assert!(uses.len() <= 16);
        }
    }
    places.sort_unstable();
    assert_eq!(places, (0..world.domain.accounts()).collect::<Vec<_>>());
    assert!(world.domain.accounts() <= jig_accounts_world::LIMITS.accounts);
}
#[test]
fn seeded_account_changes_replay() {
    for seed in 0..64 {
        assert_eq!(changes(seed), changes(seed), "seed {seed}");
    }
}
