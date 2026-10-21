use jig_domain_tasks::{
    Accepted, Active, Amendment, Control, End, Event, Funder, Hold, HoldKind, Holding, Key, Kind, Line, Name, Party,
    Phase, Refusal, Status, Stored, Taken, TaskResult,
};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::{ReplyTo, Rng, Token};
use std::collections::BTreeSet;

const DRAW_SOURCE: u64 = 731;
const DRAWN_SEEDS: usize = 64;
const STEPS: usize = 96;
const PERSON: Party = Party::Person(9);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Reached {
    Amended,
    Circle,
    Kept,
    Parked,
    Met,
    QueueLeft,
    QueueReturned,
}
const MARKERS: [Reached; 7] = [
    Reached::Amended,
    Reached::Circle,
    Reached::Kept,
    Reached::Parked,
    Reached::Met,
    Reached::QueueLeft,
    Reached::QueueReturned,
];

// Each operation carries its complete draw. A future Compound case can carry
// Vec<jig_domain_tasks::Operation> and send those amends/controls in one event
// once staged-circle integration has landed.
#[derive(Debug)]
enum Operation {
    Amend { task: u64, dependencies: Vec<u64> },
    HoldRelease { task: u64 },
    Claim { task: u64 },
    End { task: u64, failed: bool },
    Cancel { task: u64 },
    Restart,
}

fn name(value: u8) -> Name {
    Name { connector: 1, path: Box::new([Box::new([value])]) }
}
fn write(value: u8, kind: u16) -> Holding {
    Holding::Write { resource: name(value), kind }
}
fn slot() -> Holding {
    Holding::Slot { pool: name(2), kind: 2 }
}

fn population(seed: u64) -> World {
    let mut world = World::new(seed, LIMITS);
    world.configure_holds(
        1,
        vec![
            Kind { connector: 1, kind: 1, hold: HoldKind::Exclusive { taken: Taken::Waits } },
            Kind { connector: 1, kind: 2, hold: HoldKind::Pooled { taken: Taken::Waits } },
            Kind { connector: 1, kind: 3, hold: HoldKind::Shared },
        ],
    );
    world.send(Event::Slots { pool: name(2), slots: 2 });
    let mut first = task(1, &[]);
    first.numbers.budget = 400;
    first.authority.budget.spend = 400;
    first.holdings = Box::new([write(1, 1), write(3, 3)]);
    let mut second = task(2, &[]);
    second.holdings = Box::new([write(1, 1)]);
    let mut third = task(3, &[]);
    third.holdings = Box::new([slot(), write(3, 3)]);
    let mut fourth = task(4, &[]);
    fourth.holdings = Box::new([slot()]);
    assert_eq!(world.make(PERSON, vec![first, second, third, fourth]), Reply::Made(vec![1, 2, 3, 4]));
    let mut fifth = task(5, &[]);
    fifth.funder = Funder::Task(1);
    let mut sixth = task(6, &[]);
    sixth.funder = Funder::Task(1);
    sixth.holdings = Box::new([slot()]);
    assert_eq!(world.make(Party::Task(1), vec![fifth, sixth]), Reply::Made(vec![5, 6]));
    assert_eq!(world.record(2).phase, Phase::Waiting);
    assert_eq!(world.record(6).phase, Phase::Waiting);
    world
}

fn live(world: &World) -> Vec<u64> {
    let mut live = Vec::new();
    for key in world.records.keys() {
        if let Key::Live(number) = key {
            live.push(*number);
        }
    }
    live
}

fn pick(random: &mut Rng, choices: &[u64]) -> u64 {
    *choices.get(usize::try_from(random.below(choices.len() as u64)).expect("small draw")).expect("nonempty choices")
}

fn draw(world: &World, random: &mut Rng) -> Operation {
    let task = pick(random, &live(world));
    match random.below(12) {
        0..=5 => {
            let mut dependencies = world.record(task).dependencies.to_vec();
            match random.below(3) {
                0 => {
                    let candidate = pick(random, &live(world));
                    if !dependencies.contains(&candidate) {
                        dependencies.push(candidate);
                    }
                }
                1 => {
                    dependencies.pop();
                }
                2 => {
                    dependencies.clear();
                    // Ended names deliberately exercise both met and refusal.
                    dependencies.push(random.below(6) + 1);
                }
                _ => unreachable!(),
            }
            Operation::Amend { task, dependencies }
        }
        6 => Operation::HoldRelease { task },
        7 | 8 => Operation::Claim { task },
        9 => Operation::End { task, failed: random.chance(200) },
        10 => Operation::Cancel { task },
        11 => Operation::Restart,
        _ => unreachable!(),
    }
}

