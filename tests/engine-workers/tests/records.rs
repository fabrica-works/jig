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
        assert_eq!(
            records,
            [
                Record::Hello,
                Record::Turn,
                Record::Call,
                Record::Answer,
                Record::Activity,
                Record::Rejected,
                Record::Exhausted
            ]
        );
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
    fn answering() -> skein_fake_channel::ScriptedPeer {
        let mut peer = reporting();
        peer.play(fixtures::answer_frame());
        peer
    }
    fn application_parts(world: &World) -> Vec<(Token, Token, Vec<u8>)> {
        let mut collected = Vec::new();
        for request in world.root() {
            if let Request::Up { channel, up: jig_domain_host::LinkUp::Answer { answer, .. }, parts } = request {
                let jig_domain_host::Ending::Ended { work, .. } = &answer.ending else {
                    panic!("the fixture ended");
                };
                assert_eq!(parts.len(), 2);
                assert_eq!(work.left, Some(parts[0].token));
                assert_eq!(work.saved, Some(parts[1].token));
                for part in parts {
                    collected.push((*channel, part.token, part.bytes.to_vec()));
                }
            }
        }
        collected
    }
    #[test]
    fn application_part_tokens_never_repeat_when_a_worker_is_replaced() {
        let mut world = world();
        for _ in 0..3 {
            world.connect(answering());
            world.run();
        }
        assert_eq!(
            application_parts(&world),
            vec![
                (Token::new(1), Token::new(1), b"left".to_vec()),
                (Token::new(1), Token::new(2), b"saved".to_vec()),
                (Token::new(2), Token::new(3), b"left".to_vec()),
                (Token::new(2), Token::new(4), b"saved".to_vec()),
                (Token::new(3), Token::new(5), b"left".to_vec()),
                (Token::new(3), Token::new(6), b"saved".to_vec()),
            ]
        );
    }
}
