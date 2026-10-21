use jig_domain_tasks::{
    Active, Amendment, Class, Control, End, Event, Hold, Key, Operation, Party, Phase, Refusal, Stored,
};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::{List, ReplyTo};

const KEPT: u64 = 1;
const DEPENDENCY: u64 = 2;
const PERSON: u64 = 9;
const SEED: u64 = 870;
// One amendment followed by one release.
const PAIR: u32 = 2;

fn response(world: &mut World, event: impl FnOnce(ReplyTo) -> Event) -> Reply {
    let to = world.to();
    let call = to.into_token();
    world.send(event(ReplyTo::new(call)));
    world.replies[&call.raw()].clone()
}

fn amendment(world: &World, release: bool) -> Amendment {
    Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        release: release.then_some(world.record(KEPT).revisions.hold),
        spec: None,
        wake: None,
        dependencies: Some(Box::new([DEPENDENCY])),
        authority: None,
        llm: None,
        reason: Box::new([]),
    }
}

fn pair() -> World {
    let mut world = World::new(SEED, LIMITS);
    world.make(Party::Person(PERSON), vec![task(KEPT, &[]), task(DEPENDENCY, &[])]);
    world
}

fn keep(world: &mut World, release: bool) -> Reply {
    let amendment = amendment(world, release);
    let message = world.record(KEPT).last_message + 1;
    response(world, |reply_to| Event::Amend {
        places: Box::new([]),
        reply_to,
        by: Party::Person(PERSON),
        task: KEPT,
        message,
        stop_run: false,
        funder: None,
        amendment,
    })
}

fn unclaimable(world: &mut World) {
    assert_eq!(world.record(KEPT).phase, Phase::Active(Active::Idle));
    assert!(!world.activations.contains(&KEPT));
    assert!(!world.runs.contains_key(&KEPT));
    let attempt = world.record(KEPT).attempt + 1;
    assert!(matches!(response(world, |reply_to| Event::Claim {
        reply_to, task: KEPT, attempt, resumed: false, budget: 1, account: None, writes: Box::new([]),
    }), Reply::Refused(problem) if problem.why == Refusal::State));
}

#[test]
fn a_kept_agents_retry_alarm_leaves_it_idle_without_a_claim() {
    let mut world = pair();
    world.claim(KEPT, KEPT);
    world.terminal(KEPT, End::Failed(Class::Transient));
    keep(&mut world, false);
    world.advance();
    unclaimable(&mut world);
}

#[test]
fn a_held_kept_agent_releases_idle_without_a_claim() {
    let mut world = pair();
    world.send(Event::Hold { task: KEPT, why: Hold::Budget });
    keep(&mut world, false);
    world.activations.clear();
    response(&mut world, |reply_to| Event::Control {
        reply_to,
        by: Party::Person(PERSON),
        task: KEPT,
        control: Control::Release { again: None, message: None },
    });
    unclaimable(&mut world);
}

#[test]
fn an_amendment_adding_a_dependency_and_releasing_keeps_the_agent_idle() {
    let mut world = pair();
    world.send(Event::Hold { task: KEPT, why: Hold::Budget });
    world.activations.clear();
    assert_eq!(keep(&mut world, true), Reply::Done);
    assert_eq!(world.record(KEPT).waiting_on.as_ref(), [DEPENDENCY]);
    unclaimable(&mut world);
}

#[test]
fn a_compound_amendment_then_release_keeps_the_agent_idle() {
    let mut world = pair();
    world.send(Event::Hold { task: KEPT, why: Hold::Budget });
    world.activations.clear();
    let mut operations = List::with_capacity(PAIR);
    operations
        .push(Operation::Amend {
            task: KEPT,
            message: 1,
            stop_run: false,
            amendment: Box::new(amendment(&world, false)),
        })
        .expect("amendment fixture");
    operations.push(Operation::Release { task: KEPT, again: None, message: None }).expect("release fixture");
    assert_eq!(
        response(&mut world, |reply_to| Event::Compound {
            reply_to,
            person: PERSON,
            project: 1,
            operations,
            period: 0,
            pool_budget: 1_000,
            period_budget: 10_000,
        }),
        Reply::Done
    );
    unclaimable(&mut world);
}

