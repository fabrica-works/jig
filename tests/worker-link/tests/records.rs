use jig_domain_host as host;
use jig_protocol_worker_link::{Event, Request};
use jig_protocol_worker_wire as wire;
use jig_worker_link_world::{World, fixtures};
use skein_lib::{Duration, Reader, Token};
fn world() -> World {
    let mut world = World::new(&fixtures::limits(), 1);
    world.engine(fixtures::engine_accepting());
    world.run();
    world.down(Event::Hello { hello: fixtures::hello() });
    world
}
fn assignment(world: &World) -> &host::Assignment {
    world
        .root()
        .iter()
        .find_map(|request| match request {
            Request::Down { down: host::LinkDown::Assign { assignment }, .. } => Some(assignment),
            Request::Down { .. }
            | Request::Ready
            | Request::Lost
            | Request::Value { .. }
            | Request::Dial
            | Request::Lower { .. }
            | Request::Close { .. }
            | Request::Abort { .. }
            | Request::Fact { .. }
            | Request::Resend { .. }
            | Request::RelayCancelled { .. } => None,
        })
        .expect("assignment")
}
fn down(world: &World) -> Vec<&host::LinkDown> {
    world
        .root()
        .iter()
        .filter_map(|request| match request {
            Request::Down { down, .. } => Some(down),
            Request::Ready
            | Request::Lost
            | Request::Value { .. }
            | Request::Dial
            | Request::Lower { .. }
            | Request::Close { .. }
            | Request::Abort { .. }
            | Request::Fact { .. }
            | Request::Resend { .. }
            | Request::RelayCancelled { .. } => None,
        })
        .collect()
}
fn value(request: &Request, table: host::GrantTable, expires: skein_lib::Time) {
    let Request::Value { table: owner, account, generation, expires: until, value, account_id } = request else {
        panic!("value: {request:?}")
    };
    assert_eq!((*owner, *account, *generation, *until), (table, 1, 3, expires));
    assert_eq!(value.expose(), fixtures::TOKEN_ONE);
    assert_eq!(account_id.expose(), fixtures::ACCOUNT_ONE);
}
#[test]
fn each_up_record_reaches_the_engine_in_order() {
    let mut world = world();
    for (up, parts) in fixtures::up_records() {
        world.down(Event::Up { up, parts });
    }
    let channels = world.records();
    let records = &channels[0];
    let limits = fixtures::codec();
    assert_eq!(
        records.iter().map(|record| record.kind).collect::<Vec<_>>(),
        [0x0100, 0x0101, 0x0102, 0x0103, 0x0104, 0x0105, 0x0106]
    );
    check_up_records(records, &limits);
}
#[test]
fn each_down_record_reaches_the_root_in_order() {
    let mut world = world();
    for frame in fixtures::down_frames() {
        world.play(frame);
    }
    let records = down(&world);
    check_down_records(&world, &records);
    world.down(Event::Up { up: fixtures::call(b"call"), parts: Box::new([]) });
    world.play(fixtures::relayed(b"call", false));
    world.play(fixtures::turn_busy());
    let records = down(&world);
    assert_eq!(records.len(), 7);
    assert_eq!(
        *records[6],
        host::LinkDown::Relayed {
            run: Token::new(7),
            attempt: Token::new(2),
            call: b"call".as_slice().into(),
            answer: host::Relayed::Answer { error: false, text: b"answer".as_slice().into() },
        }
    );
    world.advance(Duration::from_millis(200));
    assert!(world.root().iter().any(|request| matches!(request, Request::Resend { run, attempt, turn: 1 }
        if *run == Token::new(7) && *attempt == Token::new(2))));
}
#[test]
fn grant_values_reach_the_layer_before_the_assignment() {
    let mut world = world();
    world.advance(Duration::from_secs(1));
    world.play(fixtures::assignment_frame(fixtures::window()));
    assert_eq!(world.order(), ["value", "value", "down"]);
    let expires = world.now().saturating_add(Duration::from_secs(60));
    value(&world.values()[0], host::GrantTable::Accounts, expires);
    value(&world.values()[1], host::GrantTable::Application, expires);
    assert_eq!(assignment(&world).assignment.grants.len(), 2);
}
#[test]
fn an_undecodable_assignment_is_refused_as_framing() {
    let mut world = world();
    world.play(fixtures::frame(0x0180, b"broken"));
    assert_eq!(world.refusals(), [(3, Box::new([]) as Box<[u8]>)]);
    assert!(down(&world).is_empty());
    assert!(world.values().is_empty());
}
#[test]
fn a_transcript_piece_that_does_not_split_is_refused_as_invalid() {
    for transcript in [b"\0".as_slice(), b"\0\0\0\x05short".get(..8).expect("past end"), b"\0\0\0\x01a\0"] {
        let mut world = world();
        world.play(fixtures::assignment_frame(transcript.into()));
        assert_eq!(assignment(&world).turns.as_ref(), [Box::new([]) as Box<[u8]>]);
        assert!(world.refusals().is_empty(), "the root judges smith's transcript");
        world.down(Event::Up {
            up: host::LinkUp::Answer {
                run: Token::new(7),
                attempt: Token::new(2),
                answer: host::Answer {
                    read: None,
                    refused: Box::new([]),
                    turns: 0,
                    spent: 0,
                    ending: host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Transcript)),
                },
            },
            parts: Box::new([]),
        });
        let records = world.records();
        let answer = wire::Answer::decode(&fixtures::codec(), &mut Reader::new(&records[0][1].body)).expect("refusal");
        let wire::Ending::Refused(refusal) = answer.ending() else { panic!("refused") };
        let wire::Refusal::Invalid(invalid) = refusal.value() else { panic!("invalid") };
        assert_eq!(invalid.value(), &wire::Invalid::Transcript);
    }
}

