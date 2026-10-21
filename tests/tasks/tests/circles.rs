use jig_domain_tasks::{
    Amendment, Circle, Control, Event, Hold, HoldKind, HoldSource, Holding, Kind, Limits, Name, Operation, Party,
    Phase, Refusal, Stored, Taken, Was,
};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::{Duration, List};

const ROOT: u64 = 1;
const REQUESTER: u64 = 2;
const HOLDER: u64 = 3;
const DEPENDENT: u64 = 4;
const PREREQUISITE: u64 = 5;
const WAITER: u64 = 6;
const LATER: u64 = 7;
const OTHER_HOLDER: u64 = 8;
const PERSON: u64 = 9;
const CONNECTOR: u16 = 1;
const KIND: u16 = 1;
const RESOURCE: u8 = 1;
const SEED: u64 = 320;

fn name() -> Name {
    Name { connector: CONNECTOR, path: Box::new([Box::new([RESOURCE])]) }
}

fn holding(pooled: bool) -> Holding {
    if pooled { Holding::Slot { pool: name(), kind: KIND } } else { Holding::Write { resource: name(), kind: KIND } }
}

fn configured(limits: &Limits, pooled: bool) -> World {
    let mut world = World::new(SEED, *limits);
    let hold =
        if pooled { HoldKind::Pooled { taken: Taken::Waits } } else { HoldKind::Exclusive { taken: Taken::Waits } };
    world.configure_holds(CONNECTOR, vec![Kind { connector: CONNECTOR, kind: KIND, hold }]);
    if pooled {
        world.send(Event::Slots { pool: name(), slots: 2 });
    }
    world
}

fn introduce(world: &mut World, left: u64, right: u64) {
    let reply_to = world.to();
    world.send(Event::Introduce { reply_to, by: ROOT, left, right });
}

// HOLDER waits for DEPENDENT, which waits for REQUESTER. Once WAITER under
// REQUESTER joins HOLDER's queue, those four waits would circle.
fn delayed(pooled: bool, limits: &Limits) -> World {
    let mut world = configured(limits, pooled);
    world.make(Party::Person(PERSON), vec![task(ROOT, &[])]);
    let mut holder = task(HOLDER, &[]);
    holder.holdings = Box::new([holding(pooled)]);
    world.make(Party::Task(ROOT), vec![task(REQUESTER, &[]), holder, task(PREREQUISITE, &[])]);
    if pooled {
        let mut other = task(OTHER_HOLDER, &[]);
        other.holdings = Box::new([holding(true)]);
        world.make(Party::Task(ROOT), vec![other]);
    }
    introduce(&mut world, HOLDER, REQUESTER);
    introduce(&mut world, REQUESTER, PREREQUISITE);
    world.make(Party::Task(HOLDER), vec![task(DEPENDENT, &[REQUESTER])]);
    let mut waiter = task(WAITER, &[PREREQUISITE]);
    waiter.holdings = Box::new([holding(pooled)]);
    world.make(Party::Task(REQUESTER), vec![waiter]);
    world
}

fn finish_prerequisite(world: &mut World) {
    world.claim(PREREQUISITE, PREREQUISITE);
    world.finish(PREREQUISITE);
    world.settle(PREREQUISITE);
}

fn named_circle(world: &World, number: u64) -> &Circle {
    let Phase::Held { was: Was::Waiting, why: Hold::Circle, source: Some(source) } = &world.record(number).phase else {
        panic!("waiter held for its circle: {:?}", world.record(number).phase)
    };
    let HoldSource::Circle(circle) = source.as_ref() else { panic!("circle source") };
    circle
}

fn release(world: &mut World, number: u64) {
    let reply_to = world.to();
    world.send(Event::Control {
        reply_to,
        by: Party::Person(PERSON),
        task: number,
        control: Control::Release { again: None, message: None },
    });
}

