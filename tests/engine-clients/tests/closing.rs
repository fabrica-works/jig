use jig_client_protocol_wire::v8 as w;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_engine_clients_world::fixtures::{self, Peer};
use jig_protocol_engine_clients::{Event, Settings, Verdict};
use skein_lib::{List, Reader, Token, Writer};
fn signed() -> Peer {
    Peer::signed_in(&fixtures::settings(), fixtures::FIRST_SIGN_IN)
}
fn watched() -> (Peer, [u8; 16], Token) {
    let mut peer = signed();
    let stream = peer.open_stream();
    let watcher = peer.open_watch(stream, &fixtures::frame_subject());
    (peer, stream, watcher)
}
fn provider() -> Peer {
    let mut providers = List::with_capacity(1);
    providers.push(9).expect("one provider");
    Peer::started(&Settings { providers, ..fixtures::settings() })
}
fn provider_identity() -> people::Identity {
    people::Identity {
        key: people::IdentityKey { provider: 9, subject: Box::from(b"person".as_slice()) },
        login: Box::from(b"person".as_slice()),
        name: Box::from(b"Person".as_slice()),
    }
}
fn provider_exchange(peer: &mut Peer) -> (Token, Token) {
    let socket = peer.post(b"/jig/v1/sign-in/provider/9", Box::from(b"proof".as_slice()), None);
    let (exchange, provider, call) = peer.take_provider().expect("provider call");
    assert_eq!(provider, 9);
    assert_eq!(call.target.as_ref(), b"/jig/v1/sign-in/provider/9");
    assert_eq!(call.body.as_ref(), b"proof");
    (socket, exchange)
}
fn sign_out(peer: &mut Peer) -> (Token, skein_lib::ReplyTo) {
    let socket = peer.authenticated_post(b"/jig/v1/sign-out", Box::new([]));
    let Some(core::Event::People(people::Event::SignOut { reply_to, sign_in })) = peer.take_core() else {
        panic!("sign-out reaches people");
    };
    assert_eq!(sign_in, fixtures::FIRST_SIGN_IN);
    (socket, reply_to)
}
fn answer(peer: &Peer, socket: Token) -> w::Answer {
    w::Answer::decode(&fixtures::limits().wire, &mut Reader::new(peer.answered(socket).expect("answer")))
        .expect("wire answer")
}
fn sign_in_answer(peer: &Peer, socket: Token) -> w::SignInAnswer {
    w::SignInAnswer::decode(&fixtures::limits().wire, &mut Reader::new(peer.answered(socket).expect("answer")))
        .expect("wire sign-in answer")
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
    let ask = w::Ask::StartChat(
        w::StartChat::new(&lim, w::StartChatParts { project: 3, words: Box::from(b"hello".as_slice()) }).expect("ask"),
    );
    let mut bytes = Writer::new(usize::try_from(ask.measure()).expect("ask size"));
    ask.encode(&mut bytes).expect("ask bytes");
    let record =
        w::Keyed::new(&lim, w::KeyedParts { key: [5; 16], owner: w::Owner::Jig, version: 8, request: bytes.finish() })
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
    assert_eq!(answer(&peer, retry), w::Answer::SignedOut);
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
fn a_provider_identity_signs_in_as_the_link_does() {
    let mut peer = provider();
    let (socket, exchange) = provider_exchange(&mut peer);
    let identity = provider_identity();
    peer.event(Event::Provider { exchange, verdict: Verdict::Identity(identity.clone()) });
    let Some(core::Event::SignIn { reply_to, identity: found }) = peer.take_core() else { panic!("sign-in") };
    assert_eq!(found, identity);
    peer.answer_request(
        reply_to,
        people::Reply::SignedIn {
            person: fixtures::FIRST_PERSON,
            sign_in: fixtures::FIRST_SIGN_IN,
            expires: fixtures::expires(),
        },
    );
    assert_eq!(peer.answered(socket), None, "before digest durability");
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    let signed = peer.sign_in_answer(socket);
    assert_eq!(signed.party(), fixtures::FIRST_PERSON);
    assert_eq!(signed.expires(), fixtures::expires().as_nanos());
    assert_eq!(signed.token().expose().len(), 40);
}
#[test]
fn a_provider_refusal_signs_nobody_in() {
    let mut peer = provider();
    let (socket, exchange) = provider_exchange(&mut peer);
    peer.event(Event::Provider { exchange, verdict: Verdict::Refused });
    assert_eq!(sign_in_answer(&peer, socket), w::SignInAnswer::Refused);
    assert!(peer.take_core().is_none());
    assert_eq!(peer.digest_kept(), None);
}
#[test]
fn an_unconfigured_provider_is_not_routed() {
    let mut peer = provider();
    for (label, path) in [
        ("unknown", b"/jig/v1/sign-in/provider/8".as_slice()),
        ("overflow", b"/jig/v1/sign-in/provider/65536"),
        ("non-number", b"/jig/v1/sign-in/provider/no"),
        ("empty", b"/jig/v1/sign-in/provider/"),
    ] {
        let socket = peer.post(path, Box::new([]), None);
        assert_eq!(peer.status(socket), Some(404), "{label}");
        assert!(peer.take_provider().is_none(), "{label}");
        assert!(peer.take_core().is_none(), "{label}");
    }
}
#[test]
fn after_stop_a_new_exchange_is_answered_not_ready() {
    let mut peer = signed();
    let socket = peer.accepted_post(b"/jig/v1/request", keyed());
    peer.event(Event::Stop);
    assert_eq!(answer(&peer, socket), w::Answer::NotReady);
    assert!(peer.take_core().is_none());
    assert_eq!(peer.socket_closes(Token::new(99)), 1);
    assert_eq!(peer.closed(), 0);
}
#[test]
fn after_stop_a_read_is_served() {
    let mut peer = signed();
    let lim = fixtures::limits().wire;
    let value = w::Read::Frame;
    let mut bytes = Writer::new(usize::try_from(value.measure()).expect("read size"));
    value.encode(&mut bytes).expect("read bytes");
    let record = w::Owned::new(&lim, w::OwnedParts { owner: w::Owner::Jig, version: 8, record: bytes.finish() })
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
    let owned = w::Owned::new(&lim, w::OwnedParts { owner: w::Owner::Jig, version: 8, record: Box::new([0]) })
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
    peer.release_delivery(watcher, fixtures::frame_document(7));
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
    peer.release_delivery(watcher, fixtures::frame_document(7));
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
    assert_eq!(answer(&peer, socket), w::Answer::Busy);
    assert_eq!(peer.socket_closes(socket), 1);
    assert_eq!(peer.closed(), 1);
}
#[test]
fn a_close_waits_for_its_listener_to_open() {
    use jig_protocol_engine_clients::{Component, Request};
    use skein_lib::{Env, Queue, Time, Wall};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fixtures::limits() };
    let mut out = Queue::with_capacity(8);
    let mut component = Component::new(Settings { link: None, ..fixtures::settings() }, &env, &mut out).expect("start");
    assert!(matches!(out.pop(), Some(Request::Io(skein_io::Request::Listen { .. }))));
    component.down(&env, Event::Close, &mut out);
    assert!(!component.up(&env, &mut out), "the pending listener has not closed");
    assert!(out.is_empty());
    component.down(
        &env,
        Event::Io(skein_io::Event::Listening {
            owner: Token::new(0),
            listener: Token::new(99),
            addr: fixtures::settings().listener,
        }),
        &mut out,
    );
    assert!(matches!(out.pop(), Some(Request::Io(skein_io::Request::Close { entity })) if entity == Token::new(99)));
    assert!(!component.up(&env, &mut out), "waiting for io's close terminal");
    component.down(&env, Event::Io(skein_io::Event::Closed { owner: Token::new(0) }), &mut out);
    assert!(component.up(&env, &mut out));
    assert!(matches!(out.pop(), Some(Request::Closed)));
    assert!(!component.up(&env, &mut out));
    assert!(out.is_empty());
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
fn after_stop_an_application_provider_is_answered_not_ready() {
    let mut peer = provider();
    let socket = peer.accepted_post(b"/jig/v1/sign-in/provider/9", Box::from(b"proof".as_slice()));
    peer.event(Event::Stop);
    assert_eq!(sign_in_answer(&peer, socket), w::SignInAnswer::NotReady);
    assert!(peer.take_provider().is_none());
    assert!(peer.take_core().is_none());
    assert_eq!(peer.digest_kept(), None);
}
#[test]
fn after_stop_a_pending_provider_identity_commits_nothing() {
    let mut peer = provider();
    let (socket, exchange) = provider_exchange(&mut peer);
    peer.event(Event::Stop);
    peer.event(Event::Provider { exchange, verdict: Verdict::Identity(provider_identity()) });
    assert_eq!(sign_in_answer(&peer, socket), w::SignInAnswer::NotReady);
    assert!(peer.take_core().is_none());
    assert_eq!(peer.digest_kept(), None);
}
#[test]
fn abort_upgrades_sockets_waiting_for_their_graceful_close() {
    let mut peer = signed();
    let stream = peer.open_stream();
    peer.hold_closes();
    peer.event(Event::Close);
    assert_eq!(peer.closed(), 0, "io has not settled its closes");
    assert_eq!(peer.socket_closes(peer.stream_socket(stream)), 1);
    peer.event(Event::Abort);
    assert_eq!(peer.socket_closes(peer.stream_socket(stream)), 2, "the drain is upgraded once to abort");
    assert_eq!(peer.closed(), 1);
}
