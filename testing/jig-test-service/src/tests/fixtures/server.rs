use crate::server::{Routed, Server};
use skein_io as io;
use skein_lib::{
    List, Queue, Token,
    stream::{Down, Up},
};
pub const SETTLE_STEPS: u32 = 128;
pub const SOCKET: Token = Token::new(100);
pub const OWNER: Token = Token::new(1);
pub const LISTENER: Token = Token::new(99);
pub const REQUEST: &[u8] = b"POST /jig/v1/read HTTP/1.1\r\nHost: 127.0.0.1:7100\r\nContent-Length: 0\r\n\r\n";
pub fn server() -> Server {
    Server::new(
        skein_io::Addr::Ip(core::net::SocketAddr::new(core::net::IpAddr::from([127, 0, 0, 1]), 7100)),
        &crate::limits::profile_limits(jig_protocol_config::Profile::Small).server,
    )
}
pub fn listening() -> skein_io::Event {
    skein_io::Event::Listening {
        owner: Token::new(0),
        listener: LISTENER,
        addr: skein_io::Addr::Ip(core::net::SocketAddr::new(core::net::IpAddr::from([127, 0, 0, 1]), 7100)),
    }
}
pub fn accepted(socket: Token) -> skein_io::Event {
    skein_io::Event::Accepted {
        owner: Token::new(0),
        socket,
        peer: skein_io::Addr::Ip(core::net::SocketAddr::new(core::net::IpAddr::from([127, 0, 0, 1]), 7100)),
    }
}
pub fn bytes(owner: Token, bytes: &[u8]) -> skein_io::Event {
    skein_io::Event::Stream { owner, up: Up::Bytes(bytes.into()) }
}
pub fn calls(out: &mut Queue<Routed>) -> Box<[(Box<[u8]>, skein_http::Method)]> {
    let mut calls = List::with_capacity(out.len());
    while let Some(routed) = out.pop() {
        if let Routed::Call { head, .. } = routed {
            calls.push((head.target, head.method)).expect("queued calls");
        }
    }
    calls.into_boxed()
}
pub fn server_drain(server: &mut Server, out: &mut Queue<Routed>) {
    for _ in 0_u32..SETTLE_STEPS {
        if !server.advance(out) {
            return;
        }
    }
    panic!("server did not settle");
}
pub fn server_call(server: &mut Server, out: &mut Queue<Routed>) -> Token {
    server.up(listening(), out);
    server.up(accepted(SOCKET), out);
    server_drain(server, out);
    server_bytes(server, out, REQUEST);
    while let Some(routed) = out.pop() {
        if let Routed::Call { call, .. } = routed {
            return call;
        }
    }
    panic!("accepted call");
}
pub fn server_response(server: &mut Server, out: &mut Queue<Routed>, call: Token) {
    use skein_http::server::{Body, Request, Response};
    server.answer(call, Request::Discard, out);
    server.answer(
        call,
        Request::Respond(Response { status: 200, headers: Box::new([]), body: Body::None, close: false }),
        out,
    );
    for _ in 0_u32..SETTLE_STEPS {
        server.advance(out);
        if let Some(routed) = out.pop() {
            match routed {
                Routed::Io(skein_io::Request::Stream { down: Down::Demand { room, .. }, .. }) if room > 0 => {
                    server.up(skein_io::Event::Stream { owner: OWNER, up: Up::Room }, out);
                }
                Routed::Http { event: skein_http::server::Event::Done(_), .. } => return,
                Routed::Io(_) | Routed::Call { .. } | Routed::Http { .. } => {}
            }
        }
    }
    panic!("response did not finish");
}

pub fn server_bytes(server: &mut Server, out: &mut Queue<Routed>, request: &[u8]) {
    server.up(skein_io::Event::Stream { owner: OWNER, up: Up::Room }, out);
    server_drain(server, out);
    for line in request.split_inclusive(|byte| *byte == b'\n') {
        server.up(bytes(OWNER, line), out);
        server_drain(server, out);
    }
}