#[test]
fn a_grandchild_queuing_for_its_grandparents_hold_is_refused_whole() {
    let mut world = configured(&LIMITS, false);
    let mut grandparent = task(ROOT, &[]);
    grandparent.holdings = Box::new([holding(false)]);
    world.make(Party::Person(PERSON), vec![grandparent]);
    world.make(Party::Task(ROOT), vec![task(REQUESTER, &[])]);
    let mut grandchild = task(WAITER, &[]);
    grandchild.holdings = Box::new([holding(false)]);
    assert!(matches!(world.make(Party::Task(REQUESTER), vec![grandchild]), Reply::Refused(problem)
        if problem.why == Refusal::Circle { cut: false }
        && problem.blocked_by.as_deref() == Some(&[REQUESTER, WAITER, ROOT])));
    assert_eq!(world.live(), 2);
}

#[test]
fn a_circle_among_new_members_names_the_members_in_wait_order() {
    let mut world = configured(&LIMITS, false);
    assert!(matches!(world.make(Party::Person(PERSON), vec![task(REQUESTER, &[WAITER]), task(WAITER, &[REQUESTER])]),
        Reply::Refused(problem) if problem.why == Refusal::Circle { cut: false }
            && problem.blocked_by.as_deref() == Some(&[REQUESTER, WAITER])));
}

#[test]
fn a_queue_circle_lifts_at_its_place_and_counts_its_bound_afresh() {
    let mut world = delayed(false, &LIMITS);
    finish_prerequisite(&mut world);
    assert_eq!(named_circle(&world, WAITER).tasks.as_ref(), [WAITER, HOLDER, DEPENDENT, REQUESTER]);
    let place = world.record(WAITER).hold_wait_since.expect("first join saves its place");
    world.elapse(Duration::from_secs(6));
    let mut later = task(LATER, &[]);
    later.holdings = Box::new([holding(false)]);
    world.make(Party::Person(PERSON), vec![later]);
    let reply_to = world.to();
    world.send(Event::Control {
        reply_to,
        by: Party::Task(HOLDER),
        task: DEPENDENT,
        control: Control::Cancel { reason: Box::new([]), grace: None, message: None },
    });
    world.settle(DEPENDENT);
    assert_eq!(world.record(WAITER).phase, Phase::Waiting);
    assert_eq!(world.record(WAITER).hold_wait_since, Some(place));
    assert_eq!(world.record(WAITER).hold_bound_since, Some(world.env.wall));
    assert!(world.record(LATER).hold_wait_since > Some(place));
    world.elapse(Duration::from_secs(5));
    assert_eq!(world.record(WAITER).phase, Phase::Waiting, "the old bound has passed");
    world.claim(HOLDER, HOLDER);
    world.finish(HOLDER);
    world.settle(HOLDER);
    assert!(world.record(WAITER).holds_taken);
    assert!(!world.record(LATER).holds_taken);
}

#[test]
fn a_pool_shrink_holds_its_circling_waiter_and_a_raise_lifts_it() {
    let mut world = delayed(true, &LIMITS);
    finish_prerequisite(&mut world);
    let place = world.record(WAITER).hold_wait_since;
    world.send(Event::Slots { pool: name(), slots: 1 });
    assert_eq!(named_circle(&world, WAITER).tasks.as_ref(), [WAITER, HOLDER, DEPENDENT, REQUESTER]);
    world.elapse(Duration::from_secs(1));
    world.send(Event::Slots { pool: name(), slots: 2 });
    assert_eq!(world.record(WAITER).phase, Phase::Waiting);
    assert_eq!(world.record(WAITER).hold_wait_since, place);
    assert_eq!(world.record(WAITER).hold_bound_since, Some(world.env.wall));
    assert!(!world.record(WAITER).holds_taken);
}

