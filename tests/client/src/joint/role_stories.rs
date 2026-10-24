//! The authenticated roles are setup; every business act is a person on the view
//! (domain/people.md, 11; client/domain/testing.md, 5–6).
use super::*;
use jig_fake_parties::{Act, Party, Role as PartyRole};
use jig_fake_workers::Call;

fn receiving_limits() -> jig_client_test_domain::Limits {
    let mut limits = crate::test::limits();
    limits.client.domain.text = 128;
    limits.client.domain.turn_text = 128;
    limits.client.domain.turn_bytes = 128;
    limits.jig.text = 128;
    limits.jig.rows = 2;
    limits.jig.projects = 2;
    limits.jig.roles = 4;
    limits.jig.delegates = 2;
    limits.first.text = 128;
    limits
}

fn choice_limits() -> jig_client_test_domain::Limits {
    let mut limits = receiving_limits();
    limits.client.domain.text = 128;
    limits.client.domain.turn_text = 128;
    limits.jig.text = 128;
    limits.first.text = 128;
    limits
}
fn choice_client(seed: u64) -> (jig_client_test_domain::Domain, jig_client_test_view::View<'static>) {
    crate::test::new(&choice_limits(), seed)
}

fn client(seed: u64) -> (jig_client_test_domain::Domain, jig_client_test_view::View<'static>) {
    crate::test::new(&receiving_limits(), seed)
}

fn world(seed: u64, scripts: Vec<Box<[Script]>>) -> World<Test, Joint> {
    configured(seed, scripts, |_, _| {})
}

fn configured(
    seed: u64,
    scripts: Vec<Box<[Script]>>,
    configure: fn(&mut jig_test_domain::Config, &jig_test_domain::Limits),
) -> World<Test, Joint> {
    configured_with_slots(seed, scripts, 1, configure)
}

fn configured_with_slots(
    seed: u64,
    scripts: Vec<Box<[Script]>>,
    slots: u32,
    configure: fn(&mut jig_test_domain::Config, &jig_test_domain::Limits),
) -> World<Test, Joint> {
    configured_with_bounds(seed, scripts, slots, configure, &receiving_limits())
}

fn configured_with_bounds(
    seed: u64,
    scripts: Vec<Box<[Script]>>,
    slots: u32,
    configure: fn(&mut jig_test_domain::Config, &jig_test_domain::Limits),
    receiving: &jig_client_test_domain::Limits,
) -> World<Test, Joint> {
    let (mut configuration, mut owner) = jig_core_world::peers::fixture(seed, 0);
    configuration.core.owners = Box::new([]);
    owner.core.fleet.slots = slots;
    // Four adoption identities and their four later public client sign-ins.
    owner.core.people.sign_ins = 8;
    // Four adoption peer watches plus eight public watches per client.
    owner.core.views.watchers = 36;
    owner.core.documents.rows = 2;
    owner.core.documents.lines = 2;
    owner.core.documents.chips = 2;
    owner.core.documents.text_bytes = receiving.jig.text;
    // This scenario offers its agent and human executor, not a procedure.
    let kinds = Box::new([jig_domain_authority::Executor::Charter(1), jig_domain_authority::Executor::Role(1)]);

    let mut authority_rules = configuration.core.defaults.rules.clone();
    authority_rules.ceiling.delegation.kinds = kinds.clone();
    let mut policy = configuration.core.seed_projects[0].policy.policy.clone();
    policy.ceiling.delegation.kinds = kinds.clone();
    for role in &mut policy.roles {
        role.authority.delegation.kinds = kinds.clone();
    }
    policy.roles[0].authority.budget.spend = 500;
    configuration.core.defaults.rules = (jig_domain_authority::Domain::new(authority_rules, owner.core.authority)
        .expect("role scenario bounds"))
    .rules()
    .clone();
    configuration.core.seed_projects[0].policy.policy = policy;
    configure(&mut configuration, &owner);
    jig_core_world::peers::journal(&mut owner);
    let roles = [PartyRole::Owner, PartyRole::Maintainer, PartyRole::Member, PartyRole::Observer];
    let parties = roles
        .iter()
        .enumerate()
        .map(|(i, role)| {
            Party::new(
                1,
                *role,
                10_000 * (i as u64 + 1),
                Box::new([Act::SignIn { provider: 0, subject: (7 + i as u64).to_be_bytes().into() }]),
            )
        })
        .collect();
    let mut driver = Driver::new(
        configuration,
        owner,
        Peers::new(vec![Worker::new(7, slots, scripts)], parties),
        Time::ZERO,
        clock(Time::ZERO),
    );
    let holdings = driver
        .world
        .peers
        .as_ref()
        .expect("setup peers")
        .parties
        .iter()
        .enumerate()
        .map(|(i, party)| people::Holding {
            person: party.person.expect("setup sign-in"),
            role: match i {
                0 => people::Role::Owner,
                1 => people::Role::Maintainer,
                2 => people::Role::Member,
                3 => people::Role::Observer,
                _ => unreachable!("four setup roles"),
            },
        })
        .collect();
    driver.send(
        Time::ZERO,
        clock(Time::ZERO),
        jig_test_domain::Event::Core(jig_domain_core::Event::People(people::Event::Roles { project: 1, holdings })),
    );
    let codec = jig_protocol_transcript::Limits {
        smith: smith_transcript::v6::CEILINGS,
        bytes: 1_048_576,
        turn: client::TurnLimits { messages: 8, blocks: 8, text: 128 },
    };
    let identities = (1..=4)
        .map(|id| {
            (
                id,
                people::Identity {
                    key: people::IdentityKey { provider: 0, subject: (id + 6_u64).to_be_bytes().into() },
                    login: b"login".as_slice().into(),
                    name: b"Person".as_slice().into(),
                },
            )
        })
        .collect();
    let mut front = Joint::new(Session::new(driver, *receiving, codec), identities, clock);
    front.unsigned_on_bind();
    front.session.capture_task_offers();
    front.session.stream_heartbeat(Duration::from_millis(50));
    World::new(Settings { seed, ..Settings::default() }, front)
}

