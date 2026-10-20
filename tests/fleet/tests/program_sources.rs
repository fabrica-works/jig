use jig_domain_fleet::{self as fleet, Event, Request};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
const CHANNEL: Token = Token::new(101);
struct World {
    domain: fleet::Domain,
    env: Env<fleet::Limits>,
    out: Queue<Request>,
}
impl World {
    fn new(names: &[&[u8]]) -> Self {
        let limits = fleet::Limits { program_sources: 2, ..jig_fleet_world::LIMITS };
        let mut world = Self {
            domain: fleet::Domain::new(&limits, jig_fleet_world::fixtures::needs()),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::with_capacity(fleet::max_out(&limits)),
        };
        assert!(world.step(Event::Sources { sources: names.iter().map(|name| source(name)).collect() }).is_empty());
        world
    }
    fn step(&mut self, event: Event) -> Vec<Request> {
        fleet::step(&mut self.domain, &self.env, event, &mut self.out);
        let mut requests = Vec::new();
        while let Some(request) = self.out.pop() {
            requests.push(request);
        }
        self.domain.reclaim();
        requests
    }
    fn hello(&mut self, channel: Token) -> Vec<Request> {
        self.step(Event::Hello {
            channel,
            hello: fleet::Hello {
                slots: 2,
                stop_bound: Duration::from_secs(1),
                workstreams: Box::new([]),
                hosting: Box::new([]),
                bounds: jig_fleet_world::fixtures::bounds(),
            },
        })
    }
    fn inventory(&mut self, names: &[&[u8]]) -> Vec<Request> {
        self.step(Event::Logins {
            channel: CHANNEL,
            logins: names
                .iter()
                .map(|name| fleet::Login { name: source(name), state: fleet::LoginState::NotSignedIn })
                .collect(),
        })
    }
    fn available(&self) -> bool {
        let available = self.domain.free_kind(fleet::Kinds::Workers, 1, 0).is_some();
        assert_eq!(self.domain.probe(fleet::Kinds::Workers, 1, &[], &[]).is_some(), available);
        assert_eq!(
            self.domain
                .placement_available(fleet::PlacementConstraint { channel: CHANNEL, kind: fleet::HostKind::Worker }),
            available
        );
        available
    }
}
fn source(name: &[u8]) -> fleet::SourceName {
    fleet::SourceName::new(name).expect("fixture source")
}
#[test]
fn even_empty_inventory_is_a_placement_barrier() {
    let mut world = World::new(&[]);
    assert!(matches!(&world.hello(CHANNEL)[..], [Request::Sources { sources, .. }] if sources.is_empty()));
    assert!(!world.available());
    assert!(world.domain.awaiting_logins(CHANNEL));
    assert!(world.inventory(&[]).is_empty());
    assert!(world.available());
    assert!(!world.domain.awaiting_logins(CHANNEL));
}
#[test]
fn old_whole_snapshot_crossing_a_replacement_updates_nothing() {
    let mut world = World::new(&[b"a", b"b"]);
    world.hello(CHANNEL);
    world.inventory(&[b"a", b"b"]);
    assert!(world.available());
    // Reordering the same set cannot start an indistinguishable barrier.
    assert!(world.step(Event::Sources { sources: Box::new([source(b"b"), source(b"a")]) }).is_empty());
    assert!(world.available());
    let sent = world.step(Event::Sources { sources: Box::new([source(b"a")]) });
    assert!(matches!(&sent[..], [Request::Sources { sources, .. }] if sources.as_ref() == [source(b"a")]));
    assert!(!world.available());
    assert!(
        world
            .step(Event::Logins {
                channel: CHANNEL,
                logins: Box::new([
                    fleet::Login { name: source(b"a"), state: fleet::LoginState::Usable },
                    fleet::Login { name: source(b"b"), state: fleet::LoginState::Usable },
                ])
            })
            .is_empty()
    );
    assert!(
        world
            .domain
            .program_logins(CHANNEL)
            .expect("old inventory")
            .iter()
            .all(|row| row.state == fleet::LoginState::NotSignedIn)
    );
    assert!(!world.available());
    world.inventory(&[b"a"]);
    assert!(world.available());
}
#[test]
fn partial_foreign_or_duplicate_snapshots_fault_the_link() {
    for names in [&[b"a".as_slice()][..], &[b"a".as_slice(), b"foreign"], &[b"a".as_slice(), b"a"]] {
        let mut world = World::new(&[b"a", b"b"]);
        world.hello(CHANNEL);
        assert!(matches!(&world.inventory(names)[..], [Request::Refuse { channel: CHANNEL }]));
        assert!(!world.available());
        assert_eq!(world.domain.program_logins(CHANNEL), None);
    }
}
#[test]
fn only_absence_lifts_the_conservative_personal_login_restriction() {
    for state in [
        fleet::LoginState::Usable,
        fleet::LoginState::SignedInOtherwise(jig_domain_fleet::LoginMethod::new(b"api-key").expect("method")),
        fleet::LoginState::Failed { reason: jig_domain_fleet::SourceFailure::Exited, detail: Box::new([]) },
        fleet::LoginState::Failed { reason: jig_domain_fleet::SourceFailure::Deadline, detail: Box::new([]) },
    ] {
        let mut world = World::new(&[b"a"]);
        world.hello(CHANNEL);
        world.step(Event::Logins { channel: CHANNEL, logins: Box::new([fleet::Login { name: source(b"a"), state }]) });
        assert!(!world.available());
        world.inventory(&[b"a"]);
        assert!(world.available());
    }
}
#[test]
fn an_unconfigured_program_counts_as_absent() {
    let mut world = World::new(&[b"a"]);
    world.hello(CHANNEL);
    world.step(Event::Logins {
        channel: CHANNEL,
        logins: Box::new([fleet::Login {
            name: source(b"a"),
            state: fleet::LoginState::Failed { reason: fleet::SourceFailure::Unconfigured, detail: Box::new([]) },
        }]),
    });
    assert!(world.available());
}
#[test]
fn an_unawaited_sign_out_faults_the_link() {
    let mut world = World::new(&[b"a"]);
    world.hello(CHANNEL);
    world.inventory(&[b"a"]);
    assert_eq!(
        world.step(Event::SignedOut { channel: CHANNEL, name: source(b"a"), result: fleet::SignOutResult::SignedOut }),
        vec![Request::Refuse { channel: CHANNEL }]
    );
    assert!(!world.available());
}
#[test]
fn a_lost_sign_out_cannot_complete_a_new_channel_call() {
    let mut world = World::new(&[b"a"]);
    world.hello(CHANNEL);
    world.inventory(&[b"a"]);
    assert!(matches!(
        &world.step(Event::SignOut { to: ReplyTo::new(Token::new(1)), channel: CHANNEL, name: source(b"a") })[..],
        [Request::SignOut { .. }]
    ));
    assert!(matches!(&world.step(Event::Lost { channel: CHANNEL })[..], [Request::SignedOut { result: None, .. }]));
    assert!(world.step(Event::Lost { channel: CHANNEL }).is_empty());
    let replacement = Token::new(202);
    assert!(matches!(&world.hello(replacement)[..], [Request::Sources { .. }]));
    assert!(!world.available());
    world.step(Event::SignOut { to: ReplyTo::new(Token::new(2)), channel: replacement, name: source(b"a") });
    assert!(
        world
            .step(Event::SignedOut { channel: CHANNEL, name: source(b"a"), result: fleet::SignOutResult::SignedOut })
            .is_empty()
    );
    let ended = world.step(Event::SignedOut {
        channel: replacement,
        name: source(b"a"),
        result: fleet::SignOutResult::SignedOut,
    });
    let Some(Request::SignedOut { to, result: Some(fleet::SignOutResult::SignedOut), .. }) = ended.into_iter().next()
    else {
        panic!("sign-out completion")
    };
    assert_eq!(to.into_token(), Token::new(2));
}
