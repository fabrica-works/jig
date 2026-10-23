use super::*;
use skein_lib::{List, Secret, Wall};
pub const HOST: &[u8] = b"127.0.0.1:8080";
pub const PUBLIC: &[u8] = b"app.example.com";
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
pub fn fresh_mount() -> Mount {
    Mount::new(Settings { prefix: Box::from(b"/jig".as_slice()), public: None }, &env().limits)
}
pub fn public_mount() -> Mount {
    listening(Settings { prefix: Box::from(b"/jig".as_slice()), public: Some(Box::from(PUBLIC)) })
}
pub fn prefixed(prefix: &[u8]) -> Mount {
    listening(Settings { prefix: Box::from(prefix), public: None })
}
fn listening(settings: Settings) -> Mount {
    let mut mount = Mount::new(settings, &env().limits);
    let mut hosts = List::with_capacity(1);
    hosts.push(Box::from(HOST)).expect("one host");
    mount.down(&env(), Event::Listening { hosts }, &mut Queue::with_capacity(1));
    mount
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
    let mut fields =
        List::with_capacity(u32::try_from(headers.len()).expect("headers").checked_add(2).expect("defaults"));
    let mut host = false;
    let mut content = false;
    for header in headers {
        host |= header.name.eq_ignore_ascii_case(b"host");
        content |= header.name.eq_ignore_ascii_case(b"content-type");
        fields.push(header.clone()).expect("supplied header");
    }
    if !host {
        fields.push(skein_http::Header { name: Box::from(b"host".as_slice()), value: Box::from(HOST) }).expect("host");
    }
    if method == Method::Post && !content {
        fields
            .push(skein_http::Header {
                name: Box::from(b"content-type".as_slice()),
                value: Box::from(b"application/octet-stream".as_slice()),
            })
            .expect("content type");
    }
    http::Call {
        method,
        target: Box::from(target),
        headers: fields.into_boxed(),
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

pub const TOKEN: [u8; 40] = [7; 40];
pub const PARTY: u64 = 7;
pub const IN_A_DAY: u64 = 86_400_000_000_000;
pub const COOKIE_ON_LISTENER: &[u8] = b"jig-sign-in=BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBw; Path=/jig; HttpOnly; SameSite=Strict; Max-Age=86400";
pub const COOKIE_SECURE: &[u8] = b"jig-sign-in=BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBw; Path=/jig; HttpOnly; SameSite=Strict; Max-Age=86400; Secure";
pub const COOKIE_CLEARED: &[u8] = b"jig-sign-in=; Path=/jig; HttpOnly; SameSite=Strict; Max-Age=0";
pub const COOKIE_CLEARED_SECURE: &[u8] = b"jig-sign-in=; Path=/jig; HttpOnly; SameSite=Strict; Max-Age=0; Secure";
pub fn browser_header() -> [skein_http::Header; 1] {
    [skein_http::Header { name: Box::from(b"jig-client".as_slice()), value: Box::from(b"browser".as_slice()) }]
}
pub fn browser() -> [skein_http::Header; 2] {
    let [header] = browser_header();
    [header, cookie_header()]
}
pub fn cookie_header() -> skein_http::Header {
    skein_http::Header {
        name: Box::from(b"cookie".as_slice()),
        value: Box::from(
            b"other=kept; jig-sign-in=BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBw; after=kept".as_slice(),
        ),
    }
}
pub fn signed_in_reply(token: &[u8; 40], expires: u64) -> kit::Reply {
    use jig_client_protocol_wire::v17 as w;
    let answer = w::SignInAnswer::SignedIn(
        w::SignedIn::new(
            &w::CEILINGS,
            w::SignedInParts { token: Some(Secret::new(Box::new(*token))), party: PARTY, expires },
        )
        .expect("signed in"),
    );
    let mut bytes = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("sign-in size"));
    answer.encode(&mut bytes).expect("sign in");
    kit::Reply::Record(bytes.finish())
}
pub fn signed_in(bytes: &[u8]) -> jig_client_protocol_wire::v17::SignedIn {
    use jig_client_protocol_wire::v17 as w;
    let w::SignInAnswer::SignedIn(value) =
        w::SignInAnswer::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("sign-in answer")
    else {
        panic!("signed in")
    };
    value
}

pub fn fields(fields: &[(&[u8], &[u8])]) -> Box<[skein_http::Header]> {
    let mut headers = List::with_capacity(u32::try_from(fields.len()).expect("header count"));
    for (name, value) in fields {
        headers.push(skein_http::Header { name: Box::from(*name), value: Box::from(*value) }).expect("header");
    }
    headers.into_boxed()
}
pub fn names_a_cors_field(headers: &[skein_http::Header]) -> bool {
    for header in headers {
        if header.name.starts_with(b"access-control-") {
            return true;
        }
    }
    false
}

pub fn browser_answer(host: &[u8], path: &[u8], reply: kit::Reply) -> Box<[skein_http::Header]> {
    let mut mount = public_mount();
    let mut out = Queue::with_capacity(16);
    let [browser] = browser_header();
    let host = skein_http::Header { name: Box::from(b"host".as_slice()), value: Box::from(host) };
    let call = call(&mut mount, &mut out, Method::Post, path, &[browser, host], b"");
    component(&mut out);
    self::reply(&mut mount, &mut out, call, reply);
    let (_, _, headers) = answered(&mut mount, &mut out, call);
    headers
}

pub fn read_by_host(mount: &mut Mount, out: &mut Queue<Request>, host: &[u8]) -> Token {
    call(mount, out, Method::Post, b"/jig/v1/read", &fields(&[(b"host", host)]), b"")
}

pub fn refused_host(mount: &mut Mount, host: &[u8]) {
    let mut out = Queue::with_capacity(16);
    let call = read_by_host(mount, &mut out, host);
    for request in &out {
        if let Request::Component(_) = request {
            panic!("refused host reached the component: {host:?}");
        }
    }
    let (status, _, _) = answered(mount, &mut out, call);
    assert_eq!(status, 421, "{host:?}");
}

pub fn cookie_for(via: Via) -> skein_http::Header {
    cookie(&public_mount().settings, via, &credential(), skein_lib::Duration::from_nanos(IN_A_DAY))
}
