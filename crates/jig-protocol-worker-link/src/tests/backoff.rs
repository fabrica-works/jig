use super::fixtures;
use crate::backoff::Backoff;
use skein_lib::Duration;
fn backoff(seed: u64) -> Backoff {
    Backoff::new(&fixtures::LIMITS, seed)
}
#[test]
fn backoff_spans_double_to_the_ceiling_within_their_jitter() {
    let mut backoff = backoff(fixtures::SEED_ONE);
    let bounds = [(50, 100), (100, 200), (200, 400), (400, 800), (800, 1600), (800, 1600)];
    for (low, high) in bounds {
        let span = backoff.next();
        assert!(span >= Duration::from_millis(low) && span <= Duration::from_millis(high), "{span:?}");
    }
}
#[test]
fn a_proved_channel_starts_the_backoff_over() {
    let mut backoff = backoff(fixtures::SEED_ONE);
    for _ in 0_u32..6 {
        backoff.next();
    }
    backoff.proved();
    let span = backoff.next();
    assert!(span >= Duration::from_millis(50) && span <= Duration::from_millis(100), "{span:?}");
}
#[test]
fn two_seeds_draw_different_spans() {
    let mut first = backoff(fixtures::SEED_ONE);
    let mut second = backoff(2);
    assert_ne!(first.next(), second.next());
}
#[test]
fn an_oversized_credential_is_refused_before_dialling() {
    let mut credential = fixtures::credential();
    credential.name = b"a-name-beyond-sixteen-bytes".as_slice().into();
    let result = crate::Component::new(&fixtures::LIMITS, credential, 1);
    assert_eq!(result.expect_err("too large"), crate::Unusable::Credential);
}
