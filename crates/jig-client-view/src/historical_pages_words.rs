//! Retained page sayings (client/domain/view.md, 7).
use crate::{
    Owner,
    phrases::{
        Form, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment, SpanUnit,
        WallFormat,
    },
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
/// Closed captions for supplied historical facts and their absence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoricalPhrase {
    EndedDelegates,
    EndedRow,
    At,
    AtAbsent,
    FundingHistory,
    PeriodAbsent,
    Began,
    Next,
    NextAbsent,
    ParentAbsent,
    Parent,
    Made,
    Open,
    Closed,
    Budget,
    Spent,
    SpentBelow,
    Reserved,
}
pub const ALL: [HistoricalPhrase; 18] = [
    HistoricalPhrase::EndedDelegates,
    HistoricalPhrase::EndedRow,
    HistoricalPhrase::At,
    HistoricalPhrase::AtAbsent,
    HistoricalPhrase::FundingHistory,
    HistoricalPhrase::PeriodAbsent,
    HistoricalPhrase::Began,
    HistoricalPhrase::Next,
    HistoricalPhrase::NextAbsent,
    HistoricalPhrase::ParentAbsent,
    HistoricalPhrase::Parent,
    HistoricalPhrase::Made,
    HistoricalPhrase::Open,
    HistoricalPhrase::Closed,
    HistoricalPhrase::Budget,
    HistoricalPhrase::Spent,
    HistoricalPhrase::SpentBelow,
    HistoricalPhrase::Reserved,
];
impl HistoricalPhrase {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        PhraseId {
            owner: Owner::Jig,
            number: match self {
                Self::EndedDelegates => 9500,
                Self::EndedRow => 9501,
                Self::At => 9502,
                Self::AtAbsent => 9503,
                Self::FundingHistory => 9504,
                Self::PeriodAbsent => 9505,
                Self::Began => 9506,
                Self::Next => 9507,
                Self::NextAbsent => 9508,
                Self::ParentAbsent => 9509,
                Self::Parent => 9510,
                Self::Made => 9511,
                Self::Open => 9512,
                Self::Closed => 9513,
                Self::Budget => 9514,
                Self::Spent => 9515,
                Self::SpentBelow => 9516,
                Self::Reserved => 9517,
            },
        }
    }
    fn hole(self) -> Option<HoleKind> {
        match self {
            Self::EndedDelegates => None,
            Self::EndedRow => Some(HoleKind::Number),
            Self::At => Some(HoleKind::Wall),
            Self::AtAbsent => None,
            Self::FundingHistory => None,
            Self::PeriodAbsent => None,
            Self::Began => Some(HoleKind::Wall),
            Self::Next => Some(HoleKind::Wall),
            Self::NextAbsent => None,
            Self::ParentAbsent => None,
            Self::Parent => None,
            Self::Made => Some(HoleKind::Number),
            Self::Open => None,
            Self::Closed => None,
            Self::Budget => Some(HoleKind::Amount),
            Self::Spent => Some(HoleKind::Amount),
            Self::SpentBelow => Some(HoleKind::Amount),
            Self::Reserved => Some(HoleKind::Amount),
        }
    }
    fn label(self) -> &'static [u8] {
        match self {
            Self::EndedDelegates => b"Ended delegates",
            Self::EndedRow => b"Ended delegate task: ",
            Self::At => b"Recorded ending time: ",
            Self::AtAbsent => b"Ending time not supplied",
            Self::FundingHistory => b"Historical funding records",
            Self::PeriodAbsent => b"Period times not supplied",
            Self::Began => b"Period began: ",
            Self::Next => b"Next period begins: ",
            Self::NextAbsent => b"Next period time not supplied",
            Self::ParentAbsent => b"Parent funder not supplied",
            Self::Parent => b"Parent funder",
            Self::Made => b"Tasks made: ",
            Self::Open => b"Funding record open",
            Self::Closed => b"Funding record closed",
            Self::Budget => b"Budget: ",
            Self::Spent => b"Spent by this funder: ",
            Self::SpentBelow => b"Spent below this funder: ",
            Self::Reserved => b"Reserved: ",
        }
    }
}
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut result = List::with_capacity(u32::try_from(ALL.len()).expect("historical phrases fit"));
    for phrase in ALL {
        let holes = match phrase.hole() {
            Some(kind) => Box::from([HoleShape { kind, bytes: 0 }]),
            None => Box::from([]),
        };
        result.push(PhraseShape { id: phrase.id(), holes, plural: None }).expect("historical shapes fit");
    }
    result.into_boxed()
}
#[must_use]
pub fn language() -> Language {
    let mut result = List::with_capacity(u32::try_from(ALL.len()).expect("historical phrases fit"));
    for phrase in ALL {
        let mut segments = List::with_capacity(2);
        segments.push(Segment::Text(copy_of(phrase.label()))).expect("historical label fits");
        if let Some(kind) = phrase.hole() {
            segments.push(Segment::Hole { index: 0, kind }).expect("historical hole fits");
        }
        result
            .push(Phrase {
                id: phrase.id(),
                forms: Box::new([Form { class: PluralClass::Other, segments: segments.into_boxed() }]),
            })
            .expect("historical phrases fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: result.into_boxed(),
    }
}
