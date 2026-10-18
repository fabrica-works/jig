use super::*;
use skein_http::Method;
use skein_lib::stream::{Down, Up};
mod fixtures;
#[test]
fn a_read_post_becomes_the_component_s_read_exchange() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/read", &fixtures::bearer(), b"\x01\x02");
    assert_eq!(
        *fixtures::component(&mut out),
        [kit::FromTransport::Exchange {
            exchange: fixtures::EXCHANGE,
            credential: Some(fixtures::credential()),
            message: kit::Message::Read(Box::new([1, 2]))
        }]
    );
}
#[test]
fn a_reply_is_written_as_200_with_its_record_and_no_store() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/read", &[], b"");
    fixtures::component(&mut out);
    fixtures::reply(&mut mount, &mut out, call, kit::Reply::Record(Box::new([3, 4])));
    let (status, body, headers) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!((status, body.as_ref()), (200, &[3, 4][..]));
    assert_eq!(fixtures::header(&headers, b"cache-control"), Some(b"no-store".as_slice()));
}
#[test]
fn a_record_the_component_cannot_read_is_written_as_400() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/read", &[], b"");
    fixtures::component(&mut out);
    fixtures::reply(&mut mount, &mut out, call, kit::Reply::NotARecord);
    let (status, body, _) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!((status, body.as_ref()), (400, &[][..]));
}
#[test]
fn a_body_past_the_route_s_bound_is_413_and_reaches_nothing() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(32);
    let mut body = skein_lib::List::with_capacity(65537);
    for _ in 0_u32..65537 {
        body.push(7).expect("body");
    }
    let body = body.into_boxed();
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/read", &[], body.as_ref());
    let mut count = 0_u32;
    for request in &out {
        if let Request::Component(_) = request {
            count = count.saturating_add(1);
        }
    }
    assert_eq!(count, 0);
    let (status, body, _) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!((status, body.as_ref()), (413, &[][..]));
}
#[test]
fn a_path_under_the_prefix_with_no_route_is_404() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/absent", &[], b"");
    let (status, body, _) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!((status, body.as_ref()), (404, &[][..]));
}
#[test]
fn a_streamed_record_is_one_event_and_taken_once_written() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    fixtures::call(&mut mount, &mut out, Method::Get, b"/jig/v1/stream", &[], b"");
    fixtures::component(&mut out);
    fixtures::streamed(&mut mount, &mut out, kit::Streamed::Record(Box::new([1, 2, 3])));
    let (bytes, taken) = fixtures::sent(&mut mount, &mut out);
    assert_eq!((bytes.as_ref(), taken), (b"data: AQID\n\n".as_slice(), 1));
    assert!(out.is_empty());
}
#[test]
fn a_heartbeat_is_written_as_a_comment_line() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    fixtures::call(&mut mount, &mut out, Method::Get, b"/jig/v1/stream", &[], b"");
    fixtures::component(&mut out);
    fixtures::streamed(&mut mount, &mut out, kit::Streamed::Heartbeat);
    let (bytes, taken) = fixtures::sent(&mut mount, &mut out);
    assert_eq!((bytes.as_ref(), taken), (b":\n\n".as_slice(), 1));
}
#[test]
fn the_same_routes_answer_under_another_prefix() {
    let mut mount = fixtures::prefixed(b"/api");
    let mut out = Queue::with_capacity(16);
    fixtures::call(&mut mount, &mut out, Method::Post, b"/api/v1/read", &[], b"\x01\x02");
    assert_eq!(
        *fixtures::component(&mut out),
        [kit::FromTransport::Exchange {
            exchange: fixtures::EXCHANGE,
            credential: None,
            message: kit::Message::Read(Box::new([1, 2]))
        }]
    );
}
#[test]
fn a_lost_connection_drops_the_answer_and_nothing_else() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(32);
    fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/request", &[], b"");
    fixtures::component(&mut out);
    fixtures::call_named(&mut mount, &mut out, fixtures::OTHER, Method::Post, b"/jig/v1/request", &[], b"");
    fixtures::component(&mut out);
    fixtures::event(&mut mount, &mut out, fixtures::EXCHANGE, http::Event::Closed);
    out = Queue::with_capacity(32);
    fixtures::reply(&mut mount, &mut out, fixtures::EXCHANGE, kit::Reply::Record(Box::new([7])));
    assert!(out.is_empty());
    fixtures::reply(&mut mount, &mut out, fixtures::OTHER, kit::Reply::Record(Box::new([9])));
    let (status, body, _) = fixtures::answered(&mut mount, &mut out, fixtures::OTHER);
    assert_eq!((status, body.as_ref()), (200, &[9][..]));
}
#[test]
fn a_chunked_request_body_read_as_it_arrives_is_the_same_whatever_its_cuts() {
    const BODY: &[u8] = b"a chunked record";
    for size in [BODY.len(), 1] {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(64);
        mount.down(
            &fixtures::env(),
            Event::Call {
                call: fixtures::EXCHANGE,
                head: fixtures::head(Method::Post, b"/jig/v1/request", &fixtures::bearer(), http::Body::Chunked),
            },
            &mut out,
        );
        fixtures::pump(&mut mount, &mut out);
        for piece in BODY.chunks(size) {
            fixtures::event(&mut mount, &mut out, fixtures::EXCHANGE, http::Event::Body(Up::Bytes(Box::from(piece))));
        }
        fixtures::event(&mut mount, &mut out, fixtures::EXCHANGE, http::Event::Body(Up::End));
        assert_eq!(
            *fixtures::component(&mut out),
            [kit::FromTransport::Exchange {
                exchange: fixtures::EXCHANGE,
                credential: Some(fixtures::credential()),
                message: kit::Message::Request(Box::from(BODY))
            }]
        );
    }
}
