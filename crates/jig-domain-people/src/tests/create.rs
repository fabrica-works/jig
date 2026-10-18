use super::*;
const SIGN_IN: u64 = 10;
const PARTY: u64 = 1;
#[test]
fn a_members_create_routes_once_to_the_core() {
    let mut test = Test::new(LIMITS);
    test.member();
    let ask = Ask::Create { project: 1, batch: Box::new([fixtures::member(b"one")]) };
    let rows = test.request(SIGN_IN, 1, ask.clone());
    let request = route(&rows);
    assert_eq!(rows.len(), 1);
    assert!(test.request(SIGN_IN, 1, ask.clone()).is_empty());
    test.send(Event::Decided { request, outcome: Outcome::Created { first: PARTY, last: PARTY, given: None } });
    assert_eq!(
        reply(&test.request(SIGN_IN, 1, ask)),
        Reply::Outcome(Outcome::Created { first: PARTY, last: PARTY, given: None })
    );
}
#[test]
fn an_observers_create_is_refused_by_role() {
    let mut test = Test::new(LIMITS);
    test.member();
    test.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: PARTY, role: Role::Observer }]) });
    let rows = test.request(SIGN_IN, 1, Ask::Create { project: 1, batch: Box::new([fixtures::member(b"one")]) });
    let request = route(&rows);
    let outcome = Outcome::Refused(Refusal::Role { held: Some(Role::Observer) });
    assert_eq!(reply(&test.send(Event::Decided { request, outcome })), Reply::Outcome(outcome));
}
#[test]
fn an_empty_create_is_refused_as_empty() {
    let mut test = Test::new(LIMITS);
    test.member();
    assert_eq!(
        reply(&test.request(SIGN_IN, 1, Ask::Create { project: 1, batch: Box::new([]) })),
        Reply::Refused(Refusal::Limit { which: LimitKind::Empty })
    );
}
#[test]
fn a_create_past_its_batch_limit_is_refused() {
    let mut test = Test::new(LIMITS);
    test.member();
    let batch = Box::new([fixtures::member(b"one"), fixtures::member(b"two"), fixtures::member(b"three")]);
    assert_eq!(
        reply(&test.request(SIGN_IN, 1, Ask::Create { project: 1, batch })),
        Reply::Refused(Refusal::Limit { which: LimitKind::Batch })
    );
}
#[test]
fn a_create_whose_words_pass_their_bound_is_refused() {
    let mut test = Test::new(LIMITS);
    test.member();
    assert_eq!(
        reply(&test.request(
            SIGN_IN,
            1,
            Ask::Create { project: 1, batch: Box::new([fixtures::member(b"too many words")]) }
        )),
        Reply::Refused(Refusal::Limit { which: LimitKind::Words })
    );
}
