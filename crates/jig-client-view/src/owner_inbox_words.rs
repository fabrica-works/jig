//! Exact Inbox sayings, composed into each checked normal book (view.md, 7).
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
pub enum InboxPhrase {
    Question,
    TaskProposal,
    PartyProposal,
    Escalation,
    PersonTask,
    Reply,
    Result,
    Project,
    Activity,
    GoalContextPending,
}
pub const ALL: [InboxPhrase; 10] = [
    InboxPhrase::Question,
    InboxPhrase::TaskProposal,
    InboxPhrase::PartyProposal,
    InboxPhrase::Escalation,
    InboxPhrase::PersonTask,
    InboxPhrase::Reply,
    InboxPhrase::Result,
    InboxPhrase::Project,
    InboxPhrase::Activity,
    InboxPhrase::GoalContextPending,
];
impl InboxPhrase {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        PhraseId {
            owner: Owner::Jig,
            number: match self {
                Self::Question => 8200,
                Self::TaskProposal => 8201,
                Self::PartyProposal => 8202,
                Self::Escalation => 8203,
                Self::PersonTask => 8204,
                Self::Reply => 8205,
                Self::Result => 8206,
                Self::Project => 8207,
                Self::Activity => 8208,
                Self::GoalContextPending => 8209,
            },
        }
    }
    fn holes(self) -> u32 {
        match self {
            Self::Question | Self::Escalation | Self::Reply | Self::Result | Self::Project | Self::Activity => 1,
            Self::TaskProposal | Self::PartyProposal => 2,
            Self::PersonTask | Self::GoalContextPending => 0,
        }
    }
    fn label(self) -> &'static [u8] {
        match self {
            Self::Question => b"Question message: ",
            Self::TaskProposal => b"Task proposer: ",
            Self::PartyProposal => b"Party proposer: ",
            Self::Escalation => b"Escalation revision: ",
            Self::PersonTask => b"Person task",
            Self::Reply => b"Reply message: ",
            Self::Result => b"Result position: ",
            Self::Project => b"Project: ",
            Self::Activity => b"Activity position: ",
            Self::GoalContextPending => b"Live goal membership awaits source context",
        }
    }
}
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut values = List::with_capacity(10);
    for phrase in ALL {
        let mut holes = List::with_capacity(phrase.holes());
        for _ in 0..phrase.holes() {
            holes.push(HoleShape { kind: HoleKind::Number, bytes: 0 }).expect("closed Inbox holes fit");
        }
        values
            .push(PhraseShape { id: phrase.id(), holes: holes.into_boxed(), plural: None })
            .expect("Inbox shapes fit");
    }
    values.into_boxed()
}
#[must_use]
pub fn language() -> Language {
    let mut values = List::with_capacity(10);
    for phrase in ALL {
        let mut segments = List::with_capacity(
            phrase.holes().checked_mul(2).expect("closed holes fit").checked_add(1).expect("closed label fits"),
        );
        segments.push(Segment::Text(copy_of(phrase.label()))).expect("label fits");
        for index in 0..phrase.holes() {
            if index > 0 {
                segments.push(Segment::Text(copy_of(b" / proposal: "))).expect("separator fits");
            }
            segments
                .push(Segment::Hole {
                    index: u16::try_from(index).expect("closed hole index fits"),
                    kind: HoleKind::Number,
                })
                .expect("hole fits");
        }
        values
            .push(Phrase {
                id: phrase.id(),
                forms: Box::new([Form { class: PluralClass::Other, segments: segments.into_boxed() }]),
            })
            .expect("Inbox phrases fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: values.into_boxed(),
    }
}
