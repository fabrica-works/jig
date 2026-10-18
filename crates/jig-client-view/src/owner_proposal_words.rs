//! Checked lifecycle pending proposal sayings (client/domain/view.md, 7).
use crate::Owner;
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::phrases::{
    Form, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment, SpanUnit,
    WallFormat,
};
/// Closed pending proposal captions, composed into normal owner words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalPhrase {
    Proposal,
    Number,
    Project,
    Reason,
    Holder,
    AsHolder,
    NotHolder,
    Batch,
    Effect,
    Attempt,
    Completion,
    Position,
    Purpose,
    ProcedureStep,
    Amend,
    Before,
    After,
    AmendReason,
    AddressAbsent,
    ReleaseRevision,
    ReleaseAbsent,
    Widen,
    Release,
    HoldAbsent,
    Funding,
    WaitingSince,
}
pub const ALL: [ProposalPhrase; 26] = [
    ProposalPhrase::Proposal,
    ProposalPhrase::Number,
    ProposalPhrase::Project,
    ProposalPhrase::Reason,
    ProposalPhrase::Holder,
    ProposalPhrase::AsHolder,
    ProposalPhrase::NotHolder,
    ProposalPhrase::Batch,
    ProposalPhrase::Effect,
    ProposalPhrase::Attempt,
    ProposalPhrase::Completion,
    ProposalPhrase::Position,
    ProposalPhrase::Purpose,
    ProposalPhrase::ProcedureStep,
    ProposalPhrase::Amend,
    ProposalPhrase::Before,
    ProposalPhrase::After,
    ProposalPhrase::AmendReason,
    ProposalPhrase::AddressAbsent,
    ProposalPhrase::ReleaseRevision,
    ProposalPhrase::ReleaseAbsent,
    ProposalPhrase::Widen,
    ProposalPhrase::Release,
    ProposalPhrase::HoldAbsent,
    ProposalPhrase::Funding,
    ProposalPhrase::WaitingSince,
];
impl ProposalPhrase {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        PhraseId {
            owner: Owner::Jig.opaque(),
            number: match self {
                Self::Proposal => 8400,
                Self::Number => 8401,
                Self::Project => 8402,
                Self::Reason => 8403,
                Self::Holder => 8404,
                Self::AsHolder => 8405,
                Self::NotHolder => 8406,
                Self::Batch => 8407,
                Self::Effect => 8408,
                Self::Attempt => 8409,
                Self::Completion => 8410,
                Self::Position => 8411,
                Self::Purpose => 8412,
                Self::ProcedureStep => 8430,
                Self::Amend => 8413,
                Self::Before => 8414,
                Self::After => 8415,
                Self::AmendReason => 8416,
                Self::AddressAbsent => 8417,
                Self::ReleaseRevision => 8418,
                Self::ReleaseAbsent => 8419,
                Self::Widen => 8420,
                Self::Release => 8421,
                Self::HoldAbsent => 8422,
                Self::Funding => 8423,
                Self::WaitingSince => 8429,
            },
        }
    }
    fn hole(self) -> Option<HoleKind> {
        match self {
            Self::Number
            | Self::Project
            | Self::Attempt
            | Self::Completion
            | Self::Position
            | Self::Purpose
            | Self::ProcedureStep
            | Self::ReleaseRevision => Some(HoleKind::Number),
            Self::WaitingSince => Some(HoleKind::Wall),
            Self::Proposal
            | Self::Reason
            | Self::Holder
            | Self::AsHolder
            | Self::NotHolder
            | Self::Batch
            | Self::Effect
            | Self::Amend
            | Self::Before
            | Self::After
            | Self::AmendReason
            | Self::AddressAbsent
            | Self::ReleaseAbsent
            | Self::Widen
            | Self::Release
            | Self::HoldAbsent
            | Self::Funding => None,
        }
    }
    fn label(self) -> &'static [u8] {
        match self {
            Self::Proposal => b"Pending proposal",
            Self::Number => b"Proposal number: ",
            Self::Project => b"Proposal project: ",
            Self::Reason => b"Proposal reason",
            Self::Holder => b"Waiting holder",
            Self::AsHolder => b"Proposed as holder",
            Self::NotHolder => b"Not proposed as holder",
            Self::Batch => b"Proposed batch",
            Self::Effect => b"Proposed effect",
            Self::Attempt => b"Effect attempt: ",
            Self::Completion => b"Effect completion: ",
            Self::Position => b"Effect position: ",
            Self::Purpose => b"Effect purpose: ",
            Self::ProcedureStep => b"Procedure step: ",
            Self::Amend => b"Proposed amendment",
            Self::Before => b"Before proposed change",
            Self::After => b"After proposed change",
            Self::AmendReason => b"Amendment reason",
            Self::AddressAbsent => b"Address change not supplied",
            Self::ReleaseRevision => b"Proposed release revision: ",
            Self::ReleaseAbsent => b"Proposed release not supplied",
            Self::Widen => b"Proposed authority widening",
            Self::Release => b"Proposed release",
            Self::HoldAbsent => b"Release hold not supplied",
            Self::Funding => b"Supplied funding offers",
            Self::WaitingSince => b"Waiting since: ",
        }
    }
}

#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut values = List::with_capacity(26);
    for phrase in ALL {
        let holes: Box<[HoleShape]> = match phrase.hole() {
            Some(kind) => Box::new([HoleShape { kind, bytes: 0 }]),
            None => Box::new([]),
        };
        values.push(PhraseShape { id: phrase.id(), holes, plural: None }).expect("closed pending proposal shapes fit");
    }
    values.into_boxed()
}
#[must_use]
pub fn language() -> Language {
    let mut values = List::with_capacity(26);
    for phrase in ALL {
        let segments: Box<[Segment]> = match phrase.hole() {
            Some(kind) => Box::new([Segment::Text(copy_of(phrase.label())), Segment::Hole { index: 0, kind }]),
            None => Box::new([Segment::Text(copy_of(phrase.label()))]),
        };
        values
            .push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("closed pending proposal sayings fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: values.into_boxed(),
    }
}
