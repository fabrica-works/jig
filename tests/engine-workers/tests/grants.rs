mod grants {
    use jig_domain_host::{self as host, LinkDown};
    use jig_engine_workers_world::{World, fixtures};
    use jig_protocol_engine_workers::{Event, GrantValue, Request, Scheme};
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
    fn send_grant(world: &mut World, channel: Token, form: host::Form, value: GrantValue) {
        world.down(Event::Down {
            channel,
            down: LinkDown::Grant { run: Token::new(7), attempt: Token::new(2), grant: fixtures::grant(form) },
            parts: Box::new([]),
            values: Box::new([Some(value)]),
        });
        world.run();
    }
    fn withdrawn() -> GrantValue {
        fixtures::values()[0].take().expect("withdrawn value")
    }
    fn application() -> GrantValue {
        fixtures::values()[1].take().expect("application value")
    }
    fn credential(grant: &wire::Grant) -> (&wire::Scheme, &[u8]) {
        let wire::GrantValue::Credential(value) = grant.value() else { panic!("credential value") };
        (value.scheme(), value.bytes().expose())
    }
    fn withdrawn_parts(grant: &wire::Grant) -> (&[u8], &[u8]) {
        let wire::GrantValue::Withdrawn(value) = grant.value() else { panic!("withdrawn value") };
        (value.id().expose(), value.bearer().expose())
    }
    fn application_secret(grant: &wire::Grant) -> &[u8] {
        let wire::GrantValue::Application(value) = grant.value() else { panic!("application value") };
        value.secret().expose()
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
    fn a_grant_of_each_arm_reaches_the_worker_with_its_value() {
        let (mut world, connection, channel) = joined();
        send_grant(
            &mut world,
            channel,
            host::Form::Credential(host::Scheme::Bearer),
            fixtures::credential_value(Scheme::Bearer),
        );
        send_grant(
            &mut world,
            channel,
            host::Form::Credential(host::Scheme::Key),
            fixtures::credential_value(Scheme::Key),
        );
        send_grant(&mut world, channel, host::Form::Withdrawn, withdrawn());
        send_grant(&mut world, channel, host::Form::Application, application());
        let grants = received_grants(&world, connection);
        assert_eq!(grants.len(), 4);
        for record in &grants {
            assert_eq!((record.task(), record.attempt()), (7, 2));
            let grant = record.grant();
            assert_eq!((grant.number(), grant.generation()), (1, 4));
            assert_eq!(grant.valid(), skein_lib::Duration::from_secs(30));
        }
        assert_eq!(credential(grants[0].grant()), (&wire::Scheme::Bearer, fixtures::TOKEN_ONE));
        assert_eq!(credential(grants[1].grant()), (&wire::Scheme::Key, fixtures::TOKEN_ONE));
        assert_eq!(withdrawn_parts(grants[2].grant()), (fixtures::ACCOUNT_ID, fixtures::TOKEN_ONE));
        assert_eq!(application_secret(grants[3].grant()), fixtures::TOKEN_TWO);
    }
    #[test]
    fn a_grant_whose_value_is_unlike_its_form_is_not_sent() {
        let (mut world, connection, channel) = joined();
        send_grant(
            &mut world,
            channel,
            host::Form::Credential(host::Scheme::Bearer),
            fixtures::credential_value(Scheme::Key),
        );
        assert!(received_grants(&world, connection).is_empty());
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
        assert_eq!(withdrawn_parts(assignment_grant(&assignment, 0)), (fixtures::ACCOUNT_ID, fixtures::TOKEN_ONE));
        assert_eq!(application_secret(assignment_grant(&assignment, 1)), fixtures::TOKEN_TWO);
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
        assert_eq!(application_secret(assignment_grant(&assignment, 0)), fixtures::TOKEN_TWO);
    }
    #[test]
    fn a_down_record_for_a_lost_channel_is_dropped() {
        let (mut world, connection, channel) = joined();
        world.down(Event::Gone { connection });
        send_grant(&mut world, channel, host::Form::Withdrawn, withdrawn());
        assert!(received_grants(&world, connection).is_empty());
        assert_eq!(world.dropped_down(), 1);
    }
    #[test]
    fn a_down_record_before_the_hello_is_dropped() {
        let mut world = world();
        let connection = world.connect(fixtures::worker(fixtures::WORKER_A, fixtures::SECRET_A, fixtures::PAYLOAD));
        world.run();
        send_grant(&mut world, Token::new(1), host::Form::Withdrawn, withdrawn());
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
                grant: fixtures::grant(host::Form::Withdrawn),
            },
            parts: Box::new([]),
            values: Box::new([None]),
        });
        world.run();
        assert!(received_grants(&world, connection).is_empty());
    }
}
