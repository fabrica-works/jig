use jig_protocol_worker_link::{Component, Event, MAX_OUT_DOWN, worst_case};
use jig_worker_link_world::fixtures;
use skein_lib::{Queue, Secret, Time, Token};
use skein_world::domain::heap::{self, Meter};
#[global_allocator]
static HEAP: heap::Counting = heap::Counting;
#[test]
fn the_link_at_its_limits_stays_within_its_worst_case() {
    let limits = fixtures::limits();
    let bound = worst_case(&limits).expect("small bound");
    let mut out = Queue::with_capacity(MAX_OUT_DOWN);
    let mut credential = fixtures::credential();
    credential.name = vec![b'w'; usize::try_from(limits.sizes.name).expect("small name")].into();
    credential.secret = Secret::new(vec![b's'; usize::try_from(limits.sizes.secret).expect("small secret")].into());
    let meter = Meter::new();
    meter.start();
    let mut component = Component::new(&limits, credential, 1).expect("full credential");
    component.fire(Time::ZERO, &mut out);
    let dial = out.pop().expect("dial");
    drop(dial);
    component.down(Time::ZERO, Event::Dialled { connection: Token::new(1) }, &mut out);
    let measured = meter.end();
    meter.check(measured, bound, &limits);
}
