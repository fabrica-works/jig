mod records {
    use jig_engine_workers_world::{
        World,
        fixtures::{self, SECRET_A, WORKER_A},
    };
    use jig_protocol_engine_workers::Request;
    use skein_lib::Token;
    fn world() -> World {
        World::new(&fixtures::limits(), &fixtures::workers(), 1)
    }
    fn reporting() -> skein_fake_channel::ScriptedPeer {
        fixtures::reporting_worker(WORKER_A, SECRET_A)
    }
    #[derive(Debug, PartialEq, Eq)]
    enum Record {
        Hello,
        Turn,
        Call,
        Answer,
        Activity,
        Rejected,
        Exhausted,
        Lost,
    }
    fn record(request: &Request) -> Record {
        use jig_domain_host::LinkUp;
        match request {
            Request::Hello { .. } => Record::Hello,
            Request::Up { up, .. } => match up {
                LinkUp::Turn { .. } => Record::Turn,
                LinkUp::Call { .. } => Record::Call,
                LinkUp::Answer { .. } => Record::Answer,
                LinkUp::Activity { .. } => Record::Activity,
                LinkUp::Rejected { .. } => Record::Rejected,
                LinkUp::Exhausted { .. } => Record::Exhausted,
            },
            Request::Lost { .. } => Record::Lost,
            Request::Take { .. }
            | Request::TurnAway { .. }
            | Request::Lower { .. }
            | Request::Close { .. }
            | Request::Abort { .. }
            | Request::Closed => panic!("an up record"),
        }
    }
    fn observed(world: &World) -> Vec<Record> {
        world.root().iter().map(record).collect()
    }
    #[test]
    fn each_up_record_reaches_the_root_in_order() {
        let mut peer = reporting();
        for frame in fixtures::up_frames() {
            peer.play(frame);
        }
        let mut world = world();
        world.connect(peer);
        world.run();
        let records = observed(&world);
        assert_eq!(records, [Record::Hello, Record::Turn, Record::Activity, Record::Rejected, Record::Exhausted]);
    }
    #[test]
    fn an_undecodable_turn_body_is_refused_as_framing() {
        let mut peer = reporting();
        peer.play(fixtures::frame(0x0101, b"bad"));
        let mut world = world();
        let connection = world.connect(peer);
        world.run();
        assert_eq!(world.refusal(connection), Some((3, Box::from(b"".as_slice()))));
        let records = observed(&world);
        assert_eq!(records, [Record::Hello, Record::Lost]);
        let Request::Lost { channel } = world.root()[1] else {
            panic!("lost");
        };
        assert_eq!(channel, Token::new(1));
    }
}
