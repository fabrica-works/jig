use jig_client_protocol_wire::v7 as w;
use jig_engine_clients_world::fixtures::{self, Peer};
use jig_protocol_engine_clients::{Component, Unusable};
use skein_lib::{Env, Queue, Time, Wall};
fn peer() -> Peer {
    Peer::started(&fixtures::settings())
}
fn pending_sign_in(peer: &mut Peer) -> skein_lib::Token {
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
    exchange
}
fn complete_sign_in(peer: &mut Peer) -> w::SignedIn {
    let exchange = pending_sign_in(peer);
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    peer.sign_in_answer(exchange)
}
#[test]
fn the_printed_link_signs_the_first_owner_in() {
    let mut peer = peer();
    let answer = complete_sign_in(&mut peer);
    assert_eq!(answer.party(), fixtures::FIRST_PERSON);
    assert_eq!(answer.expires(), fixtures::expires().as_nanos());
}
#[test]
fn a_spent_link_is_refused_and_a_fresh_one_printed() {
    let mut peer = peer();
    let link = peer.printed_link();
    complete_sign_in(&mut peer);
    let exchange = peer.post_link(&link);
    assert_eq!(peer.answered(exchange), Some(b"\x01".as_slice()));
    assert_ne!(peer.printed_link(), link);
}
#[test]
fn the_token_leaves_only_after_its_digest_is_kept() {
    let mut peer = peer();
    let exchange = pending_sign_in(&mut peer);
    assert_eq!(peer.answered(exchange), None);
    assert_eq!(peer.digest_kept(), Some(fixtures::FIRST_SIGN_IN));
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    let answer = peer.sign_in_answer(exchange);
    assert_eq!(answer.party(), fixtures::FIRST_PERSON);
    assert_eq!(
        u64::from_be_bytes(answer.token().expose()[..8].try_into().expect("sign-in number")),
        fixtures::FIRST_SIGN_IN
    );
}
#[test]
fn a_failed_keep_hands_no_token_out() {
    let mut peer = peer();
    let exchange = pending_sign_in(&mut peer);
    peer.fail_keep();
    assert_eq!(peer.answered(exchange), None);
}
#[test]
fn an_unknown_bearer_is_answered_signed_out() {
    let mut peer = peer();
    let exchange = peer.post(
        b"/jig/v1/request",
        Box::new([0]),
        Some(b"Bearer AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
    );
    assert_eq!(peer.answered(exchange), Some(b"\x03".as_slice()));
}
#[test]
fn two_tokens_drawn_in_turn_differ() {
    let mut peer = peer();
    let first = complete_sign_in(&mut peer);
    let spent = peer.printed_link();
    peer.post_link(&spent);
    let second = complete_sign_in(&mut peer);
    assert_ne!(first.token(), second.token());
}
#[test]
fn a_link_on_a_non_loopback_listener_refuses_the_start() {
    let mut settings = fixtures::settings();
    settings.listener = skein_io::Addr::Ip("192.0.2.1:8080".parse().expect("address"));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fixtures::limits() };
    assert!(matches!(Component::new(settings, &env, &mut Queue::with_capacity(8)), Err(Unusable::Listener)));
}
#[test]
fn a_sign_in_refused_above_the_fill_mark_is_answered_refused() {
    let mut peer = peer();
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    peer.refuse_sign_in();
    assert_eq!(peer.answered(exchange), Some(b"\x01".as_slice()));
}
#[test]
fn no_core_event_holds_a_token_or_digest() {
    let mut peer = peer();
    let link = peer.printed_link();
    peer.post_link(&link);
    assert_eq!(
        peer.core_record(),
        "SignIn { reply_to: ReplyTo(Token(2)), identity: Identity { key: IdentityKey { provider: 0, subject: [111, 119, 110, 101, 114] }, login: [111, 119, 110, 101, 114], name: [79, 119, 110, 101, 114] } }"
    );
}
