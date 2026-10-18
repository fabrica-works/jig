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
    assert_eq!(*f::calls(&mut out), [(Box::from(b"/jig/v1/read".as_slice()), Method::Post)]);
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
    assert_eq!(*f::calls(&mut out), [(Box::from(b"/jig/v1/read".as_slice()), Method::Post)]);
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

use crate::server::Answer;
use skein_http::Method;
type Row = (Method, &'static [u8], &'static [(&'static [u8], &'static [u8])], Answer);
const ANSWERS: &[Row] = &[
    (Method::Get, b"/", &[], Answer::Page),
    (Method::Get, b"/tasks/7", &[], Answer::Page),
    (Method::Get, b"/jig/v1/sign-in/link?secret=abc", &[], Answer::Page),
    (Method::Post, b"/jig/v1/read", &[], Answer::Mount),
    (Method::Get, b"/jig/v1/stream", &[], Answer::Mount),
    (Method::Get, b"/bundle/0123456789abcdef/boot.js", &[], Answer::File),
    (Method::Get, b"/bundle/fedcba9876543210/boot.js", &[], Answer::NotFound),
    (Method::Get, b"/bundle/0123456789abcdef/missing.js", &[], Answer::NotFound),
    (Method::Get, b"/robots.txt", &[], Answer::Root),
    (Method::Get, b"/", &[(b"if-none-match", b"\"0011223344556677\"")], Answer::NotModified),
    (Method::Post, b"/", &[], Answer::NotAllowed),
];
#[test]
fn each_path_is_answered_by_its_part_of_the_server() {
    let server = f::serving(f::bundle());
    for (method, target, headers, expected) in ANSWERS {
        assert_eq!(server.route(&f::head(*method, target, headers)), *expected, "{method:?} {target:?}");
    }
}
#[test]
fn with_no_bundle_a_page_get_is_404() {
    let reply = f::answer(&mut f::server(), b"GET /tasks/7 HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(reply.status, 404);
    assert_eq!(reply.body.as_ref(), b"");
}
#[test]
fn a_bundle_file_is_immutable_and_its_kind_follows_its_suffix() {
    let reply = f::answer(
        &mut f::serving(f::bundle()),
        b"GET /bundle/0123456789abcdef/boot.js HTTP/1.1\r\nHost: localhost\r\n\r\n",
    );
    assert_eq!(reply.status, 200);
    assert_eq!(*reply.fields(b"Cache-Control"), [b"public, max-age=31536000, immutable".as_slice()]);
    assert_eq!(*reply.fields(b"Content-Type"), [b"text/javascript".as_slice()]);
    assert_eq!(reply.body.as_ref(), b"boot();");
}
#[test]
fn the_page_is_revalidated_by_its_etag() {
    const REQUESTS: &[(&[u8], u16, &[u8])] = &[
        (b"GET /tasks/7 HTTP/1.1\r\nHost: localhost\r\n\r\n", 200, b"<!doctype html><title>t</title>"),
        (b"GET /tasks/7 HTTP/1.1\r\nHost: localhost\r\nIf-None-Match: \"0011223344556677\"\r\n\r\n", 304, b""),
        (
            b"GET /tasks/7 HTTP/1.1\r\nHost: localhost\r\nIf-None-Match: \"different\"\r\n\r\n",
            200,
            b"<!doctype html><title>t</title>",
        ),
    ];
    for (request, status, body) in REQUESTS {
        let reply = f::answer(&mut f::serving(f::bundle()), request);
        assert_eq!(reply.status, *status, "{request:?}");
        assert_eq!(*reply.fields(b"ETag"), [b"\"0011223344556677\"".as_slice()]);
        assert_eq!(*reply.fields(b"Cache-Control"), [b"no-cache".as_slice()]);
        assert_eq!(reply.body.as_ref(), *body);
    }
}
#[test]
fn each_answer_carries_the_reference_policy_s_five_fields() {
    const REQUESTS: &[(&[u8], u16)] = &[
        (b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n", 200),
        (b"GET /bundle/0123456789abcdef/boot.js HTTP/1.1\r\nHost: localhost\r\n\r\n", 200),
        (b"GET /robots.txt HTTP/1.1\r\nHost: localhost\r\n\r\n", 200),
        (b"GET / HTTP/1.1\r\nHost: localhost\r\nIf-None-Match: \"0011223344556677\"\r\n\r\n", 304),
        (b"GET /bundle/wrong/boot.js HTTP/1.1\r\nHost: localhost\r\n\r\n", 404),
        (b"POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n", 405),
    ];
    for (request, status) in REQUESTS {
        let reply = f::answer(&mut f::serving(f::bundle()), request);
        assert_eq!(reply.status, *status, "{request:?}");
        for (name, value) in f::POLICY_FIELDS {
            assert_eq!(*reply.fields(name), [*value], "{name:?} for {request:?}");
        }
    }
}
#[test]
fn the_mount_s_answer_gains_the_policy_but_keeps_its_own_fields() {
    let mut server = f::server();
    let mut out = Queue::with_capacity(128);
    let call = f::server_call(&mut server, &mut out);
    server.answer(call, skein_http::server::Request::Discard, &mut out);
    server.answer(
        call,
        skein_http::server::Request::Respond(skein_http::server::Response {
            status: 204,
            headers: Box::from([
                skein_http::Header { name: b"Cache-Control".as_slice().into(), value: b"no-store".as_slice().into() },
                skein_http::Header {
                    name: b"x-CoNtEnT-tYpE-oPtIoNs".as_slice().into(),
                    value: b"nosniff".as_slice().into(),
                },
                skein_http::Header {
                    name: b"Referrer-Policy".as_slice().into(),
                    value: b"same-origin".as_slice().into(),
                },
            ]),
            body: skein_http::server::Body::None,
            close: false,
        }),
        &mut out,
    );
    let reply = f::wire_response(&mut server, &mut out);
    assert_eq!(reply.status, 204);
    assert_eq!(*reply.fields(b"Cache-Control"), [b"no-store".as_slice()]);
    for (name, value) in f::POLICY_FIELDS {
        let expected = if name.eq_ignore_ascii_case(b"referrer-policy") { b"same-origin".as_slice() } else { value };
        assert_eq!(*reply.fields(name), [expected], "{name:?}");
    }
}
#[test]
fn the_link_route_s_page_carries_no_referrer() {
    let reply = f::answer(
        &mut f::serving(f::bundle()),
        b"GET /jig/v1/sign-in/link?secret=abc HTTP/1.1\r\nHost: localhost\r\n\r\n",
    );
    assert_eq!(reply.status, 200);
    assert_eq!(reply.body.as_ref(), b"<!doctype html><title>t</title>");
    assert_eq!(*reply.fields(b"Referrer-Policy"), [b"no-referrer".as_slice()]);
}
#[test]
fn the_root_file_is_served_with_the_page_s_cache_policy() {
    let reply = f::answer(&mut f::serving(f::bundle()), b"GET /robots.txt HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(reply.status, 200);
    assert_eq!(reply.body.as_ref(), b"User-agent: *\nDisallow: /\n");
    assert_eq!(*reply.fields(b"Content-Type"), [b"text/plain; charset=utf-8".as_slice()]);
    assert_eq!(*reply.fields(b"Cache-Control"), [b"no-cache".as_slice()]);
    assert_eq!(*reply.fields(b"ETag"), [b"\"0011223344556677\"".as_slice()]);
}
#[test]
fn a_bundle_file_larger_than_one_send_is_delivered_in_full() {
    let mut bundle = f::bundle();
    let limits = crate::limits::profile_limits(jig_protocol_config::Profile::Small).server;
    let length = usize::try_from(limits.http.send).expect("send size").checked_add(1).expect("larger file");
    bundle.files.get_mut(&Box::from(b"boot.js".as_slice())).expect("boot file").bytes =
        skein_lib::bytes::zeroed(length);
    let reply =
        f::answer(&mut f::serving(bundle), b"GET /bundle/0123456789abcdef/boot.js HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(reply.status, 200);
    assert_eq!(reply.body.as_ref(), skein_lib::bytes::zeroed(length).as_ref());
}

#[test]
fn the_bundle_is_installed_before_the_service_loop() {
    let document = f::with(&f::engine_document(), br#""components":[]"#, br#""components":["clients"]"#);
    let mut service = f::started(&document);
    assert_eq!(service.serve(f::bundle()), Ok(()));
    let clients = f::engine(&mut service).clients.as_mut().expect("clients");
    assert_eq!(clients.server.route(&f::head(Method::Get, b"/tasks/7", &[])), Answer::Page);
}
#[test]
fn a_bundle_is_refused_after_the_service_loop_starts() {
    let document = f::with(&f::engine_document(), br#""components":[]"#, br#""components":["clients"]"#);
    let mut service = f::started(&document);
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.serve(f::bundle()), Err(crate::Refused));
}
#[test]
fn a_bundle_is_refused_when_the_service_has_no_server() {
    let mut service = f::started(&f::engine_document());
    assert_eq!(service.serve(f::bundle()), Err(crate::Refused));
}

type KindRow = (&'static [u8], Option<&'static [u8]>);
const KINDS: &[KindRow] = &[
    (b"index.html", Some(b"text/html; charset=utf-8")),
    (b"boot.js", Some(b"text/javascript")),
    (b"app.wasm", Some(b"application/wasm")),
    (b"app.css", Some(b"text/css")),
    (b"image.svg", Some(b"image/svg+xml")),
    (b"image.png", Some(b"image/png")),
    (b"image.webp", Some(b"image/webp")),
    (b"favicon.ico", Some(b"image/x-icon")),
    (b"font.woff2", Some(b"font/woff2")),
    (b"robots.txt", Some(b"text/plain; charset=utf-8")),
    (b"app.webmanifest", Some(b"application/manifest+json")),
    (b"script.exe", None),
    (b"path.js/file", None),
    (b"no-extension", None),
];
#[test]
fn only_the_closed_list_of_file_extensions_has_a_content_type() {
    for (path, expected) in KINDS {
        assert_eq!(crate::server::kind(path), *expected, "{path:?}");
    }
}

#[test]
fn a_native_http_client_accepts_the_mount_s_answer_with_the_policy() {
    use skein_http::server;
    let mut server = f::server();
    let mut out = Queue::with_capacity(128);
    let call = f::server_call(&mut server, &mut out);
    let headers = [
        (b"Content-Type".as_slice(), b"application/octet-stream".as_slice()),
        (b"Cache-Control".as_slice(), b"no-store".as_slice()),
        (b"X-Content-Type-Options".as_slice(), b"nosniff".as_slice()),
        (b"Set-Cookie".as_slice(), b"jig=credential; HttpOnly; SameSite=Strict; Path=/jig".as_slice()),
    ];
    server.answer(call, server::Request::Discard, &mut out);
    server.answer(
        call,
        server::Request::Respond(server::Response {
            status: 200,
            headers: f::head(Method::Get, b"/", &headers).headers,
            body: server::Body::None,
            close: false,
        }),
        &mut out,
    );
    let response = f::wire_response(&mut server, &mut out);
    // The protocol world's native peer reserves room for the reference origin policy.
    let head = f::native_head(&response, 16);
    assert_eq!(head.status, 200);
}
