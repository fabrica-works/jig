use jig_channel_world::stories;
use skein_world::domain::Verdict;

#[test]
fn drawn_channel_worlds_keep_the_referee_s_rules() {
    for seed in 1..=12 {
        let outcome = stories::drawn(seed);
        assert_eq!(outcome.verdict, Verdict::Passed, "drawn seed {seed}");
        assert_eq!(outcome.answers_acknowledged, 1, "drawn seed {seed}");
    }
}