fn start(words: &[u8]) -> Vec<Step<jig_client_test_domain::Address>> {
    let within = Duration::from_secs(1);
    vec![
        Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
        Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
        Step::See { find: Find::named(Role::TextArea, b"First words"), within },
        Step::Type { find: Find::named(Role::TextArea, b"First words"), words: words.into() },
        Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
        Step::See { find: Find::text(words), within },
    ]
}

#[test]
fn a_chat_proposing_a_goal_its_member_cannot_fund_is_accepted_by_a_maintainer() {
    let mut world = world(
        8513,
        vec![
            Box::new([
                Script::Call(Call::ProposeGoal {
                    words: b"fund a larger goal".as_slice().into(),
                    budget: 150,
                    priority: 7,
                }),
                Script::Park,
            ]),
            Box::new([Script::Finish { report: b"goal completed".as_slice().into() }]),
        ],
    );
    let member = world.client_with(3, crate::stories::conversation(), receiving_limits(), client);
    world.person(member, start(b"please propose"));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let maintainer = world.client_with(2, crate::stories::inbox(), receiving_limits(), client);
    let within = Duration::from_secs(1);
    world.person(
        maintainer,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::text(b"fund a larger goal"), within },
            Step::See { find: Find::exactly_named(Role::Button, b"Accept"), within },
            Step::Press { find: Find::exactly_named(Role::Button, b"Accept") },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Accepted"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    let actual = world.front().session.driver.world.peers.as_ref().expect("actual peers").parties[1]
        .person
        .expect("actual maintainer");
    assert!(world.front().session.driver.world.store.rows.values().any(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(task)))
        if task.tracked == Some(7) && task.requester == jig_domain_tasks::Party::Person(actual)
        && task.numbers.budget == 150)));
}

