//! The starter alone may finish a waiting sign-in (accounts.md, section 5).
use jig_accounts_world::{LIMITS, World, provider, providers};
use jig_domain_accounts::{
    Answer, Change, End, Event, Failure, Holder, Refusal, Request, SignInRedirectWhy as Why, Signing,
};
use skein_lib::Token;

fn world(sign_ins: u32) -> World {
    let mut world = World::with_limits(
        jig_domain_accounts::Limits { sign_ins, ..LIMITS },
        providers(vec![provider(1, true, false), provider(2, true, false)]),
        &[],
    );
    world.step(Event::Want);
    world
}
fn start(request: u64, by: u64, provider: u32) -> Event {
    Event::SignIn { request: Token::new(request), holder: Holder::Party(by), by, provider }
}
fn visit(source: u32) -> Event {
    Event::Visited { source, address: b"https://issuer.test/visit".as_slice().into() }
}
fn finish(request: u64, source: u32, by: u64) -> Event {
    Event::Change {
        request: Token::new(request),
        holder: Holder::Party(by),
        change: Change::FinishSignIn { source, carried: 90, by },
    }
}
fn refused(request: u64, why: Why) -> Vec<Request> {
    vec![Request::Answer {
        request: Token::new(request),
        answer: Answer::Refused { refusal: Refusal::SignInRedirect { why } },
    }]
}
fn redirected(source: u32) -> Vec<Request> {
    vec![Request::Signing(Signing::Redirected { source, carried: 90 })]
}

#[test]
fn a_starters_pasted_address_is_handed_to_its_sign_in_and_answered_finishing() {
    let mut w = world(1);
    w.step(start(1, 7, 1));
    w.step(visit(0));
    assert_eq!(w.step(finish(2, 0, 7)), redirected(0));
    assert_eq!(
        w.step(Event::Finishing { source: 0 }),
        vec![Request::Answer { request: Token::new(2), answer: Answer::Finishing { source: 0 } }]
    );
    assert!(w.ended().is_empty());
    assert_eq!(w.step(finish(3, 0, 7)), refused(3, Why::NotWaiting));
}
#[test]
fn anothers_source_a_missing_source_an_unvisited_and_an_ended_sign_in_answer_the_same() {
    let mut w = world(1);
    w.step(start(1, 7, 1));
    let before = w.step(finish(2, 0, 7));
    w.step(visit(0));
    let another = w.step(finish(2, 0, 8));
    let missing = w.step(finish(2, 99, 7));
    w.step(Event::SignInFailed { source: 0, failure: Failure::Cancelled });
    let ended = w.step(finish(2, 0, 7));
    assert_eq!(before, refused(2, Why::NotWaiting));
    assert_eq!(another, before);
    assert_eq!(missing, before);
    assert_eq!(ended, before);
}
#[test]
fn a_second_finish_waits_for_the_first_and_a_refused_finish_allows_another() {
    let mut w = world(1);
    w.step(start(1, 7, 1));
    w.step(visit(0));
    assert_eq!(w.step(finish(2, 0, 7)), redirected(0));
    assert_eq!(w.step(finish(3, 0, 7)), refused(3, Why::NotWaiting));
    assert_eq!(w.step(Event::FinishRefused { source: 0, why: Why::State }), refused(2, Why::State));
    assert!(w.ended().is_empty());
    assert_eq!(w.step(finish(4, 0, 7)), redirected(0));
}
#[test]
fn an_ending_sign_in_answers_its_pending_finish_not_waiting_first() {
    for terminal in [
        Event::SignInFailed { source: 0, failure: Failure::Cancelled },
        Event::SignInFailed { source: 0, failure: Failure::Misconfigured },
        Event::SignedIn { source: 0, subject: b"subject".as_slice().into(), display: Box::new([]), declined: false },
        Event::SignedIn { source: 0, subject: Box::new([]), display: Box::new([]), declined: false },
        Event::Close,
    ] {
        let mut w = world(1);
        w.step(start(1, 7, 1));
        w.step(visit(0));
        w.step(finish(2, 0, 7));
        let out = w.step(terminal);
        assert_eq!(out.first(), refused(2, Why::NotWaiting).first());
        assert!(w.step(Event::Finishing { source: 0 }).is_empty());
        assert!(w.step(Event::FinishRefused { source: 0, why: Why::State }).is_empty());
    }
}
#[test]
fn starting_again_cancels_ones_waiting_sign_in_and_reuses_its_room() {
    let mut w = world(1);
    w.step(start(1, 7, 1));
    w.step(visit(0));
    w.step(finish(2, 0, 7));
    let out = w.step(start(3, 7, 1));
    assert_eq!(out[0], Request::Signing(Signing::CancelSignIn { source: 0 }));
    assert_eq!(&out[1..2], refused(2, Why::NotWaiting));
    assert_eq!(out.last(), Some(&Request::Signing(Signing::SignIn { source: 1, provider: 1 })));
    assert!(
        matches!(w.ended().as_slice(), [end] if end.source == 0 && end.end == End::Failed { failure: Failure::Cancelled })
    );
    assert!(w.step(Event::SignInFailed { source: 0, failure: Failure::Cancelled }).is_empty());
    assert_eq!(
        w.step(visit(1)),
        vec![Request::Answer {
            request: Token::new(3),
            answer: Answer::Started { source: 1, address: b"https://issuer.test/visit".as_slice().into() }
        }]
    );
}
#[test]
fn starting_again_leaves_another_party_another_provider_and_a_taken_sign_in_alone() {
    let mut w = world(4);
    for (request, by, provider) in [(1, 7, 1), (2, 8, 1), (3, 7, 2)] {
        w.step(start(request, by, provider));
    }
    for source in 0..3 {
        w.step(visit(source));
    }
    let out = w.step(start(4, 7, 1));
    assert_eq!(out.first(), Some(&Request::Signing(Signing::CancelSignIn { source: 0 })));
    assert_eq!(w.ended().len(), 1);
    assert_eq!(w.step(finish(5, 1, 8)), redirected(1));
    assert_eq!(w.step(finish(6, 2, 7)), redirected(2));
    w.step(Event::Finishing { source: 1 });
    let out = w.step(start(7, 8, 1));
    assert!(!out.iter().any(|request| matches!(request, Request::Signing(Signing::CancelSignIn { .. }))));
    assert!(w.ended().is_empty());
}
