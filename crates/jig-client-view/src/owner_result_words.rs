//! Checked result metadata words (client/domain/view.md, 7).
//! IDs 6200..6222 belong to this vocabulary within the reserved 6200..6260 range.
use crate::{
    Owner,
    phrases::{
        Form, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment, SpanUnit,
        WallFormat,
    },
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};

/// Result owner facts and explicit absence, selected as checked language data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultPhrase {
    /// Metadata heading.
    Details,
    /// Supplied task contract.
    Contract,
    /// Task requester number.
    RequesterTask,
    /// Person requester number.
    RequesterPerson,
    /// Deployment requester project.
    RequesterDeployment,
    /// Supplied chooser identity.
    Chooser,
    /// Absent chooser identity.
    ChooserAbsent,
    /// Delegate reference count.
    Delegates,
    /// Available follow-up reference count.
    Followups,
    /// Unavailable follow-up references.
    FollowupsAbsent,
    /// Proposal with a task maker.
    ProposalTask,
    /// Proposal with a party maker.
    ProposalParty,
    /// Absent proposal reference.
    ProposalAbsent,
    /// Live linked task.
    Live,
    /// Retained ended task.
    Retained,
    /// Stored task number.
    Stored,
    /// Supplied retained clock.
    At,
    /// Unavailable retained clock.
    AtUnknown,
    /// Stored completed status.
    Done,
    /// Stored failed status.
    Failed,
    /// Stored cancelled status.
    Cancelled,
    /// Unavailable stored status.
    StatusUnknown,
    /// Aggregate metadata loss.
    Cut,
}
/// Complete closed vocabulary in stable identity order.
pub const ALL: [ResultPhrase; 23] = [
    ResultPhrase::Details,
    ResultPhrase::Contract,
    ResultPhrase::RequesterTask,
    ResultPhrase::RequesterPerson,
    ResultPhrase::RequesterDeployment,
    ResultPhrase::Chooser,
    ResultPhrase::ChooserAbsent,
    ResultPhrase::Delegates,
    ResultPhrase::Followups,
    ResultPhrase::FollowupsAbsent,
    ResultPhrase::ProposalTask,
    ResultPhrase::ProposalParty,
    ResultPhrase::ProposalAbsent,
    ResultPhrase::Live,
    ResultPhrase::Retained,
    ResultPhrase::Stored,
    ResultPhrase::At,
    ResultPhrase::AtUnknown,
    ResultPhrase::Done,
    ResultPhrase::Failed,
    ResultPhrase::Cancelled,
    ResultPhrase::StatusUnknown,
    ResultPhrase::Cut,
];
impl ResultPhrase {
    /// Stable identity used to compose languages before rendering.
    #[must_use]
    pub const fn id(self) -> PhraseId {
        let number = match self {
            Self::Details => 6200,
            Self::Contract => 6201,
            Self::RequesterTask => 6202,
            Self::RequesterPerson => 6203,
            Self::RequesterDeployment => 6204,
            Self::Chooser => 6205,
            Self::ChooserAbsent => 6206,
            Self::Delegates => 6207,
            Self::Followups => 6208,
            Self::FollowupsAbsent => 6209,
            Self::ProposalTask => 6210,
            Self::ProposalParty => 6211,
            Self::ProposalAbsent => 6212,
            Self::Live => 6213,
            Self::Retained => 6214,
            Self::Stored => 6215,
            Self::At => 6216,
            Self::AtUnknown => 6217,
            Self::Done => 6218,
            Self::Failed => 6219,
            Self::Cancelled => 6220,
            Self::StatusUnknown => 6221,
            Self::Cut => 6222,
        };
        PhraseId { owner: Owner::Jig, number }
    }
    /// Typed hole contract independent of language order.
    #[must_use]
    pub fn shape(self) -> PhraseShape {
        let holes: Box<[HoleShape]> = match self {
            Self::Details => Box::new([]),
            Self::Contract => Box::new([]),
            Self::RequesterTask => Box::new([HoleShape { kind: HoleKind::Number, bytes: 0 }]),
            Self::RequesterPerson => Box::new([HoleShape { kind: HoleKind::Number, bytes: 0 }]),
            Self::RequesterDeployment => Box::new([HoleShape { kind: HoleKind::Number, bytes: 0 }]),
            Self::Chooser => Box::new([HoleShape { kind: HoleKind::Number, bytes: 0 }]),
            Self::ChooserAbsent => Box::new([]),
            Self::Delegates => Box::new([HoleShape { kind: HoleKind::Number, bytes: 0 }]),
            Self::Followups => Box::new([HoleShape { kind: HoleKind::Number, bytes: 0 }]),
            Self::FollowupsAbsent => Box::new([]),
            Self::ProposalTask => Box::new([
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Number, bytes: 0 },
            ]),
            Self::ProposalParty => Box::new([
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Number, bytes: 0 },
            ]),
            Self::ProposalAbsent => Box::new([]),
            Self::Live => Box::new([]),
            Self::Retained => Box::new([]),
            Self::Stored => Box::new([HoleShape { kind: HoleKind::Number, bytes: 0 }]),
            Self::At => Box::new([HoleShape { kind: HoleKind::Wall, bytes: 0 }]),
            Self::AtUnknown => Box::new([]),
            Self::Done => Box::new([]),
            Self::Failed => Box::new([]),
            Self::Cancelled => Box::new([]),
            Self::StatusUnknown => Box::new([]),
            Self::Cut => Box::new([]),
        };
        PhraseShape { id: self.id(), holes, plural: None }
    }
}
/// Shapes for whole phrasebook checking.
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut rows = List::with_capacity(23);
    for phrase in ALL {
        rows.push(phrase.shape()).expect("closed result shapes fit");
    }
    rows.into_boxed()
}
/// English language data; rendering uses the loaded selected language.
#[must_use]
pub fn language() -> Language {
    let mut rows = List::with_capacity(23);
    for phrase in ALL {
        let segments: Box<[Segment]> = match phrase {
            ResultPhrase::Details => Box::new([Segment::Text(copy_of(b"Result owner details"))]),
            ResultPhrase::Contract => Box::new([Segment::Text(copy_of(b"Task contract"))]),
            ResultPhrase::RequesterTask => Box::new([
                Segment::Text(copy_of(b"Requested by task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            ResultPhrase::RequesterPerson => Box::new([
                Segment::Text(copy_of(b"Requested by person ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            ResultPhrase::RequesterDeployment => Box::new([
                Segment::Text(copy_of(b"Requested by deployment in project ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            ResultPhrase::Chooser => Box::new([
                Segment::Text(copy_of(b"Choice made by person ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            ResultPhrase::ChooserAbsent => Box::new([Segment::Text(copy_of(b"Choice maker not supplied"))]),
            ResultPhrase::Delegates => Box::new([
                Segment::Text(copy_of(b"Delegate task references: ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            ResultPhrase::Followups => Box::new([
                Segment::Text(copy_of(b"Follow-up task references: ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            ResultPhrase::FollowupsAbsent => Box::new([Segment::Text(copy_of(b"Follow-up facts unavailable"))]),
            ResultPhrase::ProposalTask => Box::new([
                Segment::Text(copy_of(b"Proposal from task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", number ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
            ResultPhrase::ProposalParty => Box::new([
                Segment::Text(copy_of(b"Proposal from party ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", number ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
            ResultPhrase::ProposalAbsent => Box::new([Segment::Text(copy_of(b"Proposal not supplied"))]),
            ResultPhrase::Live => Box::new([Segment::Text(copy_of(b"Live task"))]),
            ResultPhrase::Retained => Box::new([Segment::Text(copy_of(b"Retained ended task"))]),
            ResultPhrase::Stored => {
                Box::new([Segment::Text(copy_of(b"Stored task ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            ResultPhrase::At => {
                Box::new([Segment::Text(copy_of(b"Ended at ")), Segment::Hole { index: 0, kind: HoleKind::Wall }])
            }
            ResultPhrase::AtUnknown => Box::new([Segment::Text(copy_of(b"Ended time unavailable"))]),
            ResultPhrase::Done => Box::new([Segment::Text(copy_of(b"Done"))]),
            ResultPhrase::Failed => Box::new([Segment::Text(copy_of(b"Failed"))]),
            ResultPhrase::Cancelled => Box::new([Segment::Text(copy_of(b"Cancelled"))]),
            ResultPhrase::StatusUnknown => Box::new([Segment::Text(copy_of(b"Stored status unavailable"))]),
            ResultPhrase::Cut => Box::new([Segment::Text(copy_of(b"Some result metadata was omitted"))]),
        };
        rows.push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("closed result words fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Minutes,
        wall: WallFormat::Clock,
        phrases: rows.into_boxed(),
    }
}
/// Largest result saying across every checked language.
#[must_use]
pub fn phrase_bytes(book: &crate::phrases::Phrasebook) -> Option<u32> {
    let mut largest = 0;
    for phrase in ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    Some(largest)
}
