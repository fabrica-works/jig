use crate::server::{Routed, Server};
use skein_lib::{
    List, Queue, Token,
    stream::{Down, Up},
};
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
    for _ in 0_u32..128 {
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
    for _ in 0_u32..128 {
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
