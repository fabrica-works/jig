//! Delayed shell acknowledgements remain tied to the platform move that made
//! them (client/domain/view.md, 11; client/domain/testing.md, 4).
use super::{ClientId, Settings, Until, World};
use crate::{
    harness::{Front, FrontFault, Observed},
    test::Test,
};
use jig_client_test_domain::{Address, Event, Place, Request};
use jig_domain_client::address::ProjectSection;
use skein_lib::{Time, Wall};
use skein_world::domain::{Span, Verdict};

#[derive(Default)]
struct Receipts {
    went: Vec<(ClientId, Address)>,
    hello: u32,
}
impl Front<Test> for Receipts {
    fn bind(&mut self, _client: ClientId, _party: u64) {}
    fn wire(&mut self, _client: ClientId, _now: Time, _wall: Wall, _request: Request) {}
    fn poll(&mut self, _now: Time, _wall: Wall) -> Vec<(ClientId, Event)> {
        Vec::new()
    }
    fn next_due(&self) -> Option<Time> {
        None
    }
    fn observed(&mut self) -> Vec<Observed> {
        Vec::new()
    }
    fn fault(&mut self, _now: Time, _fault: FrontFault) {}
    fn reload(&mut self, _client: ClientId, _now: Time) {}
    fn received(&mut self, client: ClientId, event: &Event) {
        if matches!(event, Event::Jig(jig_domain_client::Event::Hello { .. })) {
            self.hello += 1;
        }
        if let Event::Went { place: Place::Address(address) } = event {
            self.went.push((client, *address));
        }
    }
    fn sign_in(&mut self, _client: ClientId, _now: Time, _wall: Wall) {}
    fn sign_out(&mut self, _: ClientId, _: Time, _: Wall) {}
}
fn world() -> (World<Test, Receipts>, ClientId) {
    let mut world = World::new(Settings { latency: Span::millis(2, 2), ..Settings::default() }, Receipts::default());
    let client = world.client(7, crate::stories::inbox());
    (world, client)
}
fn project(section: ProjectSection) -> Address {
    Address::Jig(jig_domain_client::address::Address::Project { project: 1, section })
}
fn drain(world: &mut World<Test, Receipts>) -> super::Outcome {
    let outcome = world.run(Until::Steps(2));
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(world.schedule.is_empty(), "all shell acknowledgements settled");
    outcome
}

#[test]
fn the_current_show_is_delivered_after_latency_once_despite_repeated_shows() {
    let (mut world, client) = world();
    let address = project(ProjectSection::Settings);
    world.platform_show(client, address, false);
    world.platform_show(client, address, false);
    world.platform_show(client, address, true);
    let outcome = world.run(Until::Steps(1));
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(world.front().went.is_empty(), "Went is not delivered synchronously");
    assert_eq!(world.schedule.next_time(), Some(Time::from_nanos(2_000_000)));
    let outcome = drain(&mut world);
    assert_eq!(outcome.dropped, 0);
    assert_eq!(world.front().went, [(client, address)]);
    assert_eq!(world.clients[client.0].history, [crate::stories::inbox(), address]);
    assert_eq!(world.clients[client.0].at, 1);
}

#[test]
fn a_newer_show_supersedes_an_older_callback_in_the_same_reload_epoch() {
    let (mut world, client) = world();
    let overview = project(ProjectSection::Overview);
    let settings = project(ProjectSection::Settings);
    world.platform_show(client, overview, false);
    world.platform_show(client, settings, false);
    let outcome = drain(&mut world);
    assert_eq!(outcome.dropped, 1);
    assert_eq!(world.front().went, [(client, settings)]);
    assert_eq!(world.clients[client.0].history, [crate::stories::inbox(), overview, settings]);
    assert_eq!(world.clients[client.0].at, 2);
}

#[test]
fn returning_to_an_address_does_not_revive_its_older_callback() {
    let (mut world, client) = world();
    let overview = project(ProjectSection::Overview);
    let settings = project(ProjectSection::Settings);
    world.platform_show(client, overview, false);
    world.platform_show(client, settings, false);
    world.platform_show(client, overview, false);
    let outcome = drain(&mut world);
    assert_eq!(outcome.dropped, 2);
    assert_eq!(world.front().went, [(client, overview)]);
    assert_eq!(world.clients[client.0].history, [crate::stories::inbox(), overview, settings, overview]);
    assert_eq!(world.clients[client.0].at, 3);
}

