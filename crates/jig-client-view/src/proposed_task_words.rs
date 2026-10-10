//! Checked words for exact batch member facts (client/domain/view.md, 7).
//! IDs 5000–5023 are reserved for this closed vocabulary.
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
pub enum ProposedPhrase {
    ExecutorCharter,
    ExecutorProcedure,
    ExecutorParty,
    ExecutorRole,
    FunderTask,
    FunderPool,
    FunderRecurring,
    FunderPeriod,
    ContractReport,
    ContractApproval,
    ContractVerdict,
    ContractChange,
    VerdictChoice,
    FieldCap,
    ContractCut,
    TaskCut,
    PartConnector,
    PartNumber,
    PartResource,
    PartPool,
    PartResult,
    PartProcedure,
    PartEffect,
    PartOutbox,
}
pub const ALL: [ProposedPhrase; 24] = [
    ProposedPhrase::ExecutorCharter,
    ProposedPhrase::ExecutorProcedure,
    ProposedPhrase::ExecutorParty,
    ProposedPhrase::ExecutorRole,
    ProposedPhrase::FunderTask,
    ProposedPhrase::FunderPool,
    ProposedPhrase::FunderRecurring,
    ProposedPhrase::FunderPeriod,
    ProposedPhrase::ContractReport,
    ProposedPhrase::ContractApproval,
    ProposedPhrase::ContractVerdict,
    ProposedPhrase::ContractChange,
    ProposedPhrase::VerdictChoice,
    ProposedPhrase::FieldCap,
    ProposedPhrase::ContractCut,
    ProposedPhrase::TaskCut,
    ProposedPhrase::PartConnector,
    ProposedPhrase::PartNumber,
    ProposedPhrase::PartResource,
    ProposedPhrase::PartPool,
    ProposedPhrase::PartResult,
    ProposedPhrase::PartProcedure,
    ProposedPhrase::PartEffect,
    ProposedPhrase::PartOutbox,
];
impl ProposedPhrase {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        let number = match self {
            Self::ExecutorCharter => 5000,
            Self::ExecutorProcedure => 5001,
            Self::ExecutorParty => 5002,
            Self::ExecutorRole => 5003,
            Self::FunderTask => 5004,
            Self::FunderPool => 5005,
            Self::FunderRecurring => 5006,
            Self::FunderPeriod => 5007,
            Self::ContractReport => 5008,
            Self::ContractApproval => 5009,
            Self::ContractVerdict => 5010,
            Self::ContractChange => 5011,
            Self::VerdictChoice => 5012,
            Self::FieldCap => 5013,
            Self::ContractCut => 5014,
            Self::TaskCut => 5015,
            Self::PartConnector => 5016,
            Self::PartNumber => 5017,
            Self::PartResource => 5018,
            Self::PartPool => 5019,
            Self::PartResult => 5020,
            Self::PartProcedure => 5021,
            Self::PartEffect => 5022,
            Self::PartOutbox => 5023,
        };
        PhraseId { owner: Owner::Jig, number }
    }
    #[must_use]
    pub fn shape(self) -> PhraseShape {
        let count = match self {
            Self::ExecutorCharter => 1,
            Self::ExecutorProcedure => 2,
            Self::ExecutorParty => 1,
            Self::ExecutorRole => 1,
            Self::FunderTask => 1,
            Self::FunderPool => 3,
            Self::FunderRecurring => 3,
            Self::FunderPeriod => 2,
            Self::ContractReport => 1,
            Self::ContractApproval => 0,
            Self::ContractVerdict => 0,
            Self::ContractChange => 3,
            Self::VerdictChoice => 3,
            Self::FieldCap => 1,
            Self::ContractCut => 0,
            Self::TaskCut => 0,
            Self::PartConnector => 1,
            Self::PartNumber => 1,
            Self::PartResource => 0,
            Self::PartPool => 0,
            Self::PartResult => 0,
            Self::PartProcedure => 0,
            Self::PartEffect => 0,
            Self::PartOutbox => 0,
        };
        let mut holes = List::with_capacity(count);
        for _ in 0..count {
            holes.push(HoleShape { kind: HoleKind::Number, bytes: 0 }).expect("typed holes fit");
        }
        PhraseShape { id: self.id(), holes: holes.into_boxed(), plural: None }
    }
}
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut rows = List::with_capacity(u32::try_from(ALL.len()).expect("phrase count fits"));
    for phrase in ALL {
        rows.push(phrase.shape()).expect("all shapes fit");
    }
    rows.into_boxed()
}
#[must_use]
pub fn language() -> Language {
    let mut rows = List::with_capacity(u32::try_from(ALL.len()).expect("phrase count fits"));
    for phrase in ALL {
        let segments: Box<[Segment]> = match phrase {
            ProposedPhrase::ExecutorCharter => Box::new([
                Segment::Text(copy_of(b"Agent charter ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            ProposedPhrase::ExecutorProcedure => Box::new([
                Segment::Text(copy_of(b"Procedure connector ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", code ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
            ProposedPhrase::ExecutorParty => {
                Box::new([Segment::Text(copy_of(b"Person ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            ProposedPhrase::ExecutorRole => {
                Box::new([Segment::Text(copy_of(b"Project role ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            ProposedPhrase::FunderTask => Box::new([
                Segment::Text(copy_of(b"Funded by task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            ProposedPhrase::FunderPool => Box::new([
                Segment::Text(copy_of(b"Funded by pool in project ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", person ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b", period ")),
                Segment::Hole { index: 2, kind: HoleKind::Number },
            ]),
            ProposedPhrase::FunderRecurring => Box::new([
                Segment::Text(copy_of(b"Funded by recurring task in project ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", task ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b", period ")),
                Segment::Hole { index: 2, kind: HoleKind::Number },
            ]),
            ProposedPhrase::FunderPeriod => Box::new([
                Segment::Text(copy_of(b"Funded by project period ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", period ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
            ProposedPhrase::ContractReport => Box::new([
                Segment::Text(copy_of(b"Report word byte limit ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            ProposedPhrase::ContractApproval => Box::new([Segment::Text(copy_of(b"Approval of exact state"))]),
            ProposedPhrase::ContractVerdict => Box::new([Segment::Text(copy_of(b"Verdict contract"))]),
            ProposedPhrase::ContractChange => Box::new([
                Segment::Text(copy_of(b"Change contract ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", kind ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b", word byte limit ")),
                Segment::Hole { index: 2, kind: HoleKind::Number },
            ]),
            ProposedPhrase::VerdictChoice => Box::new([
                Segment::Text(copy_of(b"Verdict choice ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", word byte limit ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b", follow-up limit ")),
                Segment::Hole { index: 2, kind: HoleKind::Number },
            ]),
            ProposedPhrase::FieldCap => Box::new([
                Segment::Text(copy_of(b"Field value byte limit ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            ProposedPhrase::ContractCut => Box::new([Segment::Text(copy_of(b"Some contract choices were omitted"))]),
            ProposedPhrase::TaskCut => Box::new([Segment::Text(copy_of(b"Some proposed task facts were omitted"))]),
            ProposedPhrase::PartConnector => {
                Box::new([Segment::Text(copy_of(b"Connector ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            ProposedPhrase::PartNumber => {
                Box::new([Segment::Text(copy_of(b"Part number ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            ProposedPhrase::PartResource => Box::new([Segment::Text(copy_of(b"Resource reference"))]),
            ProposedPhrase::PartPool => Box::new([Segment::Text(copy_of(b"Pool reference"))]),
            ProposedPhrase::PartResult => Box::new([Segment::Text(copy_of(b"Result reference"))]),
            ProposedPhrase::PartProcedure => Box::new([Segment::Text(copy_of(b"Procedure reference"))]),
            ProposedPhrase::PartEffect => Box::new([Segment::Text(copy_of(b"Effect reference"))]),
            ProposedPhrase::PartOutbox => Box::new([Segment::Text(copy_of(b"Outbox reference"))]),
        };
        rows.push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("all phrases fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Minutes,
        wall: WallFormat::Clock,
        phrases: rows.into_boxed(),
    }
}
