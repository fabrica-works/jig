use super::*;
use fixtures::{ATTEMPT, Harness, KEY, limits, message, signed_in};
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
    let exchange =
        h.sign_in(With::Link { secret: Secret::new(Box::new(LINK_SECRET)), held: None }).expect("sign-in exchange");
    assert!(h.link.credential().is_none());
    h.answer(exchange, &signed_in(PARTY, &CREDENTIAL));
    assert!(h.has(&Request::Signing { signing: Signing::SignedIn { party: PARTY } }));
    h.send(ATTEMPT, KEY, message());
    assert_eq!(h.credentials().as_slice().last(), Some(&Some(Secret::new(Box::new(CREDENTIAL)))));
}
#[test]
fn a_held_credential_is_carried_without_an_exchange() {
    let mut h = Harness::new(&limits());
    assert_eq!(h.sign_in(held()), None);
    assert!(h.out.is_empty());
    h.send(ATTEMPT, KEY, message());
    assert_eq!(h.credentials().as_slice(), &[Some(Secret::new(Box::new(CREDENTIAL)))]);
}
#[test]
fn a_not_ready_sign_in_is_again_and_holds_nothing() {
    let mut h = Harness::new(&limits());
    let exchange = h.sign_in(With::Link { secret: Secret::new(Box::new(LINK_SECRET)), held: None }).unwrap();
    h.answer(exchange, &[2]);
    assert!(h.has(&Request::Signing { signing: Signing::Again }));
    assert!(h.link.credential().is_none());
}
#[test]
fn a_refused_secret_is_refused_and_holds_nothing() {
    let mut h = Harness::new(&limits());
    let exchange = h.sign_in(With::Link { secret: Secret::new(Box::new(LINK_SECRET)), held: None }).unwrap();
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

#[test]
fn a_sign_in_answer_with_no_token_leaves_none_held() {
    let mut h = Harness::new(&limits());
    assert_eq!(h.sign_in(held()), None);
    let generation = h.link.generation;
    let exchange =
        h.sign_in(With::Link { secret: Secret::new(Box::new(LINK_SECRET)), held: None }).expect("sign-in exchange");
    let answer = w::SignInAnswer::SignedIn(
        w::SignedIn::new(&limits().wire, w::SignedInParts { token: None, party: PARTY, expires: 10 })
            .expect("transport credential"),
    );
    let mut bytes = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("short sign-in"));
    answer.encode(&mut bytes).expect("measured sign-in");
    h.answer(exchange, &bytes.finish());
    assert!(h.link.credential().is_none());
    assert_eq!(h.link.generation, generation.checked_add(1).expect("next generation"));
    assert!(h.has(&Request::Signing { signing: Signing::SignedIn { party: PARTY } }));
}

#[test]
fn a_link_carries_the_attempts_held_credential_and_keeps_it_on_a_tokenless_answer() {
    let mut h = Harness::new(&limits());
    h.sign_in(With::Held(Secret::new(Box::new(NEW_CREDENTIAL))));
    let exchange = h
        .sign_in(With::Link {
            secret: Secret::new(Box::new(LINK_SECRET)),
            held: Some(Secret::new(Box::new(CREDENTIAL))),
        })
        .expect("link exchange");
    assert_eq!(h.credentials().as_slice().last(), Some(&Some(Secret::new(Box::new(CREDENTIAL)))));
    let answer = w::SignInAnswer::SignedIn(
        w::SignedIn::new(&limits().wire, w::SignedInParts { token: None, party: PARTY, expires: 10 })
            .expect("kept sign-in"),
    );
    let mut bytes = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("answer size"));
    answer.encode(&mut bytes).expect("answer");
    h.answer(exchange, &bytes.finish());
    assert_eq!(h.link.credential(), Some(&Secret::new(Box::new(CREDENTIAL))));
    assert!(h.has(&Request::Signing { signing: Signing::SignedIn { party: PARTY } }));
    h.send(ATTEMPT, KEY, message());
    assert_eq!(h.credentials().as_slice().last(), Some(&Some(Secret::new(Box::new(CREDENTIAL)))));
}

#[test]
fn a_link_answer_with_a_new_token_replaces_the_attempts_held_credential() {
    let mut h = Harness::new(&limits());
    let exchange = h
        .sign_in(With::Link {
            secret: Secret::new(Box::new(LINK_SECRET)),
            held: Some(Secret::new(Box::new(CREDENTIAL))),
        })
        .expect("link exchange");
    h.answer(exchange, &signed_in(PARTY, &NEW_CREDENTIAL));
    assert_eq!(h.link.credential(), Some(&Secret::new(Box::new(NEW_CREDENTIAL))));
}

#[test]
fn a_refused_not_ready_or_lost_link_installs_none_of_its_held_credential() {
    for (body, expected) in
        [(Some(&[1][..]), Signing::Refused), (Some(&[2][..]), Signing::Again), (None, Signing::Lost)]
    {
        let mut h = Harness::new(&limits());
        let exchange = h
            .sign_in(With::Link {
                secret: Secret::new(Box::new(LINK_SECRET)),
                held: Some(Secret::new(Box::new(CREDENTIAL))),
            })
            .expect("link exchange");
        match body {
            Some(body) => h.answer(exchange, body),
            None => h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out),
        }
        assert!(h.has(&Request::Signing { signing: expected }));
        assert_eq!(h.link.credential(), None);
    }
}

#[test]
fn busy_and_full_sign_in_answers_decode_and_trailing_bytes_are_lost() {
    let full = w::SignInAnswer::Full(
        w::SignInsFull::new(&w::CEILINGS, w::SignInsFullParts { sign_ins: 2, earliest: 400, link_until: 300 }).unwrap(),
    );
    for (record, expected) in [
        (w::SignInAnswer::Busy, Signing::Busy),
        (
            full,
            Signing::Full {
                sign_ins: 2,
                earliest: skein_lib::Wall::from_nanos(400),
                link_until: skein_lib::Wall::from_nanos(300),
            },
        ),
    ] {
        for trailing in [false, true] {
            let mut h = Harness::new(&limits());
            let exchange = h
                .sign_in(With::Link {
                    secret: Secret::new(Box::new(LINK_SECRET)),
                    held: Some(Secret::new(Box::new(CREDENTIAL))),
                })
                .unwrap();
            let length = usize::try_from(record.measure()).unwrap().checked_add(usize::from(trailing)).unwrap();
            let mut bytes = skein_lib::Writer::new(length);
            record.encode(&mut bytes).unwrap();
            if trailing {
                bytes.put(&[0]).unwrap();
            }
            h.answer(exchange, &bytes.finish());
            let mut told: u32 = 0;
            for request in &h.out {
                if let Request::Signing { signing } = request {
                    assert_eq!(signing, if trailing { &Signing::Lost } else { &expected });
                    told += 1;
                }
            }
            assert_eq!(told, 1);
            assert!(h.link.credential().is_none());
        }
    }
}