fn amend(world: &mut World, task: u64, dependencies: &[u64], seen: &mut BTreeSet<Reached>) {
    let before = world.record(task).clone();
    let reply_to = world.to();
    let call = reply_to.into_token().raw();
    let reply_to = ReplyTo::new(Token::new(call));
    world.send(Event::Amend {
        reply_to,
        by: PERSON,
        task,
        message: before.last_message + 1,
        stop_run: false,
        funder: None,
        amendment: Amendment {
            tracked: None,
            addressed: None,
            instructions: None,
            release: None,
            spec: None,
            wake: None,
            dependencies: Some(dependencies.into()),
            authority: None,
            llm: None,
            reason: Box::new([1]),
        },
    });
    match world.replies.get(&call).expect("amendment reply") {
        Reply::Done => {
            seen.insert(Reached::Amended);
            let after = world.record(task);
            if matches!(after.phase, Phase::Active(_)) && !after.waiting_on.is_empty() {
                seen.insert(Reached::Kept);
            }
            if before.phase == Phase::Waiting && before.waiting_on.is_empty() && !after.waiting_on.is_empty() {
                seen.insert(Reached::QueueLeft);
            }
            for dependency in dependencies {
                if !before.dependencies.contains(dependency)
                    && let Some(Stored::Ended(row)) = world.records.get(&Key::Ended(*dependency))
                    && matches!(row.phase, Phase::Ended(jig_domain_tasks::Ending::Done(_)))
                {
                    assert!(!after.waiting_on.contains(dependency));
                    let mut recorded = false;
                    for added in &after.added {
                        if added.task == *dependency {
                            recorded = true;
                        }
                    }
                    assert!(recorded, "done dependency's add is recorded");
                    seen.insert(Reached::Met);
                }
            }
        }
        Reply::Refused(problem) => {
            if let Refusal::Circle { .. } = problem.why {
                let members = problem.blocked_by.as_ref().expect("circle names its members");
                assert!(!members.is_empty(), "circle names at least one member");
                seen.insert(Reached::Circle);
            } else {
                assert!(
                    matches!(
                        problem.why,
                        Refusal::State
                            | Refusal::Unknown
                            | Refusal::Dependencies
                            | Refusal::DependencyEnded { how: Status::Failed | Status::Cancelled }
                    ),
                    "unexpected amendment refusal: {:?}",
                    problem.why
                );
            }
        }
        reply @ (Reply::Made(_) | Reply::Acknowledged(_) | Reply::Turn(_) | Reply::WriterWaiting(_)) => {
            panic!("unexpected amendment reply: {reply:?}");
        }
    }
}

fn drain(world: &mut World) {
    for (task, _) in world.stops.clone() {
        if world.runs.contains_key(&task) {
            assert_eq!(world.terminal(task, End::Parked), Reply::Acknowledged(Accepted::New));
        }
    }
    while let Some(task) = world.closing.first().copied() {
        world.settle(task);
    }
}

fn control(world: &mut World, task: u64, control: Control) {
    let reply_to = world.to();
    let call = reply_to.into_token().raw();
    let reply_to = ReplyTo::new(Token::new(call));
    world.send(Event::Control { reply_to, by: PERSON, task, control });
    match world.replies.get(&call).expect("control reply") {
        Reply::Done => {}
        Reply::Refused(problem) if problem.why == Refusal::State => {}
        reply @ (Reply::Made(_)
        | Reply::Refused(_)
        | Reply::Acknowledged(_)
        | Reply::Turn(_)
        | Reply::WriterWaiting(_)) => {
            panic!("unexpected control reply: {reply:?}");
        }
    }
}

// i3-0517.md: a released failed-dependency hold can incorrectly become Due.
// Use only observed requester endings to avoid claiming that known domain gap.
fn dependencies_done(world: &World, task: u64) -> bool {
    for dependency in &world.record(task).dependencies {
        if !matches!(world.results.get(dependency), Some(jig_domain_tasks::Ending::Done(_))) {
            return false;
        }
    }
    true
}

