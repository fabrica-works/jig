use super::fixtures::{self as f, Exchange};
use crate::{Event, Request, SecretsRequest};
use jig_client_protocol_wire::v8 as w;
use jig_domain_people as p;
use skein_lib::{Reader, ReplyTo, Token};
#[test]
fn a_made_chat_is_answered_done_with_its_task() {
    let mut exchange = Exchange::new();
    let (owner, to) = exchange.ask();
    assert!(unanswered(&exchange, owner));
    exchange.reply(to, p::Reply::Outcome(p::Outcome::Said { task: 8, message: 9 }));
    assert_eq!(exchange.final_answer(owner), f::started());
    assert!(exchange.component.pending.exchanges.is_empty());
    assert!(exchange.out.is_empty());
}
#[test]
fn a_moved_hold_is_refused_with_its_facts() {
    let mut exchange = Exchange::new();
    let (owner, to) = exchange.ask();
    exchange.reply(
        to,
        p::Reply::Outcome(p::Outcome::Refused(p::Refusal::Moved {
            now: p::Moved::Hold { revision: 31, reason: p::HoldReason::Dependency { task: 9 } },
        })),
    );
    assert_eq!(exchange.final_answer(owner), f::moved_hold());
}
#[test]
fn again_busy_is_answered_busy_and_not_kept() {
    let mut exchange = Exchange::new();
    for (reply, expected) in [
        (p::Reply::Again(p::Again::Busy), w::KeyedAnswer::Busy),
        (p::Reply::Outcome(p::Outcome::Again(p::Again::NotReady)), w::KeyedAnswer::NotReady),
    ] {
        let (owner, to) = exchange.ask();
        exchange.reply(to, reply);
        assert_eq!(exchange.answer(owner), expected);
        assert!(exchange.component.pending.exchanges.is_empty());
        assert!(exchange.out.is_empty());
        exchange.component.connections.remove(&owner);
    }
}
#[test]
fn an_ended_sign_in_is_answered_signed_out() {
    let mut exchange = Exchange::new();
    let (owner, to) = exchange.ask();
    exchange.component.down(&exchange.env, Event::SignInEnded { sign_in: f::NUMBER }, &mut exchange.out);
    assert_eq!(exchange.answer(owner), w::KeyedAnswer::SignedOut);
    let Some(Request::Secrets(SecretsRequest::Erase { owner: secret_owner })) = exchange.out.pop() else {
        panic!("erase digest");
    };
    assert_eq!(secret_owner, crate::SecretOwner::SignIn(41));
    assert!(exchange.component.pending.exchanges.is_empty());
    exchange.reply(to, p::Reply::Outcome(p::Outcome::Said { task: 8, message: 9 }));
    assert_eq!(exchange.answer(owner), w::KeyedAnswer::SignedOut);
    assert!(exchange.out.is_empty());
}
#[test]
fn an_application_outcome_is_carried_under_its_family() {
    let mut exchange = Exchange::new();
    let (owner, to) = exchange.application();
    let number = to.into_token();
    exchange
        .reply(ReplyTo::new(number), p::Reply::Outcome(p::Outcome::Applied { connector: 9, token: Token::new(91) }));
    assert!(unanswered(&exchange, owner));
    answer_application(&mut exchange, ReplyTo::new(number), Box::from(b"outcome".as_slice()));
    assert_eq!(exchange.answer(owner), f::application_answer());
    assert!(exchange.component.pending.exchanges.is_empty());
    assert!(exchange.out.is_empty());
}
#[test]
fn a_lost_connection_drops_the_answer_and_nothing_else() {
    let mut exchange = Exchange::new();
    let (owner, to) = exchange.ask();
    let (other_owner, other_to) = exchange.ask();
    exchange.component.down(&exchange.env, Event::Io(skein_io::Event::Closed { owner }), &mut exchange.out);
    exchange.reply(to, p::Reply::Outcome(p::Outcome::Said { task: 8, message: 9 }));
    assert_eq!(exchange.component.pending.exchanges.len(), 1);
    assert!(exchange.out.is_empty());
    exchange.reply(other_to, p::Reply::Outcome(p::Outcome::Said { task: 8, message: 9 }));
    assert_eq!(exchange.final_answer(other_owner), f::started());
    assert!(exchange.component.pending.exchanges.is_empty());
}
#[test]
fn owner_outcomes_keep_every_supplied_fact() {
    let limits = f::wire();
    for cases in [f::owner_outcomes().as_slice(), f::decision_outcomes().as_slice(), f::seen_outcomes().as_slice()] {
        for (reply, expected) in cases {
            assert_eq!(f::translated(reply, &limits), *expected);
        }
    }
}

