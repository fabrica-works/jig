//! Starting truth and person's scripts shared by focused and random worlds.
#![allow(clippy::wildcard_imports, reason = "world adapters translate the complete owner vocabulary")]
use crate::{Scripted, Settings, Test, World, scripted::Family, test::list};
use jig_client::{address::Address, contract::*};
use jig_scripted_engine::{Bounds, Script, Truth};
use jig_test_client_domain::Address as RootAddress;
use skein_lib::{Duration, Wall};
/// Creates a bounded fixture word sequence.
#[must_use]
pub fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), 32).expect("fixture text fits")
}
/// Creates a signed-in party with a chat and goal charter.
#[must_use]
pub fn frame(party: u64) -> Frame {
    Frame {
        party: Party { number: party, kind: PartyKind::Person, name: text(b"Pat") },
        projects: list([ProjectRole {
            project: 1,
            name: text(b"First"),
            roles: list([1]),
            chats: list([Charter { number: 1, name: text(b"Chat") }]),
            goals: list([Charter { number: 1, name: text(b"Goal") }]),
            pool: Budget { total: 100, own: 0, ended: 0, reserved: 0, left: 100, funder: Actor::Party(party) },
            chat_budget: 10,
        }]),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        updates: Count { value: 0, limited: false },
        versions: VersionRange { first: 1, last: 1 },
        retention: Duration::from_secs(1000),
        deployment_role: Some(1),
    }
}
/// Starts the standard fixture at the requested seed.
#[must_use]
pub fn world(seed: u64) -> World<Test, Scripted> {
    configured(Settings { seed, ..Settings::default() })
}
/// Starts the standard fixture with explicit loop settings.
#[must_use]
pub fn configured(settings: Settings) -> World<Test, Scripted> {
    let seed = settings.seed;
    let mut truth = Truth::default();
    truth.frames.insert(7, frame(7));
    truth.sign_ins.insert(7, 7);
    World::new(
        settings,
        Scripted::new(
            Script {
                truth,
                bounds: Bounds {
                    text: 32,
                    rows: 8,
                    document_bytes: 65536,
                    commit_delay: Duration::from_millis(5),
                    heartbeat: Duration::from_millis(20),
                    watches: 64,
                    watches_per_party: 32,
                },
                rules: vec![],
            },
            seed,
            Family::default(),
        ),
    )
}
/// Names the party’s inbox.
#[must_use]
pub fn inbox() -> RootAddress {
    RootAddress::Jig(Address::Inbox { project: None })
}

fn authority() -> Authority {
    Authority {
        budget: 1,
        grants: list([]),
        task_kinds: list([]),
        depth: 1,
        tasks: 1,
        deadline: None,
        tools: list([]),
        note_scopes: list([]),
    }
}
fn budget() -> Budget {
    Budget { total: 1, own: 0, ended: 0, reserved: 0, left: 1, funder: Actor::Party(7) }
}
/// Creates the initial chat task and its write offer.
#[must_use]
pub fn task() -> Task {
    Task {
        chip: Chip {
            task: 9,
            title: text(b"Chat nine"),
            executor: Executor::Agent { charter: 1 },
            phase: Phase::Running,
            details: Box::new(ChipPhase::Procedure {
                panel: Part::Panel { connector: jig_test_client_domain::FIRST, task: 9 },
            }),
        },
        goal: false,
        priority: Some(1),
        lineage_chips: list([]),
        inputs: list([]),
        project: 1,
        requester: Actor::Party(7),
        lineage: list([]),
        spec: text(b"spec"),
        spec_revision: Revision { value: 1 },
        instructions: text(b"instructions"),
        wake: WakePolicy { words: false, results: false, topics: list([]), subscriptions: list([]) },
        result_contract: ResultContract::Report,
        dependencies: list([]),
        references: list([]),
        resources: list([]),
        budget: budget(),
        authority: authority(),
        hold: None,
        escalation: None,
        proposals: list([]),
        activity: 0,
        deadline: None,
        changed: Wall::EPOCH,
        acts: list([Offered { act: ActKind::Write, admission: Admission::Allowed }]),
        parts: list([]),
        closing: None,
        spend: list([]),
        result: None,
        runtime: Runtime::Idle,
        impact: Impact {
            tasks: PhaseCounts { waiting: 0, running: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 },
            live_runs: 0,
            proposals: 0,
            effects_pending: 0,
            effects_settling: 0,
            takeover_budget: 0,
        },
        delegates: list([]),
        writer_slots: list([]),
    }
}

/// Starts the scripted first-chat decision fixture.
#[must_use]
pub fn chat_world(seed: u64) -> World<Test, Scripted> {
    let mut world = world(seed);
    let task = task();
    world
        .front()
        .engine
        .script(jig_scripted_engine::Rule {
            party: Some(7),
            ask: Ask::StartChat { project: 1, words: text(b"Hello") },
            answer: Answer::Done(Outcome::Made { what: Made::Chat { task: 9 } }),
            changes: vec![jig_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Task(9),
                value: ReadValue::Task(Box::new(task)),
            }],
        })
        .expect("chat rule fits");
    world
}
/// Names the party’s conversation list.
#[must_use]
pub fn conversation() -> RootAddress {
    RootAddress::Jig(Address::Conversations { project: None })
}
/// Names the initial chat’s activity page.
#[must_use]
pub fn chat() -> RootAddress {
    RootAddress::Jig(Address::Task { task: 9, section: jig_client::address::TaskSection::Activity })
}
/// Starts with the initial chat already durable.
#[must_use]
pub fn chat_ready(seed: u64) -> World<Test, Scripted> {
    let mut world = world(seed);
    let now = world.now();
    world
        .front()
        .engine
        .act(
            now,
            Wall::EPOCH,
            jig_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Task(9),
                value: ReadValue::Task(Box::new(task())),
            },
        )
        .expect("task fits");
    world
}
