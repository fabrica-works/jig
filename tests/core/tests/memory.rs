use jig_core_world::world;
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Env, Time, Wall};
use skein_world::domain::heap::{self, Meter};

#[global_allocator]
static HEAP: heap::Counting = heap::Counting;

fn core_children_routes_and_an_in_flight_decision_fit_the_checked_bound() {
    let limits = world::limits();
    let bound = core::worst_case(&limits.core).expect("walking limits price every core owner");
    let configuration = world::config(91);
    let mut observer = jig_core_world::observations::Observer::new(&configuration);
    let config = configuration.core;
    let meter = Meter::new();
    meter.start();
    let mut domain = core::Core::new(config, &limits.core).expect("fixture core configuration");
    let measured = meter.end();
    meter.check(measured, bound, &limits.core);

    domain.restart_begin();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.core };
    for event in [
        core::Event::People(people::Event::Restored),
        core::Event::Tasks(tasks::Event::Restored),
        core::Event::Fleet(fleet::Event::Loaded),
        core::Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
            form: accounts::Form::Credential(accounts::Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        })),
    ] {
        let room = core::room(&limits.core, &event).expect("event route has representable room");
        assert!(room.writes <= limits.journal.writes);
        assert!(room.held <= limits.journal.held);
        meter.start();
        let requests = core::step(&mut domain, &env, event);
        let measured = meter.end();
        drop(requests);
        let held = meter.check(measured, bound, &limits.core);
        observer.observe(0, jig_core_world::referee::Observed::Heap { held, maximum: bound });
    }
    observer.observe(0, jig_core_world::referee::Observed::Heap { held: meter.held(), maximum: bound });
}

fn room_and_memory_refuse_unrepresentable_or_missing_limits() {
    let limits = world::limits().core;
    assert!(core::worst_case(&core::Limits { connectors: 0, ..limits }).is_some());
    assert!(core::worst_case(&core::Limits { resume_bytes: 0, ..limits }).is_none());
    assert!(core::worst_case(&core::Limits { call_records: 0, ..limits }).is_none());
    assert!(
        core::worst_case(&core::Limits { tasks: tasks::Limits { tasks: u32::MAX, ..limits.tasks }, ..limits })
            .is_none()
    );
}

fn both_worlds_check_the_counted_heap_at_every_iteration() {
    let mut walking = world::World::goal(190);
    walking.run_goal();
    drop(walking);
    let mut effects = jig_core_world::effects::World::new(191, false);
    effects.delegate();
    effects.call(2, 9000);
    effects.restart(191, false);
    assert!(jig_core_world::observations::root_bound(&world::limits()).is_some());
}

// Each named scenario preserves its own fixture and original assertions.
#[test]
fn memory_scenario_family() {
    core_children_routes_and_an_in_flight_decision_fit_the_checked_bound();
    room_and_memory_refuse_unrepresentable_or_missing_limits();
    both_worlds_check_the_counted_heap_at_every_iteration();
}

#[test]
fn settled_finishing_queries_allocate_nothing_and_empty_closure_prices_one_output() {
    let limits = world::limits().core;
    let meter = Meter::new();
    meter.start();
    let mut domain = core::Core::new(world::config(192).core, &limits).expect("fixture core");
    domain.counters = core::Counters::new(domain.counters.deployment());
    let setup = meter.end();
    let bound = core::worst_case(&limits).expect("validated core heap bound");
    meter.check(setup, bound, &limits);
    let held = meter.held();
    meter.start();
    for _ in 0..100 {
        assert!(!domain.finish_owed());
    }
    let queries = meter.end();
    assert_eq!((queries.held(), queries.peak()), (held, held));
    meter.check(queries, bound, &limits);
    meter.start();
    let core::Requests::Out(mut out) =
        core::finish_decision(&mut domain, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits });
    let closed = meter.end();
    assert!(matches!(out.pop(), Some(core::Request::Decided)));
    assert!(out.is_empty());
    let one_output = skein_lib::Queue::<core::Request>::worst_case(1).expect("one output fits");
    assert_eq!(closed.peak() - held, one_output);
    drop(out);
    meter.check(closed, bound, &limits);
    meter.start();
    let mut quiet = skein_lib::Queue::with_capacity(0);
    jig_domain_views::step(
        &mut domain.views,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.views },
        jig_domain_views::Event::Text {
            run: skein_lib::Token::new(41),
            turn: jig_domain_views::TurnName { attempt: 1, turn: 1, retry: 0 },
            offset: 0,
            text: Box::new([b'x']),
            call: None,
        },
        &mut quiet,
    );
    let retained = meter.end();
    assert!(quiet.is_empty());
    meter.check(retained, bound, &limits);
    let held = meter.held();
    meter.start();
    for _ in 0..100 {
        assert!(domain.finish_owed());
    }
    let queries = meter.end();
    assert_eq!((queries.held(), queries.peak()), (held, held), "retained metadata inspection also allocates nothing");
    meter.check(queries, bound, &limits);
}
