use super::*;
#[test]
fn a_send_becomes_one_post_with_its_key() {
    let (mut h, _) = fixtures::Harness::sending();
    assert_eq!(h.out.len(), 1);
    let Request::Exchange { message: Message::Request(body), .. } = h.out.pop().unwrap() else {
        panic!("expected exchange")
    };
    let record = w::Keyed::decode(&fixtures::limits().wire, &mut Reader::new(&body)).unwrap();
    assert_eq!(record.key(), &fixtures::KEY);
    assert_eq!(record.owner(), &w::Owner::Jig);
    assert_eq!(record.version(), 10);
}
#[test]
fn done_is_handed_up_final_with_its_outcome() {
    let (mut h, exchange) = fixtures::Harness::sending();
    h.answer(exchange, &fixtures::answer_done_chat(31));
    assert_eq!(
        h.answered(fixtures::ATTEMPT),
        Some((
            RequestEnvelope::Final,
            Some(&Answered::Jig(c::Answer::Done(c::Outcome::Made {
                what: c::Made::Tasks { tasks: fixtures::list([31]) }
            })))
        ))
    );
}
#[test]
fn the_client_domain_never_holds_a_service_credential() {
    let cases = [
        (fixtures::answer_done_service(&fixtures::SERVICE_CREDENTIAL), true),
        (fixtures::answer_done_service(&[]), false),
    ];
    for (body, shown) in cases {
        let (mut h, exchange) = fixtures::Harness::sending();
        h.answer(exchange, &body);
        assert_eq!(
            h.answered(fixtures::ATTEMPT),
            Some((
                RequestEnvelope::Final,
                Some(&Answered::Jig(c::Answer::Done(c::Outcome::Made { what: c::Made::Service { party: 7 } })))
            ))
        );
        assert_eq!(
            h.has(&Request::ServiceCredential {
                party: 7,
                credential: skein_lib::Secret::new(Box::new(fixtures::SERVICE_CREDENTIAL))
            }),
            shown
        );
        let count = if shown { 3 } else { 2 };
        assert_eq!(h.out.len(), count);
        h.answer(exchange, &body);
        assert_eq!(h.out.len(), count);
    }
}
#[test]
fn busy_is_handed_up_as_again_with_no_answer() {
    let (mut h, exchange) = fixtures::Harness::sending();
    h.answer(exchange, &fixtures::answer_busy());
    assert_eq!(h.answered(fixtures::ATTEMPT), Some((RequestEnvelope::Again, None)));
}
#[test]
fn signed_out_is_handed_up_as_signed_out() {
    let (mut h, exchange) = fixtures::Harness::sending();
    h.answer(exchange, &fixtures::answer_signed_out());
    assert_eq!(h.answered(fixtures::ATTEMPT), Some((RequestEnvelope::SignedOut, None)));
}
#[test]
fn a_failed_exchange_is_handed_up_lost() {
    let (mut h, exchange) = fixtures::Harness::sending();
    h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out);
    h.lost();
}
#[test]
fn a_final_answer_that_does_not_decode_is_lost_and_reported() {
    let (mut h, exchange) = fixtures::Harness::sending();
    h.answer(exchange, &fixtures::malformed_final());
    h.lost();
    assert!(h.has(&Request::Fact { fact: Fact::UndecodableAnswer }));
}
#[test]
fn a_late_answer_cannot_answer_an_abandoned_attempt_twice() {
    let (mut h, exchange) = fixtures::Harness::sending();
    h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out);
    h.answer(exchange, &fixtures::answer_done_chat(31));
    assert_eq!(h.terminals(), 1);
    h.lost();
}
#[test]
fn malformed_outer_bodies_are_lost() {
    let (mut h, exchange) = fixtures::Harness::sending();
    h.answer(exchange, &[255]);
    h.lost();
}

#[test]
fn a_final_refusal_keeps_its_revision_and_whole_hold_facts() {
    let (mut h, exchange) = fixtures::Harness::sending();
    h.answer(exchange, &fixtures::refusal_with_hold(fixtures::fact_bytes(&fixtures::hold_budget_fact())));
    let expected = c::Answer::Refused {
        refusal: c::Refusal::Moved {
            now: c::Revision { value: u64::MAX },
            what: c::Moved::Hold {
                reason: Some(c::CurrentHold::Document(Box::new(c::HoldReason::Budget { spent: u64::MAX, limit: 16 }))),
            },
        },
        at: None,
    };
    assert_eq!(h.answered(fixtures::ATTEMPT), Some((RequestEnvelope::Final, Some(&Answered::Jig(expected)))));
}
#[test]
fn malformed_piece_lengths_in_a_final_refusal_are_lost_and_reported() {
    for rows in [Box::from([0, 0, 0].as_slice()), Box::from([0, 0, 0, 3, 1].as_slice())] {
        let (mut h, exchange) = fixtures::Harness::sending();
        h.answer(exchange, &fixtures::refusal_with_hold(rows));
        h.lost();
        assert!(h.has(&Request::Fact { fact: Fact::UndecodableAnswer }));
    }
}
#[test]
fn an_invalid_client_limit_literal_is_refused() {
    let mut invalid = fixtures::limits();
    invalid.wire.owned_record = 16_777_217;
    assert_eq!(Link::new(&invalid).expect_err("invalid link limits"), Unusable::Limits);
}

