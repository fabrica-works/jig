use jig_engine_workers_world::{
    World,
    fixtures::{self, SECRET_A, SECRET_B, WORKER_A, WORKER_B},
    referee,
};
use jig_protocol_engine_workers::{Event, Request};
use skein_lib::{Duration, Rng, Token};

fn drawn_worker(world: &mut World, rng: &mut Rng, coverage: &mut [bool; 7]) {
    let fault = usize::try_from(rng.between(0, 6)).expect("seven faults");
    coverage[fault] = true;
    let (name, secret) = if rng.between(0, 1) == 0 { (WORKER_A, SECRET_A) } else { (WORKER_B, SECRET_B) };
    let secret = if fault == 0 { b"wrong".as_slice() } else { secret };
    let connection = world.connect(fixtures::reporting_worker(name, secret));
    world.run();
    if world.gone(connection) {
        return;
    }
    match fault {
        0 | 1 => {}
        2 => {
            world.play(connection, fixtures::frame(0x0101, &[255]));
        }
        3 => {
            world.end_stream(connection);
        }
        4 => {
            world.fail_stream(connection);
        }
        5 => {
            world.withhold_write_room(connection);
            let Some(Request::Hello { channel, .. }) = world.root().last() else {
                panic!("hello");
            };
            world.down(Event::TurnBusy { channel: *channel, run: Token::new(7), attempt: Token::new(2), turn: 3 });
        }
        6 => {
            let frame = fixtures::turn_frame();
            world.inject(connection, &frame.bytes()[..10]);
            world.end_stream(connection);
        }
        _ => unreachable!(),
    }
    world.run();
}

#[test]
fn fuzzy_workers_cuts_and_stalls_keep_each_channel_told_once() {
    let mut coverage = [false; 7];
    for seed in 0..40 {
        let mut world = World::new(&fixtures::limits(), &fixtures::workers(), seed);
        let mut rng = Rng::new(seed);
        for _ in 0..6 {
            drawn_worker(&mut world, &mut rng, &mut coverage);
            world.advance(Duration::from_millis(rng.between(1, 400)));
        }
        world.down(Event::Stop);
        world.down(Event::Close);
        world.advance(Duration::from_secs(30));
        referee::judge(world.root()).unwrap_or_else(|why| panic!("seed {seed}: {why}"));
        assert!(matches!(world.root().last(), Some(Request::Closed)), "seed {seed}: closed");
    }
    assert_eq!(coverage, [true; 7]);
}
