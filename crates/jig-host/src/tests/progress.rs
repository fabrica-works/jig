//! Live progress through the owner boundary and borrowed public observations.
use super::*;

#[test]
fn slot_wait_and_resumption_emit_bounded_observations_without_duplicate_transitions() {
    let mut h = Harness::new();
    let (owner, _, _) = h.live(1);
    while h.domain.pop_fact().is_some() {}
    let run = Token::new(1);
    let attempt = Token::new(1001);
    h.step(Event::Yielded { owner });
    assert_eq!(h.domain.pop_fact(), Some(crate::Fact::Waiting { run, attempt }));
    h.step(Event::Yielded { owner });
    assert_eq!(h.domain.pop_fact(), None);
    h.step(fixtures::inbound(run, attempt, Token::new(42), Box::from(&b"words"[..])));
    assert_eq!(h.domain.pop_fact(), Some(crate::Fact::Resumed { run, attempt }));
    h.step(fixtures::inbound(run, attempt, Token::new(43), Box::from(&b"more"[..])));
    assert_eq!(h.domain.pop_fact(), None);
    h.step(Event::Yielded { owner });
    assert_eq!(h.domain.pop_fact(), Some(crate::Fact::Waiting { run, attempt }));
    h.step(fixtures::called(owner, Token::new(44), fixtures::deliver(Box::from(&b"work"[..]))));
    assert_eq!(h.domain.pop_fact(), Some(crate::Fact::Resumed { run, attempt }));
    assert_eq!(h.domain.facts_lost(), 0);
    assert_eq!(h.domain.hosted(), 1, "waiting retains the admitted slot");
}
