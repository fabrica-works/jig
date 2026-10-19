//! A real worker ends a person-started chat; the person reads its retained task.
use super::*;

fn world() -> World<Test, Joint> {
    let mut owner = jig_core_world::world::limits();
    owner.core.documents.rows = 2;
    owner.core.documents.lines = 2;
    owner.core.documents.chips = 2;
    owner.core.documents.text_bytes = 32;
    let worker = Worker::new(
        1,
        1,
        vec![Box::new([
            Script::Silence { for_: Duration::from_millis(100) },
            Script::Finish { report: b"finished".as_slice().into() },
        ])],
    );
    let driver = Driver::new(
        jig_core_world::world::config(8512),
        owner,
        Peers::new(vec![worker], Vec::new()),
        Time::ZERO,
        clock(Time::ZERO),
    );
    let codec = jig_protocol_transcript::Limits {
        smith: smith_transcript::v5::CEILINGS,
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
    World::new(Settings { seed: 8512, ..Settings::default() }, front)
}

#[test]
fn a_person_reads_the_whole_historical_task_and_result_after_the_engine_finishes_their_chat() {
    let mut world = world();
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
            Step::See { find: Find::text(b"Hello"), within },
            Step::Pause { span: Duration::from_millis(200) },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let task = world.front().session.driver.world.peers.as_ref().expect("actual worker peer").workers[0]
        .seen
        .first()
        .expect("actual assigned chat")
        .task;
    assert!(world.front().session.driver.world.store.rows.values().any(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(value)))
        if value.number==task && matches!(value.phase,jig_domain_tasks::Phase::Ended(_))
    )));
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Region, b"Historical object"), within },
            Step::See { find: Find::text(b"finished"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    assert!(
        !world
            .tree(client)
            .nodes()
            .iter()
            .any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Cancel"))
    );
    let rows = &world.front().session.driver.world.store.rows;
    assert_eq!(
        rows.values()
            .filter(|row| matches!(
                row,
                jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { .. }))
            ))
            .count(),
        1
    );
}

#[test]
fn a_person_revisits_the_whole_historical_task_and_result_by_its_fresh_public_address() {
    let mut world = world();
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
            Step::See { find: Find::text(b"Hello"), within },
            Step::Pause { span: Duration::from_millis(200) },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let task = world.front().session.driver.world.peers.as_ref().expect("actual worker peer").workers[0]
        .seen
        .first()
        .expect("actual assigned chat")
        .task;
    assert!(world.front().session.driver.world.store.rows.values().any(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(value)))
        if value.number==task && matches!(value.phase,jig_domain_tasks::Phase::Ended(_))
    )));
    world.person(
        client,
        vec![
            Step::Go { address: jig_client_test_domain::Address::Jig(jig_client_view::address::Address::task(task)) },
            Step::See { find: Find::named(Role::Region, b"Historical object"), within },
            Step::See { find: Find::text(b"finished"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    assert!(
        !world
            .tree(client)
            .nodes()
            .iter()
            .any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Cancel"))
    );
    let rows = &world.front().session.driver.world.store.rows;
    assert_eq!(
        rows.values()
            .filter(|row| matches!(
                row,
                jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { .. }))
            ))
            .count(),
        1
    );
}
