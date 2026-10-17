use super::*;
const CASES: [(&str, bool, bool); 4] = [
    ("beyond offer", false, false),
    ("within offer", true, false),
    ("wrong answer beyond", false, true),
    ("wrong answer within", true, true),
];
#[test]
fn an_explicit_amendment_proposal_is_answered_with_its_proposal_number() {
    for (name, bounded, changed) in CASES {
        let (setup, ask, answer) = super::fixtures::proposed_amendment(bounded, changed);
        let mut engine = Engine::new(setup, 1).unwrap();
        let result = engine.decide(1, &ask, Wall::EPOCH);
        assert_eq!(result, if changed { Err(Invalid::Script) } else { Ok((answer, vec![])) }, "{name}");
    }
}
