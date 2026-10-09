//! Fixtures for durable document and owner-page reads.
use crate::{
    effects::World,
    reads::{Parties, ask, latest},
};
use jig_core::{self as core, documents, reads};
use jig_core_people as people;
use jig_core_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token};

#[must_use]
pub fn world(seed: u64) -> World {
    read_world(seed, None)
}

fn read_world(seed: u64, reply_bound: Option<u32>) -> World {
    let (configuration, mut limits) = crate::effects::fixture(seed, false);
    if let Some(bound) = reply_bound {
        limits.core.tasks.inbox_messages = bound;
        limits.core.people.inbox_entries = bound;
    }
    limits.core.documents.rows = 2;
    if reply_bound.is_none() {
        limits.core.people.inbox_entries = 1;
    }
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

/// A temporal cache prefix must remain a prefix even when a later turn has an earlier wall time.
pub fn a_backward_wall_correction_keeps_reply_paging_whole_after_restart() {
    let mut world = read_world(766, Some(2));
    let task = world.assigned.as_ref().expect("chat").task;
    world.advance(Duration::from_secs(2));
    world.turn(1, 1, None, b"before correction one");
    world.turn(2, 2, None, b"before correction two");
    world.correct_wall_backwards(Duration::from_secs(1));
    for turn in 3..=5 {
        world.turn(turn, u64::from(turn), None, b"after correction");
    }
    let activity = conversation(&mut world, task, 210);
    let mut expected = activity
        .iter()
        .filter_map(|line| match line.line {
            tasks::Line::Turn { reply, .. } => Some((line.at, reply, line.position)),
            tasks::Line::Words { .. }
            | tasks::Line::Heard(_)
            | tasks::Line::Notified { .. }
            | tasks::Line::RunBegan { .. }
            | tasks::Line::RunEnded { .. }
            | tasks::Line::Decided { .. }
            | tasks::Line::Milestone { .. } => None,
        })
        .collect::<Vec<_>>();
    expected.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(expected.iter().map(|row| row.1).collect::<Vec<_>>(), [2, 1, 5, 4, 3]);
    world.restart(766, false);
    let mut found = Vec::new();
    let mut from = None;
    for page in 0..8 {
        ask(
            &mut world,
            250 + page,
            reads::Read::Inbox {
                group: reads::Group::Updates,
                filter: reads::Filter { project: Some(1), goal: None },
                from: from.take(),
            },
        );
        let reads::Answer::Page(value) = latest(&world) else { panic!("reply inbox page") };
        assert!(value.rows.len() <= 2);
        for row in &value.rows {
            let reads::PageRow::Inbox(entry) = row else { panic!("reply row") };
            let documents::InboxObject::Reply { message } = entry.object else { panic!("reply ordinal") };
            let conversation = entry.reply.as_ref().expect("whole committed conversation");
            assert_eq!(conversation.last_reply, 5, "the ordinal is independent of the retained prefix");
            assert_eq!(conversation.seen, 0);
            found.push((entry.at.expect("reply wall time"), message, entry.activity.expect("immutable load position")));
        }
        from.clone_from(&value.next);
        if from.is_none() {
            break;
        }
    }
    assert_eq!(found, expected, "cold live and stored pages return every reply exactly once in wall order");
}
