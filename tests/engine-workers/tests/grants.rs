mod grants {
    use jig_domain_host::{self as host, LinkDown};
    use jig_engine_workers_world::{World, fixtures};
    use jig_protocol_engine_workers::{Event, Request};
    use jig_protocol_worker_wire as wire;
    use skein_lib::{Reader, Token};

    fn world() -> World {
        World::new(&fixtures::limits(), &fixtures::workers(), 1)
    }
    fn assignment_grant(assignment: &wire::Assignment, index: u32) -> &wire::Grant {
        assignment.grants().get(index).expect("grant")
    }
    fn joined() -> (World, Token, Token) {
        let mut world = world();
        let connection = world.connect(fixtures::reporting_worker(fixtures::WORKER_A, fixtures::SECRET_A));
        world.run();
        let Request::Hello { channel, .. } = world.root()[0] else {
            panic!("hello first");
        };
        (world, connection, channel)
    }
    fn codec() -> wire::v3::Limits {
        wire::sizes::limits(&fixtures::SIZES).expect("small codec")
    }
    fn received_assignment(world: &World, connection: Token) -> wire::Assignment {
        let record =
            world.observed(connection).iter().find(|record| record.kind == 0x0180).expect("assignment crossed");
        wire::Assignment::decode(&codec(), &mut Reader::new(&record.body)).expect("assignment decodes")
    }
    fn send_grant(world: &mut World, channel: Token, table: host::GrantTable) {
        let mut values = fixtures::values();
        let index = usize::from(table != host::GrantTable::Accounts);
        world.down(Event::Down {
            channel,
            down: LinkDown::Grant { run: Token::new(7), attempt: Token::new(2), grant: fixtures::grant(table) },
            parts: Box::new([]),
            values: Box::new([values[index].take()]),
        });
        world.run();
    }
    fn received_grants(world: &World, connection: Token) -> Vec<wire::Granted> {
        world
            .observed(connection)
            .iter()
            .filter(|record| record.kind == 0x0187)
            .map(|record| wire::Granted::decode(&codec(), &mut Reader::new(&record.body)).expect("grant decodes"))
            .collect()
    }
    #[test]
    fn a_grant_of_each_table_reaches_the_worker_with_its_value() {
        let (mut world, connection, channel) = joined();
        send_grant(&mut world, channel, host::GrantTable::Accounts);
        send_grant(&mut world, channel, host::GrantTable::Application);
        let grants = received_grants(&world, connection);
        assert_eq!(grants.len(), 2);
        for (record, table, value, account_id) in [
            (&grants[0], wire::GrantTable::Accounts, fixtures::TOKEN_ONE, fixtures::ACCOUNT_ID),
            (&grants[1], wire::GrantTable::Application, fixtures::TOKEN_TWO, b"".as_slice()),
        ] {
            assert_eq!((record.task(), record.attempt()), (7, 2));
            let grant = record.grant();
            assert_eq!(grant.table(), &table);
            assert_eq!((grant.account(), grant.generation()), (1, 4));
            assert_eq!(grant.valid(), skein_lib::Duration::from_secs(30));
            assert_eq!(grant.value().expose(), value);
            assert_eq!(grant.account_id().expose(), account_id);
        }
    }
    #[test]
    fn an_assignment_reaches_the_worker_with_its_start_and_identity() {
        let (mut world, connection, channel) = joined();
        world.down(fixtures::assignment_down(channel));
        world.run();
        let assignment = received_assignment(&world, connection);
        assert_eq!((assignment.task(), assignment.attempt()), (7, 2));
        assert_eq!(assignment.identity(), &fixtures::IDENTITY);
        assert_eq!(assignment.charter(), b"charter");
        assert_eq!(assignment.transcript(), &[0, 0, 0, 2, 0, 127, 0, 0, 0, 3, 255, 1, 2]);
        assert_eq!(assignment.workspace().as_ref().expect("workspace").items(), b"items");
        assert_eq!(assignment.messages().len(), 1);
        assert_eq!(assignment.messages().get(0).expect("message").words(), b"message");
        assert_eq!(assignment.grants().len(), 2);
        assert_eq!(assignment_grant(&assignment, 0).value().expose(), fixtures::TOKEN_ONE);
        assert_eq!(assignment_grant(&assignment, 0).account_id().expose(), fixtures::ACCOUNT_ID);
        assert_eq!(assignment_grant(&assignment, 1).value().expose(), fixtures::TOKEN_TWO);
        assert!(assignment_grant(&assignment, 1).account_id().expose().is_empty());
    }
    #[test]
    fn an_assignment_drops_the_grant_the_table_no_longer_holds() {
        let (mut world, connection, channel) = joined();
        let mut values = fixtures::values();
        values[0] = None;
        world.down(Event::Down {
            channel,
            down: LinkDown::Assign { assignment: fixtures::assignment() },
            parts: fixtures::parts(),
            values,
        });
        world.run();
        let assignment = received_assignment(&world, connection);
        assert_eq!(assignment.grants().len(), 1);
        assert_eq!(assignment_grant(&assignment, 0).table(), &wire::GrantTable::Application);
        assert_eq!(assignment_grant(&assignment, 0).value().expose(), fixtures::TOKEN_TWO);
    }
    #[test]
    fn a_down_record_for_a_lost_channel_is_dropped() {
        let (mut world, connection, channel) = joined();
        world.down(Event::Gone { connection });
        send_grant(&mut world, channel, host::GrantTable::Accounts);
        assert!(received_grants(&world, connection).is_empty());
        assert_eq!(world.dropped_down(), 1);
    }
    #[test]
    fn a_down_record_before_the_hello_is_dropped() {
        let mut world = world();
        let connection = world.connect(fixtures::worker(fixtures::WORKER_A, fixtures::SECRET_A, fixtures::PAYLOAD));
        world.run();
        send_grant(&mut world, Token::new(1), host::GrantTable::Accounts);
        assert!(received_grants(&world, connection).is_empty());
        assert_eq!(world.dropped_down(), 1);
    }
    #[test]
    fn a_missing_part_is_dropped_and_counted() {
        let (mut world, connection, channel) = joined();
        world.down(Event::Down {
            channel,
            down: LinkDown::Assign { assignment: fixtures::assignment() },
            parts: Box::new([]),
            values: fixtures::values(),
        });
        world.run();
        assert!(world.observed(connection).iter().all(|record| record.kind != 0x0180));
        assert_eq!(world.dropped_down(), 1);
    }
    #[test]
    fn a_grant_the_table_no_longer_holds_sends_no_record() {
        let (mut world, connection, channel) = joined();
        world.down(Event::Down {
            channel,
            down: LinkDown::Grant {
                run: Token::new(7),
                attempt: Token::new(2),
                grant: fixtures::grant(host::GrantTable::Accounts),
            },
            parts: Box::new([]),
            values: Box::new([None]),
        });
        world.run();
        assert!(received_grants(&world, connection).is_empty());
    }
}
