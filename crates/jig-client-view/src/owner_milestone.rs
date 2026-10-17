//! Standalone lifecycle milestones (client/domain/objects.md, 11; view.md, 7).
//! Ui is borrowed only by existing admitted result follow-up fragments.
use crate::JigBindings;
use crate::owner_milestone_words::MilestonePhrase as P;
use crate::words::Words;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use crate::{Borrowed, Limits};
use jig_client::contract::{ClosingStage, OwnerChipEnding, OwnerMilestoneLine, OwnerMilestonePhase, RunStage};

/// Checked storage for one whole milestone and its nested result/source fragments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MilestoneFragment {
    pub nodes: u32,
    pub text_bytes: u32,
    pub text_per_node: u32,
    pub depth: u32,
    pub bindings: u32,
    pub slots: u32,
}
/// Finite whole row envelope, including existing result follow-ups and raw parts.
#[must_use]
pub fn max_nodes(limits: &Limits) -> Option<u32> {
    32_u32
        .checked_add(crate::proposed_task::part_nodes(limits)?)?
        .checked_add(skein_ui::markdown::max_nodes(limits.text)?.checked_mul(2)?)?
        .checked_add(crate::result_nodes(limits)?)
}
fn ending_fits(value: &OwnerChipEnding, limits: &Limits) -> Option<()> {
    match value {
        OwnerChipEnding::Done(result) => {
            crate::owner_chip::result_nodes(result, limits)?;
        }
        OwnerChipEnding::Failed(reason) => {
            if !reason.bytes.fits(limits.text) {
                return None;
            }
        }
        OwnerChipEnding::Cancelled { reason, result, .. } => {
            if !reason.bytes.fits(limits.text) {
                return None;
            }
            if let Some(result) = result {
                crate::owner_chip::result_nodes(result, limits)?;
            }
        }
    }
    Some(())
}
/// Validate every nested supplied text/list/part before returning a whole envelope.
#[must_use]
pub fn required(value: &OwnerMilestoneLine, limits: &Limits, words: &Words) -> Option<MilestoneFragment> {
    match &value.phase {
        OwnerMilestonePhase::Waiting | OwnerMilestonePhase::Active(_) => {}
        OwnerMilestonePhase::Closing { ending, .. } | OwnerMilestonePhase::Ended(ending) => {
            ending_fits(ending, limits)?;
        }
        OwnerMilestonePhase::Held { source, ending, .. } => {
            if let Some(source) = source {
                crate::owner_chip::part_fits(source, limits)?;
            }
            if let Some(ending) = ending {
                ending_fits(ending, limits)?;
            }
        }
    }
    worst_case(limits, words)
}
/// Checked worst-case tree/text/binding envelope from the normal selected language.
#[must_use]
pub fn worst_case(limits: &Limits, words: &Words) -> Option<MilestoneFragment> {
    let nodes = max_nodes(limits)?;
    let phrase = crate::phrases::jig_capacity(&words.book)?;
    let text_per_node = crate::literal_segment::max_bytes(limits.text)?.max(phrase).checked_mul(2)?;
    Some(MilestoneFragment {
        nodes,
        text_bytes: nodes.checked_mul(text_per_node)?,
        text_per_node,
        depth: 20,
        bindings: nodes,
        slots: nodes,
    })
}
fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    let saying = words.book.say(words.language, phrase.id(), holes).expect("checked milestone words and typed holes");
    b.text(Run::Plain, &saying.text);
}
fn active(b: &mut Builder, stage: RunStage, words: &Words) {
    match stage {
        RunStage::Idle => text(b, words, P::Idle, &[]),
        RunStage::Due => text(b, words, P::Due, &[]),
        RunStage::Preparing => text(b, words, P::Preparing, &[]),
        RunStage::Claimed { attempt } => text(b, words, P::Claimed, &[Hole::Number(attempt)]),
        RunStage::Running { attempt } => text(b, words, P::Running, &[Hole::Number(attempt)]),
        RunStage::BackingOff { until } => text(b, words, P::BackingOff, &[Hole::Wall(words.adjusted_wall(until))]),
    }
}
fn closing(b: &mut Builder, stage: ClosingStage, words: &Words) {
    match stage {
        ClosingStage::Run { attempt } => text(b, words, P::Run, &[Hole::Number(attempt)]),
        ClosingStage::Delegates => text(b, words, P::Delegates, &[]),
        ClosingStage::Effects => text(b, words, P::Effects, &[]),
        ClosingStage::Releases => text(b, words, P::Releases, &[]),
        ClosingStage::Settled => text(b, words, P::Settled, &[]),
    }
}
/// Render only supplied milestone facts, borrowing existing result follow-ups.
pub fn render(
    b: &mut Builder,
    value: &OwnerMilestoneLine,
    client: &Borrowed<'_>,
    words: &Words,
    bindings: &mut JigBindings,
) {
    b.open(Role::Region);
    let caption = words.book.say(words.language, P::Milestone.id(), &[]).expect("checked milestone region name");
    b.name(&caption.text);
    text(b, words, P::Milestone, &[]);
    text(b, words, P::Task, &[Hole::Number(value.task)]);
    text(b, words, P::Position, &[Hole::Number(value.position)]);
    match value.at {
        Some(at) => text(b, words, P::At, &[Hole::Wall(words.adjusted_wall(at))]),
        None => text(b, words, P::AtAbsent, &[]),
    }
    match &value.phase {
        OwnerMilestonePhase::Waiting => text(b, words, P::Waiting, &[]),
        OwnerMilestonePhase::Active(stage) => {
            text(b, words, P::Active, &[]);
            active(b, *stage, words);
        }
        OwnerMilestonePhase::Closing { stage, ending } => {
            text(b, words, P::Closing, &[]);
            closing(b, *stage, words);
            crate::owner_chip::ending(b, ending, client, words, bindings);
        }
        OwnerMilestonePhase::Held { before, why, source, ending } => {
            text(b, words, P::Held, &[]);
            crate::owner_chip::prior(b, *before, words);
            crate::owner_chip::reason(b, *why, words);
            match source {
                Some(source) => crate::proposed_task::part(b, source, words, bindings),
                None => text(b, words, P::SourceAbsent, &[]),
            }
            match ending {
                Some(ending) => crate::owner_chip::ending(b, ending, client, words, bindings),
                None => text(b, words, P::EndingAbsent, &[]),
            }
        }
        OwnerMilestonePhase::Ended(ending) => {
            text(b, words, P::Ended, &[]);
            crate::owner_chip::ending(b, ending, client, words, bindings);
        }
    }
    b.close();
}
