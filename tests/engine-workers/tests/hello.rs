mod hello {
    use jig_engine_workers_world::{
        World,
        fixtures::{self, PAYLOAD, SECRET_A, WORKER_A},
    };
    use jig_protocol_engine_workers::{Event, Request};
    use skein_lib::{Duration, Token};
    fn world() -> World {
        World::new(&fixtures::limits(), &fixtures::workers(), 1)
    }
    fn reporting(world: &mut World) -> Token {
        world.connect(fixtures::reporting_worker(WORKER_A, SECRET_A))
    }
    fn first_channel(world: &World) -> Token {
        let Request::Hello { channel, .. } = &world.root()[0] else {
            panic!("first is hello");
        };
        *channel
    }
    #[test]
    fn the_root_hears_a_channel_only_with_its_hello() {
        let mut world = world();
        let connection = world.connect(fixtures::worker(WORKER_A, SECRET_A, PAYLOAD));
        world.run();
        assert!(world.root().is_empty());
        world.play(connection, fixtures::hello_frame());
        world.run();
        assert_eq!(first_channel(&world), Token::new(1));
    }
    #[test]
    fn a_turn_before_the_hello_is_refused_as_framing() {
        let mut peer = fixtures::worker(WORKER_A, SECRET_A, PAYLOAD);
        peer.play(fixtures::turn_frame());
        let mut world = world();
        let connection = world.connect(peer);
        world.run();
        assert_eq!(world.refusal(connection), Some((3, Box::from(b"".as_slice()))));
        assert!(world.root().is_empty());
    }
    #[test]
    fn a_hello_past_its_deadline_closes_the_channel_unheard() {
        let mut world = world();
        let connection = world.connect(fixtures::worker(WORKER_A, SECRET_A, PAYLOAD));
        world.run();
        world.advance(Duration::from_secs(2));
        world.play(connection, fixtures::hello_frame());
        world.run();
        assert!(world.gone(connection));
        assert!(world.root().is_empty());
    }
    #[test]
    fn a_hello_not_admitted_is_refused_with_the_fleets_text() {
        let mut world = world();
        let connection = reporting(&mut world);
        world.run();
        world.down(Event::NotAdmitted { channel: first_channel(&world), text: Box::from(b"agent bounds".as_slice()) });
        world.run();
        assert_eq!(world.refusal(connection), Some((260, Box::from(b"agent bounds".as_slice()))));
    }
    #[test]
    fn a_replaced_channel_is_lost_before_its_successors_hello() {
        let mut world = world();
        reporting(&mut world);
        world.run();
        reporting(&mut world);
        world.run();
        assert_eq!(world.root().len(), 3);
        let Request::Lost { channel } = world.root()[1] else {
            panic!("old channel lost");
        };
        assert_eq!(channel, Token::new(1));
        let Request::Hello { channel, .. } = world.root()[2] else {
            panic!("successor hello");
        };
        assert_eq!(channel, Token::new(2));
    }
    #[test]
    fn channel_tokens_start_at_one_and_never_repeat() {
        let mut world = world();
        for _ in 0..3 {
            reporting(&mut world);
            world.run();
        }
        let channels: Vec<_> = world
            .root()
            .iter()
            .filter_map(|request| if let Request::Hello { channel, .. } = request { Some(*channel) } else { None })
            .collect();
        assert_eq!(channels, [Token::new(1), Token::new(2), Token::new(3)]);
    }
}
