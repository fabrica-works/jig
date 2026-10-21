use jig_client_protocol_wire::v11 as w;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_engine_clients_world::fixtures::{self, Peer};
use jig_protocol_engine_clients::{Event, MintRefusal};
use skein_lib::{Reader, Token, Writer};
fn signed() -> Peer {
    Peer::signed_in(&fixtures::settings(), fixtures::FIRST_SIGN_IN)
}
fn watched() -> (Peer, [u8; 16], Token) {
    let mut peer = signed();
    let stream = peer.open_stream();
    let watcher = peer.open_watch(stream, &fixtures::me_subject());
    (peer, stream, watcher)
}
fn sign_out(peer: &mut Peer) -> (Token, skein_lib::ReplyTo) {
    let socket = peer.authenticated_post(b"/jig/v1/sign-out", Box::new([]));
    let Some(core::Event::People(people::Event::SignOut { reply_to, sign_in })) = peer.take_core() else {
        panic!("sign-out reaches people");
    };
    assert_eq!(sign_in, fixtures::FIRST_SIGN_IN);
    (socket, reply_to)
}
fn answer(peer: &Peer, socket: Token) -> w::KeyedAnswer {
    w::KeyedAnswer::decode(&fixtures::limits().wire, &mut Reader::new(peer.answered(socket).expect("answer")))
        .expect("wire answer")
}
fn end(watch: u64, why: w::WatchEnd) -> w::StreamEvent {
    w::StreamEvent::Ended(w::Ended::new(&fixtures::limits().wire, w::EndedParts { watch, why }).expect("ending"))
}
fn second_stream(peer: &mut Peer) -> [u8; 16] {
    let spent = peer.printed_link();
    peer.post_link(&spent);
    let socket = peer.post_link(&peer.printed_link());
    peer.answer_signed_in(42, fixtures::FIRST_PERSON);
    peer.answer_kept(42);
    peer.remember_sign_in(socket);
    peer.open_stream()
}
fn keyed() -> Box<[u8]> {
    let lim = fixtures::limits().wire;
    let ask = w::Ask::Message(
        w::Message::new(&lim, w::MessageParts { project: 3, task: 8, words: Box::from(b"hello".as_slice()) })
            .expect("ask"),
    );
    let mut bytes = Writer::new(usize::try_from(ask.measure()).expect("ask size"));
    ask.encode(&mut bytes).expect("ask bytes");
    let record = w::Keyed::new(
        &lim,
        w::KeyedParts { key: [5; 16], owner: w::Owner::Jig, version: 11, request: bytes.finish(), value: None },
    )
    .expect("keyed");
    let mut bytes = Writer::new(usize::try_from(record.measure()).expect("keyed size"));
    record.encode(&mut bytes).expect("keyed bytes");
    bytes.finish()
}
#[test]
fn signing_out_erases_the_digest_and_refuses_the_token() {
    let mut peer = signed();
    peer.hold_erases();
    let (socket, to) = sign_out(&mut peer);
    assert_eq!(peer.answered(socket), None, "before the held reply");
    peer.answer_request(to, people::Reply::SignedOut);
    assert_eq!(peer.answered(socket), None, "before the digest erase is durable");
    assert!(peer.has_digest(fixtures::FIRST_SIGN_IN));
    peer.answer_erased();
    assert!(!peer.has_digest(fixtures::FIRST_SIGN_IN));
    assert_eq!(
        w::SignOutAnswer::decode(&fixtures::limits().wire, &mut Reader::new(peer.answered(socket).expect("answer"))),
        Ok(w::SignOutAnswer::SignedOut)
    );
    let retry = peer.authenticated_post(b"/jig/v1/request", Box::new([]));
    assert_eq!(answer(&peer, retry), w::KeyedAnswer::SignedOut);
    assert!(peer.take_core().is_none());
}
#[test]
fn a_sign_in_whose_record_is_erased_ends_its_stream() {
    let (mut peer, stream, watcher) = watched();
    peer.event(Event::SignInEnded { sign_in: fixtures::FIRST_SIGN_IN });
    assert!(!peer.has_digest(fixtures::FIRST_SIGN_IN));
    assert!(peer.stream_closed(stream));
    assert!(peer.unwatched(watcher));
    assert_eq!(peer.socket_closes(peer.stream_socket(stream)), 1);
}
#[test]
fn after_stop_a_new_exchange_is_answered_not_ready() {
    let mut peer = signed();
    let socket = peer.accepted_post(b"/jig/v1/request", keyed());
    peer.event(Event::Stop);
    assert_eq!(answer(&peer, socket), w::KeyedAnswer::NotReady);
    assert!(peer.take_core().is_none());
    assert_eq!(peer.socket_closes(Token::new(99)), 1);
    assert_eq!(peer.closed(), 0);
}
#[test]
fn after_stop_a_read_is_served() {
    let mut peer = signed();
    let lim = fixtures::limits().wire;
    let value = w::Read::Me;
    let mut bytes = Writer::new(usize::try_from(value.measure()).expect("read size"));
    value.encode(&mut bytes).expect("read bytes");
    let record = w::Owned::new(&lim, w::OwnedParts { owner: w::Owner::Jig, version: 11, record: bytes.finish() })
        .expect("read owner");
    let mut bytes = Writer::new(usize::try_from(record.measure()).expect("owner size"));
    record.encode(&mut bytes).expect("owner bytes");
    let socket = peer.accepted_post(b"/jig/v1/read", bytes.finish());
    peer.event(Event::Stop);
    let Some(core::Event::Read { reply, sign_in, read }) = peer.take_core() else { panic!("read served") };
    assert_eq!(sign_in, fixtures::FIRST_SIGN_IN);
    assert_eq!(read, core::reads::Read::Frame);
    peer.answer_read(reply, core::reads::Answer::None(core::reads::Why::Unknown));
    assert_eq!(
        w::ReadAnswer::decode(&lim, &mut Reader::new(peer.answered(socket).expect("read answer"))),
        Ok(w::ReadAnswer::None(
            w::ReadAnswerNone::new(&lim, w::ReadAnswerNoneParts { value: w::ReadNone::Unknown })
                .expect("literal unknown")
        )),
        "read"
    );
}
#[test]
fn after_stop_a_new_watch_is_ended_as_stopping() {
    let mut peer = signed();
    let stream = peer.open_stream();
    let lim = fixtures::limits().wire;
    let owned = w::Owned::new(&lim, w::OwnedParts { owner: w::Owner::Jig, version: 11, record: Box::new([0]) })
        .expect("frame subject");
    let open = w::Open::new(&lim, w::OpenParts { stream, watch: 1, subject: owned }).expect("watch open");
    let mut bytes = Writer::new(usize::try_from(open.measure()).expect("open size"));
    open.encode(&mut bytes).expect("open bytes");
    let socket = peer.accepted_post(b"/jig/v1/watch", bytes.finish());
    peer.event(Event::Stop);
    assert_eq!(
        w::OpenAnswer::decode(&lim, &mut Reader::new(peer.answered(socket).expect("watch answer"))),
        Ok(w::OpenAnswer::Admitted)
    );
    assert_eq!(peer.events(stream).last(), Some(&end(1, w::WatchEnd::Stopping)));
    assert!(peer.take_core().is_none(), "local stopping ending makes no new core watch");
    assert!(!peer.stream_closed(stream));
}
#[test]
fn close_ends_each_stream_then_says_closed() {
    let (mut peer, first, watcher) = watched();
    let second = second_stream(&mut peer);
    peer.stop_reading(first);
    peer.release_delivery(watcher, fixtures::me_document(7));
    peer.event(Event::Close);
    assert_eq!(peer.closed(), 0, "the last stream has unread bytes");
    assert!(!peer.stream_closed(first));
    peer.event(Event::Close);
    peer.read_all(first);
    assert_eq!(peer.events(first).last(), Some(&end(1, w::WatchEnd::Closed)));
    for (label, stream) in [("first", first), ("second", second)] {
        assert!(peer.stream_closed(stream), "{label}");
        assert_eq!(peer.socket_closes(peer.stream_socket(stream)), 1, "{label}");
    }
    assert_eq!(peer.closed(), 1);
}
#[test]
fn abort_drops_a_stream_without_waiting_for_its_last_bytes() {
    let (mut peer, stream, watcher) = watched();
    peer.stop_reading(stream);
    peer.release_delivery(watcher, fixtures::me_document(7));
    let bytes = peer.stream_bytes(stream).len();
    peer.event(Event::Close);
    assert_eq!(peer.closed(), 0);
    peer.event(Event::Abort);
    assert_eq!(peer.closed(), 1);
    assert_eq!(peer.stream_bytes(stream).len(), bytes);
    assert!(peer.stream_closed(stream));
    assert_eq!(peer.socket_closes(peer.stream_socket(stream)), 1);
}
#[test]
fn close_keeps_an_exchange_until_its_released_answer_is_written() {
    let mut peer = signed();
    let socket = peer.authenticated_post(b"/jig/v1/request", keyed());
    let Some(core::Event::People(people::Event::Ask { reply_to, .. })) = peer.take_core() else { panic!("ask") };
    peer.event(Event::Close);
    assert_eq!(peer.closed(), 0, "waiting for the released answer");
    assert_eq!(peer.answered(socket), None);
    assert_eq!(peer.socket_closes(socket), 0);
    peer.answer_request(reply_to, people::Reply::Again(people::Again::Busy));
    assert_eq!(answer(&peer, socket), w::KeyedAnswer::Busy);
    assert_eq!(peer.socket_closes(socket), 1);
    assert_eq!(peer.closed(), 1);
}
#[test]
fn after_stop_a_sign_in_and_a_sign_out_are_answered_not_ready() {
    let mut peer = signed();
    let spent = peer.printed_link();
    peer.post_link(&spent);
    let link = peer.printed_link();
    let link_socket = peer.accepted_link(&link);
    let service_socket = peer.accepted_post(b"/jig/v1/sign-in/deployment", Box::new([]));
    let out_socket = peer.accepted_post(b"/jig/v1/sign-out", Box::new([]));
    peer.event(Event::Stop);
    for (label, socket, expected) in [
        ("link", link_socket, b"\x02".as_slice()),
        ("deployment", service_socket, b"\x02"),
        ("sign-out", out_socket, b"\x01"),
    ] {
        assert_eq!(peer.answered(socket), Some(expected), "{label}: not_ready wire record");
    }
    assert!(peer.has_digest(fixtures::FIRST_SIGN_IN));
    assert_eq!(peer.digest_kept(), None);
    assert_eq!(peer.printed_link(), link, "no replacement link");
    assert!(peer.take_core().is_none());
}
#[test]
fn after_stop_a_mint_is_refused_not_ready() {
    let mut peer = Peer::started(&fixtures::settings());
    peer.event(Event::Stop);
    let exchange = peer.mint(fixtures::application_person());
    assert_eq!(peer.mint_refusal(exchange), Some(MintRefusal::NotReady));
    assert!(peer.take_core().is_none());
    assert_eq!(peer.digest_kept(), None);
}
#[test]
fn after_stop_a_pending_mint_commits_nothing() {
    let mut peer = Peer::started(&fixtures::settings());
    let exchange = peer.mint(fixtures::application_person());
    peer.event(Event::Stop);
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
    assert_eq!(peer.mint_refusal(exchange), Some(MintRefusal::NotReady));
    assert!(peer.take_core().is_none());
    assert_eq!(peer.digest_kept(), None);
}
