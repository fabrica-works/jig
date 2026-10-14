//! One shared authentic core/worker/client fixture, with externally supplied seeds and bounds.
use super::{Joint, engine::Driver, session::Session};
use crate::{Settings, Test, World};
use jig_core_world::peers::Peers;
use jig_domain_client::contract as client;
use jig_domain_people as people;
use jig_fake_workers::{Script, Worker};
use skein_lib::{Duration, Time, Wall};
use std::collections::BTreeMap;

pub(super) fn clock(now: Time) -> Wall {
    Wall::from_nanos(1_000_000_000_000 + now.as_nanos())
}

pub(super) fn turn() -> Box<[u8]> {
    use smith_transcript::v3 as source;
    let limits = source::CEILINGS;
    let text = source::Text::new(&limits, source::TextParts { text: b"abc".as_slice().into(), replay: None })
        .expect("supplied worker text");
    let blocks = crate::test::list([source::Block::Text(text)]);
    let message = source::Message::new(&limits, source::MessageParts { role: source::Role::Assistant, blocks })
        .expect("supplied worker message");
    let usage = source::Usage::new(
        &limits,
        source::UsageParts {
            input: Some(1),
            output: Some(1),
            cache_read: Some(0),
            cache_write: Some(0),
            reasoning: None,
        },
    )
    .expect("supplied worker usage");
    let value = source::Turn::new(
        &limits,
        source::TurnParts {
            model: b"model".as_slice().into(),
            effort: None,
            kind: source::TurnKind::Ordinary,
            affinity: None,
            endpoint: b"fixture".as_slice().into(),
            dialect: b"fixture".as_slice().into(),
            place: 1,
            usage,
            spent: 1,
            messages: crate::test::list([message]),
        },
    )
    .expect("whole actual codec turn");
    let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).expect("measured turn fits"));
    value.encode(&mut writer).expect("actual codec encoding");
    writer.finish()
}

#[cfg(test)]
pub(super) fn world(body: Box<[u8]>) -> World<Test, Joint> {
    configured_world(body, |_| {})
}

#[cfg(test)]
pub(super) fn configured_world(body: Box<[u8]>, configure: fn(&mut Joint)) -> World<Test, Joint> {
    bounded_world(body, configure, |_| {})
}

#[cfg(test)]
pub(super) fn bounded_world(
    body: Box<[u8]>,
    configure: fn(&mut Joint),
    bounds: fn(&mut jig_test_domain::Limits),
) -> World<Test, Joint> {
    seeded_world(8510, body, configure, bounds)
}

#[cfg(test)]
pub(super) fn seeded_world(
    seed: u64,
    body: Box<[u8]>,
    configure: fn(&mut Joint),
    bounds: fn(&mut jig_test_domain::Limits),
) -> World<Test, Joint> {
    seeded_world_with_receiving(seed, body, configure, bounds, crate::test::limits())
}

pub(super) fn seeded_world_with_receiving(
    seed: u64,
    body: Box<[u8]>,
    configure: fn(&mut Joint),
    bounds: fn(&mut jig_test_domain::Limits),
    receiving: jig_test_domain_client::Limits,
) -> World<Test, Joint> {
    let mut owner = jig_core_world::world::limits();
    bounds(&mut owner);
    owner.core.documents.rows = 2;
    owner.core.documents.lines = 2;
    owner.core.documents.chips = 2;
    owner.core.documents.text_bytes = 32;
    let room = jig_domain_core::room_max(&owner.core).expect("declared whole core bounds");
    let connectors = jig_test_connector::MAX_OUT.checked_mul(2).expect("two connector routes");
    owner.journal.writes = owner.journal.writes.max(room.writes.checked_add(connectors).expect("whole writes"));
    owner.journal.held = owner.journal.held.max(
        room.held
            .checked_add(connectors)
            .expect("whole held deliveries")
            .checked_mul(owner.journal.commits)
            .expect("bounded journal flights"),
    );
    let worker = Worker::new(
        1,
        1,
        vec![Box::new([
            Script::Silence { for_: Duration::from_millis(100) },
            Script::Text { turn: 1, offset: 0, text: b"abc".as_slice().into() },
            Script::Silence { for_: Duration::from_millis(300) },
            Script::Turn { body, cost: 1, read: None },
            Script::Wait,
        ])],
    );
    let driver = Driver::new(
        jig_core_world::world::config(seed),
        owner,
        Peers::new(vec![worker], Vec::new()),
        Time::ZERO,
        clock(Time::ZERO),
    );
    let codec = jig_protocol_transcript::Limits {
        smith: smith_transcript::v3::CEILINGS,
        bytes: 1_048_576,
        turn: client::TurnLimits { messages: 2, blocks: 2, text: 32 },
    };
    let identity = people::Identity {
        key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
        login: b"login".as_slice().into(),
        name: b"Person".as_slice().into(),
    };
    let mut front = Joint::new(Session::new(driver, receiving, codec), BTreeMap::from([(1, identity)]), clock);
    front.unsigned_on_bind();
    front.session.stream_heartbeat(Duration::from_millis(50));
    front.session.capture_task_offers();
    configure(&mut front);
    World::new(Settings { seed, ..Settings::default() }, front)
}
