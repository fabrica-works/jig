//! Closed immutable-history sayings (client/domain/view.md, 7).
//! IDs7600..7655 belong to this additive fragment, independent of request editing.
use crate::Owner;
use crate::ui::phrases::{
    Form, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment, SpanUnit,
    WallFormat,
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryPhrase {
    History,
    Task,
    Revision,
    Cancelled,
    Released,
    Amended,
    Prioritised,
    Moved,
    Proposed,
    Passed,
    Accepted,
    Rejected,
    Withdrawn,
    Reason,
    At,
    AtAbsent,
    SnapshotsAbsent,
    Stored,
    Before,
    After,
    Proposal,
    Proposer,
    AsHolder,
    NotAsHolder,
    Pending,
    Since,
    HolderTask,
    HolderPerson,
    HolderPolicy,
    Batch,
    Effect,
    Amend,
    Widen,
    Release,
    Attempt,
    Completion,
    Position,
    Purpose,
    ProcedureStep,
    AddressRevision,
    Party,
    Role,
    AddressAbsent,
    Instructions,
    InstructionsAbsent,
    HoldRevision,
    ReleaseAbsent,
    Spec,
    SpecAbsent,
    Authority,
    AuthorityAbsent,
    Wake,
    WakeAbsent,
    Dependencies,
    Dependency,
    DependenciesAbsent,
    DependenciesCut,
}
pub const ALL: [HistoryPhrase; 57] = [
    HistoryPhrase::History,
    HistoryPhrase::Task,
    HistoryPhrase::Revision,
    HistoryPhrase::Cancelled,
    HistoryPhrase::Released,
    HistoryPhrase::Amended,
    HistoryPhrase::Prioritised,
    HistoryPhrase::Moved,
    HistoryPhrase::Proposed,
    HistoryPhrase::Passed,
    HistoryPhrase::Accepted,
    HistoryPhrase::Rejected,
    HistoryPhrase::Withdrawn,
    HistoryPhrase::Reason,
    HistoryPhrase::At,
    HistoryPhrase::AtAbsent,
    HistoryPhrase::SnapshotsAbsent,
    HistoryPhrase::Stored,
    HistoryPhrase::Before,
    HistoryPhrase::After,
    HistoryPhrase::Proposal,
    HistoryPhrase::Proposer,
    HistoryPhrase::AsHolder,
    HistoryPhrase::NotAsHolder,
    HistoryPhrase::Pending,
    HistoryPhrase::Since,
    HistoryPhrase::HolderTask,
    HistoryPhrase::HolderPerson,
    HistoryPhrase::HolderPolicy,
    HistoryPhrase::Batch,
    HistoryPhrase::Effect,
    HistoryPhrase::Amend,
    HistoryPhrase::Widen,
    HistoryPhrase::Release,
    HistoryPhrase::Attempt,
    HistoryPhrase::Completion,
    HistoryPhrase::Position,
    HistoryPhrase::Purpose,
    HistoryPhrase::ProcedureStep,
    HistoryPhrase::AddressRevision,
    HistoryPhrase::Party,
    HistoryPhrase::Role,
    HistoryPhrase::AddressAbsent,
    HistoryPhrase::Instructions,
    HistoryPhrase::InstructionsAbsent,
    HistoryPhrase::HoldRevision,
    HistoryPhrase::ReleaseAbsent,
    HistoryPhrase::Spec,
    HistoryPhrase::SpecAbsent,
    HistoryPhrase::Authority,
    HistoryPhrase::AuthorityAbsent,
    HistoryPhrase::Wake,
    HistoryPhrase::WakeAbsent,
    HistoryPhrase::Dependencies,
    HistoryPhrase::Dependency,
    HistoryPhrase::DependenciesAbsent,
    HistoryPhrase::DependenciesCut,
];
impl HistoryPhrase {
    #[must_use]
    pub fn id(self) -> PhraseId {
        let number = match self {
            Self::History => 7600,
            Self::Task => 7601,
            Self::Revision => 7602,
            Self::Cancelled => 7603,
            Self::Released => 7604,
            Self::Amended => 7605,
            Self::Prioritised => 7606,
            Self::Moved => 7607,
            Self::Proposed => 7608,
            Self::Passed => 7609,
            Self::Accepted => 7610,
            Self::Rejected => 7611,
            Self::Withdrawn => 7612,
            Self::Reason => 7613,
            Self::At => 7614,
            Self::AtAbsent => 7615,
            Self::SnapshotsAbsent => 7616,
            Self::Stored => 7617,
            Self::Before => 7618,
            Self::After => 7619,
            Self::Proposal => 7620,
            Self::Proposer => 7621,
            Self::AsHolder => 7622,
            Self::NotAsHolder => 7623,
            Self::Pending => 7624,
            Self::Since => 7625,
            Self::HolderTask => 7626,
            Self::HolderPerson => 7627,
            Self::HolderPolicy => 7628,
            Self::Batch => 7629,
            Self::Effect => 7630,
            Self::Amend => 7631,
            Self::Widen => 7632,
            Self::Release => 7633,
            Self::Attempt => 7634,
            Self::Completion => 7635,
            Self::Position => 7636,
            Self::Purpose => 7637,
            Self::ProcedureStep => 7656,
            Self::AddressRevision => 7638,
            Self::Party => 7639,
            Self::Role => 7640,
            Self::AddressAbsent => 7641,
            Self::Instructions => 7642,
            Self::InstructionsAbsent => 7643,
            Self::HoldRevision => 7644,
            Self::ReleaseAbsent => 7645,
            Self::Spec => 7646,
            Self::SpecAbsent => 7647,
            Self::Authority => 7648,
            Self::AuthorityAbsent => 7649,
            Self::Wake => 7650,
            Self::WakeAbsent => 7651,
            Self::Dependencies => 7652,
            Self::Dependency => 7653,
            Self::DependenciesAbsent => 7654,
            Self::DependenciesCut => 7655,
        };
        PhraseId { owner: Owner::Jig.opaque(), number }
    }
    fn hole(self) -> Option<HoleKind> {
        match self {
            Self::History
            | Self::Cancelled
            | Self::Released
            | Self::Amended
            | Self::Moved
            | Self::Proposed
            | Self::Passed
            | Self::Accepted
            | Self::Rejected
            | Self::Withdrawn
            | Self::Reason
            | Self::AtAbsent
            | Self::SnapshotsAbsent
            | Self::Stored
            | Self::Before
            | Self::After
            | Self::AsHolder
            | Self::NotAsHolder
            | Self::Pending
            | Self::Batch
            | Self::Effect
            | Self::Amend
            | Self::Widen
            | Self::Release
            | Self::AddressAbsent
            | Self::Instructions
            | Self::InstructionsAbsent
            | Self::ReleaseAbsent
            | Self::Spec
            | Self::SpecAbsent
            | Self::Authority
            | Self::AuthorityAbsent
            | Self::Wake
            | Self::WakeAbsent
            | Self::Dependencies
            | Self::DependenciesAbsent
            | Self::DependenciesCut => None,
            Self::Task
            | Self::Revision
            | Self::Prioritised
            | Self::Proposal
            | Self::Proposer
            | Self::HolderTask
            | Self::HolderPerson
            | Self::HolderPolicy
            | Self::Attempt
            | Self::Completion
            | Self::Position
            | Self::Purpose
            | Self::ProcedureStep
            | Self::AddressRevision
            | Self::Party
            | Self::Role
            | Self::HoldRevision
            | Self::Dependency => Some(HoleKind::Number),
            Self::At | Self::Since => Some(HoleKind::Wall),
        }
    }
    fn label(self) -> &'static [u8] {
        match self {
            Self::History => b"Semantic history",
            Self::Task => b"Task: ",
            Self::Revision => b"Revision: ",
            Self::Cancelled => b"Cancelled",
            Self::Released => b"Released",
            Self::Amended => b"Amended",
            Self::Prioritised => b"Priority: ",
            Self::Moved => b"Moved",
            Self::Proposed => b"Proposed",
            Self::Passed => b"Proposal passed",
            Self::Accepted => b"Proposal accepted",
            Self::Rejected => b"Proposal rejected",
            Self::Withdrawn => b"Proposal withdrawn",
            Self::Reason => b"Supplied reason",
            Self::At => b"History time: ",
            Self::AtAbsent => b"History time not supplied",
            Self::SnapshotsAbsent => b"Stored snapshots not supplied",
            Self::Stored => b"Stored task overrides; historical effective instructions are not supplied",
            Self::Before => b"Before",
            Self::After => b"After",
            Self::Proposal => b"Proposal: ",
            Self::Proposer => b"Raw task-origin proposer: ",
            Self::AsHolder => b"Proposed as holder",
            Self::NotAsHolder => b"Not proposed as holder",
            Self::Pending => b"Proposal pending",
            Self::Since => b"Pending since: ",
            Self::HolderTask => b"Holder task: ",
            Self::HolderPerson => b"Holder person: ",
            Self::HolderPolicy => b"Holder policy project: ",
            Self::Batch => b"Retained batch",
            Self::Effect => b"Retained effect",
            Self::Amend => b"Retained amendment",
            Self::Widen => b"Retained widening",
            Self::Release => b"Retained release",
            Self::Attempt => b"Source attempt: ",
            Self::Completion => b"Source completion: ",
            Self::Position => b"Source effect position: ",
            Self::Purpose => b"Source purpose: ",
            Self::ProcedureStep => b"Procedure step: ",
            Self::AddressRevision => b"Person revision: ",
            Self::Party => b"Addressed party: ",
            Self::Role => b"Addressed role: ",
            Self::AddressAbsent => b"Address replacement not supplied",
            Self::Instructions => b"Retained instruction replacement",
            Self::InstructionsAbsent => b"Instruction replacement not supplied",
            Self::HoldRevision => b"Release hold revision: ",
            Self::ReleaseAbsent => b"Release not supplied",
            Self::Spec => b"Retained specification replacement",
            Self::SpecAbsent => b"Specification replacement not supplied",
            Self::Authority => b"Retained authority",
            Self::AuthorityAbsent => b"Authority replacement not supplied",
            Self::Wake => b"Retained wake replacement",
            Self::WakeAbsent => b"Wake replacement not supplied",
            Self::Dependencies => b"Retained dependency replacement",
            Self::Dependency => b"Dependency task: ",
            Self::DependenciesAbsent => b"Dependency replacement not supplied",
            Self::DependenciesCut => b"Retained dependency list incomplete",
        }
    }
}
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut values = List::with_capacity(57);
    for phrase in ALL {
        let holes: Box<[HoleShape]> = match phrase.hole() {
            Some(kind) => Box::new([HoleShape { kind, bytes: 0 }]),
            None => Box::new([]),
        };
        values.push(PhraseShape { id: phrase.id(), holes, plural: None }).expect("closed history shapes fit");
    }
    values.into_boxed()
}
#[must_use]
pub fn language() -> Language {
    let mut values = List::with_capacity(57);
    for phrase in ALL {
        let segments: Box<[Segment]> = match phrase.hole() {
            Some(kind) => Box::new([Segment::Text(copy_of(phrase.label())), Segment::Hole { index: 0, kind }]),
            None => Box::new([Segment::Text(copy_of(phrase.label()))]),
        };
        values
            .push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("closed history sayings fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: values.into_boxed(),
    }
}
