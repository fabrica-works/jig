//! People act on the public tree over actual core, worker and applied store doors
//! (client/domain/testing.md, sections 5–6; domain/people.md, section 11).
use super::{Joint, engine::Driver, session::Session};
use crate::{Settings, Test, Until, World};
use jig_client::contract as client;
use jig_client_view::Role;
use jig_core_people as people;
use jig_core_world::peers::Peers;
use jig_fake_person::{Find, Step};
use jig_fake_workers::{Script, Worker};
use skein_lib::{Duration, Time, Token, Wall};
use std::collections::BTreeMap;

fn clock(now: Time) -> Wall {
    Wall::from_nanos(1_000_000_000_000 + now.as_nanos())
}

fn turn() -> Box<[u8]> {
    use smith_transcript::v2 as source;
    let limits = source::CEILINGS;
    let text = source::Text::new(&limits, source::TextParts { text: b"abc".as_slice().into(), replay: None })
        .expect("supplied worker text");
    let blocks = crate::test::list([source::Block::Text(text)]);
    let message = source::Message::new(&limits, source::MessageParts { role: source::Role::Assistant, blocks })
        .expect("supplied worker message");
    let usage = source::Usage::new(&limits, source::UsageParts { input: 1, output: 1, cache_read: 0, cache_write: 0 })
        .expect("supplied worker usage");
    let value = source::Turn::new(
        &limits,
        source::TurnParts {
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

fn world(body: Box<[u8]>) -> World<Test, Joint> {
    configured_world(body, |_| {})
}

fn configured_world(body: Box<[u8]>, configure: fn(&mut Joint)) -> World<Test, Joint> {
    bounded_world(body, configure, |_| {})
}

fn bounded_world(
    body: Box<[u8]>,
    configure: fn(&mut Joint),
    bounds: fn(&mut jig_test_domain::Limits),
) -> World<Test, Joint> {
    let mut owner = jig_core_world::world::limits();
    bounds(&mut owner);
    owner.core.documents.rows = 2;
    owner.core.documents.lines = 2;
    owner.core.documents.chips = 2;
    owner.core.documents.text_bytes = 32;
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
        jig_core_world::world::config(8510),
        owner,
        Peers::new(vec![worker], Vec::new()),
        Time::ZERO,
        clock(Time::ZERO),
    );
    let codec = jig_client_transcript::Limits {
        smith: smith_transcript::v2::CEILINGS,
        bytes: 1_048_576,
        turn: client::TurnLimits { messages: 2, blocks: 2, text: 32 },
    };
    let identity = people::Identity {
        key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
        login: b"login".as_slice().into(),
        name: b"Person".as_slice().into(),
    };
    let mut front =
        Joint::new(Session::new(driver, crate::test::limits(), codec), BTreeMap::from([(1, identity)]), clock);
    front.unsigned_on_bind();
    front.session.stream_heartbeat(Duration::from_millis(50));
    front.session.capture_task_offers();
    configure(&mut front);
    World::new(Settings { seed: 8510, ..Settings::default() }, front)
}

fn report(
    world: &mut World<Test, Joint>,
    assignment: &jig_fake_workers::Assignment,
    activity: jig_core_fleet::Activity,
) {
    let now = world.now();
    world.front().session.driver.send(
        now,
        clock(now),
        jig_test_domain::Event::Core(jig_core::Event::Fleet(jig_core_fleet::Event::Observe {
            channel: Token::new(1),
            run: Token::new(assignment.task),
            attempt: Token::new(assignment.attempt),
            report: jig_core_fleet::Report::Activity(activity),
        })),
    );
}

fn same_pattern(source: &jig_core::documents::GrantLine, native: &client::OwnerGrant) {
    assert_eq!((source.connector, source.kind), (native.connector, native.kind));
    assert_eq!(source.segments.capacity(), native.pattern.segments.capacity());
    assert_eq!(source.segments.as_slice(), native.pattern.segments.as_slice());
    match (&source.last, &native.pattern.last) {
        (jig_core::documents::PatternEnd::Exact(a), client::OwnerPatternEnd::Exact(b))
        | (jig_core::documents::PatternEnd::Open(a), client::OwnerPatternEnd::Open(b)) => assert_eq!(a, b),
        pair => panic!("different whole pattern ends: {pair:?}"),
    }
}

fn same_authority(source: &jig_core::documents::AuthorityLine, native: &client::OwnerAuthorityDocument) {
    let authority = &native.authority;
    assert_eq!(source.cut, native.cut);
    assert_eq!((source.tools.0, source.notes.0), (authority.tools, authority.notes));
    assert_eq!((source.ceiling.spend, source.ceiling.deadline), (authority.spend, authority.deadline));
    assert_eq!((source.tasks, source.depth), (authority.delegation.tasks, authority.delegation.depth));
    assert_eq!(source.grants.capacity(), authority.grants.capacity());
    assert_eq!(source.grants.len(), authority.grants.len());
    for (a, b) in source.grants.iter().zip(authority.grants.iter()) {
        same_pattern(a, b);
    }
    assert_eq!(source.executors.capacity(), authority.delegation.kinds.capacity());
    assert_eq!(source.executors.len(), authority.delegation.kinds.len());
    for (a, b) in source.executors.iter().zip(authority.delegation.kinds.iter()) {
        match (a, b) {
            (jig_core_tasks::AuthorityExecutor::Charter(a), client::OwnerAuthorityExecutor::Charter(b))
            | (jig_core_tasks::AuthorityExecutor::Procedure(a), client::OwnerAuthorityExecutor::Procedure(b))
            | (jig_core_tasks::AuthorityExecutor::Role(a), client::OwnerAuthorityExecutor::Role(b)) => assert_eq!(a, b),
            pair => panic!("different executor permission: {pair:?}"),
        }
    }
    assert_eq!(source.note_resources.capacity(), authority.note_resources.capacity());
    assert_eq!(source.note_resources.len(), authority.note_resources.len());
    for (a, b) in source.note_resources.iter().zip(authority.note_resources.iter()) {
        assert_eq!(a.connector, b.connector);
        assert_eq!(a.segments.capacity(), b.pattern.segments.capacity());
        assert_eq!(a.segments.as_slice(), b.pattern.segments.as_slice());
        match (&a.last, &b.pattern.last) {
            (jig_core::documents::PatternEnd::Exact(a), client::OwnerPatternEnd::Exact(b))
            | (jig_core::documents::PatternEnd::Open(a), client::OwnerPatternEnd::Open(b)) => assert_eq!(a, b),
            pair => panic!("different whole note scope: {pair:?}"),
        }
    }
}

fn same_recipient(source: &jig_core::acts::Recipient, native: &client::Recipient) {
    match (source, native) {
        (jig_core::acts::Recipient::Task(a), client::Recipient::Task(b))
        | (jig_core::acts::Recipient::Party(a), client::Recipient::Party(b)) => assert_eq!(a, b),
        (
            jig_core::acts::Recipient::Role { project: a, role: ar },
            client::Recipient::Role { project: b, role: br },
        ) => {
            assert_eq!((a, ar), (b, br));
        }
        (
            jig_core::acts::Recipient::Roles { project: a, roles: ar },
            client::Recipient::Roles { project: b, roles: br },
        ) => {
            assert_eq!(a, b);
            assert_eq!(ar.capacity(), br.capacity());
            assert_eq!(ar.as_slice(), br.as_slice());
        }
        pair => panic!("different exact recipient: {pair:?}"),
    }
}

fn same_task_admission(source: &jig_core::acts::Admission, native: &client::Admission) {
    match (source, native) {
        (jig_core::acts::Admission::Allowed, client::Admission::Allowed) => {}
        (jig_core::acts::Admission::Proposed { to: a }, client::Admission::Proposed { to: b }) => same_recipient(a, b),
        (jig_core::acts::Admission::Refused(a), client::Admission::Refused(b)) => {
            assert_eq!(super::wire::refusals::refusal(*a), *b);
        }
        (
            jig_core::acts::Admission::Bounded {
                within:
                    jig_core::acts::Bounds::Amend {
                        budget_left: a,
                        grants: ag,
                        implications: ai,
                        authority: aa,
                        ceiling: ac,
                    },
                beyond: ab,
            },
            client::Admission::Bounded {
                within:
                    client::Bounds::Amend { budget_left: b, grants: bg, implications: bi, authority: ba, ceiling: bc },
                beyond: bb,
            },
        ) => {
            assert_eq!(a, b);
            assert_eq!(ag.capacity(), bg.capacity());
            assert_eq!(ag.len(), bg.len());
            for (a, b) in ag.iter().zip(bg.iter()) {
                same_pattern(a, b);
            }
            assert_eq!(ai.capacity(), bi.capacity());
            assert_eq!(ai.len(), bi.len());
            for (a, b) in ai.iter().zip(bi.iter()) {
                assert_eq!((a.connector, a.kind, a.implies), (b.connector, b.kind, b.implies));
            }
            same_authority(aa, ba);
            same_authority(ac, bc);
            match (ab, bb) {
                (jig_core::acts::Beyond::Proposed { to: a }, client::Beyond::Proposed { to: b }) => {
                    same_recipient(a, b);
                }
                (jig_core::acts::Beyond::Refused(a), client::Beyond::Refused(b)) => {
                    assert_eq!(super::wire::refusals::refusal(*a), *b);
                }
                pair => panic!("different exact beyond-bound judgment: {pair:?}"),
            }
        }
        pair => panic!("different actual Task admission: {pair:?}"),
    }
}

#[test]
fn signing_in_and_starting_a_chat_on_the_engine() {
    let body = turn();
    let mut world = world(body.clone());
    let client = world.client(1, crate::stories::conversation());
    let within = Duration::from_secs(1);
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::named(Role::TextArea, b"First words"), within },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: b"Hello".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::See { find: Find::text(b"Reported work starting"), within },
            Step::See { find: Find::text(b"Reported work writing"), within },
            Step::See { find: Find::text(b"abc"), within },
            Step::Pause { span: Duration::from_millis(350) },
            Step::See { find: Find::text(b"abc"), within },
        ],
    );
    world.run(Until::Time(Time::from_nanos(30_000_000)));
    let assignment = world.front().session.driver.world.peers.as_ref().expect("ordinary peers").workers[0]
        .seen
        .first()
        .expect("worker received actual assignment")
        .clone();
    report(&mut world, &assignment, jig_core_fleet::Activity::Starting);
    world.run(Until::Time(Time::from_nanos(80_000_000)));
    report(&mut world, &assignment, jig_core_fleet::Activity::Writing);
    world.run(Until::Time(Time::from_nanos(180_000_000)));
    let offers = world.front().session.task_offers(client, assignment.task).expect("admitted actual Task packet");
    assert_eq!(offers.source.capacity(), 7);
    assert_eq!(offers.native.capacity(), 7);
    for (source, native) in offers.source.iter().zip(offers.native.iter()) {
        same_task_admission(&source.admission, &native.admission);
    }
    let source_observation = offers.source_task.chip.details.observation.as_ref().expect("actual reported work");
    assert_eq!(source_observation.work, jig_core::documents::ChipWork::Writing);
    let client::ChipPhase::Owner { details, .. } = offers.native_task.chip.details.as_ref() else {
        panic!("whole actual chip metadata")
    };
    let observation = details.observation.as_ref().expect("translated actual reported work");
    assert_eq!(observation.work, client::OwnerChipWork::Writing);
    assert_eq!(source_observation.channel, observation.channel);
    assert_eq!(source_observation.since, observation.since);
    assert_eq!(source_observation.reported_at, observation.reported_at);
    assert_eq!(source_observation.started, observation.started);
    assert_eq!(
        offers.source.iter().map(|value| value.act).collect::<Vec<_>>(),
        vec![
            jig_core::acts::Act::Write,
            jig_core::acts::Act::Amend,
            jig_core::acts::Act::TakeOver,
            jig_core::acts::Act::Close,
            jig_core::acts::Act::Stop,
            jig_core::acts::Act::Cancel,
            jig_core::acts::Act::MarkSeen,
        ]
    );
    assert_eq!(
        offers.native.iter().map(|value| value.act).collect::<Vec<_>>(),
        vec![
            client::ActKind::Write,
            client::ActKind::Amend,
            client::ActKind::TakeOver,
            client::ActKind::Close,
            client::ActKind::Stop,
            client::ActKind::Cancel,
            client::ActKind::MarkSeen,
        ]
    );
    assert!(world.tree(client).nodes().iter().any(|node| node.name.as_deref() == Some(b"Provisional".as_slice())));
    assert!(!world.front().session.driver.world.store.rows.values().any(|value| matches!(
        value,
        jig_test_domain::Record::Core(jig_core::Record::Core(jig_core::CoreRecord::Turn(_)))
    )));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let rows = &world.front().session.driver.world.store.rows;
    assert_eq!(
        rows.values()
            .filter(|value| matches!(value,
                jig_test_domain::Record::Core(jig_core::Record::Core(jig_core::CoreRecord::Turn(value)))
                if value.task == assignment.task && value.attempt == assignment.attempt && value.turn == 1
                    && value.transcript == body
            ))
            .count(),
        1
    );
    assert_eq!(
        rows.values()
            .filter(|value| matches!(
                value,
                jig_test_domain::Record::Core(jig_core::Record::People(people::Stored::Answer { .. }))
            ))
            .count(),
        1
    );
    assert!(!world.tree(client).nodes().iter().any(|node| node.name.as_deref() == Some(b"Provisional".as_slice())));
}

