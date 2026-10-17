//! The whole vocabulary of held reasons (client/domain/objects.md, sections 3 and 5).
use crate::JigBindings;
use crate::phrases::jig::JigPhrase;
use crate::words::{Words, name};
use skein_ui::Builder;
use skein_ui::phrases::Hole;

use jig_client::contract::HoldReason;

/// Say the actual held reason and every fact it carries; parts remain connector-owned.
pub fn reason(b: &mut Builder, reason: &HoldReason, words: &Words, bindings: &mut JigBindings) {
    match reason {
        HoldReason::Tries { failures } => {
            words.text(b, JigPhrase::TriesExhausted, &[]);
            for failure in failures {
                words.text(b, JigPhrase::Attempt, &[Hole::Number(failure.attempt)]);
                super::chip::failure(b, failure.class, words);
                words.text(b, JigPhrase::Reason, &[name(failure.reason.bytes())]);
                words.wall(b, failure.at);
            }
        }
        HoldReason::Stalled { since, waiting } => {
            words.text(b, JigPhrase::StalledOn, &[name(waiting.bytes())]);
            words.text(b, JigPhrase::Since, &[Hole::Wall(words.adjusted_wall(*since))]);
        }
        HoldReason::Dependency { task, how } => {
            words.text(b, JigPhrase::DependencyEnded, &[Hole::Number(*task)]);
            words.ending(b, how);
        }
        HoldReason::Drift { part } => {
            words.text(b, JigPhrase::DriftHold, &[]);
            super::common::part(b, part, words, bindings);
        }
        HoldReason::EffectFailed { part } => {
            words.text(b, JigPhrase::EffectFailedHold, &[]);
            super::common::part(b, part, words, bindings);
        }
        HoldReason::EffectUncertain { part } => {
            words.text(b, JigPhrase::EffectUncertainHold, &[]);
            super::common::part(b, part, words, bindings);
        }
        HoldReason::Budget { spent, limit } => {
            words.text(b, JigPhrase::BudgetHold, &[Hole::Amount(*spent), Hole::Amount(*limit)]);
        }
        HoldReason::Deadline { at } => words.text(b, JigPhrase::DeadlineHold, &[Hole::Wall(words.adjusted_wall(*at))]),
        HoldReason::Stopped { by, at } => {
            words.text(b, JigPhrase::StoppedHold, &[name(by.name.bytes()), Hole::Wall(words.adjusted_wall(*at))]);
        }
        HoldReason::Resource { resource, holder } => {
            words.text(b, JigPhrase::ResourceHold, &[]);
            super::common::part(b, resource, words, bindings);
            words.actor(b, *holder);
        }
        HoldReason::Pool { part } => {
            words.text(b, JigPhrase::PoolHold, &[]);
            super::common::part(b, part, words, bindings);
        }
        HoldReason::Procedure { part } => {
            words.text(b, JigPhrase::ProcedureHold, &[]);
            super::common::part(b, part, words, bindings);
        }
    }
}