#[test]
fn restored_due_and_preparing_kept_agents_resume_idle_without_claims() {
    for phase in [Active::Due, Active::Preparing] {
        let mut world = pair();
        keep(&mut world, false);
        let Some(Stored::Live(row)) = world.records.get_mut(&Key::Live(KEPT)) else { panic!("kept fixture") };
        row.phase = Phase::Active(phase);
        world.restart();
        unclaimable(&mut world);
    }
}

#[test]
fn a_restored_kept_backoff_keeps_its_alarm_and_then_becomes_idle() {
    let mut world = pair();
    world.claim(KEPT, KEPT);
    world.terminal(KEPT, End::Failed(Class::Transient));
    keep(&mut world, false);
    let phase = world.record(KEPT).phase.clone();
    world.restart();
    assert_eq!(world.record(KEPT).phase, phase);
    world.advance();
    unclaimable(&mut world);
}

#[test]
fn an_escalation_release_reholds_a_circle_and_keeps_its_queue_place() {
    use jig_domain_tasks::{EscalationDecision, EscalationHolder, HoldKind, Holding, Kind, Name, Taken};
    use skein_lib::Duration;
    const WAITER: u64 = 3;
    const LATER: u64 = 4;
    const ENTRY: u64 = 100;
    const PREREQUISITE: u64 = 5;
    let mut world = World::new(SEED, LIMITS);
    world.configure_holds(1, vec![Kind { connector: 1, kind: 1, hold: HoldKind::Exclusive { taken: Taken::Waits } }]);
    let holding = Holding::Write { resource: Name { connector: 1, path: Box::new([Box::new([1])]) }, kind: 1 };
    let mut holder = task(KEPT, &[]);
    holder.holdings = Box::new([holding.clone()]);
    world.make(Party::Person(PERSON), vec![holder, task(DEPENDENCY, &[])]);
    let mut waiter = task(WAITER, &[PREREQUISITE]);
    waiter.holdings = Box::new([holding.clone()]);
    world.make(Party::Task(DEPENDENCY), vec![task(PREREQUISITE, &[]), waiter]);
    keep(&mut world, false);
    world.claim(PREREQUISITE, PREREQUISITE);
    world.finish(PREREQUISITE);
    world.settle(PREREQUISITE);
    let place = world.record(WAITER).hold_wait_since;
    let revision = world.record(WAITER).escalation.revision();
    world.send(Event::RoutedEscalation {
        task: WAITER,
        revision,
        holder: EscalationHolder::Person(PERSON),
        entry: ENTRY,
    });
    let reply_to = world.to();
    world.stage(Event::DecideEscalation {
        reply_to,
        task: WAITER,
        revision,
        by: PERSON,
        entry: None,
        decision: EscalationDecision::Release,
    });
    world.durable();
    // Crash after committing the decision and before delivering its reply.
    world.restart();
    assert!(matches!(world.record(WAITER).phase, Phase::Held { why: Hold::Circle, .. }));
    assert_eq!(world.record(WAITER).hold_wait_since, place);
    world.elapse(Duration::from_secs(1));
    let mut later = task(LATER, &[]);
    later.holdings = Box::new([holding]);
    world.make(Party::Person(PERSON), vec![later]);
    let mut remove = amendment(&world, false);
    remove.dependencies = Some(Box::new([]));
    let message = world.record(KEPT).last_message + 1;
    response(&mut world, |reply_to| Event::Amend {
        places: Box::new([]),
        reply_to,
        by: Party::Person(PERSON),
        task: KEPT,
        message,
        stop_run: false,
        funder: None,
        amendment: remove,
    });
    assert_eq!(world.record(WAITER).phase, Phase::Waiting);
    assert_eq!(world.record(WAITER).hold_wait_since, place);
    assert_eq!(world.record(WAITER).hold_bound_since, Some(world.env.wall));
    assert!(world.record(LATER).hold_wait_since > place);
    world.claim(KEPT, KEPT);
    world.finish(KEPT);
    world.settle(KEPT);
    assert!(world.record(WAITER).holds_taken);
    assert!(!world.record(LATER).holds_taken);
}
