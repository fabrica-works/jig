use super::*;
use skein_lib::{List, Secret, Wall};
pub const EXCHANGE: Token = Token::new(11);
pub const OTHER: Token = Token::new(12);
pub fn env() -> Env<Limits> {
    let limits = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients_http;
    Env {
        now: Time::ZERO,
        wall: Wall::EPOCH,
        limits: Limits {
            calls: limits.calls,
            streams: limits.streams,
            body: limits.body,
            read: limits.read,
            send: limits.send,
            event: limits.event,
        },
    }
}
pub fn mount() -> Mount {
    prefixed(b"/jig")
}
pub fn prefixed(prefix: &[u8]) -> Mount {
    Mount::new(Settings { prefix: Box::from(prefix), loopback: true }, &env().limits)
}
pub fn credential() -> Secret {
    Secret::new(Box::new([7; 40]))
}
pub fn bearer() -> [skein_http::Header; 1] {
    [skein_http::Header {
        name: Box::from(b"authorization".as_slice()),
        value: Box::from(b"Bearer BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBw".as_slice()),
    }]
}
pub fn head(method: Method, target: &[u8], headers: &[skein_http::Header], body: http::Body) -> http::Call {
    http::Call {
        method,
        target: Box::from(target),
        headers: Box::from(headers),
        body,
        version: skein_http::Version::Http11,
    }
}
pub fn pump(mount: &mut Mount, out: &mut Queue<Request>) {
    while mount.up(&env(), out) {}
}
pub fn event(mount: &mut Mount, out: &mut Queue<Request>, name: Token, event: http::Event) {
    mount.down(&env(), Event::Http { call: name, event }, out);
    pump(mount, out);
}
pub fn call(
    mount: &mut Mount,
    out: &mut Queue<Request>,
    method: Method,
    target: &[u8],
    headers: &[skein_http::Header],
    body: &[u8],
) -> Token {
    call_named(mount, out, EXCHANGE, method, target, headers, body)
}
pub fn call_named(
    mount: &mut Mount,
    out: &mut Queue<Request>,
    name: Token,
    method: Method,
    target: &[u8],
    headers: &[skein_http::Header],
    body: &[u8],
) -> Token {
    mount.down(
        &env(),
        Event::Call {
            call: name,
            head: head(method, target, headers, http::Body::Length(u64::try_from(body.len()).expect("test body"))),
        },
        out,
    );
    pump(mount, out);
    if !body.is_empty() {
        event(mount, out, name, http::Event::Body(Up::Bytes(Box::from(body))));
    }
    event(mount, out, name, http::Event::Body(Up::End));
    name
}
pub fn component(out: &mut Queue<Request>) -> Box<[kit::FromTransport]> {
    let mut messages = List::with_capacity(16);
    while let Some(request) = out.pop() {
        if let Request::Component(message) = request {
            messages.push(message).expect("messages");
        }
    }
    messages.into_boxed()
}
pub fn reply(mount: &mut Mount, out: &mut Queue<Request>, name: Token, reply: kit::Reply) {
    mount.down(&env(), Event::Component(kit::ToTransport::Reply { exchange: name, reply }), out);
    pump(mount, out);
}
pub fn streamed(mount: &mut Mount, out: &mut Queue<Request>, message: kit::Streamed) {
    mount.down(&env(), Event::Component(kit::ToTransport::Streamed { stream: EXCHANGE, message }), out);
    pump(mount, out);
}
pub fn answered(
    mount: &mut Mount,
    out: &mut Queue<Request>,
    name: Token,
) -> (u16, Box<[u8]>, Box<[skein_http::Header]>) {
    let mut status = None;
    let mut headers: Box<[skein_http::Header]> = Box::new([]);
    let mut body = List::with_capacity(65536);
    while let Some(request) = out.pop() {
        if let Request::Http { call, request } = request {
            assert_eq!(call, name);
            match request {
                http::Request::Respond(response) => {
                    status = Some(response.status);
                    headers = response.headers;
                }
                http::Request::Reply(Down::Send(bytes)) => append(&mut body, &bytes),
                http::Request::Reply(Down::Demand { .. }) => event(mount, out, name, http::Event::Reply(Up::Room)),
                http::Request::Next
                | http::Request::Body(_)
                | http::Request::Discard
                | http::Request::Reply(_)
                | http::Request::Close => {}
            }
        }
    }
    (status.expect("response"), body.into_boxed(), headers)
}
pub fn header<'a>(headers: &'a [skein_http::Header], name: &[u8]) -> Option<&'a [u8]> {
    for header in headers {
        if header.name.as_ref() == name {
            return Some(&header.value);
        }
    }
    None
}
pub fn sent(mount: &mut Mount, out: &mut Queue<Request>) -> (Box<[u8]>, u32) {
    let mut bytes = List::with_capacity(65536);
    let mut taken = 0_u32;
    while let Some(request) = out.pop() {
        match request {
            Request::Http { request: http::Request::Reply(Down::Demand { .. }), .. } => {
                event(mount, out, EXCHANGE, http::Event::Reply(Up::Room));
            }
            Request::Http { request: http::Request::Reply(Down::Send(piece)), .. } => append(&mut bytes, &piece),
            Request::Component(kit::FromTransport::StreamTaken { stream }) => {
                assert_eq!(stream, EXCHANGE);
                taken = taken.saturating_add(1);
            }
            Request::Http { .. } | Request::Component(_) => {}
        }
    }
    (bytes.into_boxed(), taken)
}

fn append(out: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        out.push(*byte).expect("bounded output");
    }
}
