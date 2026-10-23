use jig_client_protocol_wire::v13 as w;
use jig_engine_clients_world::fixtures::{self, Peer};
use jig_protocol_engine_clients::{Event, MintRefusal};
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
fn a_link_answered_not_ready_is_not_spent() {
    let mut peer = peer();
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    let Some(jig_domain_core::Event::SignIn { reply_to, .. }) = peer.take_core() else { panic!("sign-in") };
    peer.answer_request(reply_to, jig_domain_people::Reply::Again(jig_domain_people::Again::NotReady));
    assert_eq!(peer.answered(exchange), Some(b"\x02".as_slice()), "not_ready wire record");
    let answer = complete_sign_in(&mut peer);
    assert_eq!(answer.party(), fixtures::FIRST_PERSON);
    assert_eq!(peer.printed_link(), link, "no replacement link");
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
        u64::from_be_bytes(
            answer.token().as_ref().expect("native token").expose()[..8].try_into().expect("sign-in number")
        ),
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
    assert_eq!(peer.answered(exchange), Some(b"\x04".as_slice()));
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
        "SignIn { reply_to: ReplyTo(Token(1)), identity: Identity { key: IdentityKey { provider: 0, subject: [111, 119, 110, 101, 114] }, login: [111, 119, 110, 101, 114], name: [79, 119, 110, 101, 114] }, held: None }"
    );
}

#[test]
fn a_mint_for_an_application_provider_signs_in_as_the_link_does() {
    let mut peer = peer();
    let exchange = peer.mint(fixtures::application_person());
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
    assert!(peer.minted(exchange).is_none(), "before digest durability");
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    let (credential, expires) = peer.minted(exchange).expect("minted");
    assert_eq!(u64::from_be_bytes(credential.expose()[..8].try_into().expect("number")), fixtures::FIRST_SIGN_IN);
    assert_eq!(expires, fixtures::expires());
    assert_eq!(credential.expose().len(), 40);
}
fn assert_mint_refused(identity: jig_domain_people::Identity, why: MintRefusal) {
    let mut peer = peer();
    let exchange = peer.mint(identity);
    assert_eq!(peer.mint_refusal(exchange), Some(why));
    assert!(peer.take_core().is_none());
    assert_eq!(peer.digest_kept(), None);
}
#[test]
fn a_mint_with_a_login_that_is_not_utf8_is_refused_as_bound() {
    let identity = jig_domain_people::Identity { login: Box::new([255]), ..fixtures::application_person() };
    assert_mint_refused(identity, MintRefusal::Bound);
}
#[test]
fn a_mint_for_the_link_s_provider_is_refused_as_not_application() {
    assert_mint_refused(fixtures::first_owner(), MintRefusal::NotApplication);
}
#[test]
fn a_mint_with_a_name_past_its_bound_is_refused_as_bound() {
    let identity =
        jig_domain_people::Identity { name: vec![b'x'; 1025].into_boxed_slice(), ..fixtures::application_person() };
    assert_mint_refused(identity, MintRefusal::Bound);
}
#[test]
fn a_mint_the_people_child_answers_busy_is_refused_busy() {
    let mut peer = peer();
    let exchange = peer.mint(fixtures::application_person());
    let Some(jig_domain_core::Event::SignIn { reply_to, .. }) = peer.take_core() else { panic!("sign-in") };
    peer.event(Event::Reply { to: reply_to, reply: jig_domain_people::Reply::Again(jig_domain_people::Again::Busy) });
    assert_eq!(peer.mint_refusal(exchange), Some(MintRefusal::Busy));
    assert_eq!(peer.digest_kept(), None);
}

fn bearer(token: &[u8]) -> Box<[u8]> {
    let mut bytes = skein_lib::Writer::new(7 + (token.len() * 8).div_ceil(6));
    bytes.put(b"Bearer ").expect("prefix");
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Url, token, &mut bytes).expect("40-byte credential");
    bytes.finish()
}
fn fresh_link(peer: &mut Peer) -> Box<[u8]> {
    let spent = peer.printed_link();
    let exchange = peer.post_link(&spent);
    assert_eq!(peer.answered(exchange), Some(b"\x01".as_slice()));
    let fresh = peer.printed_link();
    assert_ne!(fresh, spent);
    fresh
}

#[test]
fn a_live_credential_keeps_the_sign_in_without_minting_and_spends_the_link() {
    let mut peer = peer();
    let first = complete_sign_in(&mut peer);
    let bearer = bearer(first.token().as_ref().expect("credential").expose());
    let link = fresh_link(&mut peer);
    let exchange = peer.post(b"/jig/v1/sign-in/link", peer.link_body(&link), Some(&bearer));
    let Some(jig_domain_core::Event::SignIn { reply_to, identity, held }) = peer.take_core() else { panic!("sign-in") };
    assert_eq!(held, Some(fixtures::FIRST_SIGN_IN));
    assert_eq!(identity, fixtures::first_owner());
    peer.answer_request(
        reply_to,
        jig_domain_people::Reply::SignedIn {
            person: fixtures::FIRST_PERSON,
            sign_in: fixtures::FIRST_SIGN_IN,
            expires: fixtures::expires(),
        },
    );
    let answer = peer.sign_in_answer(exchange);
    assert_eq!(answer.token(), &None);
    assert_eq!((answer.party(), answer.expires()), (first.party(), first.expires()));
    assert_eq!(peer.digest_kept(), None);
    assert!(peer.has_digest(fixtures::FIRST_SIGN_IN));
    assert_ne!(fresh_link(&mut peer), link);
}

