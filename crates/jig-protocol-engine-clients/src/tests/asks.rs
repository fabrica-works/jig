use super::fixtures::{self as f, Exchange};
use crate::{
    Request, ToApplication,
    translate::{Untranslatable, asks::ask},
};
use jig_client_protocol_wire::v8 as w;
use jig_domain_people as p;
#[test]
fn a_start_chat_record_becomes_the_core_s_start_chat() {
    let limits = f::wire();
    assert_eq!(
        ask(f::start(&limits), &limits),
        Ok(p::Ask::StartChat { project: 3, words: Box::from(b"hello".as_slice()) })
    );
}
#[test]
fn a_decision_record_keeps_its_revision() {
    let limits = f::wire();
    let cases = [
        (w::Decision::PassedUp, p::EscalationDecision::Pass),
        (
            w::Decision::Released(
                w::DecisionReleased::new(&limits, w::DecisionReleasedParts { again: None }).expect("release"),
            ),
            p::EscalationDecision::Release,
        ),
    ];
    for (decision, expected) in cases {
        let value = w::Ask::DecideEscalation(
            w::DecideEscalation::new(
                &limits,
                w::DecideEscalationParts { project: 3, task: 8, revision: f::revision(u64::MAX), decision },
            )
            .expect("decision"),
        );
        assert_eq!(
            ask(value, &limits),
            Ok(p::Ask::DecideEscalation { project: 3, task: 8, revision: u64::MAX, decision: expected })
        );
    }
}
#[test]
fn an_amendment_record_keeps_its_release_and_reason() {
    for (release, expected_release, reason) in
        [(None, None, b"".as_slice()), (Some(f::revision(4)), Some(4), b"supplied".as_slice())]
    {
        assert_eq!(
            ask(f::amendment(release.clone(), reason), &f::wire()),
            Ok(p::Ask::Amend {
                project: 3,
                task: 8,
                revision: 6,
                release: expected_release,
                amendment: p::Amendment {
                    addressed: None,
                    instructions: Some(Box::new([])),
                    spec: None,
                    wake: None,
                    dependencies: Some(Box::new([])),
                    authority: None,
                    reason: Box::from(reason)
                }
            })
        );
    }
}
fn invalid() -> w::Ask {
    let limits = f::wire();
    w::Ask::DecideEscalation(
        w::DecideEscalation::new(
            &limits,
            w::DecideEscalationParts { project: 3, task: 8, revision: f::revision(6), decision: w::Decision::Accepted },
        )
        .expect("valid record with inapplicable decision"),
    )
}
#[test]
fn an_ask_that_does_not_translate_reaches_no_core() {
    let value = invalid();
    assert_eq!(ask(value.clone(), &f::wire()), Err(Untranslatable { path: w::Path::DecideEscalationDecision }));
    let mut exchange = Exchange::new();
    for bytes in [f::encode_ask(&value), f::encode_ask(&f::malformed_piece()), Box::new([255]), {
        let bytes = f::encode_ask(&f::start(&f::wire()));
        let mut out = skein_lib::Writer::new(bytes.len().checked_add(1).expect("tail size"));
        out.put(&bytes).expect("ask");
        out.put(&[0]).expect("tail");
        out.finish()
    }] {
        let owner = exchange.post(f::keyed(w::Owner::Jig, 8, bytes));
        exchange.authenticate();
        assert!(exchange.out.is_empty());
        assert_eq!(exchange.response(owner), (400, b"".as_slice()));
    }
}
#[test]
fn an_application_record_is_routed_with_its_digest() {
    let mut exchange = Exchange::new();
    let family = w::Owner::Family(w::Family::new(&f::wire(), w::FamilyParts { number: 9 }).expect("family"));
    exchange.post(f::keyed(family, 2, Box::from(b"abc".as_slice())));
    exchange.authenticate();
    let Some(Request::Application(ToApplication::Request { sign_in, key, family, version, record, digest, .. })) =
        exchange.out.pop()
    else {
        panic!("application request");
    };
    assert_eq!((sign_in, key, family, version, record.as_ref()), (41, [5; 16], 9, 2, b"abc".as_slice()));
    assert_eq!(
        digest,
        [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03,
            0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad
        ]
    );
}
#[test]
fn a_third_exchange_past_the_client_s_bound_is_busy() {
    let mut exchange = Exchange::new();
    let body = f::keyed(w::Owner::Jig, 8, f::encode_ask(&f::start(&f::wire())));
    for _ in 0..2_u32 {
        exchange.post(body.clone());
        exchange.authenticate();
        let Some(Request::Core(_)) = exchange.out.pop() else {
            panic!("admitted ask");
        };
    }
    let owner = exchange.post(body);
    exchange.authenticate();
    assert!(exchange.out.is_empty());
    assert_eq!(exchange.response(owner), (200, b"\x01".as_slice()));
}

#[test]
fn every_other_ask_family_keeps_its_supplied_fields() {
    let limits = f::wire();
    let cases = f::other_asks();
    for (wire, expected) in cases {
        assert_eq!(ask(wire, &limits), Ok(expected));
    }
}

#[test]
fn policy_authority_keeps_every_length_prefixed_grant_and_resource() {
    let limits = f::wire();
    let (wire, expected) = f::authority_ask();
    assert_eq!(ask(wire.clone(), &limits), Ok(expected));
}
#[test]
fn a_piece_past_its_decoded_sum_is_refused() {
    let limits = f::wire();
    let (wire, _) = f::authority_ask();
    let small = w::Limits { owner_authority_grants: 19, ..limits };
    assert_eq!(ask(wire, &small), Err(Untranslatable { path: w::Path::OwnerAuthorityGrants }));
}

#[test]
fn an_undecodable_application_record_is_answered_bad_request() {
    let mut exchange = Exchange::new();
    let family = w::Owner::Family(w::Family::new(&f::wire(), w::FamilyParts { number: 9 }).expect("family"));
    let owner = exchange.post(f::keyed(family, 2, Box::new([255])));
    exchange.authenticate();
    let Some(Request::Application(ToApplication::Request { reply_to, .. })) = exchange.out.pop() else {
        panic!("application request");
    };
    exchange.component.down(
        &exchange.env,
        crate::Event::Application(crate::FromApplication::Undecodable { to: reply_to }),
        &mut exchange.out,
    );
    assert_eq!(exchange.response(owner), (400, b"".as_slice()));
    assert!(exchange.out.is_empty());
}
