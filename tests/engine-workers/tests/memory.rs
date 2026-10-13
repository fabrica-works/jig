use jig_engine_workers_world::fixtures;
use jig_protocol_engine_workers::{Component, Event, MAX_OUT_DOWN, Request, worst_case};
use skein_lib::{Queue, Time, Token};
use skein_world::domain::heap::{self, Meter};

#[global_allocator]
static HEAP: heap::Counting = heap::Counting;

fn checked_step(meter: &Meter, measured: heap::Measured, bound: u64, out: &mut Queue<Request>) {
    while let Some(request) = out.pop() {
        drop(request);
    }
    meter.check(measured, bound, &"engine worker opening");
}

#[test]
fn the_component_at_its_limits_stays_within_its_worst_case() {
    let limits = fixtures::limits();
    let bound = worst_case(&limits).expect("bounded component");
    let meter = Meter::new();
    meter.start();
    let mut component = Component::new(&limits, fixtures::workers(), fixtures::PAYLOAD).expect("valid component");
    let mut out = Queue::with_capacity(MAX_OUT_DOWN);
    let measured = meter.end();
    checked_step(&meter, measured, bound, &mut out);
    for connection in 1..=3 {
        meter.start();
        component.down(Time::ZERO, Event::Accepted { connection: Token::new(connection) }, &mut out);
        let measured = meter.end();
        checked_step(&meter, measured, bound, &mut out);
    }
    assert!(meter.held() > 0, "all connection machines are retained");
}

#[test]
fn every_connection_at_its_cap_stays_within_the_worst_case() {
    use jig_engine_workers_world::World;
    let limits = fixtures::limits();
    let bound = worst_case(&limits).expect("bounded component");
    let cap = jig_protocol_worker_wire::sizes::engine_cap(&limits.sizes).expect("cap");
    let meter = Meter::new();
    meter.start();
    let mut world = World::new(&limits, &fixtures::workers(), 1);
    let peers = [(fixtures::WORKER_A, fixtures::SECRET_A), (fixtures::WORKER_B, fixtures::SECRET_B)];
    for (name, secret) in peers {
        let (connection, channel) = world.reporting(name, secret);
        world.withhold_write_room(connection);
        for _ in 0..cap.frames {
            world.down(Event::TurnBusy { channel, run: Token::new(7), attempt: Token::new(2), turn: 3 });
        }
        assert!(!world.gone(connection), "every frame fits the cap");
    }
    world.connect(fixtures::worker(fixtures::WORKER_C, fixtures::SECRET_C, fixtures::PAYLOAD));
    let measured = meter.end();
    meter.check(measured, bound, &"engine channels at cap, including byte-world overhead");
    assert!(meter.held() > 0);
}