#[test]
fn a_person_reads_actual_project_settings_and_returns_to_its_overview() {
    let mut world = world(turn());
    let client = world.client(1, crate::stories::conversation());
    let within = Duration::from_secs(1);
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::named(Role::Button, b"Project 1"), within },
            Step::Press { find: Find::named(Role::Button, b"Project 1") },
            Step::See { find: Find::named(Role::Button, b"Settings"), within },
            Step::Press { find: Find::named(Role::Button, b"Settings") },
            Step::See { find: Find::text(b"Period spend: 1000"), within },
            Step::See { find: Find::text(b"Period spend: 500"), within },
            Step::See { find: Find::text(b"Escalation role: 0"), within },
            Step::See { find: Find::text(b"Configured pool period: 1"), within },
            Step::See { find: Find::named(Role::Button, b"Overview"), within },
            Step::Press { find: Find::named(Role::Button, b"Overview") },
            Step::See { find: Find::named(Role::Button, b"Settings"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let rows = &world.front().session.driver.world.store.rows;
    assert_eq!(
        rows.values()
            .filter(|value| matches!(
                value,
                jig_test_domain::Record::Core(jig_core::Record::People(people::Stored::Answer { .. }))
            ))
            .count(),
        0
    );
    assert_eq!(
        rows.values()
            .filter(|value| matches!(
                value,
                jig_test_domain::Record::Core(jig_core::Record::People(people::Stored::SignIn { .. }))
            ))
            .count(),
        1
    );
    assert!(!rows.values().any(|value| matches!(
        value,
        jig_test_domain::Record::Core(jig_core::Record::Core(jig_core::CoreRecord::Turn(_)))
    )));
}

#[path = "deployment_story.rs"]
mod deployment;

#[path = "historical_story.rs"]
mod historical;

#[path = "people_stories.rs"]
mod person_stories;

#[path = "role_stories.rs"]
mod role_stories;

#[path = "offered_acts.rs"]
mod offered_acts;

mod fault_stories;
