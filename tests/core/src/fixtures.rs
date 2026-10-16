//! Shared host bounds for the domain world.
use jig_domain_core as core;
use jig_domain_tasks as tasks;
pub const WORKSPACE_CREDENTIAL: u32 = 7;
pub const CREDENTIAL_RESOURCE: u64 = 1;
#[must_use]
pub fn bounds() -> jig_domain_fleet::Bounds {
    use skein_lib::Duration;
    jig_domain_fleet::Bounds {
        models: Box::new([
            jig_domain_fleet::ModelBound { name: b"scripted".as_slice().into(), window: 8192, output: 1024 },
            jig_domain_fleet::ModelBound { name: b"research".as_slice().into(), window: 8192, output: 1024 },
        ]),
        turns: u32::MAX,
        spend: u64::MAX,
        time: Duration::from_secs(3600),
        waiting: Duration::from_secs(3600),
        checks: Duration::from_nanos(1),
        message_bytes: u32::MAX,
        inbox: u32::MAX,
        unacknowledged: u32::MAX,
    }
}

#[must_use]
pub fn live_row(world: &crate::effects::World, task: u64) -> &jig_domain_tasks::TaskRecord {
    match world
        .store
        .rows
        .get(&jig_test_domain::Key::Core(jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Live(task))))
    {
        Some(jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(row)))) => row,
        row => panic!("live task row missing: {row:?}"),
    }
}

pub const QUERY_REQUESTER: u64 = 1;

/// A requesting party's task query, shared by the independent watch stories.
pub const OWN_TASKS: jig_domain_core::documents::TaskQuery = jig_domain_core::documents::TaskQuery {
    project: None,
    requester: Some(jig_domain_tasks::Party::Person(QUERY_REQUESTER)),
    root: None,
    parent: None,
    label: None,
    phase: None,
    executor: None,
    tracked: None,
    order: jig_domain_core::documents::QueryOrder::Number,
};

/// A delegate result and one standing state subscription each reserve a result payload.
#[must_use]
pub fn notices_world(seed: u64) -> crate::effects::World {
    let (config, mut limits) = crate::effects::fixture(seed, false);
    limits.core.tasks.inbox_bytes = 256;
    crate::effects::World::configured(seed, false, config, limits)
}

/// Task numbers in a live query page, retaining its requested order.
#[must_use]
pub fn query_numbers(world: &crate::effects::World) -> Vec<u64> {
    let jig_domain_core::reads::Answer::Page(page) = crate::reads::latest(world) else { panic!("query page") };
    page.rows
        .iter()
        .map(|row| {
            let jig_domain_core::reads::PageRow::Line(line) = row else { panic!("task line") };
            line.chip.number
        })
        .collect()
}

#[must_use]
pub fn templates() -> Box<[jig_domain_core::Template]> {
    use jig_domain_people::{TemplateBudget, TemplateDeadline, TemplateExecutor};
    use jig_domain_tasks::{Contract, Flags, Retention};
    Box::new([
        jig_domain_core::Template {
            number: 1,
            name: Box::from(&b"chat"[..]),
            executor: TemplateExecutor::Charter(1),
            contract: Contract::Report { words: 32 },
            shallower: 1,
            budget: TemplateBudget::AsAllowed { cap: None },
            deadline: TemplateDeadline::None,
            tracked: None,
            flags: Flags { conversation: true, listed: false, retention: Retention::Kept },
            label: 1,
            grace: Some(skein_lib::Duration::from_millis(5000)),
        },
        jig_domain_core::Template {
            number: 2,
            name: Box::from(&b"goal"[..]),
            executor: TemplateExecutor::Chosen,
            contract: Contract::Report { words: 32 },
            shallower: 1,
            budget: TemplateBudget::Given,
            deadline: TemplateDeadline::Given,
            tracked: Some(1),
            flags: Flags { conversation: false, listed: true, retention: Retention::Kept },
            label: 2,
            grace: None,
        },
    ])
}

const HOLDER_REQUEST_BASE: u64 = 8200;
/// Submit an owner or member request with the fixture's request-token range.
pub fn ask_as(world: &mut crate::effects::World, sign_in: u64, key: u8, ask: jig_domain_people::Ask) {
    world.send(jig_test_domain::Event::Core(jig_domain_core::Event::People(jig_domain_people::Event::Ask {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(HOLDER_REQUEST_BASE + u64::from(key))),
        sign_in,
        key: [key; 16],
        ask,
    })));
}

#[must_use]
pub fn member(world: &crate::effects::World, executor: tasks::Executor, words: &[u8]) -> core::Delegate {
    let mut authority = world.assigned.as_ref().expect("task assigned").run.authority.clone();
    authority.budget.spend = 20;
    authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 1, depth: 1 };
    core::Delegate {
        executor,
        spec: tasks::Spec { words: words.into(), parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: 128 },
        authority,
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
    }
}

/// Assert the last party answer carries this refusal.
pub fn refusal(world: &crate::effects::World, why: jig_domain_people::Refusal) {
    use jig_domain_people as people;
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::Refused(why))));
}

/// A person's amendment retaining each field.
#[must_use]
pub fn person_unchanged(reason: &[u8]) -> jig_domain_people::Amendment {
    jig_domain_people::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        reason: reason.into(),
    }
}
