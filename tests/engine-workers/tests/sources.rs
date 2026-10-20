use jig_domain_host as host;
use jig_engine_workers_world::{World, fixtures};
use jig_protocol_engine_workers::{Event, Request};
use jig_protocol_worker_wire as wire;
use skein_lib::{List, Writer};
fn frame(names: &[&[u8]]) -> skein_channel::Frame {
    let mut rows = List::with_capacity(u32::try_from(names.len()).expect("rows"));
    for name in names {
        rows.push(
            wire::Login::new(
                &wire::CEILINGS,
                wire::LoginParts { name: (*name).into(), state: wire::LoginState::Usable },
            )
            .expect("wire row"),
        )
        .expect("list");
    }
    let record = wire::Logins::new(&wire::CEILINGS, wire::LoginsParts { logins: rows }).expect("wire rows");
    let mut body = Writer::new(usize::try_from(record.measure()).expect("body"));
    record.encode(&mut body).expect("encode");
    fixtures::frame(0x0108, &body.finish())
}
#[test]
fn whole_logins_and_signed_out_are_host_wide_without_attempt_fences() {
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let (connection, channel) = world.reporting(fixtures::WORKER_A, fixtures::SECRET_A);
    world.play(connection, frame(&[b"alice"]));
    world.run();
    assert!(world.root().iter().any(|request| matches!(request, Request::Up { channel: at, up: host::LinkUp::Logins { logins }, parts } if *at == channel && logins.len() == 1 && parts.is_empty())));
    let alice = host::SourceName::new(b"alice").expect("name");
    for down in [
        host::LinkDown::Sources { sources: Box::new([alice]) },
        host::LinkDown::Check { name: alice },
        host::LinkDown::SignOut { name: alice },
    ] {
        world.down(Event::Down { channel, down, parts: Box::new([]), values: Box::new([]) });
    }
    world.run();
    let kinds = world.observed(connection).iter().map(|record| record.kind).collect::<Vec<_>>();
    for kind in [0x0188, 0x0189, 0x018a] {
        assert!(kinds.contains(&kind));
    }
    let record = wire::SignedOut::new(
        &wire::CEILINGS,
        wire::SignedOutParts { name: b"alice".as_slice().into(), result: wire::SignOutResult::SignedOut },
    )
    .expect("answer");
    let mut body = Writer::new(usize::try_from(record.measure()).expect("body"));
    record.encode(&mut body).expect("encode");
    world.play(connection, fixtures::frame(0x0109, &body.finish()));
    world.run();
    assert!(world.root().iter().any(|request| matches!(request, Request::Up { up: host::LinkUp::SignedOut { name, result: host::SignOutResult::SignedOut }, .. } if *name == alice)));
}
#[test]
fn malformed_or_oversized_login_rows_are_refused_and_empty_inventory_is_valid() {
    for names in [&[b"Alice".as_slice()][..], &[b"a".as_slice(), b"b", b"c"]] {
        let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
        let (connection, _) = world.reporting(fixtures::WORKER_A, fixtures::SECRET_A);
        world.play(connection, frame(names));
        world.run();
        assert_eq!(world.refusal(connection), Some((3, Box::new([]) as Box<[u8]>)));
    }
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let (connection, _) = world.reporting(fixtures::WORKER_A, fixtures::SECRET_A);
    world.play(connection, frame(&[]));
    world.run();
    assert!(world.root().iter().any(
        |request| matches!(request, Request::Up { up: host::LinkUp::Logins { logins }, .. } if logins.is_empty())
    ));
}