#[test]
fn a_service_result_contains_its_party_and_no_credential() {
    let answer =
        crate::translate::answers::answer(&p::Reply::Outcome(p::Outcome::ServiceMade { person: 7 }), &f::wire())
            .expect("service answer");
    let mut out = skein_lib::Writer::new(25);
    answer.encode(&mut out).expect("25-byte party-only answer");
    assert_eq!(out.finish().as_ref(), &f::SERVICE_ANSWER);
}
fn unanswered(exchange: &Exchange, owner: Token) -> bool {
    exchange.component.connections.get(&owner).expect("owner").response.is_empty()
}

#[test]
fn refusal_facts_keep_every_sparse_entrance_category() {
    let limits = f::wire();
    for cases in [f::refusals().as_slice(), f::extra_refusals().as_slice()] {
        for (source, expected) in cases {
            assert_eq!(crate::translate::refusals::refusal(*source, &limits).expect("refusal fits"), *expected);
        }
    }
}
#[test]
fn moved_holds_keep_every_cause_and_revision() {
    let limits = f::wire();
    for (source, expected) in f::holds() {
        let source = p::Refusal::Moved { now: p::Moved::Hold { revision: 31, reason: source } };
        assert_eq!(crate::translate::refusals::refusal(source, &limits).expect("hold fits"), f::hold_refusal(expected));
    }
}
#[test]
fn ended_refusals_keep_one_sparse_row_and_its_wall_time() {
    let limits = f::wire();
    for (source, expected) in f::endings() {
        let actual =
            crate::translate::refusals::refusal(p::Refusal::Ended { how: source }, &limits).expect("ending fits");
        assert_eq!(actual, expected);
        if source == p::Ending::Done {
            done_ending_round_trips(&limits);
        }
    }
}
fn done_ending_round_trips(limits: &w::Limits) {
    let refusal =
        crate::translate::refusals::refusal(p::Refusal::Ended { how: p::Ending::Done }, limits).expect("ending fits");
    let value = w::Final::Refused(w::Refused::new(limits, w::RefusedParts { at: None, refusal }).expect("refusal"));
    let mut out = skein_lib::Writer::new(usize::try_from(value.measure()).expect("short final"));
    value.encode(&mut out).expect("final answer");
    let owned = w::Owned::new(
        limits,
        w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: out.finish() },
    )
    .expect("owned final");
    assert_eq!(
        jig_client_protocol_link::translate::answers::answer(w::KeyedAnswer::FinalAnswer(owned), &f::client_limits()),
        Ok(jig_client_domain::contract::Answer::Refused {
            refusal: jig_client_domain::contract::Refusal::Ended { how: jig_client_domain::contract::Ending::Done },
            at: None
        })
    );
}
#[test]
fn an_answer_past_its_bound_is_lost_without_a_partial_response() {
    let mut exchange = Exchange::new();
    for (bound, expected) in f::response_bounds() {
        let (owner, to) = exchange.application();
        exchange.env.limits.wire.owned_record = bound;
        answer_application(&mut exchange, to, Box::new([3; 257]));
        assert!(exchange.component.pending.exchanges.is_empty());
        match expected {
            Some(expected) => assert_eq!(exchange.answer(owner), expected),
            None => {
                assert!(unanswered(&exchange, owner));
                let Some(Request::Io(skein_io::Request::Abort { entity })) = exchange.out.pop() else {
                    panic!("lost answer");
                };
                assert_eq!(entity, owner);
            }
        }
        assert!(exchange.out.is_empty());
        exchange.component.connections.remove(&owner);
    }
}
fn answer_application(exchange: &mut Exchange, to: ReplyTo, record: Box<[u8]>) {
    exchange.component.down(
        &exchange.env,
        Event::Application(crate::FromApplication::Answered { to, family: 9, record }),
        &mut exchange.out,
    );
}