#[test]
fn a_links_unaccepted_credential_always_signs_in_afresh() {
    use jig_protocol_engine_clients::SecretOwner;
    for case in ["wrong digest", "expired", "ended while checking", "service", "unknown", "malformed", "none"] {
        let mut peer = peer();
        let first = complete_sign_in(&mut peer);
        let original = first.token().as_ref().expect("40-byte credential").expose();
        let mut token = original.to_vec();
        match case {
            "wrong digest" => token[39] ^= 1,
            "expired" => {
                peer.keep_digest(SecretOwner::SignIn(fixtures::FIRST_SIGN_IN), original, skein_lib::Wall::EPOCH);
            }
            "ended while checking" => peer.hold_gets(),
            "service" => {
                token[..8].copy_from_slice(&99_u64.to_be_bytes());
                peer.keep_digest(SecretOwner::Service(99), &token, fixtures::expires());
            }
            "unknown" => token[..8].copy_from_slice(&98_u64.to_be_bytes()),
            "malformed" => token.truncate(39),
            "none" => {}
            _ => unreachable!(),
        }
        let link = fresh_link(&mut peer);
        let bearer = bearer(&token);
        let exchange = peer.post(
            b"/jig/v1/sign-in/link",
            peer.link_body(&link),
            if case == "none" { None } else { Some(&bearer) },
        );
        if case == "ended while checking" {
            assert!(peer.take_core().is_none(), "Get is pending");
            peer.event(Event::SignInEnded { sign_in: fixtures::FIRST_SIGN_IN });
            peer.answer_get(); // Even the stale, accepted Get cannot resurrect the ended sign-in.
        }
        assert_eq!(peer.answered(exchange), None, "{case}: waits for people");
        peer.answer_signed_in(42, fixtures::FIRST_PERSON); // Also asserts core held: None.
        assert_eq!(peer.digest_kept(), Some(42), "{case}: new digest");
        assert_eq!(peer.answered(exchange), None, "{case}: waits for durability");
        peer.answer_kept(42);
        let answer = peer.sign_in_answer(exchange);
        assert_eq!(answer.party(), fixtures::FIRST_PERSON, "{case}");
        assert_eq!(answer.token().as_ref().expect("fresh token").len(), 40, "{case}");
        assert_ne!(answer.token(), first.token(), "{case}");
    }
}

#[test]
fn a_links_pending_credential_check_survives_stop_and_drain_as_an_admitted_sign_in() {
    for drain in [false, true] {
        let mut peer = peer();
        let first = complete_sign_in(&mut peer);
        let bearer = bearer(first.token().as_ref().expect("credential").expose());
        let link = fresh_link(&mut peer);
        peer.hold_gets();
        let exchange = peer.post(b"/jig/v1/sign-in/link", peer.link_body(&link), Some(&bearer));
        assert!(peer.take_core().is_none());
        peer.event(if drain { Event::Close } else { Event::Stop });
        assert_eq!(peer.answered(exchange), None);
        assert_eq!(peer.closed(), 0);
        peer.answer_get();
        let Some(jig_domain_core::Event::SignIn { reply_to, held: Some(fixtures::FIRST_SIGN_IN), .. }) =
            peer.take_core()
        else {
            panic!("admitted sign-in completes after stop");
        };
        peer.answer_request(
            reply_to,
            jig_domain_people::Reply::SignedIn {
                person: fixtures::FIRST_PERSON,
                sign_in: fixtures::FIRST_SIGN_IN,
                expires: fixtures::expires(),
            },
        );
        assert_eq!(peer.sign_in_answer(exchange).token(), &None);
        assert_eq!(peer.digest_kept(), None);
        assert_eq!(peer.closed(), usize::from(drain));
    }
}

#[test]
fn aborting_a_links_pending_credential_check_releases_no_sign_in_or_token() {
    let mut peer = peer();
    let first = complete_sign_in(&mut peer);
    let bearer = bearer(first.token().as_ref().expect("credential").expose());
    let link = fresh_link(&mut peer);
    peer.hold_gets();
    let exchange = peer.post(b"/jig/v1/sign-in/link", peer.link_body(&link), Some(&bearer));
    assert!(peer.take_core().is_none());
    peer.event(Event::Abort);
    assert!(peer.take_core().is_none());
    assert_eq!(peer.answered(exchange), None);
    assert_eq!(peer.digest_kept(), None);
    assert_eq!(peer.closed(), 1);
}

#[test]
fn a_lost_links_credential_check_spends_the_link_like_a_lost_core_answer() {
    let mut peer = peer();
    let first = complete_sign_in(&mut peer);
    let bearer = bearer(first.token().as_ref().expect("credential").expose());
    let link = fresh_link(&mut peer);
    peer.hold_gets();
    let exchange = peer.post(b"/jig/v1/sign-in/link", peer.link_body(&link), Some(&bearer));
    peer.cut_exchange(exchange);
    peer.answer_get();
    let Some(jig_domain_core::Event::SignIn { reply_to, held: Some(fixtures::FIRST_SIGN_IN), .. }) = peer.take_core()
    else {
        panic!("the admitted lost exchange still spends its link");
    };
    peer.answer_request(
        reply_to,
        jig_domain_people::Reply::SignedIn {
            person: fixtures::FIRST_PERSON,
            sign_in: fixtures::FIRST_SIGN_IN,
            expires: fixtures::expires(),
        },
    );
    assert_eq!(peer.answered(exchange), None);
    assert_eq!(peer.digest_kept(), None);
    assert_ne!(fresh_link(&mut peer), link);
}
