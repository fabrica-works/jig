//! Pending person work sayings (client/domain/view.md, 7).
use crate::{
    Owner,
    phrases::{
        Form, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment, SpanUnit,
        WallFormat,
    },
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersonPhrase {
    PersonTask,
    Party,
    Role,
    Taker,
    TakerAbsent,
    Since,
    SinceAbsent,
    Stall,
    StallAbsent,
    Deadline,
    DeadlineAbsent,
    Results,
    MissingResult,
    Cut,
    Choice,
    FieldsUnavailable,
    ChangeUnavailable,
    Revision,
}
pub const ALL: [PersonPhrase; 18] = [
    PersonPhrase::PersonTask,
    PersonPhrase::Party,
    PersonPhrase::Role,
    PersonPhrase::Taker,
    PersonPhrase::TakerAbsent,
    PersonPhrase::Since,
    PersonPhrase::SinceAbsent,
    PersonPhrase::Stall,
    PersonPhrase::StallAbsent,
    PersonPhrase::Deadline,
    PersonPhrase::DeadlineAbsent,
    PersonPhrase::Results,
    PersonPhrase::MissingResult,
    PersonPhrase::Cut,
    PersonPhrase::Choice,
    PersonPhrase::FieldsUnavailable,
    PersonPhrase::ChangeUnavailable,
    PersonPhrase::Revision,
];
impl PersonPhrase {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        PhraseId {
            owner: Owner::Jig.opaque(),
            number: match self {
                Self::PersonTask => 8600,
                Self::Party => 8601,
                Self::Role => 8602,
                Self::Taker => 8603,
                Self::TakerAbsent => 8604,
                Self::Since => 8605,
                Self::SinceAbsent => 8606,
                Self::Stall => 8607,
                Self::StallAbsent => 8608,
                Self::Deadline => 8609,
                Self::DeadlineAbsent => 8610,
                Self::Results => 8611,
                Self::MissingResult => 8612,
                Self::Cut => 8613,
                Self::Choice => 8614,
                Self::FieldsUnavailable => 8615,
                Self::ChangeUnavailable => 8616,
                Self::Revision => 8617,
            },
        }
    }
    fn hole(self) -> Option<HoleKind> {
        match self {
            Self::Party | Self::Role | Self::Taker | Self::Choice | Self::Revision => Some(HoleKind::Number),
            Self::Since | Self::Deadline => Some(HoleKind::Wall),
            Self::Stall => Some(HoleKind::Span),
            Self::PersonTask
            | Self::TakerAbsent
            | Self::SinceAbsent
            | Self::StallAbsent
            | Self::DeadlineAbsent
            | Self::Results
            | Self::MissingResult
            | Self::Cut
            | Self::FieldsUnavailable
            | Self::ChangeUnavailable => None,
        }
    }
    fn label(self) -> &'static [u8] {
        match self {
            Self::PersonTask => b"Person task",
            Self::Party => b"Person destination: ",
            Self::Role => b"Role destination: ",
            Self::Taker => b"Taken by person: ",
            Self::TakerAbsent => b"Taker not supplied",
            Self::Since => b"Waiting since: ",
            Self::SinceAbsent => b"Waiting time not supplied",
            Self::Stall => b"Stall duration: ",
            Self::StallAbsent => b"Stall duration not supplied",
            Self::Deadline => b"Deadline: ",
            Self::DeadlineAbsent => b"Deadline not supplied",
            Self::Results => b"Shown results",
            Self::MissingResult => b"Result not supplied",
            Self::Cut => b"Some shown results were omitted",
            Self::Choice => b"Choice code: ",
            Self::FieldsUnavailable => b"This choice requires fields the person answer cannot carry",
            Self::ChangeUnavailable => b"This contract has no positive person answer form",
            Self::Revision => b"Person revision: ",
        }
    }
}
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut result = List::with_capacity(u32::try_from(ALL.len()).expect("person phrases fit"));
    for phrase in ALL {
        let holes = match phrase.hole() {
            Some(kind) => Box::from([HoleShape { kind, bytes: 0 }]),
            None => Box::from([]),
        };
        result.push(PhraseShape { id: phrase.id(), holes, plural: None }).expect("person shapes fit");
    }
    result.into_boxed()
}
#[must_use]
pub fn language() -> Language {
    let mut result = List::with_capacity(u32::try_from(ALL.len()).expect("person phrases fit"));
    for phrase in ALL {
        let mut segments = List::with_capacity(2);
        segments.push(Segment::Text(copy_of(phrase.label()))).expect("person label fits");
        if let Some(kind) = phrase.hole() {
            segments.push(Segment::Hole { index: 0, kind }).expect("person hole fits");
        }
        result
            .push(Phrase {
                id: phrase.id(),
                forms: Box::new([Form { class: PluralClass::Other, segments: segments.into_boxed() }]),
            })
            .expect("person phrases fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: result.into_boxed(),
    }
}
