//! Compact exact lifecycle; whole metadata is expanded on the task page.
//! See client/domain/objects.md, 3, and domain/tasks.md, 5.
use crate::{
    Builder,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::contract::{ClosingStage, OwnerChipPhase, PersonAddress, RunStage};

pub(crate) fn brief(b: &mut Builder, phase: &OwnerChipPhase, words: &Words) {
    match phase {
        OwnerChipPhase::Waiting { more, holds, .. } => {
            words.text(b, JigPhrase::Waiting, &[]);
            if *more {
                words.text(b, JigPhrase::ChipDependenciesCut, &[]);
            }
            if *holds {
                words.text(b, JigPhrase::ChipWaitingHolds, &[]);
            }
        }
        OwnerChipPhase::Active(stage) => match stage {
            RunStage::Idle => words.text(b, JigPhrase::RunIdle, &[]),
            RunStage::Due => words.text(b, JigPhrase::RunDue, &[]),
            RunStage::Preparing => words.text(b, JigPhrase::RunPreparing, &[]),
            RunStage::Claimed { attempt } => words.text(b, JigPhrase::RunClaimed, &[Hole::Number(*attempt)]),
            RunStage::Running { attempt } => words.text(b, JigPhrase::RunOwnerActive, &[Hole::Number(*attempt)]),
            RunStage::BackingOff { until } => {
                words.text(b, JigPhrase::RunBackingOff, &[Hole::Wall(words.adjusted_wall(*until))])
            }
        },
        OwnerChipPhase::Person { address, taken_by, since } => {
            match address {
                PersonAddress::Party(party) => words.text(b, JigPhrase::AddressParty, &[Hole::Number(*party)]),
                PersonAddress::Role(role) => words.text(b, JigPhrase::AddressRole, &[Hole::Number(u64::from(*role))]),
            }
            if let Some(party) = taken_by {
                words.text(b, JigPhrase::TakenByNumber, &[Hole::Number(*party)]);
            }
            if let Some(at) = since {
                words.text(b, JigPhrase::Since, &[Hole::Wall(words.adjusted_wall(*at))]);
            }
        }
        OwnerChipPhase::Closing(stage) => {
            words.text(b, JigPhrase::ClosingPhase, &[]);
            match stage {
                ClosingStage::Run { attempt } => words.text(b, JigPhrase::ClosingRun, &[Hole::Number(*attempt)]),
                ClosingStage::Delegates => words.text(b, JigPhrase::ClosingDelegates, &[]),
                ClosingStage::Effects => words.text(b, JigPhrase::ClosingStageEffects, &[]),
                ClosingStage::Releases => words.text(b, JigPhrase::ClosingReleases, &[]),
                ClosingStage::Settled => words.text(b, JigPhrase::ClosingSettled, &[]),
            }
        }
        OwnerChipPhase::Held { reason, revision } => {
            words.text(b, JigPhrase::Held, &[]);
            crate::owner_chip::reason(b, *reason, words);
            words.text(b, JigPhrase::Revision, &[Hole::Number(*revision)]);
        }
        OwnerChipPhase::Ended => words.text(b, JigPhrase::Ended, &[]),
    }
}
