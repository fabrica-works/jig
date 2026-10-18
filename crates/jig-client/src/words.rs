//! Jig's enum phrases, overridable one by one (client/domain/client.md, 19).
mod english;
use crate::{
    contract::{HoldCategory, Phase, Refusal, WaitsOn},
    facts::ChatStatus,
};
use alloc::boxed::Box;
use skein_lib::List;
use skein_ui::phrases::{HoleKind, HoleShape, Language, PhraseId, PhraseShape};
/// Phrase count, for a caller composing a bounded phrasebook.
pub const PHRASES: u32 = 58;
/// Owner of jig's public enum vocabulary.
pub const OWNER: skein_ui::Owner = skein_ui::Owner { number: 65_537 };
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    Phase(Phase),
    Hold(HoldWord),
    Refusal(RefusalWord),
    Waiting(WaitingWord),
    Status(StatusWord),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldWord {
    Failures,
    Stalled,
    Dependency,
    Stopped,
    StoppedBy,
    Drift,
    Effects,
    EffectFailed,
    Uncertain,
    Budget,
    Deadline,
    HoldsWaited,
    ResourceUnavailable,
    ResourceAccess,
    PoolLost,
    Procedure,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefusalWord {
    NoEscalationHolder,
    Malformed,
    Role,
    Taken,
    NotWaiting,
    NoFurther,
    Authority,
    Limit,
    Moved,
    Ended,
    Unknown,
    NotOffered,
    KeyConflict,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitingWord {
    Party,
    Delegates,
    News,
    Nothing,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusWord {
    Starting,
    Writing,
    Tool,
    Working,
    WaitingForYou,
    WaitingFor,
    Parked,
    Retrying,
    Held,
    Closing,
    Ended,
}
impl Word {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        PhraseId { owner: OWNER, number: self.data().0 }
    }
    #[must_use]
    pub fn hold(value: &HoldCategory) -> Self {
        Self::Hold(match value {
            HoldCategory::Failures { .. } => HoldWord::Failures,
            HoldCategory::Stalled => HoldWord::Stalled,
            HoldCategory::Dependency { .. } => HoldWord::Dependency,
            HoldCategory::Stopped => HoldWord::Stopped,
            HoldCategory::StoppedBy { .. } => HoldWord::StoppedBy,
            HoldCategory::Drift => HoldWord::Drift,
            HoldCategory::Effects => HoldWord::Effects,
            HoldCategory::EffectFailed => HoldWord::EffectFailed,
            HoldCategory::Uncertain { .. } => HoldWord::Uncertain,
            HoldCategory::Budget => HoldWord::Budget,
            HoldCategory::Deadline => HoldWord::Deadline,
            HoldCategory::HoldsWaited => HoldWord::HoldsWaited,
            HoldCategory::ResourceUnavailable => HoldWord::ResourceUnavailable,
            HoldCategory::ResourceAccess => HoldWord::ResourceAccess,
            HoldCategory::PoolLost { .. } => HoldWord::PoolLost,
            HoldCategory::Procedure => HoldWord::Procedure,
        })
    }
    #[must_use]
    pub fn refusal(value: &Refusal) -> Self {
        Self::Refusal(match value {
            Refusal::NoEscalationHolder => RefusalWord::NoEscalationHolder,
            Refusal::Malformed { .. } => RefusalWord::Malformed,
            Refusal::Role { .. } => RefusalWord::Role,
            Refusal::Taken { .. } => RefusalWord::Taken,
            Refusal::NotWaiting => RefusalWord::NotWaiting,
            Refusal::NoFurther => RefusalWord::NoFurther,
            Refusal::Authority { .. } => RefusalWord::Authority,
            Refusal::Limit { .. } => RefusalWord::Limit,
            Refusal::Moved { .. } => RefusalWord::Moved,
            Refusal::Ended { .. } => RefusalWord::Ended,
            Refusal::Unknown => RefusalWord::Unknown,
            Refusal::NotOffered => RefusalWord::NotOffered,
            Refusal::KeyConflict => RefusalWord::KeyConflict,
        })
    }
    #[must_use]
    pub fn waiting(value: &WaitsOn) -> Self {
        Self::Waiting(match value {
            WaitsOn::Party { .. } => WaitingWord::Party,
            WaitsOn::Delegates => WaitingWord::Delegates,
            WaitsOn::News => WaitingWord::News,
            WaitsOn::Nothing => WaitingWord::Nothing,
        })
    }
    #[must_use]
    pub fn status(value: &ChatStatus) -> Self {
        Self::Status(match value {
            ChatStatus::Starting => StatusWord::Starting,
            ChatStatus::Writing => StatusWord::Writing,
            ChatStatus::Tool { .. } => StatusWord::Tool,
            ChatStatus::Working { .. } => StatusWord::Working,
            ChatStatus::WaitingForYou { .. } => StatusWord::WaitingForYou,
            ChatStatus::WaitingFor(_) => StatusWord::WaitingFor,
            ChatStatus::Parked => StatusWord::Parked,
            ChatStatus::Retrying { .. } => StatusWord::Retrying,
            ChatStatus::Held(_) => StatusWord::Held,
            ChatStatus::Closing => StatusWord::Closing,
            ChatStatus::Ended { .. } => StatusWord::Ended,
        })
    }
    #[must_use]
    pub fn shape(self, text: u32) -> PhraseShape {
        let mut holes = List::with_capacity(1);
        for kind in self.data().2 {
            holes
                .push(HoleShape { kind: *kind, bytes: if *kind == HoleKind::Name { text } else { 0 } })
                .expect("one bounded hole");
        }
        PhraseShape { id: self.id(), holes: holes.into_boxed(), plural: None }
    }
}
/// All public words, for composing phrasebook capacities without allocation.
pub const ALL: [Word; 58] = [
    Word::Phase(Phase::Waiting),
    Word::Phase(Phase::Idle),
    Word::Phase(Phase::Due),
    Word::Phase(Phase::Preparing),
    Word::Phase(Phase::Claimed),
    Word::Phase(Phase::Running),
    Word::Phase(Phase::BackingOff),
    Word::Phase(Phase::Active),
    Word::Phase(Phase::Closing),
    Word::Phase(Phase::Held),
    Word::Phase(Phase::Ended),
    Word::Phase(Phase::Done),
    Word::Phase(Phase::Failed),
    Word::Phase(Phase::Cancelled),
    Word::Hold(HoldWord::Failures),
    Word::Hold(HoldWord::Stalled),
    Word::Hold(HoldWord::Dependency),
    Word::Hold(HoldWord::Stopped),
    Word::Hold(HoldWord::StoppedBy),
    Word::Hold(HoldWord::Drift),
    Word::Hold(HoldWord::Effects),
    Word::Hold(HoldWord::EffectFailed),
    Word::Hold(HoldWord::Uncertain),
    Word::Hold(HoldWord::Budget),
    Word::Hold(HoldWord::Deadline),
    Word::Hold(HoldWord::HoldsWaited),
    Word::Hold(HoldWord::ResourceUnavailable),
    Word::Hold(HoldWord::ResourceAccess),
    Word::Hold(HoldWord::PoolLost),
    Word::Hold(HoldWord::Procedure),
    Word::Refusal(RefusalWord::NoEscalationHolder),
    Word::Refusal(RefusalWord::Malformed),
    Word::Refusal(RefusalWord::Role),
    Word::Refusal(RefusalWord::Taken),
    Word::Refusal(RefusalWord::NotWaiting),
    Word::Refusal(RefusalWord::NoFurther),
    Word::Refusal(RefusalWord::Authority),
    Word::Refusal(RefusalWord::Limit),
    Word::Refusal(RefusalWord::Moved),
    Word::Refusal(RefusalWord::Ended),
    Word::Refusal(RefusalWord::Unknown),
    Word::Refusal(RefusalWord::NotOffered),
    Word::Refusal(RefusalWord::KeyConflict),
    Word::Waiting(WaitingWord::Party),
    Word::Waiting(WaitingWord::Delegates),
    Word::Waiting(WaitingWord::News),
    Word::Waiting(WaitingWord::Nothing),
    Word::Status(StatusWord::Starting),
    Word::Status(StatusWord::Writing),
    Word::Status(StatusWord::Tool),
    Word::Status(StatusWord::Working),
    Word::Status(StatusWord::WaitingForYou),
    Word::Status(StatusWord::WaitingFor),
    Word::Status(StatusWord::Parked),
    Word::Status(StatusWord::Retrying),
    Word::Status(StatusWord::Held),
    Word::Status(StatusWord::Closing),
    Word::Status(StatusWord::Ended),
];
#[must_use]
pub fn phrase_shapes(text: u32) -> Box<[PhraseShape]> {
    let mut shapes = List::with_capacity(u32::try_from(ALL.len()).expect("closed word count"));
    for word in ALL {
        shapes.push(word.shape(text)).expect("all word shapes fit");
    }
    shapes.into_boxed()
}
#[must_use]
pub fn english() -> Language {
    english::language()
}
