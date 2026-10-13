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

fn full_call(run: u64, name: u8) -> jig_domain_host::LinkUp {
    jig_domain_host::LinkUp::Call {
        run: Token::new(run),
        attempt: Token::new(2),
        call: Box::new([name; 16]),
        tool: Box::new([b't'; 16]),
        writes: true,
        input: Box::new([b'i'; 16]),
        deadline: skein_lib::Duration::from_secs(60),
    }
}
fn busy_frame(run: u64, turn: u32) -> skein_channel::Frame {
    let record = jig_protocol_worker_wire::TurnBusy::new(
        &fixtures::codec(),
        jig_protocol_worker_wire::TurnBusyParts { task: run, attempt: 2, number: turn },
    )
    .expect("busy");
    fixtures::frame(0x0184, &fixtures::encoded(record.measure(), |writer| record.encode(writer)))
}
#[test]
fn relays_and_busy_turns_at_their_limits_stay_within_the_worst_case() {
    let mut limits = fixtures::limits();
    limits.sizes.slots = 2;
    limits.sizes.calls = 2;
    limits.sizes.turns = 2;
    let bound = worst_case(&limits).expect("small bound");
    let mut world = jig_worker_link_world::World::new(&limits, 1);
    world.engine(fixtures::engine_accepting_with(&limits));
    world.run();
    world.down(Event::Hello { hello: fixtures::hello() });
    let calls: Vec<_> =
        [7, 8].into_iter().flat_map(|run| [1, 2].into_iter().map(move |name| full_call(run, name))).collect();
    let busy: Vec<_> =
        [7, 8].into_iter().flat_map(|run| [1, 2].into_iter().map(move |turn| busy_frame(run, turn))).collect();
    let meter = Meter::new();
    meter.start();
    for up in calls {
        world.down(Event::Up { up, parts: Box::new([]) });
    }
    for frame in busy {
        world.play(frame);
    }
    let measured = meter.end();
    meter.check(measured, bound, &limits);
    world.advance(skein_lib::Duration::from_millis(200));
    assert_eq!(
        world
            .root()
            .iter()
            .filter(|request| matches!(request, jig_protocol_worker_link::Request::Resend { .. }))
            .count(),
        4
    );
}