#[test]
fn a_not_served_answer_holds_its_request_for_its_owner() {
    let (mut h, exchange) = fixtures::Harness::sending_family();
    h.answer(exchange, &fixtures::encoded_answer(&w::KeyedAnswer::NotServed(fixtures::not_served())));
    assert!(h.has(&Request::Hello { family: Some(19), versions: Some(c::VersionRange { first: 24, last: 24 }) }));
    assert_eq!(h.answered(fixtures::ATTEMPT), Some((RequestEnvelope::Again, None)));
}

#[test]
fn a_jig_refusal_of_a_family_request_is_handed_up_final() {
    let (mut h, exchange) = fixtures::Harness::sending_family();
    h.answer(exchange, &fixtures::answer_key_conflict());
    assert_eq!(
        h.answered(fixtures::ATTEMPT),
        Some((
            RequestEnvelope::Final,
            Some(&Answered::Jig(c::Answer::Refused { refusal: c::Refusal::KeyConflict, at: None }))
        ))
    );
    assert!(!h.has(&Request::Fact { fact: Fact::UndecodableAnswer }));
}

#[test]
fn a_jig_done_for_a_family_request_is_lost_and_reported() {
    let (mut h, exchange) = fixtures::Harness::sending_family();
    h.answer(exchange, &fixtures::answer_done_chat(31));
    h.lost();
    assert!(h.has(&Request::Fact { fact: Fact::UndecodableAnswer }));
}

#[test]
fn a_family_answer_reaches_its_owner_as_bytes_only_for_its_own_owner_and_version() {
    let (mut h, exchange) = fixtures::Harness::sending_family();
    h.answer(exchange, &fixtures::answer_family(19, 23));
    assert_eq!(
        h.answered(fixtures::ATTEMPT),
        Some((RequestEnvelope::Final, Some(&Answered::Family(fixtures::family_read()))))
    );
    for (family, version) in [(19, 24), (20, 23)] {
        let (mut h, exchange) = fixtures::Harness::sending_family();
        h.answer(exchange, &fixtures::answer_family(family, version));
        h.lost();
        assert!(h.has(&Request::Fact { fact: Fact::UndecodableAnswer }));
    }
}

#[test]
fn a_family_answer_to_a_jig_request_is_lost_and_reported() {
    let (mut h, exchange) = fixtures::Harness::sending();
    h.answer(exchange, &fixtures::answer_family(19, 23));
    h.lost();
    assert!(h.has(&Request::Fact { fact: Fact::UndecodableAnswer }));
}

#[test]
fn a_seen_over_a_conversation_names_its_task() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.send(
        fixtures::ATTEMPT,
        fixtures::KEY,
        c::Ask::Seen { through: c::SeenThrough::Conversation { task: u64::MAX, through: u64::MAX } },
    );
    let Request::Exchange { message: Message::Request(bytes), .. } = harness.out.pop().expect("exchange") else {
        panic!("request bytes")
    };
    let keyed = w::Keyed::decode(&fixtures::limits().wire, &mut Reader::new(&bytes)).expect("keyed request");
    let request = w::Ask::decode(&fixtures::limits().wire, &mut Reader::new(keyed.request())).expect("jig request");
    let w::Ask::Seen(seen) = request else { panic!("seen") };
    let w::SeenThrough::Conversation(conversation) = seen.through() else { panic!("conversation") };
    assert_eq!((conversation.task(), conversation.through()), (u64::MAX, u64::MAX));
}

const TEMPLATE_ROWS: [(u32, u32); 3] = [(2, 2), (16, 16), (32, 16)];
#[test]
fn template_overrides_keep_the_wire_ceiling_when_the_client_accepts_more_rows() {
    for (rows, expected) in TEMPLATE_ROWS {
        let mut client = fixtures::limits().client;
        client.rows = rows;
        let limits = match Limits::new(client, 4) {
            Ok(value) => value,
            Err(error) => panic!("client rows {rows}: {error:?}"),
        };
        assert_eq!(limits.wire.policy_edit_templates_overrides, expected, "client rows {rows}");
    }
}
