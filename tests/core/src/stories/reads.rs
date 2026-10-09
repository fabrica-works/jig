//! Fixtures for durable document and owner-page reads.
use crate::{
    effects::World,
    reads::{Parties, ask, latest},
};
use jig_core::{self as core, reads};
use jig_core_people as people;
use jig_core_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

#[must_use]
pub fn world(seed: u64) -> World {
    let (configuration, mut limits) = crate::effects::fixture(seed, false);
    limits.core.documents.rows = 2;
    limits.core.people.inbox_entries = 1;
    limits.core.load_slots = 1;
    limits.core.tasks.resource_reports = 4;
    limits.core.people.requests = 8;
    limits.core.views.watchers = 1;
    limits.core.views.watchers_per_party = 1;
    limits.core.views.subjects_per_change = 1;
    let room = core::room_max(&limits.core).expect("read story room");
    let connectors = jig_test_connector::MAX_OUT * 2;
    limits.journal.commits = 2;
    limits.journal.writes = room.writes + connectors;
    limits.journal.held = (room.held + connectors) * 2;
    let mut world = World::configured(seed, false, configuration, limits);
    world.reads = Some(Parties::new(world.store.applied));
    world
}

pub fn request(world: &mut World, number: u8, ask: people::Ask) {
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(30_000 + u64::from(number))),
        sign_in: world.sign_in.expect("signed in"),
        key: [number; 16],
        ask,
    })));
}

#[must_use]
pub fn conversation(world: &mut World, task: u64, first: u64) -> Vec<tasks::ActivityLine> {
    let mut before = None;
    let mut lines = Vec::new();
    for page in 0..32 {
        ask(world, first + page, reads::Read::Activity { task, before: before.take() });
        let reads::Answer::Page(value) = latest(world) else { panic!("activity page") };
        assert!(value.rows.len() <= 2);
        for row in &value.rows {
            let reads::PageRow::Activity(line) = row else { panic!("original activity") };
            lines.push(line.clone());
        }
        before.clone_from(&value.next);
        if before.is_none() {
            return lines;
        }
    }
    panic!("bounded conversation ended");
}
