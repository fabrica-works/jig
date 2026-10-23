use jig_client_protocol_wire::v16 as w;
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
fn a_held_credential_answered_with_another_sign_in_draws_a_token() {
    const NEW_SIGN_IN: u64 = fixtures::FIRST_SIGN_IN + 1;
    // A credential carries an eight-byte sign-in number and 32 drawn bytes (client.md, 8).
    const TOKEN_BYTES: usize = 40;
    let mut peer = peer();
    let bearer = fixtures::signed_in(&mut peer);
    let link = fresh_link(&mut peer);
    let exchange = peer.post(b"/jig/v1/sign-in/link", peer.link_body(&link), Some(&bearer));
    let Some(jig_domain_core::Event::SignIn { reply_to, held, .. }) = peer.take_core() else { panic!("sign-in") };
    assert_eq!(held, Some(fixtures::FIRST_SIGN_IN));
    peer.answer_request(
        reply_to,
        jig_domain_people::Reply::SignedIn {
            person: fixtures::FIRST_PERSON,
            sign_in: NEW_SIGN_IN,
            expires: fixtures::expires(),
        },
    );
    assert_eq!(peer.digest_kept(), Some(NEW_SIGN_IN));
    assert_eq!(peer.answered(exchange), None, "waits for digest durability");
    peer.answer_kept(NEW_SIGN_IN);
    let answer = peer.sign_in_answer(exchange);
    assert_eq!(answer.party(), fixtures::FIRST_PERSON);
    let token = answer.token().as_ref().expect("fresh token").expose();
    assert_eq!(token.len(), TOKEN_BYTES);
    assert_eq!(u64::from_be_bytes(token[..8].try_into().expect("sign-in number")), NEW_SIGN_IN);
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

fn retry_answers() -> [jig_domain_people::Reply; 3] {
    use jig_domain_people::{Again, Reply};
    [
        Reply::Again(Again::NotReady),
        Reply::Again(Again::Busy),
        Reply::Full { sign_ins: 2, earliest: skein_lib::Wall::from_nanos(900_000_000_000) },
    ]
}
fn pending_reply(peer: &mut Peer) -> skein_lib::ReplyTo {
    let Some(jig_domain_core::Event::SignIn { reply_to, .. }) = peer.take_core() else { panic!("sign-in") };
    reply_to
}
fn decoded(peer: &Peer, exchange: skein_lib::Token) -> w::SignInAnswer {
    let bytes = peer.answered(exchange).expect("answer");
    let mut reader = skein_lib::Reader::new(bytes);
    let answer = w::SignInAnswer::decode(&w::CEILINGS, &mut reader).expect("sign-in answer");
    assert!(reader.is_empty());
    answer
}

#[test]
fn busy_and_full_return_the_same_printed_link_and_it_can_sign_in() {
    for reply in retry_answers() {
        let mut peer = peer();
        let link = peer.printed_link();
        let exchange = peer.post_link(&link);
        let to = pending_reply(&mut peer);
        peer.advance_time(skein_lib::Duration::from_secs(1));
        let expected = match reply {
            jig_domain_people::Reply::Again(jig_domain_people::Again::NotReady) => w::SignInAnswer::NotReady,
            jig_domain_people::Reply::Again(jig_domain_people::Again::Busy) => w::SignInAnswer::Busy,
            jig_domain_people::Reply::Full { sign_ins, earliest } => w::SignInAnswer::Full(
                w::SignInsFull::new(
                    &w::CEILINGS,
                    w::SignInsFullParts { sign_ins, earliest: earliest.as_nanos(), link_until: 299_000_000_000 },
                )
                .expect("full answer within wire bounds"),
            ),
            jig_domain_people::Reply::SignedIn { .. }
            | jig_domain_people::Reply::SignedOut
            | jig_domain_people::Reply::Outcome(_)
            | jig_domain_people::Reply::Refused(_) => {
                unreachable!("retry fixtures contain only not ready, busy and full")
            }
        };
        peer.answer_request(to, reply);
        assert_eq!(decoded(&peer, exchange), expected);
        assert_eq!(peer.printed_link(), link);
        // One nanosecond before the original expiry remains usable.
        peer.advance_time(skein_lib::Duration::from_nanos(299_000_000_000 - 1));
        let exchange = peer.post_link(&link);
        peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
        peer.answer_kept(fixtures::FIRST_SIGN_IN);
        assert_eq!(peer.sign_in_answer(exchange).party(), fixtures::FIRST_PERSON);
        assert_eq!(peer.printed_link(), link);
    }
}

#[test]
fn a_returned_link_past_its_original_expiry_is_refused_and_replaced() {
    for reply in retry_answers() {
        let mut peer = peer();
        let link = peer.printed_link();
        peer.post_link(&link);
        let to = pending_reply(&mut peer);
        peer.advance_time(skein_lib::Duration::from_secs(1));
        peer.answer_request(to, reply);
        peer.advance_time(skein_lib::Duration::from_secs(299));
        let exchange = peer.post_link(&link);
        assert_eq!(decoded(&peer, exchange), w::SignInAnswer::Refused);
        assert_ne!(peer.printed_link(), link);
        assert!(peer.take_core().is_none());
    }
}

#[test]
fn a_reserved_link_answers_its_secret_busy_and_another_refused_without_drawing() {
    let mut peer = peer();
    let link = peer.printed_link();
    let original = peer.post_link(&link);
    let to = pending_reply(&mut peer);
    let crossed = peer.post_link(&link);
    assert_eq!(decoded(&peer, crossed), w::SignInAnswer::Busy);
    let other = w::LinkSignIn::new(&w::CEILINGS, w::LinkSignInParts { secret: Box::new([0; 32]) })
        .expect("real-length link secret");
    let mut bytes = skein_lib::Writer::new(usize::try_from(other.measure()).expect("short link record"));
    other.encode(&mut bytes).expect("measured link record");
    let wrong = peer.post(b"/jig/v1/sign-in/link", bytes.finish(), None);
    assert_eq!(decoded(&peer, wrong), w::SignInAnswer::Refused);
    assert_eq!(peer.printed_link(), link);
    assert!(peer.take_core().is_none());
    peer.answer_request(to, jig_domain_people::Reply::Again(jig_domain_people::Again::Busy));
    assert_eq!(decoded(&peer, original), w::SignInAnswer::Busy);
    let exchange = peer.post_link(&link);
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    assert_eq!(peer.sign_in_answer(exchange).party(), fixtures::FIRST_PERSON);
}

#[test]
fn a_core_refusal_drops_the_reservation_and_the_next_attempt_prints_a_link() {
    let mut peer = peer();
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    peer.refuse_sign_in();
    assert_eq!(decoded(&peer, exchange), w::SignInAnswer::Refused);
    assert_ne!(fresh_link(&mut peer), link);
    complete_sign_in(&mut peer);
}

#[test]
fn admission_full_answers_busy_and_leaves_the_link_open() {
    let mut limits = fixtures::limits();
    limits.exchanges = 1;
    let mut peer = Peer::with_limits(&fixtures::settings(), limits);
    let mint = peer.mint(fixtures::application_person());
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    assert_eq!(decoded(&peer, exchange), w::SignInAnswer::Busy);
    assert_eq!(peer.printed_link(), link);
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN + 1, fixtures::FIRST_PERSON);
    peer.answer_kept(fixtures::FIRST_SIGN_IN + 1);
    assert!(peer.minted(mint).is_some());
    complete_sign_in(&mut peer);
    assert_eq!(peer.printed_link(), link);
}

#[test]
fn a_mint_the_people_child_answers_full_is_refused_busy() {
    let mut peer = peer();
    let exchange = peer.mint(fixtures::application_person());
    let to = pending_reply(&mut peer);
    peer.answer_request(to, retry_answers()[2].clone());
    assert_eq!(peer.mint_refusal(exchange), Some(MintRefusal::Busy));
    assert_eq!(peer.digest_kept(), None);
}

#[test]
fn a_services_sign_in_is_busy_for_busy_or_full() {
    use jig_protocol_engine_clients::SecretOwner;
    for reply in [retry_answers()[1].clone(), retry_answers()[2].clone()] {
        let mut peer = peer();
        let token = [7; 40];
        let number = u64::from_be_bytes(token[..8].try_into().expect("eight-byte service number"));
        peer.keep_digest(SecretOwner::Service(number), &token, fixtures::expires());
        let exchange = peer.post(b"/jig/v1/sign-in/deployment", Box::new([0]), Some(&bearer(&token)));
        let to = pending_reply(&mut peer);
        peer.answer_request(to, reply);
        assert_eq!(decoded(&peer, exchange), w::SignInAnswer::Busy);
        assert_eq!(peer.digest_kept(), None);
    }
}
