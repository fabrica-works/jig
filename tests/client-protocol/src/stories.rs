//! The same people's actions and expected outcomes as the joint stories.
use crate::{Protocol, fixtures};
use jig_client_world::{ClientId, Settings, Test, Until, World, stories};
use skein_fake_person::{Find, Step};
use skein_lib::{Duration, Time};
use skein_ui::Role;
pub const SEEDS: &[u64] = &[8510, 8511];
pub type Story = World<Test, Protocol>;
#[must_use]
pub fn start() -> Vec<Step<jig_client_test_domain::Address>> {
    let mut steps = fixtures::sign_in_then(stories::start_chat_steps());
    steps.push(Step::See { find: Find::text(b"Hello"), within: Duration::from_secs(2) });
    steps.push(Step::See { find: Find::exactly_named(Role::Button, b"Stop"), within: Duration::from_secs(1) });
    steps
}
#[must_use]
pub fn ready(seed: u64) -> (Story, ClientId) {
    let mut world = fixtures::world(seed);
    let client = world.client(1, stories::conversation());
    world.person(client, start());
    finish(&mut world);
    (world, client)
}
pub fn finish(world: &mut Story) {
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    assert!(world.front().rules().records > 0 && world.front().rules().domains > 0);
}
pub fn send_words(world: &mut Story, client: ClientId, words: &[u8]) {
    world.person(
        client,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Write") },
            Step::Type { find: Find::named(Role::TextArea, b"Your answer"), words: words.into() },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(words), within: Duration::from_secs(2) },
        ],
    );
}
pub fn submit_stop(world: &mut Story, client: ClientId) {
    world.person(
        client,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Stop") },
            Step::See { find: Find::text(b"Stop the live run and hold its task"), within: Duration::from_secs(1) },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::Pause { span: Duration::from_millis(10) },
        ],
    );
    let made = world.front().requests_made();
    let cuts = world.front().cuts();
    for _ in 0..32 {
        let boundary = world.now().saturating_add(Duration::from_nanos(1));
        world.run(Until::Time(boundary));
        if world.front().requests_made() > made || world.front().cuts() > cuts {
            return;
        }
    }
    panic!("the stop crossed its transport boundary");
}
pub fn held(world: &mut Story, client: ClientId) {
    world.person(client, vec![Step::See { find: Find::text(b"Held stopped"), within: Duration::from_secs(5) }]);
    finish(world);
}
#[must_use]
pub fn pressure() -> jig_fake_parties::Party {
    use jig_fake_parties::{Act, Ask, Party, Role};
    Party::new(
        1,
        Role::Owner,
        1_000_000,
        Box::new([
            Act::SignIn { provider: 0, subject: b"ordinary pressure".as_slice().into() },
            Act::Request { key: [61; 16], ask: Ask::SeeResults { through: 0 } },
        ]),
    )
}
pub fn settle(world: &mut Story, span: Duration) {
    let boundary = world.now().saturating_add(span);
    world.run(Until::Time(boundary));
}
#[must_use]
pub fn after(world: &Story, span: Duration) -> Time {
    world.now().saturating_add(span)
}

#[must_use]
pub fn two_clients(seed: u64) -> (Story, ClientId) {
    let mut limits = fixtures::owner_limits();
    limits.core.views.watchers = 8;
    let mut world = World::new(Settings { seed, ..Settings::default() }, Protocol::bounded(seed, &limits));
    let client = world.client(1, stories::conversation());
    world.person(client, start());
    finish(&mut world);
    (world, client)
}

pub fn transient_request(seed: u64, fault: jig_client_world::FrontFault) {
    use jig_client_test_domain::{Ask, Event, Request};
    use jig_client_world::Front;
    use skein_lib::{RequestEnvelope, Token};
    let mut limits = fixtures::owner_limits();
    limits.core.people.requests = 1;
    limits.core.people.request_retention = Duration::from_millis(100);
    let mut protocol = Protocol::bounded(seed, &limits);
    let client = ClientId(0);
    Front::<Test>::bind(&mut protocol, client, 1);
    Front::<Test>::sign_in(&mut protocol, client, Time::ZERO, fixtures::wall(Time::ZERO));
    let _ = Front::<Test>::poll(&mut protocol, Time::ZERO, fixtures::wall(Time::ZERO));
    Front::<Test>::fault(&mut protocol, Time::ZERO, fault);
    let request = |attempt| Request::Send {
        attempt,
        key: [62; 16],
        ask: Ask::Jig(Box::new(jig_domain_client::contract::Ask::Stop { project: 1, task: u64::MAX })),
    };
    Front::<Test>::wire(&mut protocol, client, Time::ZERO, fixtures::wall(Time::ZERO), request(Token::new(31)));
    let events = Front::<Test>::poll(&mut protocol, Time::ZERO, fixtures::wall(Time::ZERO));
    assert!(
        events.iter().any(|(_, event)| matches!(event, Event::Answer { envelope: RequestEnvelope::Again, .. })),
        "actual busy"
    );
    let later = Time::from_nanos(100_000_001);
    let _ = Front::<Test>::poll(&mut protocol, later, fixtures::wall(later));
    Front::<Test>::wire(&mut protocol, client, later, fixtures::wall(later), request(Token::new(32)));
    let events = Front::<Test>::poll(&mut protocol, later, fixtures::wall(later));
    assert!(
        events
            .iter()
            .any(|(_, event)| matches!(event, Event::Answer { envelope: RequestEnvelope::Final, answer: Some(_), .. })),
        "busy retry is a real final owner answer"
    );
    let observations = Front::<Test>::observed(&mut protocol);
    assert_eq!(
        observations
            .iter()
            .filter(|value| matches!(value, jig_client_world::harness::Observed::Made { key, .. } if *key == [62; 16]))
            .count(),
        1,
        "busy key is made once"
    );
}
