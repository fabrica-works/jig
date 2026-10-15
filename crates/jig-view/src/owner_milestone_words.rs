//! Checked lifecycle milestone sayings (client/domain/view.md, 7).
use crate::Owner;
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::phrases::{
    Form, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment, SpanUnit,
    WallFormat,
};
/// Closed milestone captions, composed into normal owner words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MilestonePhrase {
    Milestone,
    Task,
    Position,
    At,
    AtAbsent,
    Waiting,
    Active,
    Closing,
    Held,
    Ended,
    Idle,
    Due,
    Preparing,
    Claimed,
    Running,
    BackingOff,
    Run,
    Delegates,
    Effects,
    Releases,
    Settled,
    SourceAbsent,
    EndingAbsent,
}
pub const ALL: [MilestonePhrase; 23] = [
    MilestonePhrase::Milestone,
    MilestonePhrase::Task,
    MilestonePhrase::Position,
    MilestonePhrase::At,
    MilestonePhrase::AtAbsent,
    MilestonePhrase::Waiting,
    MilestonePhrase::Active,
    MilestonePhrase::Closing,
    MilestonePhrase::Held,
    MilestonePhrase::Ended,
    MilestonePhrase::Idle,
    MilestonePhrase::Due,
    MilestonePhrase::Preparing,
    MilestonePhrase::Claimed,
    MilestonePhrase::Running,
    MilestonePhrase::BackingOff,
    MilestonePhrase::Run,
    MilestonePhrase::Delegates,
    MilestonePhrase::Effects,
    MilestonePhrase::Releases,
    MilestonePhrase::Settled,
    MilestonePhrase::SourceAbsent,
    MilestonePhrase::EndingAbsent,
];
impl MilestonePhrase {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        PhraseId {
            owner: Owner::Jig.opaque(),
            number: match self {
                Self::Milestone => 8100,
                Self::Task => 8101,
                Self::Position => 8102,
                Self::At => 8103,
                Self::AtAbsent => 8104,
                Self::Waiting => 8105,
                Self::Active => 8106,
                Self::Closing => 8107,
                Self::Held => 8108,
                Self::Ended => 8109,
                Self::Idle => 8110,
                Self::Due => 8111,
                Self::Preparing => 8112,
                Self::Claimed => 8113,
                Self::Running => 8114,
                Self::BackingOff => 8115,
                Self::Run => 8116,
                Self::Delegates => 8117,
                Self::Effects => 8118,
                Self::Releases => 8119,
                Self::Settled => 8120,
                Self::SourceAbsent => 8121,
                Self::EndingAbsent => 8122,
            },
        }
    }
    fn hole(self) -> Option<HoleKind> {
        match self {
            Self::Task | Self::Position | Self::Claimed | Self::Running | Self::Run => Some(HoleKind::Number),
            Self::At | Self::BackingOff => Some(HoleKind::Wall),
            Self::Milestone
            | Self::AtAbsent
            | Self::Waiting
            | Self::Active
            | Self::Closing
            | Self::Held
            | Self::Ended
            | Self::Idle
            | Self::Due
            | Self::Preparing
            | Self::Delegates
            | Self::Effects
            | Self::Releases
            | Self::Settled
            | Self::SourceAbsent
            | Self::EndingAbsent => None,
        }
    }
    fn label(self) -> &'static [u8] {
        match self {
            Self::Milestone => b"Lifecycle milestone",
            Self::Task => b"Task: ",
            Self::Position => b"Activity position: ",
            Self::At => b"Milestone time: ",
            Self::AtAbsent => b"Milestone time not supplied",
            Self::Waiting => b"Waiting",
            Self::Active => b"Activation",
            Self::Closing => b"Closing",
            Self::Held => b"Held",
            Self::Ended => b"Ended",
            Self::Idle => b"Activation idle",
            Self::Due => b"Activation due",
            Self::Preparing => b"Activation preparing",
            Self::Claimed => b"Claimed attempt: ",
            Self::Running => b"Running attempt: ",
            Self::BackingOff => b"Backing off until: ",
            Self::Run => b"Closing run attempt: ",
            Self::Delegates => b"Closing delegates",
            Self::Effects => b"Closing effects",
            Self::Releases => b"Closing releases",
            Self::Settled => b"Closing settled",
            Self::SourceAbsent => b"Hold source not supplied",
            Self::EndingAbsent => b"Held ending not supplied",
        }
    }
}
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut values = List::with_capacity(23);
    for phrase in ALL {
        let holes: Box<[HoleShape]> = match phrase.hole() {
            Some(kind) => Box::new([HoleShape { kind, bytes: 0 }]),
            None => Box::new([]),
        };
        values.push(PhraseShape { id: phrase.id(), holes, plural: None }).expect("closed milestone shapes fit");
    }
    values.into_boxed()
}
#[must_use]
pub fn language() -> Language {
    let mut values = List::with_capacity(23);
    for phrase in ALL {
        let segments: Box<[Segment]> = match phrase.hole() {
            Some(kind) => Box::new([Segment::Text(copy_of(phrase.label())), Segment::Hole { index: 0, kind }]),
            None => Box::new([Segment::Text(copy_of(phrase.label()))]),
        };
        values
            .push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("closed milestone sayings fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: values.into_boxed(),
    }
}
