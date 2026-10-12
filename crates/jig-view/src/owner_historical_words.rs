//! Passive historical object sayings (client/domain/view.md, 7).
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
pub enum HistoricalPhrase {
    Historical,
    At,
    AtAbsent,
    By,
    ByAbsent,
    Task,
    PersonTask,
    TaskProposal,
    PersonProposal,
    Escalation,
    Question,
    Pending,
    Accepted,
    Rejected,
    Release,
    Pass,
    DecisionAbsent,
    HoldAbsent,
    AnswerAbsent,
    EndingAbsent,
    CreatedAbsent,
    TrackedAbsent,
    Number,
    Project,
    Proposer,
    Goal,
    Budget,
    Spent,
    SpentBelow,
    Reserved,
    Dependency,
    Write,
    Slot,
    Connector,
    Kind,
    Tracked,
    Revision,
    Requester,
    DecisionBy,
    DecisionAt,
    FirstCreated,
    LastCreated,
    Message,
    Since,
    AcceptChoice,
    RejectChoice,
    BatchKind,
    EffectKind,
    AmendKind,
    WidenKind,
    ReleaseKind,
    Withdrawn,
    Stale,
}
pub const ALL: [HistoricalPhrase; 53] = [
    HistoricalPhrase::Historical,
    HistoricalPhrase::At,
    HistoricalPhrase::AtAbsent,
    HistoricalPhrase::By,
    HistoricalPhrase::ByAbsent,
    HistoricalPhrase::Task,
    HistoricalPhrase::PersonTask,
    HistoricalPhrase::TaskProposal,
    HistoricalPhrase::PersonProposal,
    HistoricalPhrase::Escalation,
    HistoricalPhrase::Question,
    HistoricalPhrase::Pending,
    HistoricalPhrase::Accepted,
    HistoricalPhrase::Rejected,
    HistoricalPhrase::Release,
    HistoricalPhrase::Pass,
    HistoricalPhrase::DecisionAbsent,
    HistoricalPhrase::HoldAbsent,
    HistoricalPhrase::AnswerAbsent,
    HistoricalPhrase::EndingAbsent,
    HistoricalPhrase::CreatedAbsent,
    HistoricalPhrase::TrackedAbsent,
    HistoricalPhrase::Number,
    HistoricalPhrase::Project,
    HistoricalPhrase::Proposer,
    HistoricalPhrase::Goal,
    HistoricalPhrase::Budget,
    HistoricalPhrase::Spent,
    HistoricalPhrase::SpentBelow,
    HistoricalPhrase::Reserved,
    HistoricalPhrase::Dependency,
    HistoricalPhrase::Write,
    HistoricalPhrase::Slot,
    HistoricalPhrase::Connector,
    HistoricalPhrase::Kind,
    HistoricalPhrase::Tracked,
    HistoricalPhrase::Revision,
    HistoricalPhrase::Requester,
    HistoricalPhrase::DecisionBy,
    HistoricalPhrase::DecisionAt,
    HistoricalPhrase::FirstCreated,
    HistoricalPhrase::LastCreated,
    HistoricalPhrase::Message,
    HistoricalPhrase::Since,
    HistoricalPhrase::AcceptChoice,
    HistoricalPhrase::RejectChoice,
    HistoricalPhrase::BatchKind,
    HistoricalPhrase::EffectKind,
    HistoricalPhrase::AmendKind,
    HistoricalPhrase::WidenKind,
    HistoricalPhrase::ReleaseKind,
    HistoricalPhrase::Withdrawn,
    HistoricalPhrase::Stale,
];
impl HistoricalPhrase {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        PhraseId {
            owner: Owner::Jig,
            number: match self {
                Self::Historical => 9600,
                Self::At => 9601,
                Self::AtAbsent => 9602,
                Self::By => 9603,
                Self::ByAbsent => 9604,
                Self::Task => 9605,
                Self::PersonTask => 9606,
                Self::TaskProposal => 9607,
                Self::PersonProposal => 9608,
                Self::Escalation => 9609,
                Self::Question => 9610,
                Self::Pending => 9611,
                Self::Accepted => 9612,
                Self::Rejected => 9613,
                Self::Release => 9614,
                Self::Pass => 9615,
                Self::DecisionAbsent => 9616,
                Self::HoldAbsent => 9617,
                Self::AnswerAbsent => 9618,
                Self::EndingAbsent => 9619,
                Self::CreatedAbsent => 9620,
                Self::TrackedAbsent => 9621,
                Self::Number => 9622,
                Self::Project => 9623,
                Self::Proposer => 9624,
                Self::Goal => 9625,
                Self::Budget => 9626,
                Self::Spent => 9627,
                Self::SpentBelow => 9628,
                Self::Reserved => 9629,
                Self::Dependency => 9630,
                Self::Write => 9631,
                Self::Slot => 9632,
                Self::Connector => 9633,
                Self::Kind => 9634,
                Self::Tracked => 9635,
                Self::Revision => 9636,
                Self::Requester => 9637,
                Self::DecisionBy => 9638,
                Self::DecisionAt => 9639,
                Self::FirstCreated => 9640,
                Self::LastCreated => 9641,
                Self::Message => 9642,
                Self::Since => 9643,
                Self::AcceptChoice => 9644,
                Self::RejectChoice => 9645,
                Self::BatchKind => 9646,
                Self::EffectKind => 9647,
                Self::AmendKind => 9648,
                Self::WidenKind => 9649,
                Self::ReleaseKind => 9650,
                Self::Withdrawn => 9651,
                Self::Stale => 9652,
            },
        }
    }
    fn hole(self) -> Option<HoleKind> {
        match self {
            Self::Historical
            | Self::AtAbsent
            | Self::By
            | Self::ByAbsent
            | Self::Task
            | Self::PersonTask
            | Self::TaskProposal
            | Self::PersonProposal
            | Self::Escalation
            | Self::Question
            | Self::Pending
            | Self::Accepted
            | Self::Rejected
            | Self::Release
            | Self::Pass
            | Self::DecisionAbsent
            | Self::HoldAbsent
            | Self::AnswerAbsent
            | Self::EndingAbsent
            | Self::CreatedAbsent
            | Self::TrackedAbsent
            | Self::Write
            | Self::Slot
            | Self::AcceptChoice
            | Self::RejectChoice
            | Self::BatchKind
            | Self::EffectKind
            | Self::AmendKind
            | Self::WidenKind
            | Self::ReleaseKind
            | Self::Withdrawn
            | Self::Stale => None,
            Self::At | Self::DecisionAt | Self::Since => Some(HoleKind::Wall),
            Self::Number
            | Self::Project
            | Self::Proposer
            | Self::Goal
            | Self::Dependency
            | Self::Connector
            | Self::Kind
            | Self::Tracked
            | Self::Revision
            | Self::Requester
            | Self::DecisionBy
            | Self::FirstCreated
            | Self::LastCreated
            | Self::Message => Some(HoleKind::Number),
            Self::Budget | Self::Spent | Self::SpentBelow | Self::Reserved => Some(HoleKind::Amount),
        }
    }
    fn label(self) -> &'static [u8] {
        match self {
            Self::Historical => b"Historical object",
            Self::At => b"Historical record time: ",
            Self::AtAbsent => b"Historical record time not supplied",
            Self::By => b"Historical record sender",
            Self::ByAbsent => b"Historical record sender not supplied",
            Self::Task => b"Historical task",
            Self::PersonTask => b"Historical person task",
            Self::TaskProposal => b"Historical task proposal",
            Self::PersonProposal => b"Historical person proposal",
            Self::Escalation => b"Historical escalation",
            Self::Question => b"Historical question",
            Self::Pending => b"Pending",
            Self::Accepted => b"Accepted",
            Self::Rejected => b"Rejected",
            Self::Release => b"Released",
            Self::Pass => b"Passed",
            Self::DecisionAbsent => b"Decision not supplied",
            Self::HoldAbsent => b"Retained hold not supplied",
            Self::AnswerAbsent => b"Answer not supplied",
            Self::EndingAbsent => b"Ending not supplied",
            Self::CreatedAbsent => b"Created task pair not supplied",
            Self::TrackedAbsent => b"Tracking not supplied",
            Self::Number => b"Number: ",
            Self::Project => b"Project: ",
            Self::Proposer => b"Proposer: ",
            Self::Goal => b"Goal: ",
            Self::Budget => b"Budget: ",
            Self::Spent => b"Spent: ",
            Self::SpentBelow => b"Spent below: ",
            Self::Reserved => b"Reserved: ",
            Self::Dependency => b"Dependency task: ",
            Self::Write => b"Write holding",
            Self::Slot => b"Slot holding",
            Self::Connector => b"Connector: ",
            Self::Kind => b"Kind: ",
            Self::Tracked => b"Tracked: ",
            Self::Revision => b"Revision: ",
            Self::Requester => b"Requester task: ",
            Self::DecisionBy => b"Decision person: ",
            Self::DecisionAt => b"Decision time: ",
            Self::FirstCreated => b"First created task: ",
            Self::LastCreated => b"Last created task: ",
            Self::Message => b"Message: ",
            Self::Since => b"Pending since: ",
            Self::AcceptChoice => b"Accept decision",
            Self::RejectChoice => b"Reject decision",
            Self::BatchKind => b"Batch decision",
            Self::EffectKind => b"Effect decision",
            Self::AmendKind => b"Amendment decision",
            Self::WidenKind => b"Authority decision",
            Self::ReleaseKind => b"Release decision",
            Self::Withdrawn => b"Withdrawn decision",
            Self::Stale => b"Stale decision",
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