fn check_up_records(records: &[skein_fake_channel::Observed], limits: &wire::v1::Limits) {
    let turn = wire::Turn::decode(limits, &mut Reader::new(&records[1].body)).expect("turn");
    assert_eq!(
        (turn.task(), turn.attempt(), turn.number(), turn.spent(), *turn.read(), turn.opens_window(), turn.body()),
        (7, 2, 1, 3, Some(5), true, b"turn".as_slice())
    );
    assert_eq!(turn.refused().get(0).expect("refusal").reason(), &wire::MessageRefusal::Full);
    let call = wire::Call::decode(limits, &mut Reader::new(&records[2].body)).expect("call");
    assert_eq!(
        (call.task(), call.attempt(), call.name(), call.tool(), call.writes(), call.input(), call.deadline()),
        (7, 2, b"call".as_slice(), b"tool".as_slice(), true, b"{}".as_slice(), Duration::from_secs(60))
    );
    let answer = wire::Answer::decode(limits, &mut Reader::new(&records[3].body)).expect("answer");
    assert_eq!(
        (answer.task(), answer.attempt(), answer.turns(), answer.spent(), *answer.read()),
        (7, 2, 1, 3, Some(5))
    );
    let wire::Ending::Ended(ended) = answer.ending() else { panic!("ended") };
    assert_eq!(
        (ended.outcome(), ended.work().left().as_deref(), ended.work().saved().as_deref()),
        (b"result".as_slice(), Some(b"left".as_slice()), Some(b"saved".as_slice()))
    );
    let fact = wire::Fact::decode(limits, &mut Reader::new(&records[4].body)).expect("fact");
    let wire::Activity::Long(long) = fact.activity() else { panic!("long") };
    assert_eq!(
        (fact.task(), fact.attempt(), long.operation(), long.span()),
        (7, 2, &wire::Operation::Command, Duration::from_secs(2))
    );
    let rejected = wire::Rejected::decode(limits, &mut Reader::new(&records[5].body)).expect("rejected");
    assert_eq!((rejected.task(), rejected.attempt(), rejected.account(), rejected.generation()), (7, 2, 1, 3));
    let exhausted = wire::Exhausted::decode(limits, &mut Reader::new(&records[6].body)).expect("exhausted");
    assert_eq!(
        (exhausted.task(), exhausted.attempt(), exhausted.account(), exhausted.retry_after()),
        (7, 2, 1, Duration::from_secs(60))
    );
}

#[test]
fn a_named_part_missing_from_the_root_is_dropped_and_counted() {
    let mut world = world();
    world.down(Event::Up { up: fixtures::answer(), parts: Box::new([]) });
    assert_eq!(world.dropped_records(), 1);
    assert_eq!(world.records()[0].iter().map(|record| record.kind).collect::<Vec<_>>(), [0x0100]);
}

fn check_down_records(world: &World, records: &[&host::LinkDown]) {
    assert_eq!(records.len(), 6);
    assert_eq!(assignment(world).turns.as_ref(), [Box::from(b"one!".as_slice()), Box::from(b"two!".as_slice())]);
    assert_eq!(
        (assignment(world).assignment.run, assignment(world).assignment.attempt),
        (Token::new(7), Token::new(2))
    );
    assert_eq!(
        *records[1],
        host::LinkDown::Message {
            run: Token::new(7),
            attempt: Token::new(2),
            name: Token::new(6),
            sender: b"sender".as_slice().into(),
            words: b"later".as_slice().into()
        }
    );
    assert_eq!(*records[2], host::LinkDown::TurnKept { run: Token::new(7), attempt: Token::new(2), turn: 1 });
    assert_eq!(*records[3], host::LinkDown::Acknowledge { run: Token::new(7), attempt: Token::new(2) });
    assert_eq!(*records[4], host::LinkDown::Cancel { run: Token::new(7), attempt: Token::new(2) });
    assert_eq!(
        *records[5],
        host::LinkDown::Grant {
            run: Token::new(7),
            attempt: Token::new(2),
            grant: host::Grant {
                table: host::GrantTable::Accounts,
                account: 1,
                generation: 3,
                valid: Duration::from_secs(60)
            }
        }
    );
}
