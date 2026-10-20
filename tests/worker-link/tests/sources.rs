use jig_domain_host as host;
use jig_protocol_worker_link::{Event, Request};
use jig_protocol_worker_wire as wire;
use jig_worker_link_world::{World, fixtures};
use skein_lib::{List, Reader, Writer};
fn world() -> World {
    let mut world = World::new(&fixtures::limits(), 19);
    world.engine(fixtures::engine_accepting());
    world.run();
    world.down(Event::Hello { hello: fixtures::hello() });
    world
}
fn sources(names: &[&[u8]]) -> skein_channel::Frame {
    let mut rows = List::with_capacity(u32::try_from(names.len()).expect("small list"));
    for name in names {
        rows.push(
            wire::ProgramSource::new(&wire::CEILINGS, wire::ProgramSourceParts { name: (*name).into() })
                .expect("wire name"),
        )
        .expect("row");
    }
    let record = wire::Sources::new(&wire::CEILINGS, wire::SourcesParts { sources: rows }).expect("wire list");
    let mut body = Writer::new(usize::try_from(record.measure()).expect("body"));
    record.encode(&mut body).expect("encode");
    fixtures::frame(0x0188, &body.finish())
}
#[test]
fn host_wide_source_records_cross_bytes_without_an_assignment() {
    let mut world = world();
    world.play(sources(&[b"alice"]));
    for kind in [0x0189, 0x018a] {
        let mut body = Writer::new(9);
        match kind {
            0x0189 => wire::Check::new(&wire::CEILINGS, wire::CheckParts { name: b"alice".as_slice().into() })
                .expect("check")
                .encode(&mut body)
                .expect("encode"),
            _ => wire::SignOut::new(&wire::CEILINGS, wire::SignOutParts { name: b"alice".as_slice().into() })
                .expect("sign-out")
                .encode(&mut body)
                .expect("encode"),
        }
        world.play(fixtures::frame(kind, &body.finish()));
    }
    let names = world
        .root()
        .iter()
        .filter_map(|request| if let Request::Down { down, .. } = request { Some(down) } else { None })
        .collect::<Vec<_>>();
    let alice = host::SourceName::new(b"alice").expect("name");
    assert_eq!(
        names,
        [
            &host::LinkDown::Sources { sources: Box::new([alice]) },
            &host::LinkDown::Check { name: alice },
            &host::LinkDown::SignOut { name: alice }
        ]
    );
    world.down(Event::Up {
        up: host::LinkUp::Logins { logins: Box::new([host::Login { name: alice, state: host::LoginState::Usable }]) },
        parts: Box::new([]),
    });
    world.down(Event::Up {
        up: host::LinkUp::SignedOut { name: alice, result: host::SignOutResult::SignedOut },
        parts: Box::new([]),
    });
    let records = world.records();
    let records = &records[0];
    let logins = records.iter().find(|record| record.kind == 0x0108).expect("login notice");
    let logins = wire::Logins::decode(&fixtures::codec(), &mut Reader::new(&logins.body)).expect("encoded logins");
    assert_eq!(logins.logins().len(), 1);
    assert_eq!(logins.logins().get(0).expect("row").name(), b"alice");
    assert!(records.iter().any(|record| record.kind == 0x0109));
}
#[test]
fn malformed_or_oversized_source_lists_are_link_faults() {
    for names in [&[b"Alice".as_slice()][..], &[b"a".as_slice(), b"b", b"c"]] {
        let mut world = world();
        world.play(sources(names));
        assert!(world.root().iter().any(|request| matches!(request, Request::Lost)));
    }
}

#[test]
fn backpressure_keeps_only_the_latest_inventory_until_the_queue_is_empty() {
    let mut world = world();
    world.withhold_write_room();
    let alice = host::SourceName::new(b"alice").expect("name");
    let latest = host::SourceName::new(b"latest").expect("name");
    for name in [alice, alice, host::SourceName::new(b"middle").expect("name"), latest] {
        world.down(Event::Up {
            up: host::LinkUp::Logins { logins: Box::new([host::Login { name, state: host::LoginState::Usable }]) },
            parts: Box::new([]),
        });
    }
    assert!(!world.root().iter().any(|request| matches!(request, Request::Lost)));
    assert!(!world.records()[0].iter().any(|record| record.kind == wire::Kind::Logins.number()));
    world.grant_write_room();
    let records = world.records();
    let inventories = records[0]
        .iter()
        .filter(|record| record.kind == wire::Kind::Logins.number())
        .map(|record| wire::Logins::decode(&fixtures::codec(), &mut Reader::new(&record.body)).expect("inventory"))
        .collect::<Vec<_>>();
    assert_eq!(inventories.len(), 2, "one queued frame and one latest pending snapshot");
    assert_eq!(inventories[0].logins().get(0).expect("first row").name(), alice.as_bytes());
    assert_eq!(inventories[1].logins().get(0).expect("latest row").name(), latest.as_bytes());
}

#[test]
fn a_lost_channel_drops_held_inventories_before_redial() {
    let mut world = world();
    world.withhold_write_room();
    let alice = host::SourceName::new(b"alice").expect("name");
    for _ in 0..3 {
        world.down(Event::Up {
            up: host::LinkUp::Logins {
                logins: Box::new([host::Login { name: alice, state: host::LoginState::Usable }]),
            },
            parts: Box::new([]),
        });
    }
    world.lose_channel();
    world.engine(fixtures::engine_accepting());
    world.advance(skein_lib::Duration::from_millis(100));
    world.down(Event::Hello { hello: fixtures::hello() });
    world.play(sources(&[]));
    world.down(Event::Up { up: host::LinkUp::Logins { logins: Box::new([]) }, parts: Box::new([]) });
    let records = world.records();
    assert_eq!(records[1].iter().map(|record| record.kind).collect::<Vec<_>>(), [0x0100, 0x0108]);
    assert!(
        wire::Logins::decode(&fixtures::codec(), &mut Reader::new(&records[1][1].body))
            .expect("fresh inventory")
            .logins()
            .is_empty()
    );
}