#[test]
fn direct_navigation_invalidates_a_pending_show_without_losing_the_new_history_entry() {
    let (mut world, client) = world();
    let overview = project(ProjectSection::Overview);
    let settings = project(ProjectSection::Settings);
    world.platform_show(client, overview, false);
    let before = world.callbacks;
    world.go(client, settings);
    assert_eq!(world.callbacks, before + 1, "direct Go reaches the root immediately");
    let outcome = drain(&mut world);
    assert_eq!(outcome.dropped, 1);
    assert!(world.front().went.is_empty(), "direct Go already delivered its one callback");
    assert_eq!(world.clients[client.0].history, [crate::stories::inbox(), overview, settings]);
    assert_eq!(world.clients[client.0].at, 2);
}

#[test]
fn back_and_forward_keep_history_and_supersede_pending_shows() {
    let (mut world, client) = world();
    let overview = project(ProjectSection::Overview);
    let settings = project(ProjectSection::Settings);
    world.platform_show(client, overview, false);
    world.platform_show(client, settings, false);
    let before = world.callbacks;
    world.back(client);
    assert_eq!(world.callbacks, before + 1, "Back reaches the root immediately");
    assert_eq!(world.clients[client.0].at, 1);
    world.forward(client);
    assert_eq!(world.callbacks, before + 2, "Forward reaches the root immediately");
    assert_eq!(world.clients[client.0].at, 2);
    let outcome = drain(&mut world);
    assert_eq!(outcome.dropped, 2);
    assert!(world.front().went.is_empty(), "history callbacks were delivered synchronously");
    assert_eq!(world.clients[client.0].history, [crate::stories::inbox(), overview, settings]);
}

#[test]
fn a_reload_still_discards_its_old_epoch_navigation_callback() {
    let (mut world, client) = world();
    let settings = project(ProjectSection::Settings);
    world.platform_show(client, settings, false);
    world.reload(client);
    let outcome = drain(&mut world);
    assert_eq!(outcome.dropped, 1);
    assert!(world.front().went.is_empty());
    assert_eq!(world.clients[client.0].epoch, 1);
    assert_eq!(world.clients[client.0].history, [crate::stories::inbox(), settings]);
    assert_eq!(world.clients[client.0].at, 1);
}

#[test]
fn one_tabs_new_show_does_not_supersede_another_tabs_callback() {
    let (mut world, first) = world();
    let second = world.tab(first);
    let overview = project(ProjectSection::Overview);
    let settings = project(ProjectSection::Settings);
    world.platform_show(first, overview, false);
    world.platform_show(second, settings, false);
    world.platform_show(first, settings, false);
    let outcome = drain(&mut world);
    assert_eq!(outcome.dropped, 1);
    assert_eq!(world.front().went, [(second, settings), (first, settings)]);
}

#[test]
fn an_older_show_arriving_after_the_current_one_cannot_reenter_the_old_page() {
    let (mut world, client) = world();
    let overview = project(ProjectSection::Overview);
    let settings = project(ProjectSection::Settings);
    world.platform_show(client, overview, false);
    world.settings.latency = Span::millis(1, 1);
    world.platform_show(client, settings, false);
    let outcome = world.run(Until::Steps(3));
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(world.schedule.is_empty());
    assert_eq!(outcome.dropped, 1);
    assert_eq!(world.front().went, [(client, settings)]);
    assert_eq!(world.clients[client.0].history, [crate::stories::inbox(), overview, settings]);
    assert_eq!(world.clients[client.0].at, 2);
}

#[test]
fn replacing_a_changed_address_keeps_the_history_slot_and_supersedes_its_callback() {
    let (mut world, client) = world();
    let overview = project(ProjectSection::Overview);
    let settings = project(ProjectSection::Settings);
    world.platform_show(client, overview, false);
    world.platform_show(client, settings, true);
    let outcome = drain(&mut world);
    assert_eq!(outcome.dropped, 1);
    assert_eq!(world.front().went, [(client, settings)]);
    assert_eq!(world.clients[client.0].history, [crate::stories::inbox(), settings]);
    assert_eq!(world.clients[client.0].at, 1);
}

#[test]
fn a_new_navigation_does_not_discard_a_current_epoch_engine_callback() {
    let (mut world, client) = world();
    let settings = project(ProjectSection::Settings);
    world.schedule.send(
        Time::from_nanos(2_000_000),
        super::Due {
            client,
            epoch: 0,
            event: Event::Jig(jig_domain_client::Event::Hello { served: true }),
            navigation: None,
        },
    );
    world.platform_show(client, settings, false);
    let outcome = drain(&mut world);
    assert_eq!(outcome.dropped, 0);
    assert_eq!(world.front().hello, 1, "the engine event retains only its reload epoch fence");
    assert_eq!(world.front().went, [(client, settings)]);
}