#[test]
fn a_person_answers_a_choice_between_two_reports() {
    let mut world = configured_with_bounds(
        8514,
        vec![Box::new([
            Script::Call(Call::Choose {
                role: 1,
                first: b"first report".as_slice().into(),
                second: b"second report".as_slice().into(),
            }),
            Script::Wait,
            Script::Finish { report: b"choice received".as_slice().into() },
        ])],
        1,
        |_, _| {},
        &choice_limits(),
    );
    world.front().session.capture_card_offers();
    let member = world.client_with(3, crate::stories::conversation(), choice_limits(), choice_client);
    world.person(member, start(b"ask a maintainer"));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    // The initiating member leaves the live chat while its maintainer answers.
    // Its original start/referee assertions remain; it need not retain a run watch.
    world.person(
        member,
        vec![Step::Go {
            address: jig_client_test_domain::Address::Jig(jig_client_view::address::Address::Project {
                project: 1,
                section: jig_client_view::address::ProjectSection::Overview,
            }),
        }],
    );
    let maintainer = world.client_with(2, crate::stories::inbox(), choice_limits(), choice_client);
    let within = Duration::from_secs(1);
    world.person(
        maintainer,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::text(b"first report or second report"), within },
            Step::See { find: Find::exactly_named(Role::Button, b"Take it"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let keys = world.front().session.card_offer_keys(maintainer);
    let person: Vec<_> = keys.into_iter().filter(|key| matches!(key, client::ObjectKey::PersonTask(_))).collect();
    assert_eq!(person.len(), 1);
    super::offered_acts::card(&mut world, maintainer, person[0], b"Person task");
    world.person(
        maintainer,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Take it") },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::exactly_named(Role::Button, b"Answer"), within },
            Step::Press { find: Find::exactly_named(Role::Button, b"Answer") },
            Step::See { find: Find::exactly_named(Role::Choice, b"Choice code: 2"), within },
            Step::Press { find: Find::exactly_named(Role::Choice, b"Choice code: 2") },
            Step::Type { find: Find::named(Role::TextArea, b"Your reason"), words: b"second report".as_slice().into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let owner = &world.front().session.driver.world;
    assert!(owner.store.rows.values().any(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(task)))
        if matches!(&task.phase, jig_domain_tasks::Phase::Ended(jig_domain_tasks::Ending::Done(
            jig_domain_tasks::TaskResult::Verdict { code: 2, words, .. })) if words.as_ref() == b"second report"))));
    assert!(
        owner.peers.as_ref().expect("actual worker").workers[0]
            .messages
            .iter()
            .any(|(_, _, _, words)| words.as_ref() == b"second report")
    );
}

#[test]
fn two_parties_decide_one_proposal_and_the_second_is_told_who() {
    let mut world = world(
        8515,
        vec![
            Box::new([
                Script::Call(Call::ProposeGoal {
                    words: b"decide this goal".as_slice().into(),
                    budget: 150,
                    priority: 7,
                }),
                Script::Park,
            ]),
            Box::new([Script::Finish { report: b"one accepted goal".as_slice().into() }]),
        ],
    );
    let member = world.client_with(3, crate::stories::conversation(), receiving_limits(), client);
    world.person(member, start(b"propose once"));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let owner = world.client_with(1, crate::stories::inbox(), receiving_limits(), client);
    let maintainer = world.client_with(2, crate::stories::inbox(), receiving_limits(), client);
    let within = Duration::from_secs(1);
    for party in [owner, maintainer] {
        world.person(
            party,
            vec![
                Step::See { find: Find::exactly_named(Role::Button, b"Sign in with Test"), within },
                Step::Press { find: Find::exactly_named(Role::Button, b"Sign in with Test") },
                Step::See { find: Find::text(b"decide this goal"), within },
                Step::See { find: Find::exactly_named(Role::Button, b"Accept"), within },
            ],
        );
    }
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    for party in [owner, maintainer] {
        let keys = world.front().session.card_offer_keys(party);
        let proposals: Vec<_> = keys.into_iter().filter(|key| matches!(key, client::ObjectKey::Proposal(_))).collect();
        assert_eq!(proposals.len(), 1, "one supplied pending proposal");
        super::offered_acts::card(&mut world, party, proposals[0], b"Proposal");
    }
    // Check the open cards before opening either confirmation. The same two
    // ready confirmations still submit together below, before either reply.
    for party in [owner, maintainer] {
        world.person(party, vec![Step::Press { find: Find::exactly_named(Role::Button, b"Accept") }]);
    }
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let first_person = world.front().session.driver.world.peers.as_ref().expect("actual parties").parties[0]
        .person
        .expect("actual owner");
    let by = format!("Decided by person: {first_person}").into_bytes();
    // Both ready confirmations send before either released watch/answer is
    // delivered; their distinct authenticated engine keys remain independent.
    for party in [owner, maintainer] {
        world.person(
            party,
            vec![
                Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
                Step::See { find: Find::text(b"Accepted"), within },
                Step::See { find: Find::text(&by), within },
            ],
        );
    }
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let engine = &world.front().session.driver.world;
    assert_eq!(
        engine
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { outcome, .. }))
        if matches!(outcome, people::Outcome::ProposalDecided { by, choice: people::ProposalChoice::Accepted, .. }
            if *by == first_person)))
            .count(),
        2
    );
    assert_eq!(
        engine
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(task)))
        if task.tracked == Some(7) && task.numbers.budget == 150))
            .count(),
        1
    );
    let assignments = &engine.peers.as_ref().expect("actual worker").workers[0].seen;
    assert_eq!(assignments.len(), 3, "the proposing chat resumes after its one accepted goal");
    let tasks: std::collections::BTreeSet<_> = assignments.iter().map(|row| row.task).collect();
    assert_eq!(tasks.len(), 2, "only the proposing chat and its accepted goal");
    let parent = assignments[0].task;
    let parent_attempts: std::collections::BTreeSet<_> =
        assignments.iter().filter(|row| row.task == parent).map(|row| row.attempt).collect();
    assert_eq!(parent_attempts.len(), 2, "the original chat parks and then resumes");
    assert_eq!(assignments.iter().filter(|row| row.task != parent).count(), 1, "one goal assignment");
}

#[test]
fn an_observer_is_refused_a_request_and_told_why() {
    let mut world = world(8516, vec![Box::new([Script::Wait])]);
    let observer = world.client_with(4, crate::stories::conversation(), receiving_limits(), client);
    let within = Duration::from_secs(1);
    let mut steps = start(b"observer request");
    // Missing creating bounds refuse locally and leave the entered words.
    steps.pop();
    steps.push(Step::See { find: Find::text(b"This act is not offered"), within });
    world.person(observer, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let engine = &world.front().session.driver.world;
    assert!(engine.peers.as_ref().expect("actual worker").workers[0].seen.is_empty());
    assert!(!engine.store.rows.values().any(|row| matches!(
        row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(_)))
    )));
    assert_eq!(
        engine
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { outcome, .. }))
        if matches!(outcome, people::Outcome::Refused(people::Refusal::Role { held: Some(people::Role::Observer) }))))
            .count(),
        0
    );
}

#[path = "engine_stories.rs"]
mod engine_stories;

#[path = "role_stories/proposed_story.rs"]
mod proposed_story;

#[path = "role_stories/card_stories.rs"]
mod card_stories;

#[path = "role_stories/fault_stories.rs"]
mod fault_stories;

#[path = "role_stories/family_stories.rs"]
mod family_stories;
