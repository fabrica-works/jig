use jig_channel_world::fixtures;
use skein_world::domain::Verdict;

#[test]
fn an_answer_acknowledged_twice_is_caught() {
    let mut referee = fixtures::referee();
    referee.answered(fixtures::TASK, fixtures::ATTEMPT);
    referee.acknowledged(fixtures::TASK, fixtures::ATTEMPT);
    referee.acknowledged(fixtures::TASK, fixtures::ATTEMPT);
    assert!(matches!(referee.verdict(), Verdict::Failed(_)));
}

#[test]
fn a_grant_value_planted_in_a_root_s_state_is_caught() {
    for (label, value) in [("token", b"planted-secret".as_slice()), ("account id", b"acc".as_slice())] {
        let mut referee = fixtures::referee();
        referee.roots(&format!("root {{ credential: {value:?} }}"), &[value]);
        assert!(matches!(referee.verdict(), Verdict::Failed(_)), "{label}");
    }
}

#[test]
fn a_refused_start_acknowledged_is_caught() {
    let mut referee = fixtures::referee();
    referee.refused(fixtures::TASK, fixtures::ATTEMPT);
    referee.answered(fixtures::TASK, fixtures::ATTEMPT);
    referee.acknowledged(fixtures::TASK, fixtures::ATTEMPT);
    assert!(matches!(referee.verdict(), Verdict::Failed(_)));
}

#[test]
fn an_attempt_placed_before_its_predecessors_stop_bound_is_caught() {
    use skein_lib::Time;
    for (label, at, failed) in [("before stop bound", 9, true), ("at stop bound", 10, false)] {
        let mut referee = fixtures::referee();
        referee.placed(fixtures::TASK, fixtures::ATTEMPT, Time::ZERO);
        referee.lost(fixtures::TASK, fixtures::ATTEMPT, Time::from_nanos(10));
        referee.placed(fixtures::TASK, fixtures::ATTEMPT + 1, Time::from_nanos(at));
        assert_eq!(matches!(referee.verdict(), Verdict::Failed(_)), failed, "{label}");
    }
}