#[test]
fn a_restored_circle_from_an_older_rule_holds_its_queue_waiter() {
    let world = delayed(false, &LIMITS);
    let mut rows: Vec<_> = world.records.values().filter(|row| row.key().live()).cloned().collect();
    for row in &mut rows {
        if let Stored::Live(waiter) = row
            && waiter.number == WAITER
        {
            waiter.dependencies = Box::new([]);
            waiter.waiting_on = Box::new([]);
            waiter.hold_wait_since = Some(world.env.wall);
        }
    }
    let mut domain = jig_tasks_world::domain(&LIMITS, SEED);
    let mut out = skein_lib::Queue::with_capacity(jig_domain_tasks::max_out(&LIMITS));
    jig_domain_tasks::step(
        &mut domain,
        &world.env,
        Event::Kinds {
            connector: CONNECTOR,
            kinds: Box::new([Kind {
                connector: CONNECTOR,
                kind: KIND,
                hold: HoldKind::Exclusive { taken: Taken::Waits },
            }]),
        },
        &mut out,
    );
    for row in rows {
        jig_domain_tasks::step(&mut domain, &world.env, Event::Restore { record: row }, &mut out);
    }
    jig_domain_tasks::step(&mut domain, &world.env, Event::Restored, &mut out);
    assert!(!out.iter().any(|request| matches!(request, jig_domain_tasks::Request::RestoreRefused { .. })));
    let waiter = domain.task(WAITER).expect("restored waiter");
    let Phase::Held { why: Hold::Circle, source: Some(source), .. } = &waiter.phase else { panic!("circle hold") };
    let HoldSource::Circle(circle) = source.as_ref() else { panic!("circle source") };
    assert_eq!(circle.tasks.as_ref(), [WAITER, HOLDER, DEPENDENT, REQUESTER]);
    assert!(waiter.hold_wait_since.is_some());
}

#[test]
fn a_release_giving_back_a_circling_queue_wait_holds_it_again() {
    let mut world = delayed(false, &LIMITS);
    finish_prerequisite(&mut world);
    let place = world.record(WAITER).hold_wait_since;
    let revision = world.record(WAITER).revisions.hold;
    release(&mut world, WAITER);
    assert_eq!(named_circle(&world, WAITER).tasks.as_ref(), [WAITER, HOLDER, DEPENDENT, REQUESTER]);
    assert_eq!(world.record(WAITER).hold_wait_since, place);
    assert_eq!(world.record(WAITER).revisions.hold, revision + 1);
}

#[test]
fn a_circle_stops_naming_tasks_at_the_configured_limit() {
    const NAMED: u32 = 2;
    let mut world = delayed(false, &Limits { circle: NAMED, ..LIMITS });
    finish_prerequisite(&mut world);
    let circle = named_circle(&world, WAITER);
    assert_eq!(circle.tasks.as_ref(), [WAITER, HOLDER]);
    assert!(circle.cut);
}

#[test]
fn a_release_of_a_holder_rechecks_its_delegate_waits_and_lifts_after_they_end() {
    let mut world = delayed(false, &LIMITS);
    finish_prerequisite(&mut world);
    world.send(Event::Hold { task: HOLDER, why: Hold::Budget });
    release(&mut world, HOLDER);
    let Phase::Held { was: Was::Active(_), why: Hold::Circle, source: Some(source) } = &world.record(HOLDER).phase
    else {
        panic!("released holder's delegate wait circles")
    };
    let HoldSource::Circle(circle) = source.as_ref() else { panic!("circle source") };
    assert_eq!(circle.tasks.as_ref(), [HOLDER, DEPENDENT, REQUESTER, WAITER]);
    let reply_to = world.to();
    world.send(Event::Control {
        reply_to,
        by: Party::Task(HOLDER),
        task: DEPENDENT,
        control: Control::Cancel { reason: Box::new([]), grace: None, message: None },
    });
    world.settle(DEPENDENT);
    assert_eq!(world.record(HOLDER).phase, Phase::Active(jig_domain_tasks::Active::Due));
}

fn amendment(world: &World, number: u64, dependencies: &[u64]) -> Operation {
    Operation::Amend {
        task: number,
        message: world.record(number).last_message + 1,
        stop_run: false,
        amendment: Box::new(Amendment {
            tracked: None,
            addressed: None,
            instructions: None,
            release: None,
            spec: None,
            wake: None,
            dependencies: Some(dependencies.into()),
            authority: None,
            llm: None,
            reason: Box::new([]),
        }),
    }
}

