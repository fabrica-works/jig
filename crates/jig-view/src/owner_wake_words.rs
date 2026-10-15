//! Checked wake language data, composable with the normal jig phrasebook.
//! See client/domain/view.md, section 7. IDs 4000–4026 are reserved for these phrases.
use crate::{
    Owner,
    phrases::{
        Form, HoleKind, HoleShape, Invalid, Language, Phrase, PhraseId, PhraseLimits, PhraseShape, Phrasebook,
        PluralClass, PluralRule, Segment, SpanUnit, WallFormat,
    },
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};

/// Closed phrases for exact owner wake facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WakePhrase {
    Policy,
    Words,
    Notices,
    News,
    Never,
    Immediate,
    Batch,
    ResultsNever,
    ResultsEach,
    ResultsLastOrFailure,
    QuestionsYes,
    QuestionsNo,
    AnswersYes,
    AnswersNo,
    TimersYes,
    TimersNo,
    Subscriptions,
    Number,
    Task,
    HeldYes,
    HeldNo,
    ResultYes,
    ResultNo,
    Timer,
    Period,
    Once,
    Topic,
}
pub const ALL: [WakePhrase; 27] = [
    WakePhrase::Policy,
    WakePhrase::Words,
    WakePhrase::Notices,
    WakePhrase::News,
    WakePhrase::Never,
    WakePhrase::Immediate,
    WakePhrase::Batch,
    WakePhrase::ResultsNever,
    WakePhrase::ResultsEach,
    WakePhrase::ResultsLastOrFailure,
    WakePhrase::QuestionsYes,
    WakePhrase::QuestionsNo,
    WakePhrase::AnswersYes,
    WakePhrase::AnswersNo,
    WakePhrase::TimersYes,
    WakePhrase::TimersNo,
    WakePhrase::Subscriptions,
    WakePhrase::Number,
    WakePhrase::Task,
    WakePhrase::HeldYes,
    WakePhrase::HeldNo,
    WakePhrase::ResultYes,
    WakePhrase::ResultNo,
    WakePhrase::Timer,
    WakePhrase::Period,
    WakePhrase::Once,
    WakePhrase::Topic,
];
impl WakePhrase {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        let number = match self {
            Self::Policy => 4000,
            Self::Words => 4001,
            Self::Notices => 4002,
            Self::News => 4003,
            Self::Never => 4004,
            Self::Immediate => 4005,
            Self::Batch => 4006,
            Self::ResultsNever => 4007,
            Self::ResultsEach => 4008,
            Self::ResultsLastOrFailure => 4009,
            Self::QuestionsYes => 4010,
            Self::QuestionsNo => 4011,
            Self::AnswersYes => 4012,
            Self::AnswersNo => 4013,
            Self::TimersYes => 4014,
            Self::TimersNo => 4015,
            Self::Subscriptions => 4016,
            Self::Number => 4017,
            Self::Task => 4018,
            Self::HeldYes => 4019,
            Self::HeldNo => 4020,
            Self::ResultYes => 4021,
            Self::ResultNo => 4022,
            Self::Timer => 4023,
            Self::Period => 4024,
            Self::Once => 4025,
            Self::Topic => 4026,
        };
        PhraseId { owner: Owner::Jig.opaque(), number }
    }
    #[must_use]
    pub fn shape(self) -> PhraseShape {
        let holes: Box<[HoleShape]> = match self {
            Self::Policy
            | Self::Words
            | Self::Notices
            | Self::News
            | Self::Never
            | Self::Immediate
            | Self::ResultsNever
            | Self::ResultsEach
            | Self::ResultsLastOrFailure
            | Self::QuestionsYes
            | Self::QuestionsNo
            | Self::AnswersYes
            | Self::AnswersNo
            | Self::TimersYes
            | Self::TimersNo
            | Self::Subscriptions
            | Self::HeldYes
            | Self::HeldNo
            | Self::ResultYes
            | Self::ResultNo
            | Self::Once => Box::new([]),
            Self::Batch | Self::Topic => Box::new([
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Number, bytes: 0 },
            ]),
            Self::Number | Self::Task | Self::Timer | Self::Period => {
                Box::new([HoleShape { kind: HoleKind::Number, bytes: 0 }])
            }
        };
        PhraseShape { id: self.id(), holes, plural: None }
    }
}
/// Shapes for a composing phrasebook.
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut rows = List::with_capacity(27);
    for phrase in ALL {
        rows.push(phrase.shape()).expect("all wake shapes fit");
    }
    rows.into_boxed()
}
/// English forms retain every numeric field; durations and times use nanoseconds.
#[must_use]
pub fn language() -> Language {
    let mut phrases = List::with_capacity(27);
    for phrase in ALL {
        let segments: Box<[Segment]> = match phrase {
            WakePhrase::Policy => Box::new([Segment::Text(copy_of(b"Wake policy"))]),
            WakePhrase::Words => Box::new([Segment::Text(copy_of(b"Words"))]),
            WakePhrase::Notices => Box::new([Segment::Text(copy_of(b"Notices"))]),
            WakePhrase::News => Box::new([Segment::Text(copy_of(b"News"))]),
            WakePhrase::Never => Box::new([Segment::Text(copy_of(b"Does not wake"))]),
            WakePhrase::Immediate => Box::new([Segment::Text(copy_of(b"Wakes immediately"))]),
            WakePhrase::Batch => Box::new([
                Segment::Text(copy_of(b"Wakes after ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" messages or ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b" ns of oldest message age")),
            ]),
            WakePhrase::ResultsNever => Box::new([Segment::Text(copy_of(b"Delegate results do not wake"))]),
            WakePhrase::ResultsEach => Box::new([Segment::Text(copy_of(b"Each delegate result wakes"))]),
            WakePhrase::ResultsLastOrFailure => {
                Box::new([Segment::Text(copy_of(b"The last delegate result or a failure wakes"))])
            }
            WakePhrase::QuestionsYes => Box::new([Segment::Text(copy_of(b"Questions wake"))]),
            WakePhrase::QuestionsNo => Box::new([Segment::Text(copy_of(b"Questions do not wake"))]),
            WakePhrase::AnswersYes => Box::new([Segment::Text(copy_of(b"Answers wake"))]),
            WakePhrase::AnswersNo => Box::new([Segment::Text(copy_of(b"Answers do not wake"))]),
            WakePhrase::TimersYes => Box::new([Segment::Text(copy_of(b"Timers wake"))]),
            WakePhrase::TimersNo => Box::new([Segment::Text(copy_of(b"Timers do not wake"))]),
            WakePhrase::Subscriptions => Box::new([Segment::Text(copy_of(b"Subscriptions"))]),
            WakePhrase::Number => {
                Box::new([Segment::Text(copy_of(b"Subscription ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            WakePhrase::Task => {
                Box::new([Segment::Text(copy_of(b"Task ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            WakePhrase::HeldYes => Box::new([Segment::Text(copy_of(b"Held state is subscribed"))]),
            WakePhrase::HeldNo => Box::new([Segment::Text(copy_of(b"Held state is not subscribed"))]),
            WakePhrase::ResultYes => Box::new([Segment::Text(copy_of(b"Result is subscribed"))]),
            WakePhrase::ResultNo => Box::new([Segment::Text(copy_of(b"Result is not subscribed"))]),
            WakePhrase::Timer => Box::new([
                Segment::Text(copy_of(b"Timer at ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" ns since epoch")),
            ]),
            WakePhrase::Period => Box::new([
                Segment::Text(copy_of(b"Repeats every ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" ns")),
            ]),
            WakePhrase::Once => Box::new([Segment::Text(copy_of(b"Fires once"))]),
            WakePhrase::Topic => Box::new([
                Segment::Text(copy_of(b"Connector ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", topic ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
        };
        phrases
            .push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("all wake English fits");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: phrases.into_boxed(),
    }
}
/// A checked standalone English phrasebook; callers select the loaded language.
pub fn english() -> Result<Phrasebook, Invalid> {
    Phrasebook::load(
        shapes(),
        Box::new([language()]),
        &PhraseLimits { languages: 1, phrases: 27, holes: 2, segments: 100, text_bytes: 2000, hole_bytes: 0 },
    )
}