pub const MOUNT_REQUEST: &[u8] = b"POST /jig/v1/read HTTP/1.1\r\nHost: 127.0.0.1:7100\r\nContent-Type: application/octet-stream\r\nContent-Length: 0\r\n\r\n";
/// A real accepted connection and parsed head, leaving the routed call for its application.
pub fn retained_call(server: &mut Server, out: &mut Queue<Routed>, socket: Token) -> (Token, Token) {
    server.up(accepted(socket), out);
    server_drain(server, out);
    let mut owner = None;
    for routed in &*out {
        match routed {
            Routed::Io(skein_io::Request::Bind { socket: found, owner: bound }) if *found == socket => {
                owner = Some(*bound);
                break;
            }
            Routed::Io(_) | Routed::Call { .. } | Routed::Http { .. } => {}
        }
    }
    let owner = owner.expect("accepted socket binding");
    let (read, room) = retained_demand(out, socket);
    assert_eq!(read, skein_lib::stream::Read::Nothing);
    assert_eq!(room, crate::limits::profile_limits(jig_protocol_config::Profile::Small).server.http.response);
    server.up(skein_io::Event::Stream { owner, up: Up::Room }, out);
    server_drain(server, out);
    for line in MOUNT_REQUEST.split_inclusive(|byte| *byte == b'\n') {
        let (read, room) = retained_demand(out, socket);
        assert_eq!(room, 0);
        match read {
            skein_lib::stream::Read::Scan { .. } => {}
            skein_lib::stream::Read::Nothing
            | skein_lib::stream::Read::Fill(_)
            | skein_lib::stream::Read::Line { .. }
            | skein_lib::stream::Read::Arrived { .. } => panic!("head line demand"),
        }
        server.up(bytes(owner, line), out);
        server_drain(server, out);
    }
    let mut call = None;
    for routed in &*out {
        match routed {
            Routed::Call { call: found, .. } => {
                call = Some(*found);
                break;
            }
            Routed::Io(_) | Routed::Http { .. } => {}
        }
    }
    let call = call.expect("parsed request head");
    (owner, call)
}

pub fn peer_owner(peers: &List<(Token, Token, Token)>, socket: Token) -> Token {
    for (found, owner, _) in peers {
        if *found == socket {
            return *owner;
        }
    }
    unreachable!("fixture peer exists")
}

/// Consume each demand before answering it, preserving the application's routed events.
fn retained_demand(out: &mut Queue<Routed>, socket: Token) -> (skein_lib::stream::Read, u32) {
    for _ in 0..out.len() {
        let routed = out.pop().expect("queued fixture output");
        match routed {
            Routed::Io(skein_io::Request::Stream { stream, down: Down::Demand { read, room } }) if stream == socket => {
                return (read, room);
            }
            Routed::Io(_) | Routed::Call { .. } | Routed::Http { .. } => out.push(routed),
        }
    }
    panic!("fixture answers an emitted demand")
}

/// The clients child and listener, with the existing client fixture's 512 request slots.
pub fn retained_clients() -> crate::clients::Clients {
    const CLIENT_REQUESTS: u32 = 512;
    let (config, _) = crate::startup::fixture_connectors(&super::engine_document());
    let limits = crate::limits::profile_limits(jig_protocol_config::Profile::Small);
    let mut clients = crate::clients::Clients::new(
        &config,
        skein_io::Addr::Ip(core::net::SocketAddr::new(core::net::IpAddr::from([127, 0, 0, 1]), 7100)),
        &super::seeds(),
        CLIENT_REQUESTS,
        &limits.server,
    )
    .expect("clients fixture");
    clients.start(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH).expect("clients start");
    assert!(clients.up(listening()));
    clients
}

/// Drain retained sockets, collecting response bytes and terminal closes.
pub fn retained_response(
    clients: &mut crate::clients::Clients,
    peers: &List<(Token, Token, Token)>,
) -> (List<u8>, u32) {
    let response_room = crate::limits::profile_limits(jig_protocol_config::Profile::Small).server.http.response;
    let mut response = List::with_capacity(response_room);
    let mut closes = 0_u32;
    for _ in 0..SETTLE_STEPS {
        clients.server.advance(&mut clients.server_out);
        clients.server_requests();
        while let Some(request) = clients.io.pop() {
            match request {
                io::Request::Stream { stream, down: Down::Demand { room, .. } } if room > 0 => {
                    let owner = peer_owner(peers, stream);
                    clients.server.up(io::Event::Stream { owner, up: Up::Room }, &mut clients.server_out);
                }
                io::Request::Stream { down: Down::Send(bytes), .. } => {
                    for byte in &bytes {
                        response.push(*byte).expect("bounded response head");
                    }
                }
                io::Request::Close { entity } => {
                    closes = closes.checked_add(1).expect("fixture socket close count");
                    let owner = peer_owner(peers, entity);
                    clients.server.up(io::Event::Closed { owner }, &mut clients.server_out);
                }
                io::Request::Listen { .. }
                | io::Request::Connect { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Stream { .. }
                | io::Request::Output { .. }
                | io::Request::Abort { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. } => {}
            }
        }
    }
    (response, closes)
}
