use super::{accepted, join, listening, server, server_bytes, server_drain};
use skein_lib::{Intake, List, Queue, bytes, stream::Read};

pub fn head(method: skein_http::Method, target: &[u8], headers: &[(&[u8], &[u8])]) -> skein_http::server::Call {
    let mut fields = List::with_capacity(u32::try_from(headers.len()).expect("fixture fields"));
    for (name, value) in headers {
        fields.push(skein_http::Header { name: (*name).into(), value: (*value).into() }).expect("fixture fields");
    }
    skein_http::server::Call {
        method,
        target: target.into(),
        version: skein_http::Version::Http11,
        headers: fields.into_boxed(),
        body: skein_http::server::Body::None,
    }
}

pub fn bundle() -> crate::server::Bundle {
    let mut files = skein_lib::Map::with_capacity(2);
    files
        .insert(
            Box::from(b"boot.js".as_slice()),
            crate::server::File { kind: b"text/javascript", bytes: Box::from(b"boot();".as_slice()) },
        )
        .expect("boot file");
    files
        .insert(
            Box::from(b"files/robots.txt".as_slice()),
            crate::server::File {
                kind: b"text/plain; charset=utf-8",
                bytes: Box::from(b"User-agent: *\nDisallow: /\n".as_slice()),
            },
        )
        .expect("root file");
    crate::server::Bundle {
        page: Box::from(b"<!doctype html><title>t</title>".as_slice()),
        etag: Box::from(b"\"0011223344556677\"".as_slice()),
        digest: Box::from(b"0123456789abcdef".as_slice()),
        files,
    }
}
pub fn serving(bundle: crate::server::Bundle) -> crate::server::Server {
    let mut server = server();
    server.serve(bundle);
    server
}
pub const REFERENCE_POLICY: &[u8] = b"default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";
pub const POLICY_FIELDS: &[(&[u8], &[u8])] = &[
    (b"Content-Security-Policy", REFERENCE_POLICY),
    (b"X-Content-Type-Options", b"nosniff"),
    (b"Cross-Origin-Opener-Policy", b"same-origin"),
    (b"Cross-Origin-Resource-Policy", b"same-origin"),
    (b"Referrer-Policy", b"no-referrer"),
];
pub struct ServerResponse {
    pub status: u16,
    pub headers: Box<[skein_http::Header]>,
    pub body: Box<[u8]>,
}
impl ServerResponse {
    pub fn fields(&self, name: &[u8]) -> Box<[&[u8]]> {
        let mut fields = List::with_capacity(u32::try_from(self.headers.len()).expect("response fields"));
        for header in &self.headers {
            if header.name.eq_ignore_ascii_case(name) {
                fields.push(header.value.as_ref()).expect("matching fields");
            }
        }
        fields.into_boxed()
    }
}

pub fn answer(server: &mut crate::server::Server, request: &[u8]) -> ServerResponse {
    let mut out = Queue::with_capacity(128);
    server.up(listening(), &mut out);
    server.up(accepted(server::SOCKET), &mut out);
    server_drain(server, &mut out);
    while out.pop().is_some() {} // server_bytes grants the initial response reservation.
    server_bytes(server, &mut out, request);
    wire_response(server, &mut out)
}
pub fn wire_response(server: &mut crate::server::Server, out: &mut Queue<crate::server::Routed>) -> ServerResponse {
    use crate::server::Routed;
    use skein_lib::stream::{Down, Up};
    let limits = crate::limits::profile_limits(jig_protocol_config::Profile::Small).server.http;
    let capacity =
        limits.send.checked_mul(2).expect("fixture body").checked_add(limits.response).expect("fixture head");
    let mut wire = Intake::with_capacity(capacity);
    for _ in 0_u32..4096 {
        let advanced = server.advance(out);
        if let Some(routed) = out.pop() {
            match routed {
                Routed::Io(skein_io::Request::Stream { down: Down::Send(bytes), .. }) => {
                    wire.append(&bytes).expect("fixture response fits");
                }
                Routed::Io(skein_io::Request::Stream { down: Down::Demand { room, .. }, .. }) if room > 0 => {
                    server.up(skein_io::Event::Stream { owner: server::OWNER, up: Up::Room }, out);
                }
                Routed::Io(skein_io::Request::Abort { .. }) => panic!("unexpected connection abort"),
                Routed::Io(_) | Routed::Http { .. } => {}
                Routed::Call { .. } => panic!("server forwarded its own route"),
            }
        } else if !advanced {
            break;
        }
    }
    let wire = wire.meet(Read::Fill(wire.len())).expect("complete wire");
    let end = bytes::find(&wire, b"\r\n\r\n").expect("complete response head");
    let status_end = bytes::find(&wire, b"\r\n").expect("status line");
    let status_line = wire.get(..status_end).expect("status");
    let mut status = 0_u16;
    for digit in status_line.get(9..12).expect("three status digits") {
        status = status
            .checked_mul(10)
            .expect("status")
            .checked_add(u16::from(digit.checked_sub(b'0').expect("digit")))
            .expect("status");
    }
    let mut headers = List::with_capacity(32);
    let mut offset = status_end.checked_add(2).expect("first field");
    for _ in 0_u32..32 {
        if offset >= end {
            break;
        }
        let rest = wire.get(offset..).expect("remaining head");
        let length = bytes::find(rest, b"\r\n").expect("field line");
        let line = rest.get(..length).expect("field");
        offset = offset.checked_add(length).expect("field end").checked_add(2).expect("next field");
        let colon = bytes::find(line, b": ").expect("header colon");
        let name = line.get(..colon).expect("field name");
        let value = line.get(colon.checked_add(2).expect("colon")..).expect("field value");
        headers.push(skein_http::Header { name: name.into(), value: value.into() }).expect("response fields");
    }
    ServerResponse {
        status,
        headers: headers.into_boxed(),
        body: bytes::copy_of(wire.get(end.checked_add(4).expect("head end")..).expect("body")),
    }
}

pub fn native_head(response: &ServerResponse, headers: u32) -> skein_http::client::Response {
    use skein_http::{Method, client as http};
    use skein_lib::{Env, Time, Wall, stream::Up};
    let env = Env {
        now: Time::ZERO,
        wall: Wall::EPOCH,
        limits: http::Limits { head: 4096, headers, request: 4096, read: 1024, send: 1024 },
    };
    let mut client = http::Client::new(&env.limits);
    let mut above = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(32);
    http::down(
        &mut client,
        &env,
        http::Request::Call(http::Call {
            method: Method::Get,
            target: b"/jig/v1/read".as_slice().into(),
            headers: head(Method::Get, b"/", &[(b"Host", b"localhost")]).headers,
            body: http::Body::None,
            close: false,
        }),
        &mut above,
        &mut below,
    );
    http::up(&mut client, &env, Up::Room, &mut above, &mut below);
    http::up(&mut client, &env, Up::Bytes(b"HTTP/1.1 200 OK\r\n".as_slice().into()), &mut above, &mut below);
    for header in &response.headers {
        let line = join(&[&header.name, b": ", &header.value, b"\r\n"]);
        http::up(&mut client, &env, Up::Bytes(line), &mut above, &mut below);
    }
    http::up(&mut client, &env, Up::Bytes(b"\r\n".as_slice().into()), &mut above, &mut below);
    let event = above.pop().expect("parsed head");
    let http::Event::Response(head) = event else { panic!("native peer refused the origin policy: {event:?}") };
    head
}
