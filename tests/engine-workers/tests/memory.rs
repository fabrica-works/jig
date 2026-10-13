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
