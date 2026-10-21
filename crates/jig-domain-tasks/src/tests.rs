use crate::{Retry, failures::backoff};
use skein_lib::{Duration, Rng};

#[test]
fn exponential_equal_jitter_saturates_without_overflow_and_replays() {
    let retry = Retry { retries: 4, base: Duration::from_millis(10), max: Duration::from_secs(1) };
    let mut first = Rng::new(42);
    let mut second = Rng::new(42);
    for times in [1, 2, 3, 4, 7, 32, 64, u32::MAX] {
        let factor = 1_u64.checked_shl(times.saturating_sub(1)).unwrap_or(u64::MAX);
        let ceiling = retry.base.saturating_mul(factor).min(retry.max);
        let pause = backoff(times, retry, &mut first);
        assert!(
            pause.as_nanos() >= ceiling.as_nanos().div_euclid(2) && pause <= ceiling,
            "pause stays in equal-jitter half-to-ceiling range"
        );
        assert_eq!(pause, backoff(times, retry, &mut second), "same seed replays backoff");
    }
}

/// One configured charter, number 1, with the one configured model, which declares no effort.
pub(crate) fn charters() -> Box<[crate::CharterModels]> {
    Box::new([crate::CharterModels { number: 1, models: Box::new([0]) }])
}

mod hold_origin;
mod keys;
mod refusals;
mod revisions;
mod waiting;

mod periods;

mod last_batch;

mod changed;

mod address;
mod instructions;

mod deadlines;

mod index;

mod payer;

mod account;

mod waits;

#[test]
fn shared_kept_rule_separates_claims_from_finish_authority() {
    use crate::{Contract, End, EndedBy, Executor, TaskResult};
    let done = End::Finished { result: TaskResult::Report { words: Box::new([]) }, cancel_delegates: false };
    let failed = End::Finished { result: TaskResult::Failure { reason: Box::new([]) }, cancel_delegates: false };
    let malformed = End::Finished {
        result: TaskResult::Verdict { fields: Box::new([]), code: 0, words: Box::new([]) },
        cancel_delegates: false,
    };
    let contract = Contract::Report { words: 1 };
    for (dependencies, ended_by, blocks, parks_done, parks_failure, parks_invalid) in [
        (&[][..], EndedBy::Result, false, false, false, false),
        (&[2][..], EndedBy::Result, true, true, false, false),
        (&[][..], EndedBy::Requester, false, true, true, true),
        (&[2][..], EndedBy::Requester, true, true, true, true),
    ] {
        let kept = crate::run::ends::pending(dependencies, ended_by);
        assert_eq!(kept.blocks_claims(), blocks);
        assert_eq!(kept.may_be_due(Executor::Agent { charter: 1 }), !blocks);
        assert_eq!(kept.parks(&done, &contract, &revisions::LIMITS), parks_done);
        assert_eq!(kept.parks(&failed, &contract, &revisions::LIMITS), parks_failure);
        assert_eq!(kept.parks(&malformed, &contract, &revisions::LIMITS), parks_invalid);
    }
}
