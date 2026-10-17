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

pub fn transient_request(seed: u64, fault: &jig_client_world::FrontFault) {
    crate::client::transient_request(seed, fault);
}
