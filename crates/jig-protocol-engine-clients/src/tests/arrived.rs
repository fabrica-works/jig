use super::fixtures::{self as f, Exchange};
use crate::{Request, connection::Connection};
use jig_client_protocol_wire::v8 as w;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_http::server as http;
use skein_lib::{
    Writer,
    stream::{Down, Read, Up},
};

fn chunked(exchange: &mut Exchange) -> Connection {
    let owner = exchange.component.name();
    let mut connection = Connection::new(owner, &exchange.env.limits, exchange.env.now);
    connection.requests.pop();
    let token = f::url(&exchange.token);
    let mut bearer = Writer::new(token.len().checked_add(7).expect("bearer size"));
    bearer.put(b"Bearer ").expect("prefix");
    bearer.put(&token).expect("token");
    let call = http::Call {
        method: skein_http::Method::Post,
        target: Box::from(b"/jig/v1/request".as_slice()),
        headers: Box::new([skein_http::Header {
            name: Box::from(b"authorization".as_slice()),
            value: bearer.finish(),
        }]),
        body: http::Body::Chunked,
        version: skein_http::Version::Http11,
    };
    exchange.component.http(&exchange.env, &mut connection, http::Event::Call(call), &mut exchange.out);
    connection
}

#[test]
fn a_chunked_request_body_read_as_it_arrives_is_the_same_whatever_its_cuts() {
    let body = f::keyed(w::Owner::Jig, 8, f::encode_ask(&f::start(&f::wire())));
    for size in [body.len(), 1] {
        let mut env = f::env();
        env.limits.http.read = 256;
        let mut exchange = Exchange::with_env(env);
        let mut connection = chunked(&mut exchange);
        for piece in body.chunks(size) {
            let Some(http::Request::Body(Down::Demand { read, room })) = connection.requests.pop() else {
                panic!("body demand")
            };
            assert_eq!(read, Read::Arrived { max: 256 });
            assert_eq!(room, 0);
            exchange.component.http(
                &exchange.env,
                &mut connection,
                http::Event::Body(Up::Bytes(Box::from(piece))),
                &mut exchange.out,
            );
        }
        exchange.component.http(&exchange.env, &mut connection, http::Event::Body(Up::End), &mut exchange.out);
        exchange.component.connections.insert(connection.socket, connection).expect("test connection");
        exchange.authenticate();
        let Some(Request::Core(event)) = exchange.out.pop() else { panic!("core ask") };
        let core::Event::People(people::Event::Ask { sign_in, ask, .. }) = *event else { panic!("people ask") };
        assert_eq!(sign_in, 41);
        assert_eq!(ask, people::Ask::Say { project: 3, task: 8, words: Box::from(b"hello".as_slice()) });
        assert!(exchange.out.is_empty());
    }
}