#[test]
fn admission_refusals_keep_the_category_without_a_guessed_bound() {
    let limits = f::wire();
    for cases in [f::admission_0().as_slice(), f::admission_1().as_slice()] {
        for (source, expected) in cases {
            assert_eq!(
                crate::translate::refusals::refusal(p::Refusal::Limit { which: *source }, &limits).expect("limit fits"),
                f::admission_refusal(expected.clone())
            );
        }
    }
}
#[test]
fn authority_refusals_keep_the_known_facts_and_optional_proposal_verdict() {
    let limits = f::wire();
    let cases = f::lacks();
    for (proposable, boolean) in
        [(None, None), (Some(p::Accepter::Nobody), Some(false)), (Some(p::Accepter::Role(0)), Some(true))]
    {
        for (source, expected) in &cases {
            let source = p::Refusal::Authority { lacking: *source, proposable };
            let expected = w::Refusal::Authority(
                w::RefusalAuthority::new(
                    &limits,
                    w::RefusalAuthorityParts { lacking: expected.clone(), proposable: boolean },
                )
                .expect("authority"),
            );
            assert_eq!(crate::translate::refusals::refusal(source, &limits).expect("authority fits"), expected);
        }
    }
}
#[test]
fn an_absent_sign_in_is_answered_signed_out_without_a_saved_answer() {
    let mut exchange = Exchange::new();
    for reply in [p::Reply::SignedOut, p::Reply::Outcome(p::Outcome::SignedOut)] {
        let (owner, to) = exchange.ask();
        exchange.reply(to, reply);
        assert_eq!(exchange.answer(owner), w::KeyedAnswer::SignedOut);
        assert!(exchange.component.pending.exchanges.is_empty());
        assert!(exchange.out.is_empty());
        exchange.component.connections.remove(&owner);
    }
}
#[test]
fn a_sparse_ending_past_its_piece_bound_is_refused() {
    let limits = w::Limits { ending_rows: 5, ..f::wire() };
    assert_eq!(
        crate::translate::refusals::refusal(p::Refusal::Ended { how: p::Ending::Done }, &limits),
        Err(w::Problem { path: w::Path::EndingRows, reason: skein_codec::Reason::Bound })
    );
}

const UNSERVED_VERSION: u32 = jig_client_protocol_wire::VERSION + 1;
const REFUSED_WATCH: u64 = 71;
#[test]
fn a_keyed_request_outside_its_owners_range_is_answered_not_served() {
    let mut exchange = Exchange::new();
    let owner = exchange.post(f::keyed(w::Owner::Jig, UNSERVED_VERSION, Box::new([255])));
    exchange.authenticate();
    assert_eq!(exchange.answer(owner), w::KeyedAnswer::NotServed(f::not_served(w::Owner::Jig)));
    assert!(exchange.out.is_empty());
}
#[test]
fn a_read_outside_its_owners_range_names_the_served_ranges() {
    let mut exchange = Exchange::new();
    let owner = exchange.post_to(b"/jig/v1/read", f::owned_read(w::Owner::Jig, UNSERVED_VERSION, Box::new([255])));
    exchange.authenticate();
    assert_eq!(exchange.read_answer(owner), w::ReadAnswer::NotServed(f::not_served(w::Owner::Jig)));
    assert!(exchange.out.is_empty());
}
#[test]
fn an_opening_outside_its_owners_range_is_answered_not_served() {
    let mut exchange = Exchange::new();
    let stream = exchange.open_stream();
    let subject = w::Owned::new(
        &f::wire(),
        w::OwnedParts { owner: w::Owner::Jig, version: UNSERVED_VERSION, record: Box::new([255]) },
    )
    .expect("opaque subject");
    let owner = exchange.open_record(stream, REFUSED_WATCH, subject);
    assert_eq!(
        w::OpenAnswer::decode(&f::wire(), &mut Reader::new(exchange.response(owner).1)),
        Ok(w::OpenAnswer::NotServed(f::not_served(w::Owner::Jig)))
    );
    assert!(exchange.out.is_empty());
}