fn compound(world: &mut World, operations: Vec<Operation>) -> Reply {
    let bound = u32::try_from(operations.len()).expect("bounded fixture");
    let mut staged = List::with_capacity(bound);
    for operation in operations {
        staged.push(operation).expect("compound fixture");
    }
    let reply_to = world.to();
    let call = reply_to.into_token();
    world.send(Event::Compound {
        reply_to: skein_lib::ReplyTo::new(call),
        person: PERSON,
        project: 1,
        operations: staged,
        period: 0,
        pool_budget: 1_000,
        period_budget: 10_000,
    });
    world.replies[&call.raw()].clone()
}

fn queued_delegate() -> World {
    let mut world = configured(&LIMITS, false);
    let mut holder = task(HOLDER, &[]);
    holder.holdings = Box::new([holding(false)]);
    world.make(Party::Person(PERSON), vec![holder, task(REQUESTER, &[])]);
    let mut waiter = task(WAITER, &[]);
    waiter.holdings = Box::new([holding(false)]);
    world.make(Party::Task(REQUESTER), vec![waiter]);
    world
}

#[test]
fn a_compound_dependency_circle_is_refused_whole_naming_both_tasks() {
    let mut world = configured(&LIMITS, false);
    world.make(Party::Person(PERSON), vec![task(HOLDER, &[]), task(REQUESTER, &[])]);
    let before = world.records.clone();
    let operations = vec![amendment(&world, HOLDER, &[REQUESTER]), amendment(&world, REQUESTER, &[HOLDER])];
    assert!(matches!(compound(&mut world, operations), Reply::Refused(problem)
        if problem.why == Refusal::Circle { cut: false }
        && problem.blocked_by.as_deref() == Some(&[REQUESTER, HOLDER])));
    assert_eq!(world.records, before);
}

#[test]
fn a_compound_removed_dependency_is_absent_from_the_circle_walk() {
    let mut world = configured(&LIMITS, false);
    world.make(Party::Person(PERSON), vec![task(HOLDER, &[REQUESTER]), task(REQUESTER, &[])]);
    let operations = vec![amendment(&world, HOLDER, &[]), amendment(&world, REQUESTER, &[HOLDER])];
    assert_eq!(compound(&mut world, operations), Reply::Done);
    assert!(world.record(HOLDER).waiting_on.is_empty());
    assert_eq!(world.record(REQUESTER).waiting_on.as_ref(), [HOLDER]);
}

#[test]
fn a_compound_release_exposes_its_waiters_queue_to_later_dependencies() {
    let mut world = queued_delegate();
    world.send(Event::Hold { task: WAITER, why: Hold::Budget });
    let before = world.records.clone();
    let operations =
        vec![Operation::Release { task: WAITER, again: None, message: None }, amendment(&world, HOLDER, &[REQUESTER])];
    assert!(matches!(compound(&mut world, operations), Reply::Refused(problem)
        if problem.why == Refusal::Circle { cut: false }
        && problem.blocked_by.as_deref() == Some(&[HOLDER, REQUESTER, WAITER])));
    assert_eq!(world.records, before);
}

#[test]
fn a_compound_stop_removes_its_waiters_queue_from_the_walk() {
    let mut world = queued_delegate();
    let operations = vec![Operation::Stop { task: WAITER }, amendment(&world, HOLDER, &[REQUESTER])];
    assert_eq!(compound(&mut world, operations), Reply::Done);
    assert_eq!(world.record(HOLDER).waiting_on.as_ref(), [REQUESTER]);
}

#[test]
fn a_compound_move_removes_the_old_requesters_delegate_wait() {
    let mut world = queued_delegate();
    let operations =
        vec![Operation::Move { task: WAITER, reason: Box::new([]) }, amendment(&world, HOLDER, &[REQUESTER])];
    assert_eq!(compound(&mut world, operations), Reply::Done);
    assert_eq!(world.record(WAITER).requester, Party::Person(PERSON));
    assert_eq!(world.record(HOLDER).waiting_on.as_ref(), [REQUESTER]);
}

