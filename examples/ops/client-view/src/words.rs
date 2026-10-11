//! Keep: checked phrase data beside jig's phrasebook (view.md, section 7).
//! Replace: the application's closed English vocabulary.
use alloc::boxed::Box;
use jig_client_view::{
    Builder, Owner, Role, Run,
    phrases::{
        Form, Hole, HoleKind, HoleShape, Invalid, Language, Phrase, PhraseId, PhraseLimits, PhraseShape, Phrasebook,
        PluralClass, PluralRule, Segment, SpanUnit, WallFormat,
    },
};
use skein_lib::{List, bytes::copy_of};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    /// Incidents.
    Incidents,
    /// Environments.
    Environments,
    /// Incident.
    Incident,
    /// Environment.
    Environment,
    /// Loading.
    Loading,
    /// Unknown.
    Unknown,
    /// Unshowable.
    Unshowable,
    /// Open.
    Open,
    /// Resolved.
    Resolved,
    /// No lead.
    NoLead,
    /// Lead.
    Lead,
    /// Service.
    Service,
    /// In environment.
    InEnvironment,
    /// Severity.
    Severity,
    /// Raised.
    Raised,
    /// Timeline.
    Timeline,
    /// Cut.
    Cut,
    /// Error rate.
    ErrorRate,
    /// Healthy.
    Healthy,
    /// Errors.
    Errors,
    /// Rate.
    Rate,
    /// Since.
    Since,
    /// Before.
    Before,
    /// Pool.
    Pool,
    /// Name.
    Name,
    /// Quota.
    Quota,
    /// Used.
    Used,
    /// Cost.
    Cost,
    /// End.
    End,
    /// Observed end.
    ObservedEnd,
    /// Ready at.
    ReadyAt,
    /// Ready.
    Ready,
    /// Not ready.
    NotReady,
    /// Until.
    Until,
    /// Move end.
    MoveEnd,
    /// Deciding.
    Deciding,
    /// Retrying.
    Retrying,
    /// Parked.
    Parked,
    /// Done.
    Done,
    /// Moved.
    Moved,
    /// Gone.
    Gone,
    /// Refused.
    Refused,
    /// Revision.
    Revision,
    /// Connector.
    Connector,
    /// Resource.
    Resource,
    /// Kind.
    Kind,
    /// Panel.
    Panel,
    /// Remediate.
    Remediate,
    /// Scale.
    Scale,
    /// Provision.
    Provision,
    /// Tear down.
    TearDown,
    /// Operation.
    Operation,
    /// Replicas.
    Replicas,
    /// Deadline.
    Deadline,
    /// Active.
    Active,
    /// Reading facts.
    ReadingFacts,
    /// Facts ready.
    FactsReady,
    /// Effect outstanding.
    EffectOutstanding,
    /// Waiting condition.
    WaitingCondition,
    /// Holding.
    Holding,
    /// Finished.
    Finished,
    /// Held.
    Held,
    /// Description.
    Description,
    /// Maximum price.
    MaximumPrice,
    /// Keyed.
    Keyed,
    /// Conditional.
    Conditional,
    /// Unrecoverable.
    Unrecoverable,
    /// Condition.
    Condition,
    /// State.
    State,
    /// Creation.
    Creation,
    /// Transition.
    Transition,
    /// Set.
    Set,
    /// Restart.
    Restart,
    /// Rollback.
    Rollback,
    /// From.
    From,
    /// To.
    To,
    /// Entry.
    Entry,
    /// Task.
    Task,
    /// Attempt.
    Attempt,
    /// Kept.
    Kept,
    /// Sent.
    Sent,
    /// Uncertain.
    Uncertain,
    /// Deleted.
    Deleted,
    /// Changed.
    Changed,
    /// Replaced.
    Replaced,
    /// Ambiguous.
    Ambiguous,
    /// Actor.
    Actor,
    /// At.
    At,
    /// Observed.
    Observed,
    /// Owned.
    Owned,
    /// Participating.
    Participating,
    /// Context.
    Context,
    /// Version.
    Version,
    /// Backend ids.
    BackendIds,
    /// Backend no ids.
    BackendNoIds,
    /// Made.
    Made,
    /// Proposal table capacity.
    ProposalLimit,
    /// Staged limit.
    StagedLimit,
    /// Watches.
    Watches,
    /// Incident limit.
    IncidentLimit,
    /// Silence.
    Silence,
    /// Held role.
    HeldRole,
    /// Past.
    Past,
    /// Exhausted.
    Exhausted,
    /// Signed out.
    SignedOut,
    /// Not allowed.
    NotAllowed,
    /// Conflict.
    Conflict,
    /// Invalid.
    Invalid,
    /// System request budget recording is deferred.
    SystemRequestBudgetNotKept,
    /// Last system response recording is deferred.
    LastSystemResponseNotKept,
    /// Read table capacity.
    ReadLimit,
}
const ALL: [Word; 111] = [
    Word::Incidents,
    Word::Environments,
    Word::Incident,
    Word::Environment,
    Word::Loading,
    Word::Unknown,
    Word::Unshowable,
    Word::Open,
    Word::Resolved,
    Word::NoLead,
    Word::Lead,
    Word::Service,
    Word::InEnvironment,
    Word::Severity,
    Word::Raised,
    Word::Timeline,
    Word::Cut,
    Word::ErrorRate,
    Word::Healthy,
    Word::Errors,
    Word::Rate,
    Word::Since,
    Word::Before,
    Word::Pool,
    Word::Name,
    Word::Quota,
    Word::Used,
    Word::Cost,
    Word::End,
    Word::ObservedEnd,
    Word::ReadyAt,
    Word::Ready,
    Word::NotReady,
    Word::Until,
    Word::MoveEnd,
    Word::Deciding,
    Word::Retrying,
    Word::Parked,
    Word::Done,
    Word::Moved,
    Word::Gone,
    Word::Refused,
    Word::Revision,
    Word::Connector,
    Word::Resource,
    Word::Kind,
    Word::Panel,
    Word::Remediate,
    Word::Scale,
    Word::Provision,
    Word::TearDown,
    Word::Operation,
    Word::Replicas,
    Word::Deadline,
    Word::Active,
    Word::ReadingFacts,
    Word::FactsReady,
    Word::EffectOutstanding,
    Word::WaitingCondition,
    Word::Holding,
    Word::Finished,
    Word::Held,
    Word::Description,
    Word::MaximumPrice,
    Word::Keyed,
    Word::Conditional,
    Word::Unrecoverable,
    Word::Condition,
    Word::State,
    Word::Creation,
    Word::Transition,
    Word::Set,
    Word::Restart,
    Word::Rollback,
    Word::From,
    Word::To,
    Word::Entry,
    Word::Task,
    Word::Attempt,
    Word::Kept,
    Word::Sent,
    Word::Uncertain,
    Word::Deleted,
    Word::Changed,
    Word::Replaced,
    Word::Ambiguous,
    Word::Actor,
    Word::At,
    Word::Observed,
    Word::Owned,
    Word::Participating,
    Word::Context,
    Word::Version,
    Word::BackendIds,
    Word::BackendNoIds,
    Word::Made,
    Word::ProposalLimit,
    Word::StagedLimit,
    Word::Watches,
    Word::IncidentLimit,
    Word::Silence,
    Word::HeldRole,
    Word::Past,
    Word::Exhausted,
    Word::SignedOut,
    Word::NotAllowed,
    Word::Conflict,
    Word::Invalid,
    Word::SystemRequestBudgetNotKept,
    Word::LastSystemResponseNotKept,
    Word::ReadLimit,
];
impl Word {
    #[must_use]
    #[expect(clippy::too_many_lines, reason = "closed application phrase identities are matched exhaustively")]
    pub const fn id(self, connector: u16) -> PhraseId {
        let number = match self {
            Self::Incidents => 0,
            Self::Environments => 1,
            Self::Incident => 2,
            Self::Environment => 3,
            Self::Loading => 4,
            Self::Unknown => 5,
            Self::Unshowable => 6,
            Self::Open => 7,
            Self::Resolved => 8,
            Self::NoLead => 9,
            Self::Lead => 10,
            Self::Service => 11,
            Self::InEnvironment => 12,
            Self::Severity => 13,
            Self::Raised => 14,
            Self::Timeline => 15,
            Self::Cut => 16,
            Self::ErrorRate => 17,
            Self::Healthy => 18,
            Self::Errors => 19,
            Self::Rate => 20,
            Self::Since => 21,
            Self::Before => 22,
            Self::Pool => 23,
            Self::Name => 24,
            Self::Quota => 25,
            Self::Used => 26,
            Self::Cost => 27,
            Self::End => 28,
            Self::ObservedEnd => 29,
            Self::ReadyAt => 30,
            Self::Ready => 31,
            Self::NotReady => 32,
            Self::Until => 33,
            Self::MoveEnd => 34,
            Self::Deciding => 35,
            Self::Retrying => 36,
            Self::Parked => 37,
            Self::Done => 38,
            Self::Moved => 39,
            Self::Gone => 40,
            Self::Refused => 41,
            Self::Revision => 42,
            Self::Connector => 43,
            Self::Resource => 44,
            Self::Kind => 45,
            Self::Panel => 46,
            Self::Remediate => 47,
            Self::Scale => 48,
            Self::Provision => 49,
            Self::TearDown => 50,
            Self::Operation => 51,
            Self::Replicas => 52,
            Self::Deadline => 53,
            Self::Active => 54,
            Self::ReadingFacts => 55,
            Self::FactsReady => 56,
            Self::EffectOutstanding => 57,
            Self::WaitingCondition => 58,
            Self::Holding => 59,
            Self::Finished => 60,
            Self::Held => 61,
            Self::Description => 62,
            Self::MaximumPrice => 63,
            Self::Keyed => 64,
            Self::Conditional => 65,
            Self::Unrecoverable => 66,
            Self::Condition => 67,
            Self::State => 68,
            Self::Creation => 69,
            Self::Transition => 70,
            Self::Set => 71,
            Self::Restart => 72,
            Self::Rollback => 73,
            Self::From => 74,
            Self::To => 75,
            Self::Entry => 76,
            Self::Task => 77,
            Self::Attempt => 78,
            Self::Kept => 79,
            Self::Sent => 80,
            Self::Uncertain => 81,
            Self::Deleted => 82,
            Self::Changed => 83,
            Self::Replaced => 84,
            Self::Ambiguous => 85,
            Self::Actor => 86,
            Self::At => 87,
            Self::Observed => 88,
            Self::Owned => 89,
            Self::Participating => 90,
            Self::Context => 91,
            Self::Version => 92,
            Self::BackendIds => 93,
            Self::BackendNoIds => 94,
            Self::Made => 95,
            Self::ProposalLimit => 96,
            Self::StagedLimit => 97,
            Self::Watches => 98,
            Self::IncidentLimit => 99,
            Self::Silence => 100,
            Self::HeldRole => 101,
            Self::Past => 102,
            Self::Exhausted => 103,
            Self::SignedOut => 104,
            Self::NotAllowed => 105,
            Self::Conflict => 106,
            Self::Invalid => 107,
            Self::SystemRequestBudgetNotKept => 108,
            Self::LastSystemResponseNotKept => 109,
            Self::ReadLimit => 110,
        };
        PhraseId { owner: Owner::Side { connector }, number }
    }
}
#[derive(Clone, Debug)]
pub struct Words {
    pub book: Phrasebook,
    pub language: u16,
    pub connector: u16,
}
impl Words {
    #[expect(clippy::too_many_lines, reason = "closed English phrase data is matched exhaustively")]
    pub fn english(connector: u16, text: u32) -> Result<Self, Invalid> {
        let mut shapes = List::with_capacity(111);
        let mut phrases = List::with_capacity(111);
        for word in ALL {
            let (literal, hole) = match word {
                Word::Incidents => (b"Incidents".as_slice(), None),
                Word::Environments => (b"Environments".as_slice(), None),
                Word::Incident => (b"Incident ".as_slice(), Some(HoleKind::Number)),
                Word::Environment => (b"Environment ".as_slice(), Some(HoleKind::Number)),
                Word::Loading => (b"Loading document".as_slice(), None),
                Word::Unknown => (b"Unknown object".as_slice(), None),
                Word::Unshowable => (b"Cannot show this part".as_slice(), None),
                Word::Open => (b"Open".as_slice(), None),
                Word::Resolved => (b"Resolved".as_slice(), None),
                Word::NoLead => (b"No lead task has been supplied".as_slice(), None),
                Word::Lead => (b"Lead task ".as_slice(), Some(HoleKind::Number)),
                Word::Service => (b"Service: ".as_slice(), Some(HoleKind::Name)),
                Word::InEnvironment => (b"Environment: ".as_slice(), Some(HoleKind::Name)),
                Word::Severity => (b"Severity ".as_slice(), Some(HoleKind::Number)),
                Word::Raised => (b"Raised at ".as_slice(), Some(HoleKind::Wall)),
                Word::Timeline => (b"Incident timeline".as_slice(), None),
                Word::Cut => (b"Earlier observations are outside this window".as_slice(), None),
                Word::ErrorRate => (b"Error rate".as_slice(), None),
                Word::Healthy => (b"Healthy replicas: ".as_slice(), Some(HoleKind::Number)),
                Word::Errors => (b"Errors: ".as_slice(), Some(HoleKind::Number)),
                Word::Rate => (b"Error rate ".as_slice(), Some(HoleKind::Number)),
                Word::Since => (b"Since ".as_slice(), Some(HoleKind::Name)),
                Word::Before => (b"Before: ".as_slice(), Some(HoleKind::Number)),
                Word::Pool => (b"Pool ".as_slice(), Some(HoleKind::Number)),
                Word::Name => (b"Name: ".as_slice(), Some(HoleKind::Name)),
                Word::Quota => (b"Quota: ".as_slice(), Some(HoleKind::Number)),
                Word::Used => (b"Slots used: ".as_slice(), Some(HoleKind::Number)),
                Word::Cost => (b"Cost: ".as_slice(), Some(HoleKind::Number)),
                Word::End => (b"Desired end: ".as_slice(), Some(HoleKind::Wall)),
                Word::ObservedEnd => (b"Observed backend lease: ".as_slice(), Some(HoleKind::Wall)),
                Word::ReadyAt => (b"Ready at ".as_slice(), Some(HoleKind::Wall)),
                Word::Ready => (b"Ready".as_slice(), None),
                Word::NotReady => (b"Not ready".as_slice(), None),
                Word::Until => (b"New end (seconds since epoch)".as_slice(), None),
                Word::MoveEnd => (b"Move end".as_slice(), None),
                Word::Deciding => (b"In flight".as_slice(), None),
                Word::Retrying => (b"Retrying".as_slice(), None),
                Word::Parked => (b"Parked".as_slice(), None),
                Word::Done => (b"End moved".as_slice(), None),
                Word::Moved => (b"Moved to revision ".as_slice(), Some(HoleKind::Number)),
                Word::Gone => (b"Gone".as_slice(), None),
                Word::Refused => (b"Request refused".as_slice(), None),
                Word::Revision => (b"Revision ".as_slice(), Some(HoleKind::Number)),
                Word::Connector => (b"Connector ".as_slice(), Some(HoleKind::Number)),
                Word::Resource => (b"Resource: ".as_slice(), Some(HoleKind::Name)),
                Word::Kind => (b"Kind ".as_slice(), Some(HoleKind::Number)),
                Word::Panel => (b"Infrastructure procedure".as_slice(), None),
                Word::Remediate | Word::Restart => (b"Restart service".as_slice(), None),
                Word::Scale => (b"Scale service".as_slice(), None),
                Word::Provision => (b"Provision environment".as_slice(), None),
                Word::TearDown => (b"Tear down environment".as_slice(), None),
                Word::Operation => (b"Operation ".as_slice(), Some(HoleKind::Number)),
                Word::Replicas => (b"Replicas: ".as_slice(), Some(HoleKind::Number)),
                Word::Deadline => (b"Deadline: ".as_slice(), Some(HoleKind::Wall)),
                Word::Active => (b"Active".as_slice(), None),
                Word::ReadingFacts => (b"Reading facts".as_slice(), None),
                Word::FactsReady => (b"Facts ready".as_slice(), None),
                Word::EffectOutstanding => (b"Effect outstanding".as_slice(), None),
                Word::WaitingCondition => (b"Waiting for condition".as_slice(), None),
                Word::Holding => (b"Acquiring hold".as_slice(), None),
                Word::Finished => (b"Done".as_slice(), None),
                Word::Held => (b"Held".as_slice(), None),
                Word::Description => (b"Effect: ".as_slice(), Some(HoleKind::Name)),
                Word::MaximumPrice => (b"Maximum price: ".as_slice(), Some(HoleKind::Number)),
                Word::Keyed => (b"Keyed recovery".as_slice(), None),
                Word::Conditional => (b"Conditional recovery".as_slice(), None),
                Word::Unrecoverable => (b"Outcome cannot be recovered automatically".as_slice(), None),
                Word::Condition => (b"Condition required".as_slice(), None),
                Word::State => (b"Observed state: ".as_slice(), Some(HoleKind::Number)),
                Word::Creation => (b"Creation".as_slice(), None),
                Word::Transition => (b"Transition".as_slice(), None),
                Word::Set => (b"Set".as_slice(), None),
                Word::Rollback => (b"Rollback service".as_slice(), None),
                Word::From => (b"From: ".as_slice(), Some(HoleKind::Name)),
                Word::To => (b"To: ".as_slice(), Some(HoleKind::Name)),
                Word::Entry => (b"Outbox entry ".as_slice(), Some(HoleKind::Number)),
                Word::Task => (b"Task ".as_slice(), Some(HoleKind::Number)),
                Word::Attempt => (b"Attempt ".as_slice(), Some(HoleKind::Number)),
                Word::Kept => (b"Kept".as_slice(), None),
                Word::Sent => (b"Sent".as_slice(), None),
                Word::Uncertain => (b"Uncertain".as_slice(), None),
                Word::Deleted => (b"Deleted by hand".as_slice(), None),
                Word::Changed => (b"Changed by hand".as_slice(), None),
                Word::Replaced => (b"Replaced by hand".as_slice(), None),
                Word::Ambiguous => (b"Hand change is ambiguous".as_slice(), None),
                Word::Actor => (b"Changed by ".as_slice(), Some(HoleKind::Name)),
                Word::At => (b"At ".as_slice(), Some(HoleKind::Wall)),
                Word::Observed => (b"Observed at ".as_slice(), Some(HoleKind::Wall)),
                Word::Owned => (b"Owned resource".as_slice(), None),
                Word::Participating => (b"Participating resource".as_slice(), None),
                Word::Context => (b"Context resource".as_slice(), None),
                Word::Version => (b"Version: ".as_slice(), Some(HoleKind::Name)),
                Word::BackendIds => (b"Backend supports operation IDs".as_slice(), None),
                Word::BackendNoIds => (b"Backend has no operation IDs".as_slice(), None),
                Word::Made => (b"Environments made: ".as_slice(), Some(HoleKind::Number)),
                Word::ProposalLimit => (b"Proposal capacity: ".as_slice(), Some(HoleKind::Number)),
                Word::StagedLimit => (b"Staged limit: ".as_slice(), Some(HoleKind::Number)),
                Word::Watches => (b"Standing watches: ".as_slice(), Some(HoleKind::Number)),
                Word::IncidentLimit => (b"Incident limit: ".as_slice(), Some(HoleKind::Number)),
                Word::Silence => (b"Silence alert ".as_slice(), Some(HoleKind::Number)),
                Word::HeldRole => (b"Your held role: ".as_slice(), Some(HoleKind::Number)),
                Word::Past => (b"The requested end is in the past".as_slice(), None),
                Word::Exhausted => (b"The request bound is exhausted".as_slice(), None),
                Word::SignedOut => (b"Sign in before moving the end".as_slice(), None),
                Word::NotAllowed => (b"This request is not allowed".as_slice(), None),
                Word::Conflict => (b"The environment facts conflict".as_slice(), None),
                Word::Invalid => (b"The request is invalid".as_slice(), None),
                Word::SystemRequestBudgetNotKept => {
                    (b"System request budget: not kept by this connector".as_slice(), None)
                }
                Word::LastSystemResponseNotKept => {
                    (b"What the system last said: not kept by this connector".as_slice(), None)
                }
                Word::ReadLimit => (b"Read capacity: ".as_slice(), Some(HoleKind::Number)),
            };
            let (holes, segments): (Box<[HoleShape]>, Box<[Segment]>) = match hole {
                Some(kind) => (
                    Box::new([HoleShape { kind, bytes: if kind == HoleKind::Name { text } else { 0 } }]),
                    Box::new([Segment::Text(copy_of(literal)), Segment::Hole { index: 0, kind }]),
                ),
                None => (Box::new([]), Box::new([Segment::Text(copy_of(literal))])),
            };
            shapes.push(PhraseShape { id: word.id(connector), holes, plural: None }).expect("closed shapes fit");
            phrases
                .push(Phrase {
                    id: word.id(connector),
                    forms: Box::new([Form { class: PluralClass::Other, segments }]),
                })
                .expect("closed English phrases fit");
        }
        let language = Language {
            plural: PluralRule::English,
            span_unit: SpanUnit::Seconds,
            wall: WallFormat::Seconds,
            phrases: phrases.into_boxed(),
        };
        let book = Phrasebook::load(shapes.into_boxed(), Box::new([language]), &phrase_limits(text))?;
        Ok(Self { book, language: 0, connector })
    }
    #[must_use]
    pub fn say(&self, word: Word, holes: &[Hole]) -> Box<[u8]> {
        self.book.say(self.language, word.id(self.connector), holes).expect("checked family phrase").text
    }
    pub fn text(&self, b: &mut Builder, word: Word, holes: &[Hole]) {
        b.text(Run::Plain, &self.say(word, holes));
    }
    pub fn status(&self, b: &mut Builder, word: Word, holes: &[Hole]) {
        let text = self.say(word, holes);
        b.open(Role::Status);
        b.name(&text);
        b.text(Run::Plain, &text);
        b.close();
    }
    #[must_use]
    pub fn capacity(&self) -> Option<u32> {
        let mut most = 0;
        for word in ALL {
            most = most.max(self.book.capacity(word.id(self.connector))?);
        }
        Some(most)
    }
}
#[must_use]
pub const fn phrase_limits(text: u32) -> PhraseLimits {
    PhraseLimits { languages: 1, phrases: 111, holes: 1, segments: 222, text_bytes: 8192, hole_bytes: text }
}
