use super::*;
use fixtures::{ATTEMPT, Harness, KEY, limits, signed_in, start_chat};
use skein_lib::Secret;
const PARTY: u64 = 7;
const CREDENTIAL: [u8; 40] = [9; 40];
const NEW_CREDENTIAL: [u8; 40] = [8; 40];
const LINK_SECRET: [u8; 32] = [3; 32];
fn held() -> With {
    With::Held(Secret::new(Box::new(CREDENTIAL)))
}
#[test]
fn a_sign_in_answer_puts_its_credential_on_each_exchange() {
    let mut h = Harness::new(&limits());
    let exchange = h.sign_in(With::Link(Secret::new(Box::new(LINK_SECRET)))).expect("sign-in exchange");
    assert!(h.link.credential().is_none());
    h.answer(exchange, &signed_in(PARTY, &CREDENTIAL));
    assert!(h.has(&Request::Signing { signing: Signing::SignedIn { party: PARTY } }));
    h.send(ATTEMPT, KEY, start_chat());
    assert_eq!(h.credentials().as_slice().last(), Some(&Some(Secret::new(Box::new(CREDENTIAL)))));
}
#[test]
fn a_held_credential_is_carried_without_an_exchange() {
    let mut h = Harness::new(&limits());
    assert_eq!(h.sign_in(held()), None);
    assert!(h.out.is_empty());
    h.send(ATTEMPT, KEY, start_chat());
    assert_eq!(h.credentials().as_slice(), &[Some(Secret::new(Box::new(CREDENTIAL)))]);
}
#[test]
fn a_not_ready_sign_in_is_again_and_holds_nothing() {
    let mut h = Harness::new(&limits());
    let exchange = h.sign_in(With::Link(Secret::new(Box::new(LINK_SECRET)))).unwrap();
    h.answer(exchange, &[2]);
    assert!(h.has(&Request::Signing { signing: Signing::Again }));
    assert!(h.link.credential().is_none());
}
#[test]
fn a_refused_secret_is_refused_and_holds_nothing() {
    let mut h = Harness::new(&limits());
    let exchange = h.sign_in(With::Link(Secret::new(Box::new(LINK_SECRET)))).unwrap();
    let record = w::SignInAnswer::Refused;
    let mut writer = skein_lib::Writer::new(1);
    record.encode(&mut writer).unwrap();
    h.answer(exchange, &writer.finish());
    assert!(h.has(&Request::Signing { signing: Signing::Refused }));
    assert!(h.link.credential().is_none());
}
const LOST: &[(&str, Option<&[u8]>, bool)] =
    &[("bad record", Some(&[255]), false), ("lost", None, false), ("not ready", Some(&[1]), true)];
#[test]
fn a_lost_sign_out_keeps_the_credential() {
    for (label, body, expected) in LOST {
        let mut h = Harness::new(&limits());
        assert_eq!(h.sign_in(held()), None);
        let exchange = h.sign_out();
        match body {
            Some(body) => h.answer(exchange, body),
            None => h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out),
        }
        assert!(
            h.has(&Request::Signing { signing: if *expected { Signing::Again } else { Signing::Lost } }),
            "{label}"
        );
        assert_eq!(h.link.credential(), Some(&Secret::new(Box::new(CREDENTIAL))), "{label}");
    }
    let mut h = Harness::new(&limits());
    assert_eq!(h.sign_in(held()), None);
    let exchange = h.sign_out();
    h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out);
    assert!(h.has(&Request::Signing { signing: Signing::Lost }));
    assert_eq!(h.link.credential(), Some(&Secret::new(Box::new(CREDENTIAL))));
}
#[test]
fn a_signed_out_answer_drops_the_credential() {
    let mut h = Harness::new(&limits());
    assert_eq!(h.sign_in(held()), None);
    let exchange = h.sign_out();
    h.answer(exchange, &[0]);
    assert!(h.has(&Request::Signing { signing: Signing::SignedOut }));
    assert!(h.link.credential().is_none());
}

#[test]
fn an_old_sign_out_cannot_end_a_new_held_credential() {
    for (label, lost) in [("signed out", false), ("lost", true)] {
        let mut h = Harness::new(&limits());
        assert_eq!(h.sign_in(held()), None);
        let exchange = h.sign_out();
        h.link.step(Time::ZERO, Event::SignIn { with: With::Held(Secret::new(Box::new(NEW_CREDENTIAL))) }, &mut h.out);
        h.out = Queue::with_capacity(32);
        if lost {
            h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out);
        } else {
            h.answer(exchange, &[0]);
        }
        assert!(h.out.is_empty(), "{label}");
        assert_eq!(h.link.credential(), Some(&Secret::new(Box::new(NEW_CREDENTIAL))), "{label}");
    }
}
