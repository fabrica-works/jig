use super::*;
use jig_protocol_client_link as link;
use skein_lib::{
    Queue, Secret, Time, Token, Writer,
    base64::{self, Alphabet},
    stream::Down,
};
use skein_world::stream::Wire;
const TOKEN: [u8; 40] = [7; 40];
fn limits() -> Limits {
    Limits {
        exchanges: 2,
        body_bytes: 128,
        http: skein_http::client::Limits { head: 1024, headers: 8, request: 1024, read: 64, send: 64 },
    }
}
struct Peer {
    shell: Connections,
    wire: Wire,
    up: Queue<link::Event>,
    below: Queue<(Conn, Down)>,
    conn: Option<Conn>,
}
impl Peer {
    fn new() -> Self {
        Self {
            shell: Connections::new(&limits()),
            wire: Wire::new(1),
            up: Queue::with_capacity(256),
            below: Queue::with_capacity(256),
            conn: None,
        }
    }
    fn exchange(&mut self) {
        self.shell.down(
            Time::ZERO,
            link::Request::Exchange {
                exchange: Token::new(42),
                path: b"/jig/v1/request",
                body: Box::from(b"ask".as_slice()),
            },
            &mut self.below,
        );
    }
    fn drive(&mut self) {
        self.shell.take(Time::ZERO, &mut self.up, &mut self.below);
        for _ in 0..4000 {
            while let Some((conn, down)) = self.below.pop() {
                self.conn = Some(conn);
                self.wire.take(down);
            }
            if let Some(up) = self.wire.answer() {
                self.shell.up(Time::ZERO, self.conn.expect("one test connection"), up, &mut self.up, &mut self.below);
            }
        }
    }
    fn respond(&mut self, response: &[u8]) {
        self.wire.write(response);
        self.drive();
    }
}
fn printed() -> Vec<u8> {
    let mut writer = Writer::new(43);
    base64::encode(Alphabet::Url, &[3; 32], &mut writer).expect("short secret");
    let mut link = b"http://127.0.0.1:8080/jig/v1/sign-in/link?secret=".to_vec();
    link.extend_from_slice(&writer.finish());
    link
}
#[test]
fn an_exchange_carries_the_bearer_token() {
    let mut peer = Peer::new();
    peer.shell.set_token(Some(Box::new(TOKEN)));
    peer.exchange();
    peer.respond(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
    let text = std::str::from_utf8(&peer.wire.received).expect("HTTP request");
    let mut writer = Writer::new(54);
    base64::encode(Alphabet::Url, &TOKEN, &mut writer).expect("bearer");
    let expected = format!("authorization: Bearer {}", std::str::from_utf8(&writer.finish()).expect("base64"));
    assert!(text.to_ascii_lowercase().contains(&expected.to_ascii_lowercase()), "{text}");
    assert!(text.to_ascii_lowercase().contains("content-type: application/octet-stream\r\n"));
    assert!(text.to_ascii_lowercase().contains("content-length: 3\r\n"));
    assert!(text.to_ascii_lowercase().contains("host: localhost\r\n"));
    assert!(
        matches!(peer.up.pop(), Some(link::Event::Answered { exchange, status: 200, body }) if exchange == Token::new(42) && body.as_ref() == b"ok")
    );
}
#[test]
fn a_connection_cut_mid_body_fails_the_exchange() {
    let mut peer = Peer::new();
    peer.exchange();
    peer.respond(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nab");
    peer.wire.eof = true;
    peer.drive();
    assert!(matches!(peer.up.pop(), Some(link::Event::Failed { exchange }) if exchange == Token::new(42)));
    assert!(peer.up.is_empty());
}
#[test]
fn the_stream_s_bytes_reach_the_link_in_order() {
    let mut peer = Peer::new();
    peer.shell.down(Time::ZERO, link::Request::OpenStream, &mut peer.below);
    peer.respond(b"HTTP/1.1 200 OK\r\n\r\ndata: first\n\ndata: second\n\n");
    assert!(matches!(peer.up.pop(), Some(link::Event::StreamOpened { status: 200 })));
    let mut bytes = Vec::new();
    while let Some(event) = peer.up.pop() {
        if let link::Event::StreamBytes { bytes: part } = event {
            bytes.extend_from_slice(&part);
        }
    }
    assert_eq!(bytes, b"data: first\n\ndata: second\n\n");
    peer.shell.down(Time::ZERO, link::Request::CloseStream, &mut peer.below);
    peer.drive();
    assert!(peer.wire.finished);
}
#[test]
fn a_link_of_another_form_is_refused() {
    let mut shell = Connections::new(&limits());
    for (label, text, expected) in [
        ("scheme", b"https://127.0.0.1:8080/".as_slice(), Unlinked::NotHttp),
        ("secret", b"http://127.0.0.1:8080/jig/v1/sign-in/link".as_slice(), Unlinked::NoSecret),
        ("address", b"http://example/".as_slice(), Unlinked::Address),
        ("short secret", b"http://127.0.0.1:8080/jig/v1/sign-in/link?secret=AA".as_slice(), Unlinked::NoSecret),
    ] {
        assert_eq!(shell.sign_in(text), Err(expected), "{label}");
    }
}
fn record(value: &jig_protocol_client_wire::v7::SignInAnswer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("short record"));
    value.encode(&mut writer).expect("measured record");
    writer.finish()
}
#[test]
fn a_sign_in_keeps_its_token_only_after_a_whole_answer() {
    use jig_protocol_client_wire::v7 as w;
    let mut peer = Peer::new();
    let exchange = peer.shell.sign_in(&printed()).expect("printed link");
    let body = record(&w::SignInAnswer::SignedIn(
        w::SignedIn::new(&w::CEILINGS, w::SignedInParts { token: Secret::new(Box::new(TOKEN)), party: 1, expires: 10 })
            .expect("token fits"),
    ));
    let head = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len());
    peer.respond(head.as_bytes());
    assert!(peer.shell.token().is_none());
    peer.respond(&body);
    assert_eq!(peer.shell.signing(), Some(Signing::SignedIn { exchange, party: 1 }));
    assert_eq!(peer.shell.token().expect("shell-owned token").expose(), &TOKEN);
}
#[test]
fn a_not_ready_signing_exchange_takes_nothing_and_can_be_sent_again() {
    use jig_protocol_client_wire::v7 as w;
    for (label, sign_out) in [("sign in", false), ("sign out", true)] {
        let mut peer = Peer::new();
        peer.shell.set_token(Some(Box::new(TOKEN)));
        let exchange =
            if sign_out { peer.shell.sign_out() } else { peer.shell.sign_in(&printed()).expect("printed link") };
        let body = if sign_out {
            let mut writer = Writer::new(1);
            w::SignOutAnswer::NotReady.encode(&mut writer).expect("one byte");
            writer.finish()
        } else {
            record(&w::SignInAnswer::NotReady)
        };
        peer.respond(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n");
        peer.respond(&body);
        assert_eq!(peer.shell.signing(), Some(Signing::Again { exchange }), "{label}");
        assert_eq!(peer.shell.token().expect("kept token").expose(), &TOKEN, "{label}");
        if sign_out {
            peer.shell.sign_out();
        } else {
            assert!(peer.shell.sign_in(&printed()).is_ok(), "{label}");
        }
    }
}
#[test]
fn a_whole_sign_out_answer_clears_the_shell_token() {
    let mut peer = Peer::new();
    peer.shell.set_token(Some(Box::new(TOKEN)));
    let exchange = peer.shell.sign_out();
    peer.respond(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n\x00");
    assert_eq!(peer.shell.signing(), Some(Signing::SignedOut { exchange }));
    assert!(peer.shell.token().is_none());
}
#[test]
fn a_lost_sign_out_keeps_the_shell_token() {
    for (label, response) in [
        ("cut", b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n\x00".as_slice()),
        ("bad record", b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n\xff".as_slice()),
        ("bad status", b"HTTP/1.1 503 Busy\r\nContent-Length: 0\r\n\r\n".as_slice()),
    ] {
        let mut peer = Peer::new();
        peer.shell.set_token(Some(Box::new(TOKEN)));
        let exchange = peer.shell.sign_out();
        peer.respond(response);
        peer.wire.eof = true;
        peer.drive();
        assert_eq!(peer.shell.signing(), Some(Signing::Lost { exchange }), "{label}");
        assert_eq!(peer.shell.token().expect("original token").expose(), &TOKEN, "{label}");
    }
}
