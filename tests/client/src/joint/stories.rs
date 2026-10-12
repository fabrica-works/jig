//! People act on the public tree over actual core, worker and applied store doors
//! (client/domain/testing.md, sections 5–6; domain/people.md, section 11).
use super::{Joint, engine::Driver, session::Session};
use crate::{Settings, Test, Until, World};
use jig_core_world::peers::Peers;
use jig_domain_client::contract as client;
use jig_domain_people as people;
use jig_fake_person::{Find, Step};
use jig_fake_workers::{Script, Worker};
use jig_view::Role;
use skein_lib::{Duration, Time, Token};
use std::collections::BTreeMap;

use super::fixture::{bounded_world, clock, configured_world, turn, world};

fn report(
    world: &mut World<Test, Joint>,
    assignment: &jig_fake_workers::Assignment,
    activity: jig_domain_fleet::Activity,
) {
    let now = world.now();
    world.front().session.driver.send(
        now,
        clock(now),
        jig_test_domain::Event::Core(jig_domain_core::Event::Fleet(jig_domain_fleet::Event::Observe {
            channel: Token::new(1),
            run: Token::new(assignment.task),
            attempt: Token::new(assignment.attempt),
            report: jig_domain_fleet::Report::Activity(activity),
        })),
    );
}

fn same_pattern(source: &jig_domain_core::documents::GrantLine, native: &client::OwnerGrant) {
    assert_eq!((source.connector, source.kind), (native.connector, native.kind));
    assert_eq!(source.segments.capacity(), native.pattern.segments.capacity());
    assert_eq!(source.segments.as_slice(), native.pattern.segments.as_slice());
    match (&source.last, &native.pattern.last) {
        (jig_domain_core::documents::PatternEnd::Exact(a), client::OwnerPatternEnd::Exact(b))
        | (jig_domain_core::documents::PatternEnd::Open(a), client::OwnerPatternEnd::Open(b)) => assert_eq!(a, b),
        pair => panic!("different whole pattern ends: {pair:?}"),
    }
}

fn same_authority(source: &jig_domain_core::documents::AuthorityLine, native: &client::OwnerAuthorityDocument) {
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
            (jig_domain_tasks::AuthorityExecutor::Charter(a), client::OwnerAuthorityExecutor::Charter(b))
            | (jig_domain_tasks::AuthorityExecutor::Procedure(a), client::OwnerAuthorityExecutor::Procedure(b))
            | (jig_domain_tasks::AuthorityExecutor::Role(a), client::OwnerAuthorityExecutor::Role(b)) => {
                assert_eq!(a, b);
            }
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
            (jig_domain_core::documents::PatternEnd::Exact(a), client::OwnerPatternEnd::Exact(b))
            | (jig_domain_core::documents::PatternEnd::Open(a), client::OwnerPatternEnd::Open(b)) => assert_eq!(a, b),
            pair => panic!("different whole note scope: {pair:?}"),
        }
    }
}

fn same_recipient(source: &jig_domain_core::acts::Recipient, native: &client::Recipient) {
    match (source, native) {
        (jig_domain_core::acts::Recipient::Task(a), client::Recipient::Task(b))
        | (jig_domain_core::acts::Recipient::Party(a), client::Recipient::Party(b)) => assert_eq!(a, b),
        (
            jig_domain_core::acts::Recipient::Role { project: a, role: ar },
            client::Recipient::Role { project: b, role: br },
        ) => {
            assert_eq!((a, ar), (b, br));
        }
        (
            jig_domain_core::acts::Recipient::Roles { project: a, roles: ar },
            client::Recipient::Roles { project: b, roles: br },
        ) => {
            assert_eq!(a, b);
            assert_eq!(ar.capacity(), br.capacity());
            assert_eq!(ar.as_slice(), br.as_slice());
        }
        pair => panic!("different exact recipient: {pair:?}"),
    }
}

fn same_task_admission(source: &jig_domain_core::acts::Admission, native: &client::Admission) {
    match (source, native) {
        (jig_domain_core::acts::Admission::Allowed, client::Admission::Allowed) => {}
        (jig_domain_core::acts::Admission::Proposed { to: a }, client::Admission::Proposed { to: b }) => {
            same_recipient(a, b);
        }
        (jig_domain_core::acts::Admission::Refused(a), client::Admission::Refused(b)) => {
            assert_eq!(super::wire::refusals::refusal(*a), *b);
        }
        (
            jig_domain_core::acts::Admission::Bounded {
                within:
                    jig_domain_core::acts::Bounds::Amend {
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
                (jig_domain_core::acts::Beyond::Proposed { to: a }, client::Beyond::Proposed { to: b }) => {
                    same_recipient(a, b);
                }
                (jig_domain_core::acts::Beyond::Refused(a), client::Beyond::Refused(b)) => {
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
    report(&mut world, &assignment, jig_domain_fleet::Activity::Starting);
    world.run(Until::Time(Time::from_nanos(80_000_000)));
    report(&mut world, &assignment, completion());
    world.run(Until::Time(Time::from_nanos(180_000_000)));
    let offers = world.front().session.task_offers(client, assignment.task).expect("admitted actual Task packet");
    assert_eq!(offers.source.capacity(), 7);
    assert_eq!(offers.native.capacity(), 7);
    for (source, native) in offers.source.iter().zip(offers.native.iter()) {
        same_task_admission(&source.admission, &native.admission);
    }
    let source_observation = offers.source_task.chip.details.observation.as_ref().expect("actual reported work");
    assert_eq!(source_observation.work, jig_domain_core::documents::ChipWork::Writing);
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
            jig_domain_core::acts::Act::Write,
            jig_domain_core::acts::Act::Amend,
            jig_domain_core::acts::Act::TakeOver,
            jig_domain_core::acts::Act::Close,
            jig_domain_core::acts::Act::Stop,
            jig_domain_core::acts::Act::Cancel,
            jig_domain_core::acts::Act::MarkSeen,
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
        jig_test_domain::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(_)))
    )));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let rows = &world.front().session.driver.world.store.rows;
    assert_eq!(
        rows.values()
            .filter(|value| matches!(value,
                jig_test_domain::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(value)))
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
                jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { .. }))
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
                jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { .. }))
            ))
            .count(),
        0
    );
    assert_eq!(
        rows.values()
            .filter(|value| matches!(
                value,
                jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::SignIn { .. }))
            ))
            .count(),
        1
    );
    assert!(!rows.values().any(|value| matches!(
        value,
        jig_test_domain::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(_)))
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

mod referee_stories;

fn completion() -> jig_domain_fleet::Activity {
    jig_domain_fleet::Activity::Long {
        operation: jig_domain_fleet::Operation::Completion,
        span: Duration::from_secs(1),
    }
}
