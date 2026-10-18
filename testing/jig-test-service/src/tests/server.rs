use super::fixtures::server::{LISTENER, OWNER, REQUEST, SOCKET};
use super::fixtures::{self as f};
use crate::server::Routed;
use skein_lib::{List, Queue, Time, Token, Wall};

#[test]
fn an_accepted_connection_s_call_goes_to_the_mount() {
    let mut server = f::server();
    let mut out = Queue::with_capacity(32);
    server.up(f::listening(), &mut out);
    server.up(f::accepted(SOCKET), &mut out);
    f::server_drain(&mut server, &mut out);
    f::server_bytes(&mut server, &mut out, REQUEST);
    assert_eq!(*f::calls(&mut out), [(Box::from(b"/jig/v1/read".as_slice()), skein_http::Method::Post)]);
}
#[test]
fn a_connection_past_the_limit_is_rejected() {
    let mut server = f::server();
    let mut out = Queue::with_capacity(32);
    for socket in 100..104 {
        server.up(f::accepted(Token::new(socket)), &mut out);
    }
    server.up(f::accepted(Token::new(104)), &mut out);
    let mut rejected = List::with_capacity(1);
    for _ in 0..out.len() {
        if let Some(Routed::Io(skein_io::Request::Reject { socket })) = out.pop() {
            rejected.push(socket).expect("one refusal");
        }
    }
    assert_eq!(rejected.as_slice(), [Token::new(104)]);
}
#[test]
fn a_kept_connection_carries_a_second_call_after_done() {
    let mut server = f::server();
    let mut out = Queue::with_capacity(32);
    let call = f::server_call(&mut server, &mut out);
    f::server_response(&mut server, &mut out, call);
    server.done(call, &mut out);
    f::server_drain(&mut server, &mut out);
    f::server_bytes(&mut server, &mut out, REQUEST);
    assert_eq!(*f::calls(&mut out), [(Box::from(b"/jig/v1/read".as_slice()), skein_http::Method::Post)]);
}
#[test]
fn a_stalled_connection_is_closed_at_its_deadline() {
    const DEADLINE: Time = Time::from_nanos(30_000_000_000);
    let mut server = f::server();
    let mut out = Queue::with_capacity(32);
    server.up(f::accepted(SOCKET), &mut out);
    assert_eq!(server.next_deadline(), Some(DEADLINE));
    let deadline = DEADLINE;
    server.clock(Time::from_nanos(deadline.as_nanos() - 1), Wall::EPOCH);
    server.fire(&mut out);
    for _ in 0..out.len() {
        if let Some(Routed::Io(skein_io::Request::Abort { .. })) = out.pop() {
            panic!("before the stall deadline");
        }
    }
    server.clock(deadline, Wall::EPOCH);
    server.fire(&mut out);
    let Some(Routed::Io(skein_io::Request::Abort { entity })) = out.pop() else { panic!("stall abort") };
    assert_eq!(entity, SOCKET);
}
#[test]
fn closing_the_server_closes_its_listener_then_its_connections() {
    let mut server = f::server();
    let mut out = Queue::with_capacity(32);
    server.up(f::listening(), &mut out);
    server.up(f::accepted(SOCKET), &mut out);
    f::server_drain(&mut server, &mut out);
    while out.pop().is_some() {}
    server.close(&mut out);
    f::server_drain(&mut server, &mut out);
    let mut closed = List::with_capacity(2);
    for _ in 0..out.len() {
        if let Some(Routed::Io(skein_io::Request::Close { entity })) = out.pop() {
            closed.push(entity).expect("listener and socket");
        }
    }
    assert_eq!(closed.as_slice(), [LISTENER, SOCKET]);
}
#[test]
fn a_close_waits_for_its_listener_to_open() {
    let mut server = f::server();
    let mut out = Queue::with_capacity(8);
    assert!(server.advance(&mut out));
    let Some(Routed::Io(skein_io::Request::Listen { .. })) = out.pop() else { panic!("listen") };
    server.close(&mut out);
    assert!(!server.advance(&mut out), "the pending listener has not closed");
    assert!(!server.closed());
    assert!(out.is_empty());
    server.up(f::listening(), &mut out);
    let Some(Routed::Io(skein_io::Request::Close { entity })) = out.pop() else { panic!("close listener") };
    assert_eq!(entity, LISTENER);
    assert!(!server.advance(&mut out), "waiting for io's close terminal");
    server.up(skein_io::Event::Closed { owner: Token::new(0) }, &mut out);
    assert!(server.closed());
    assert!(!server.advance(&mut out));
    assert!(out.is_empty());
}
#[test]
fn abort_upgrades_sockets_waiting_for_their_graceful_close() {
    let mut server = f::server();
    let mut out = Queue::with_capacity(32);
    let call = f::server_call(&mut server, &mut out);
    server.close(&mut out);
    server.done(call, &mut out);
    assert!(!server.closed(), "io has not settled its closes");
    let mut closes = 0_u32;
    for _ in 0..out.len() {
        if let Some(Routed::Io(skein_io::Request::Close { entity })) = out.pop()
            && entity == SOCKET
        {
            closes += 1;
        }
    }
    assert_eq!(closes, 1);
    server.abort(&mut out);
    f::server_drain(&mut server, &mut out);
    let mut aborts = 0_u32;
    for _ in 0..out.len() {
        if let Some(Routed::Io(skein_io::Request::Abort { entity })) = out.pop()
            && entity == SOCKET
        {
            aborts += 1;
        }
    }
    assert_eq!(closes + aborts, 2, "the drain is upgraded once to abort");
    server.up(skein_io::Event::Closed { owner: Token::new(0) }, &mut out);
    server.up(skein_io::Event::Closed { owner: OWNER }, &mut out);
    assert!(server.closed());
}

#[test]
fn a_heartbeat_waiting_for_room_keeps_its_stall_deadline() {
    const WRITING: Time = Time::from_nanos(5_000_000_000);
    const DEADLINE: Time = Time::from_nanos(35_000_000_000);
    let mut server = f::server();
    let mut out = Queue::with_capacity(32);
    let call = f::server_call(&mut server, &mut out);
    server.clock(WRITING, Wall::EPOCH);
    server.answer(
        call,
        skein_http::server::Request::Respond(skein_http::server::Response {
            status: 200,
            headers: Box::new([]),
            body: skein_http::server::Body::Chunked,
            close: false,
        }),
        &mut out,
    );
    server.answer(
        call,
        skein_http::server::Request::Reply(skein_lib::stream::Down::Demand {
            read: skein_lib::stream::Read::Nothing,
            room: 1,
        }),
        &mut out,
    );
    f::server_drain(&mut server, &mut out);
    assert_eq!(server.next_deadline(), Some(DEADLINE));
    while out.pop().is_some() {}
    server.clock(DEADLINE, Wall::EPOCH);
    server.fire(&mut out);
    let Some(Routed::Io(skein_io::Request::Abort { entity })) = out.pop() else { panic!("stalled heartbeat") };
    assert_eq!(entity, SOCKET);
}
