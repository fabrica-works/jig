//! Checked committed-outcome phrases (client/domain/view.md, 7).
//! IDs 6400..6470 belong to this owner vocabulary. Language data is
//! composed into the caller's checked Words; no language is chosen here.
use crate::Owner;
use crate::words::Words;
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::phrases::{
    Form, Hole, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment,
    SpanUnit, WallFormat,
};

/// The closed phrases for committed outcomes and their supplied facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutcomePhrase {
    /// The operations committed saying.
    Created,
    FirstTask,
    LastTask,
    GivenTask,
    GivenAmount,
    Compound,
    /// The number of committed operations.
    Operations,
    /// The read position marked seen saying.
    MarkedSeen,
    /// The note edited saying.
    NoteEdited,
    /// The service made saying.
    ServiceMade,
    /// The project adopted saying.
    Adopted,
    /// The task stopped saying.
    Stopped,
    /// The task cancelled saying.
    Cancelled,
    /// The task released saying.
    Released,
    /// The person task taken saying.
    PersonTaken,
    /// The person task handed back saying.
    PersonHandedBack,
    /// The person task answered saying.
    PersonAnswered,
    /// The task moved saying.
    Moved,
    /// The proposal decided saying.
    ProposalDecided,
    /// The words admitted saying.
    Said,
    /// The question answered saying.
    QuestionAnswered,
    /// The task amended saying.
    Amended,
    /// The proposal admitted saying.
    Proposed,
    /// The project roles set saying.
    RolesSet,
    /// The project policy changed saying.
    PolicyChanged,
    Renamed,
    ProjectMade,
    /// The person pool set saying.
    PoolSet,
    /// The escalation decided saying.
    EscalationDecided,
    /// The accepted saying.
    Accepted,
    /// The rejected saying.
    Rejected,
    /// The passed up saying.
    Passed,
    /// The withdrawn saying.
    Withdrawn,
    /// The no longer applies saying.
    Stale,
    ReleaseChoice,
    /// The chat saying.
    Chat,
    /// The cancelled delegates saying.
    CancelledCount,
    /// The kept delegates saying.
    KeptCount,
    /// The note saying.
    Note,
    /// The person saying.
    Person,
    /// The project saying.
    Project,
    /// The task saying.
    Task,
    /// The proposal number saying.
    ProposalNumber,
    /// The proposing task saying.
    TaskProposer,
    /// The proposing party saying.
    PartyProposer,
    /// The decided by person saying.
    By,
    /// The decided at saying.
    At,
    /// The message saying.
    Message,
    /// The question saying.
    Question,
    /// The hold revision saying.
    Revision,
    /// The results seen through saying.
    ResultsThrough,
    /// The replies seen through saying.
    RepliesThrough,
    /// The LLM account sign-in started saying.
    AccountSignInStarted,
    /// The LLM account signed out saying.
    AccountSignedOut,
    AccountKeyKept,
    AccountChanged,
    AccountRemoved,
    AccountProgramDeclared,
    SourceNumber,
    SignOutConfirmed,
    SignOutUnconfirmed,
    HostNotInContact,
    HostLost,
    HostFailed,
}
/// Every outcome phrase in stable owner order.
pub const ALL: [OutcomePhrase; 64] = [
    OutcomePhrase::Created,
    OutcomePhrase::FirstTask,
    OutcomePhrase::LastTask,
    OutcomePhrase::GivenTask,
    OutcomePhrase::GivenAmount,
    OutcomePhrase::Compound,
    OutcomePhrase::Operations,
    OutcomePhrase::MarkedSeen,
    OutcomePhrase::NoteEdited,
    OutcomePhrase::ServiceMade,
    OutcomePhrase::Adopted,
    OutcomePhrase::Stopped,
    OutcomePhrase::Cancelled,
    OutcomePhrase::Released,
    OutcomePhrase::PersonTaken,
    OutcomePhrase::PersonHandedBack,
    OutcomePhrase::PersonAnswered,
    OutcomePhrase::Moved,
    OutcomePhrase::ProposalDecided,
    OutcomePhrase::Said,
    OutcomePhrase::QuestionAnswered,
    OutcomePhrase::Amended,
    OutcomePhrase::Proposed,
    OutcomePhrase::RolesSet,
    OutcomePhrase::PolicyChanged,
    OutcomePhrase::Renamed,
    OutcomePhrase::ProjectMade,
    OutcomePhrase::PoolSet,
    OutcomePhrase::EscalationDecided,
    OutcomePhrase::Accepted,
    OutcomePhrase::Rejected,
    OutcomePhrase::Passed,
    OutcomePhrase::Withdrawn,
    OutcomePhrase::Stale,
    OutcomePhrase::ReleaseChoice,
    OutcomePhrase::Chat,
    OutcomePhrase::CancelledCount,
    OutcomePhrase::KeptCount,
    OutcomePhrase::Note,
    OutcomePhrase::Person,
    OutcomePhrase::Project,
    OutcomePhrase::Task,
    OutcomePhrase::ProposalNumber,
    OutcomePhrase::TaskProposer,
    OutcomePhrase::PartyProposer,
    OutcomePhrase::By,
    OutcomePhrase::At,
    OutcomePhrase::Message,
    OutcomePhrase::Question,
    OutcomePhrase::Revision,
    OutcomePhrase::ResultsThrough,
    OutcomePhrase::RepliesThrough,
    OutcomePhrase::AccountSignInStarted,
    OutcomePhrase::AccountSignedOut,
    OutcomePhrase::AccountKeyKept,
    OutcomePhrase::AccountChanged,
    OutcomePhrase::AccountRemoved,
    OutcomePhrase::AccountProgramDeclared,
    OutcomePhrase::SourceNumber,
    OutcomePhrase::SignOutConfirmed,
    OutcomePhrase::SignOutUnconfirmed,
    OutcomePhrase::HostNotInContact,
    OutcomePhrase::HostLost,
    OutcomePhrase::HostFailed,
];
impl OutcomePhrase {
    /// Stable identity for checked language composition.
    #[must_use]
    pub fn id(self) -> PhraseId {
        let number = match self {
            Self::Created => 6452,
            Self::FirstTask => 6453,
            Self::LastTask => 6454,
            Self::GivenTask => 6455,
            Self::GivenAmount => 6456,
            Self::Compound => 6450,
            Self::Operations => 6451,
            Self::MarkedSeen => 6401,
            Self::NoteEdited => 6402,
            Self::ServiceMade => 6403,
            Self::Adopted => 6404,
            Self::Stopped => 6407,
            Self::Cancelled => 6408,
            Self::Released => 6409,
            Self::PersonTaken => 6410,
            Self::PersonHandedBack => 6411,
            Self::PersonAnswered => 6412,
            Self::Moved => 6413,
            Self::ProposalDecided => 6414,
            Self::Said => 6415,
            Self::QuestionAnswered => 6416,
            Self::Amended => 6418,
            Self::Proposed => 6419,
            Self::RolesSet => 6420,
            Self::PolicyChanged => 6421,
            Self::Renamed => 6469,
            Self::ProjectMade => 6470,
            Self::PoolSet => 6422,
            Self::EscalationDecided => 6423,
            Self::Accepted => 6425,
            Self::Rejected => 6426,
            Self::Passed => 6427,
            Self::Withdrawn => 6428,
            Self::Stale => 6429,
            Self::ReleaseChoice => 6430,
            Self::Chat => 6431,
            Self::CancelledCount => 6432,
            Self::KeptCount => 6433,
            Self::Note => 6434,
            Self::Person => 6435,
            Self::Project => 6436,
            Self::Task => 6437,
            Self::ProposalNumber => 6438,
            Self::TaskProposer => 6439,
            Self::PartyProposer => 6440,
            Self::By => 6441,
            Self::At => 6442,
            Self::Message => 6443,
            Self::Question => 6444,
            Self::Revision => 6445,
            Self::ResultsThrough => 6448,
            Self::RepliesThrough => 6449,
            Self::AccountSignInStarted => 6457,
            Self::AccountSignedOut => 6458,
            Self::AccountKeyKept => 6459,
            Self::AccountChanged => 6460,
            Self::AccountRemoved => 6461,
            Self::AccountProgramDeclared => 6462,
            Self::SourceNumber => 6463,
            Self::SignOutConfirmed => 6464,
            Self::SignOutUnconfirmed => 6465,
            Self::HostNotInContact => 6466,
            Self::HostLost => 6467,
            Self::HostFailed => 6468,
        };
        PhraseId { owner: Owner::Jig.opaque(), number }
    }
    fn hole(self) -> Option<HoleKind> {
        match self {
            Self::SourceNumber
            | Self::FirstTask
            | Self::LastTask
            | Self::GivenTask
            | Self::GivenAmount
            | Self::Operations
            | Self::Chat
            | Self::CancelledCount
            | Self::KeptCount
            | Self::Note
            | Self::Person
            | Self::Project
            | Self::Task
            | Self::ProposalNumber
            | Self::TaskProposer
            | Self::PartyProposer
            | Self::By
            | Self::Message
            | Self::Question
            | Self::Revision
            | Self::ResultsThrough
            | Self::RepliesThrough => Some(HoleKind::Number),
            Self::At => Some(HoleKind::Wall),
            Self::AccountProgramDeclared
            | Self::SignOutConfirmed
            | Self::SignOutUnconfirmed
            | Self::HostNotInContact
            | Self::HostLost
            | Self::HostFailed
            | Self::Created
            | Self::Compound
            | Self::MarkedSeen
            | Self::NoteEdited
            | Self::ServiceMade
            | Self::Adopted
            | Self::Stopped
            | Self::Cancelled
            | Self::Released
            | Self::PersonTaken
            | Self::PersonHandedBack
            | Self::PersonAnswered
            | Self::Moved
            | Self::ProposalDecided
            | Self::Said
            | Self::QuestionAnswered
            | Self::Amended
            | Self::Proposed
            | Self::RolesSet
            | Self::Renamed
            | Self::ProjectMade
            | Self::PolicyChanged
            | Self::PoolSet
            | Self::EscalationDecided
            | Self::Accepted
            | Self::Rejected
            | Self::Passed
            | Self::Withdrawn
            | Self::Stale
            | Self::AccountSignInStarted
            | Self::AccountKeyKept
            | Self::AccountChanged
            | Self::AccountRemoved
            | Self::AccountSignedOut
            | Self::ReleaseChoice => None,
        }
    }
}
/// Phrase contracts independent of the selected language's word order.
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut values = List::with_capacity(64);
    for phrase in ALL {
        let holes: Box<[HoleShape]> = match phrase.hole() {
            Some(kind) => Box::new([HoleShape { kind, bytes: 0 }]),
            None => Box::new([]),
        };
        values.push(PhraseShape { id: phrase.id(), holes, plural: None }).expect("closed outcome shapes fit");
    }
    values.into_boxed()
}
/// Explicit English data for composition, never a rendering fallback.
#[must_use]
pub fn english() -> Language {
    let mut values = List::with_capacity(64);
    for phrase in ALL {
        let text: &[u8] = match phrase {
            OutcomePhrase::Created => b"Tasks created",
            OutcomePhrase::FirstTask => b"First task: ",
            OutcomePhrase::LastTask => b"Last task: ",
            OutcomePhrase::GivenTask => b"Budget given to task: ",
            OutcomePhrase::GivenAmount => b"Amount given: ",
            OutcomePhrase::Compound => b"Operations committed",
            OutcomePhrase::Operations => b"Operations: ",
            OutcomePhrase::MarkedSeen => b"Read position marked seen",
            OutcomePhrase::NoteEdited => b"Note edited",
            OutcomePhrase::ServiceMade => b"Service made",
            OutcomePhrase::Adopted => b"Project adopted",
            OutcomePhrase::Stopped => b"Task stopped",
            OutcomePhrase::Cancelled => b"Task cancelled",
            OutcomePhrase::Released => b"Task released",
            OutcomePhrase::PersonTaken => b"Person task taken",
            OutcomePhrase::PersonHandedBack => b"Person task handed back",
            OutcomePhrase::PersonAnswered => b"Person task answered",
            OutcomePhrase::Moved => b"Task moved",
            OutcomePhrase::ProposalDecided => b"Proposal decided",
            OutcomePhrase::Said => b"Words admitted",
            OutcomePhrase::QuestionAnswered => b"Question answered",
            OutcomePhrase::Amended => b"Task amended",
            OutcomePhrase::Proposed => b"Proposed",
            OutcomePhrase::RolesSet => b"Project roles set",
            OutcomePhrase::Renamed => b"Renamed the project",
            OutcomePhrase::ProjectMade => b"Made project",
            OutcomePhrase::PolicyChanged => b"Project policy changed",
            OutcomePhrase::PoolSet => b"Person pool set",
            OutcomePhrase::EscalationDecided => b"Escalation decided",
            OutcomePhrase::Accepted => b"Accepted",
            OutcomePhrase::Rejected => b"Rejected",
            OutcomePhrase::Passed => b"Passed up",
            OutcomePhrase::Withdrawn => b"Withdrawn",
            OutcomePhrase::Stale => b"No longer applies",
            OutcomePhrase::ReleaseChoice => b"Released",
            OutcomePhrase::Chat => b"Chat: ",
            OutcomePhrase::CancelledCount => b"Cancelled delegates: ",
            OutcomePhrase::KeptCount => b"Kept delegates: ",
            OutcomePhrase::Note => b"Note: ",
            OutcomePhrase::Person => b"Person: ",
            OutcomePhrase::Project => b"Project: ",
            OutcomePhrase::Task => b"Task: ",
            OutcomePhrase::ProposalNumber => b"Proposal number: ",
            OutcomePhrase::TaskProposer => b"Proposing task: ",
            OutcomePhrase::PartyProposer => b"Proposing party: ",
            OutcomePhrase::By => b"Decided by person: ",
            OutcomePhrase::At => b"Decided at: ",
            OutcomePhrase::Message => b"Message: ",
            OutcomePhrase::Question => b"Question: ",
            OutcomePhrase::Revision => b"Hold revision: ",
            OutcomePhrase::ResultsThrough => b"Results seen through: ",
            OutcomePhrase::RepliesThrough => b"Replies seen through: ",
            OutcomePhrase::AccountSignInStarted => b"LLM account sign-in started",
            OutcomePhrase::AccountKeyKept => b"LLM account key kept",
            OutcomePhrase::AccountChanged => b"LLM account changed",
            OutcomePhrase::AccountRemoved => b"LLM account removed",
            OutcomePhrase::AccountProgramDeclared => b"Program login declared",
            OutcomePhrase::SourceNumber => b"Login source: ",
            OutcomePhrase::SignOutConfirmed => b"Sign-out confirmed",
            OutcomePhrase::SignOutUnconfirmed => b"Sign-out not confirmed",
            OutcomePhrase::HostNotInContact => b"Not signed out (not in contact):",
            OutcomePhrase::HostLost => b"Not signed out (lost contact):",
            OutcomePhrase::HostFailed => b"Not signed out (failed):",

            OutcomePhrase::AccountSignedOut => b"LLM account signed out",
        };
        let segments: Box<[Segment]> = match phrase.hole() {
            Some(kind) => Box::new([Segment::Text(copy_of(text)), Segment::Hole { index: 0, kind }]),
            None => Box::new([Segment::Text(copy_of(text))]),
        };
        values
            .push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("closed outcome phrases fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: values.into_boxed(),
    }
}
/// Capacity across every loaded language for one checked phrase.
#[must_use]
pub fn capacity(words: &Words, phrase: OutcomePhrase) -> u32 {
    words.book().capacity(phrase.id()).expect("outcome vocabulary checked in Words")
}
/// Say supplied typed facts in the caller's selected checked language.
#[must_use]
pub fn say(words: &Words, phrase: OutcomePhrase, holes: &[Hole]) -> Box<[u8]> {
    words.book().say(words.language(), phrase.id(), holes).expect("checked outcome phrase and typed facts").text
}