#[test]
fn a_compound_cancel_removes_the_subtrees_dependency_and_queue_waits() {
    let mut world = queued_delegate();
    let operations = vec![
        Operation::Cancel { task: REQUESTER, reason: Box::new([]), grace: None, message: None },
        amendment(&world, HOLDER, &[REQUESTER]),
    ];
    assert_eq!(compound(&mut world, operations), Reply::Done);
    assert_eq!(world.record(HOLDER).waiting_on.as_ref(), [REQUESTER]);
}

#[test]
fn a_compound_reheld_release_does_not_restore_a_circling_queue_edge() {
    let mut world = delayed(false, &LIMITS);
    finish_prerequisite(&mut world);
    let operations =
        vec![Operation::Release { task: WAITER, again: None, message: None }, amendment(&world, HOLDER, &[REQUESTER])];
    assert_eq!(compound(&mut world, operations), Reply::Done);
    assert_eq!(world.record(HOLDER).waiting_on.as_ref(), [REQUESTER]);
    assert_eq!(named_circle(&world, WAITER).tasks.as_ref(), [WAITER, HOLDER, REQUESTER]);
}

#[test]
fn a_procedure_step_judges_earlier_amendments_before_refusing_a_circle_whole() {
    use jig_domain_tasks::{Executor, ProcedureAmendment, ProcedureDecision, ProcedureEnd};
    let mut world = configured(&LIMITS, false);
    let mut root = task(ROOT, &[]);
    root.executor = Executor::Procedure { connector: CONNECTOR, code: u32::from(KIND) };
    world.make(Party::Person(PERSON), vec![root]);
    world.make(Party::Task(ROOT), vec![task(HOLDER, &[]), task(REQUESTER, &[])]);
    let mut parts = Vec::new();
    for operation in [amendment(&world, HOLDER, &[REQUESTER]), amendment(&world, REQUESTER, &[HOLDER])] {
        let Operation::Amend { task, message, stop_run, amendment } = operation else {
            unreachable!("amendment fixture")
        };
        parts.push(ProcedureAmendment { task, message, stop_run, amendment: *amendment });
    }
    let mut decision = ProcedureDecision::new(ProcedureEnd::Wait);
    decision.amendments = parts.into_boxed_slice();
    let before = world.records.clone();
    let reply_to = world.to();
    let call = reply_to.into_token();
    world.send(Event::Procedure { reply_to: skein_lib::ReplyTo::new(call), task: ROOT, step: 1, read: None, decision });
    assert!(matches!(&world.replies[&call.raw()], Reply::Refused(problem)
        if problem.why == Refusal::Circle { cut: false }
        && problem.blocked_by.as_deref() == Some(&[REQUESTER, HOLDER])));
    assert_eq!(world.records, before);
}

#[test]
fn a_compound_amendment_that_reholds_a_returning_waiter_adds_no_queue_edge() {
    let mut world = delayed(false, &LIMITS);
    finish_prerequisite(&mut world);
    world.make(Party::Person(PERSON), vec![task(OTHER_HOLDER, &[])]);
    let mut operation = amendment(&world, WAITER, &[OTHER_HOLDER]);
    let Operation::Amend { amendment: change, .. } = &mut operation else { unreachable!("amendment fixture") };
    change.release = Some(world.record(WAITER).revisions.hold);
    compound(&mut world, vec![operation]);
    let operations = vec![amendment(&world, WAITER, &[]), amendment(&world, HOLDER, &[REQUESTER])];
    assert_eq!(compound(&mut world, operations), Reply::Done);
    assert_eq!(world.record(HOLDER).waiting_on.as_ref(), [REQUESTER]);
    assert_eq!(named_circle(&world, WAITER).tasks.as_ref(), [WAITER, HOLDER, REQUESTER]);
}
