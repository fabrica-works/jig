//! Seeded public truth and fake-person exploration (testing.md, section 7; skein's ui.md, 17.1).
use crate::{Outcome, Scripted, Test, Until, World, stories};
use jig_client_domain::contract::{ObjectKey, ReadValue};
use jig_client_scripted_engine::Change;
use jig_client_test_domain::{Address as RootAddress, family};
use jig_client_view::address::{Address, ProjectSection, TaskSection};
use skein_lib::{Rng, Wall};

/// Independently replayable story starting states for a random person.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Start {
    NewChat,
    Chat,
    Held,
    Proposal,
    Goal,
    Incident,
}
impl Start {
    pub const ALL: [Self; 6] = [Self::NewChat, Self::Chat, Self::Held, Self::Proposal, Self::Goal, Self::Incident];
}
/// Actual fixture and opening address reached by a random whole-document run.
#[derive(Clone, Copy, Debug)]
pub struct Reach {
    pub start: Start,
    pub original_address: bool,
}

fn fixture(seed: u64, start: Start) -> (World<Test, Scripted>, RootAddress) {
    match start {
        Start::NewChat => (stories::chat_world(seed), stories::conversation()),
        Start::Chat => (stories::chat_ready(seed), stories::chat()),
        Start::Held => (stories::held_chat(seed), stories::chat()),
        Start::Proposal => (stories::proposal_world(seed), stories::inbox()),
        Start::Goal => (
            stories::goal_world(seed),
            RootAddress::Jig(Address::Project { project: 1, section: ProjectSection::Goals }),
        ),
        Start::Incident => (stories::incident_world(seed), stories::incident()),
    }
}
fn explore(mut world: World<Test, Scripted>, address: RootAddress, seed: u64, memory: bool) -> Outcome {
    world.randomness(16, 150, 8);
    if memory {
        world.measure_heap();
    }
    let mut rng = Rng::new(seed);
    for party in [7, 8] {
        let first = world.client(party, address);
        world.random_person(first);
        let second = world.tab(first);
        world.random_person(second);
        if rng.chance(500) {
            let device = world.client(party, stories::inbox());
            world.random_person(device);
        }
    }
    world.run(Until::Done)
}
/// Random people across one real story's initial truth, with tabs and devices.
#[must_use]
pub fn story(seed: u64, start: Start) -> Outcome {
    let (world, address) = fixture(seed, start);
    explore(world, address, seed, true)
}
/// Random whole documents, addresses, family values and offers behind the same shell.
#[must_use]
fn world_with_heap(seed: u64, memory: bool) -> (Outcome, Reach) {
    let mut rng = Rng::new(seed);
    let start = Start::ALL[usize::try_from(rng.below(6)).expect("six starts")];
    let (mut world, mut address) = fixture(seed, start);
    let original_address = address;
    if let Some(ReadValue::Task(task)) = world.front().engine.truth().objects.get(&ObjectKey::Task(9)) {
        let mut task = task.clone();
        task.spec_revision.value = rng.between(1, 16);
        task.spec.specification.words.bytes =
            stories::text(if rng.chance(500) { b"Investigate" } else { b"Check service" });
        task.priority = Some(u32::try_from(rng.between(1, 3)).expect("small priority"));
        world
            .front()
            .engine
            .act(
                skein_lib::Time::ZERO,
                Wall::EPOCH,
                Change::Object { project: 1, key: ObjectKey::Task(9), value: ReadValue::Task(task) },
            )
            .expect("whole random task fits");
    }
    let mut family = world.front().engine.application().clone();
    for document in family.documents.values_mut() {
        match document {
            family::Document::Item(item) => {
                item.value = rng.below(64);
                item.touchable = rng.chance(500);
            }
            family::Document::Part(part) => {
                part.value = rng.below(64);
            }
            family::Document::Connector(_) => {}
        }
    }
    world
        .front()
        .engine
        .act_application(skein_lib::Time::ZERO, Wall::EPOCH, jig_client_test_domain::FIRST, 1, family, vec![])
        .expect("whole random family fits");
    match rng.below(4) {
        0 => address = stories::inbox(),
        1 => address = RootAddress::Jig(Address::Project { project: 1, section: ProjectSection::Tasks }),
        2 if world.front().engine.truth().objects.contains_key(&ObjectKey::Task(9)) => {
            address = RootAddress::Jig(Address::Task { task: 9, section: TaskSection::Overview });
        }
        _ => {}
    }
    let reach = Reach { start, original_address: address == original_address };
    (explore(world, address, seed, memory), reach)
}

/// Random whole documents with allocator accounting for the focused smoke.
#[must_use]
pub fn world(seed: u64) -> Outcome {
    world_with_heap(seed, true).0
}

/// The same story exploration without heap bookkeeping; memory tests and
/// the focused smoke retain that check independently.
#[must_use]
pub fn story_without_heap(seed: u64, start: Start) -> Outcome {
    let (world, address) = fixture(seed, start);
    explore(world, address, seed, false)
}

/// The same random documents and faults without heap bookkeeping.
#[must_use]
pub fn world_without_heap(seed: u64) -> Outcome {
    world_with_heap(seed, false).0
}

/// Random whole documents with typed reach recorded at their actual opening.
#[must_use]
pub fn world_with_coverage(seed: u64) -> (Outcome, Reach) {
    world_with_heap(seed, false)
}