// i3-0526.md: the backward pool walk misses a circle made by keeping its
// second holder. Draw those amendments, but leave that known case unsent.
fn keeps_second_pool_holder(world: &World, task: u64, dependencies: &[u64]) -> bool {
    let row = world.record(task);
    if !row.holds_taken || !matches!(row.phase, Phase::Active(_)) {
        return false;
    }
    let mut pending = false;
    for dependency in dependencies {
        if world.records.contains_key(&Key::Live(*dependency)) {
            pending = true;
        }
    }
    if !pending {
        return false;
    }
    for holding in &row.holdings {
        let Holding::Slot { pool, .. } = holding else { continue };
        for record in world.records.values() {
            if let Stored::Live(other) = record
                && other.number != task
                && other.holds_taken
                && matches!(other.phase, Phase::Active(_))
                && !other.waiting_on.is_empty()
            {
                for holding in &other.holdings {
                    if let Holding::Slot { pool: other_pool, .. } = holding
                        && other_pool == pool
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn apply(world: &mut World, operation: Operation, seen: &mut BTreeSet<Reached>, attempt: &mut u64) {
    match operation {
        Operation::Amend { task, dependencies } => {
            if !keeps_second_pool_holder(world, task, &dependencies) {
                amend(world, task, &dependencies, seen);
            }
        }
        Operation::HoldRelease { task } => {
            world.send(Event::Hold { task, why: Hold::Budget });
            drain(world);
            if world.records.contains_key(&Key::Live(task)) {
                control(world, task, Control::Release { again: None, message: None });
            }
        }
        Operation::Claim { task } => {
            if matches!(world.record(task).phase, Phase::Active(Active::Due))
                && !world.runs.contains_key(&task)
                && dependencies_done(world, task)
            {
                *attempt += 1;
                match world.claim_budget_writing(task, *attempt, 1, Box::new([])) {
                    Reply::Done => {}
                    Reply::Refused(problem) if problem.why == Refusal::Funding => {}
                    reply @ (Reply::Made(_)
                    | Reply::Refused(_)
                    | Reply::Acknowledged(_)
                    | Reply::Turn(_)
                    | Reply::WriterWaiting(_)) => {
                        panic!("unexpected claim reply: {reply:?}");
                    }
                }
            }
        }
        Operation::End { task, failed } => {
            if world.runs.contains_key(&task) {
                let kept = !world.record(task).waiting_on.is_empty();
                let result = if failed {
                    TaskResult::Failure { reason: Box::new([1]) }
                } else {
                    TaskResult::Report { words: Box::new([1]) }
                };
                match world.terminal(task, End::Finished { result, cancel_delegates: false }) {
                    Reply::Acknowledged(Accepted::New) => {
                        if kept && !failed {
                            assert!(matches!(world.record(task).phase, Phase::Active(Active::Idle)));
                            let mut parked = false;
                            for record in world.records.values() {
                                if let Stored::Activity(row) = record
                                    && row.task == task
                                    && matches!(row.line, Line::Kept { .. })
                                {
                                    parked = true;
                                }
                            }
                            assert!(parked, "kept done finish has a durable parked line");
                            seen.insert(Reached::Parked);
                        }
                    }
                    Reply::Refused(problem) if problem.why == Refusal::LiveDelegates => {}
                    reply @ (Reply::Made(_)
                    | Reply::Done
                    | Reply::Refused(_)
                    | Reply::Acknowledged(_)
                    | Reply::Turn(_)
                    | Reply::WriterWaiting(_)) => {
                        panic!("unexpected terminal reply: {reply:?}");
                    }
                }
            }
        }
        Operation::Cancel { task } => {
            control(world, task, Control::Cancel { reason: Box::new([1]), grace: None, message: None });
        }
        Operation::Restart => world.restart(),
    }
    drain(world);
}

#[test]
fn random_amendments_preserve_the_promise_and_leave_no_circle() {
    let mut random = Rng::new(DRAW_SOURCE);
    let mut seeds = BTreeSet::new();
    while seeds.len() < DRAWN_SEEDS {
        seeds.insert(random.below(65_536));
    }
    eprintln!("amendment drawn seeds={seeds:?}, steps={STEPS}");
    let mut seen = BTreeSet::new();
    let tracing = std::env::var_os("JIG_WORLD_TRACE").is_some();
    for seed in seeds {
        eprintln!("amendment seed={seed}");
        let mut world = population(seed);
        let mut random = Rng::new(seed);
        let mut attempt = 0;
        let mut left = BTreeSet::new();
        for step in 0..STEPS {
            if world.live() == 0 {
                break;
            }
            let mut queued = BTreeSet::new();
            for task in live(&world) {
                let row = world.record(task);
                if row.phase == Phase::Waiting && row.hold_wait_since.is_some() && row.waiting_on.is_empty() {
                    queued.insert(task);
                }
            }
            let operation = draw(&world, &mut random);
            if tracing {
                eprintln!("step {step}: {operation:?}");
            }
            apply(&mut world, operation, &mut seen, &mut attempt);
            for task in live(&world) {
                let row = world.record(task);
                if queued.contains(&task) && !row.waiting_on.is_empty() {
                    left.insert(task);
                }
                if left.contains(&task)
                    && row.phase == Phase::Waiting
                    && row.waiting_on.is_empty()
                    && row.hold_wait_since.is_some()
                {
                    seen.insert(Reached::QueueReturned);
                }
            }
            world.referee.assert_holding(seed);
        }
        for task in live(&world) {
            if world.records.contains_key(&Key::Live(task)) {
                control(&mut world, task, Control::Cancel { reason: Box::new([1]), grace: None, message: None });
                drain(&mut world);
            }
        }
        world.complete_cancel();
        assert_eq!(world.results.len(), 6, "seed {seed}: every task ends");
        assert_eq!(world.live(), 0);
    }
    eprintln!("amendment reach={seen:?}");
    for marker in MARKERS {
        assert!(seen.contains(&marker), "sweep reaches {marker:?}");
    }
}
