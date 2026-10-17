//! Exact owner lifecycle beside separately supplied host observations.
//! See client/domain/objects.md, 3, and view.md, 7 and 10.
use crate::JigBindings;
use crate::phrases::jig::JigPhrase;
use crate::words::Words;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Run};

use jig_client::contract::{ActiveRun, HeldRuntime, RetryNotice, RunStage, RunWork};

pub(crate) fn active(b: &mut Builder, value: &ActiveRun, words: &Words) {
    match value.stage {
        RunStage::Idle => words.text(b, JigPhrase::RunIdle, &[]),
        RunStage::Due => words.text(b, JigPhrase::RunDue, &[]),
        RunStage::Preparing => words.text(b, JigPhrase::RunPreparing, &[]),
        RunStage::Claimed { attempt } => words.text(b, JigPhrase::RunClaimed, &[Hole::Number(attempt)]),
        RunStage::Running { attempt } => words.text(b, JigPhrase::RunOwnerActive, &[Hole::Number(attempt)]),
        RunStage::BackingOff { until } => {
            words.text(b, JigPhrase::RunBackingOff, &[Hole::Wall(words.adjusted_wall(until))]);
        }
    }
    words.text(b, JigPhrase::RunPhaseSince, &[Hole::Wall(words.adjusted_wall(value.phase_since))]);
    if let Some(attempt) = value.attempt {
        words.text(b, JigPhrase::Attempt, &[Hole::Number(attempt)]);
    }
    if let Some(at) = value.placement {
        words.text(b, JigPhrase::RunPlacement, &[Hole::Wall(words.adjusted_wall(at))]);
    }
    match value.retry {
        Some(RetryNotice::Failed { class, count, allowance }) => {
            crate::cards::chip::failure(b, class, words);
            words.text(
                b,
                JigPhrase::RunRetryFailed,
                &[Hole::Number(u64::from(count)), Hole::Number(u64::from(allowance))],
            );
        }
        Some(RetryNotice::OwnerFailed { class, count, allowance }) => {
            crate::owner_chip::hold_failure(b, class, words);
            words.text(
                b,
                JigPhrase::RunRetryFailed,
                &[Hole::Number(u64::from(count)), Hole::Number(u64::from(allowance))],
            );
        }
        Some(RetryNotice::Refused { count }) => {
            words.text(b, JigPhrase::RunRetryRefused, &[Hole::Number(u64::from(count))]);
        }
        None => {}
    }
    let Some(observation) = &value.observation else {
        words.text(b, JigPhrase::RunUnobserved, &[]);
        return;
    };
    match &observation.work {
        RunWork::Preparing => words.text(b, JigPhrase::WorkPreparing, &[]),
        RunWork::Starting => words.text(b, JigPhrase::WorkStarting, &[]),
        RunWork::Active => words.text(b, JigPhrase::WorkActive, &[]),
        RunWork::Writing => words.text(b, JigPhrase::WorkWriting, &[]),
        RunWork::Tool { call, tool } => {
            let phrase = crate::live_call_words::LiveCallPhrase::ObservedCurrentCall;
            let label = words.book.say(words.language, phrase.id(), &[]).expect("checked observed call label").text;
            b.text(Run::Plain, &label);
            crate::literal_segment::render(b, tool.bytes());
            crate::literal_segment::render(b, call.id.bytes());
        }
        RunWork::Waiting => words.text(b, JigPhrase::WorkWaiting, &[]),
        RunWork::Ending => words.text(b, JigPhrase::WorkEnding, &[]),
    }
    words.text(b, JigPhrase::RunObservedSince, &[Hole::Wall(words.adjusted_wall(observation.since))]);
    words.text(b, JigPhrase::RunReportedAt, &[Hole::Wall(words.adjusted_wall(observation.reported_at))]);
    match observation.started {
        Some(at) => words.text(b, JigPhrase::RunActualStart, &[Hole::Wall(words.adjusted_wall(at))]),
        None => words.text(b, JigPhrase::RunStartUnknown, &[]),
    }
}

pub(crate) fn prior(b: &mut Builder, value: &HeldRuntime, words: &Words, bindings: &mut JigBindings) {
    words.text(b, JigPhrase::HeldPriorLifecycle, &[]);
    match value {
        HeldRuntime::Waiting => words.text(b, JigPhrase::HeldPriorWaiting, &[]),
        HeldRuntime::Active(value) => active(b, value, words),
        HeldRuntime::Closing(value) => crate::cards::common::closing(b, value, words, bindings),
    }
}
