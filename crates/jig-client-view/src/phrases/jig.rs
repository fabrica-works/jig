//! Jig's closed words and typed holes (client/domain/view.md, section 7).
use super::{HoleKind, HoleShape, PhraseId, PhraseShape};
use crate::Owner;
use alloc::boxed::Box;
use skein_lib::List;
/// Each sentence jig says, independent of language word order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JigPhrase {
    /// Loading navigation.
    LoadingNavigation,
    /// By (0).
    ByNamed,
    /// Refused.
    Refused,
    /// Leaving.
    Leaving,
    /// Live.
    Live,
    /// Offline.
    Offline,
    /// Connecting.
    Starting,
    /// Behind: waiting for current facts.
    Behind,
    /// (0) need you.
    Needs,
    /// More than (0) need you.
    MoreNeeds,
    /// (0) updates not read.
    Updates,
    /// More than (0) updates not read.
    MoreUpdates,
    /// Signed in as (0).
    SignedIn,
    /// Signed out.
    SignedOut,
    /// Work.
    Frame,
    /// Navigation.
    Navigation,
    /// Project: (0).
    Project,
    /// All projects.
    AllProjects,
    /// Go to task number.
    TaskNumber,
    /// Inbox.
    Inbox,
    /// Conversations.
    Conversations,
    /// Project.
    ProjectPage,
    /// Deployment.
    Deployment,
    /// Task (0).
    TaskPage,
    /// Open.
    Open,
    /// Being decided: review the confirmation.
    Confirming,
    /// Being decided: waiting for the engine.
    InFlight,
    /// Being decided: retrying.
    Retrying,
    /// Being decided: waiting for the engine connection or your sign-in.
    Parked,
    /// Outcome unknown: inspect the current object before acting again.
    Unknown,
    /// Inspect current object.
    LookAgain,
    /// Finding how this ended.
    Looking,
    /// Ended; its final facts are no longer available.
    GoneUnknown,
    /// Changed.
    Changed,
    /// Proposed by task (0), proposal (1).
    Proposed,
    /// Completed.
    Done,
    /// Failed: (0).
    Failed,
    /// Cancelled: (0).
    Cancelled,
    /// Withdrawn: (0).
    Withdrawn,
    /// By party (0).
    ByParty,
    /// By task (0).
    ByTask,
    /// By the deployment.
    ByDeployment,
    /// At (0).
    At,
    /// Accepted.
    Accepted,
    /// Rejected: (0).
    Rejected,
    /// Passed up.
    PassedUp,
    /// Released.
    Released,
    /// Released: make the effect again.
    ReleasedAgain,
    /// Released: do not make the effect again.
    ReleasedWithout,
    /// Left held: (0).
    LeftHeld,
    /// Decided first by (0).
    DecidedFirst,
    /// Made chat (0).
    MadeChat,
    /// Sent words to task (0), message (1).
    MadeWords,
    /// Made goal (0).
    MadeGoal,
    /// Made task (0).
    MadeTask,
    /// Made service party (0).
    MadeService,
    /// Answered person task (0).
    MadeResult,
    /// Made note (0).
    MadeNote,
    /// Project (0): requires role (1).
    RoleMissing,
    /// Your pool has (0) left; this needs (1).
    BudgetMissing,
    /// Requires effect kind (0) on connector (1), resource (2).
    GrantMissing,
    /// Delegation needs depth (0); allowed (1).
    DepthMissing,
    /// Deadline (0) exceeds latest allowed (1).
    DeadlineMissing,
    /// Needs (0) tasks; (1) left.
    TasksMissing,
    /// Agent.
    Agent,
    /// Procedure.
    Procedure,
    /// Person.
    Person,
    /// Person holding a role.
    RoleExecutor,
    /// Executor kind not allowed.
    KindMissing,
    /// Project (0): live task limit (1).
    LimitLive,
    /// Task (0): inbox limit (1).
    LimitInbox,
    /// Words exceed (0) bytes.
    LimitWords,
    /// Conversation word snapshot limit (0).
    LimitConversation,
    /// Field limit reached.
    LimitFields,
    /// Kept record limit reached.
    LimitRecords,
    /// Request limit reached.
    LimitRequests,
    /// Read limit reached.
    LimitReads,
    /// Watch limit reached.
    LimitWatches,
    /// Working set limit reached.
    LimitObjects,
    /// The hold or its reason changed.
    MovedHold,
    /// The specification changed.
    MovedSpec,
    /// The question or its options changed.
    MovedPerson,
    /// The note changed.
    MovedNote,
    /// The recipient changed.
    MovedWaiting,
    /// Current revision (0).
    Revision,
    /// No visible object with that name.
    NoObject,
    /// This act is not offered.
    NotOffered,
    /// The request key names another act.
    KeyConflict,
    /// (0).
    Amount,
    /// Waiting (0) min.
    WaitingSpan,
    /// (0) min.
    Span,
    /// Object.
    Object,
    /// Proposal.
    Proposal,
    /// Escalation.
    Escalation,
    /// Question.
    Question,
    /// Person task.
    PersonTask,
    /// Result.
    Result,
    /// Note.
    Note,
    /// In project (0).
    ContextProject,
    /// Task (0).
    ContextTask,
    /// Task (0): (1).
    ChipNumber,
    /// Waiting.
    Waiting,
    /// Waiting for news.
    Idle,
    /// Wakes on any news.
    WakeAny,
    /// Wakes at the next scheduled time.
    WakeAt,
    /// Wakes on a delegate result.
    WakeResult,
    /// Wakes on subscribed news.
    WakeTopic,
    /// Wakes on inbox messages.
    WakeInbox,
    /// Running.
    Running,
    /// Writing.
    Writing,
    /// Using tool (0).
    Tool,
    /// Attempt (0).
    Attempt,
    /// Retrying at (0): try (1) of (2).
    RetryAt,
    /// Waiting for your words.
    WaitingWords,
    /// Host lost.
    HostLost,
    /// Temporary failure.
    Temporary,
    /// Permanent failure.
    Permanent,
    /// Stalled.
    Stall,
    /// Reason: (0).
    Reason,
    /// Closing: (0) delegates and (1) effects settling.
    Closing,
    /// Held.
    Held,
    /// Ended.
    Ended,
    /// Waiting for party (0).
    WaitingParty,
    /// Waiting for task (0).
    WaitingTask,
    /// Waiting for project (0), role (1).
    WaitingRole,
    /// Since (0).
    Since,
    /// Funds (0); (1) left now, (2) after.
    Funds,
    /// From this pool.
    Pool,
    /// Why this is proposed.
    ProposalWhy,
    /// The accepted tasks will be yours.
    OwnTasks,
    /// Tasks acceptance would make.
    Batch,
    /// Proposed task (0).
    BatchTask,
    /// After proposed task (0).
    AfterTask,
    /// Budget (0).
    Budget,
    /// Before.
    Before,
    /// After.
    After,
    /// Delegation depth (0).
    Depth,
    /// Tasks allowed (0).
    TasksAllowed,
    /// Effect kind (0) on connector (1), resource (2).
    Grant,
    /// Tool (0) allowed.
    ToolAllowed,
    /// Passed by this holder.
    PassedBy,
    /// Widen authority.
    Widen,
    /// Amend task (0).
    AmendTask,
    /// Effect.
    Effect,
    /// Your answer.
    AnswerField,
    /// State to approve: (0).
    ApprovalState,
    /// Choose an option.
    Options,
    /// Cannot do it.
    CannotDo,
    /// Taken by (0).
    TakenBy,
    /// Becomes news for its requester after (0) min.
    StallSpan,
    /// Verdict (0).
    Verdict,
    /// Choice (0), by (1).
    ChoiceResult,
    /// Approved (0), by (1).
    ApprovalYes,
    /// Did not approve (0), by (1).
    ApprovalNo,
    /// Open at first reply not read: (0).
    FirstUnread,
    /// Reply.
    Reply,
    /// Loading connector content.
    LoadingPart,
    /// Loading object.
    LoadingObject,
    /// Write.
    ActWrite,
    /// Answer.
    ActAnswer,
    /// Take it.
    ActTake,
    /// Hand it back.
    ActHandBack,
    /// Accept.
    ActAccept,
    /// Reject.
    ActReject,
    /// Pass up.
    ActPassUp,
    /// Release.
    ActRelease,
    /// Leave it held.
    ActLeaveHeld,
    /// Amend.
    ActAmend,
    /// Amend and release.
    ActAmendRelease,
    /// Stop.
    ActStop,
    /// Cancel.
    ActCancel,
    /// Close.
    ActClose,
    /// Take over.
    ActTakeOver,
    /// Set goal.
    ActSetGoal,
    /// Prioritise.
    ActPrioritise,
    /// Mark read.
    ActMarkSeen,
    /// Edit note.
    ActEditNote,
    /// Delete note.
    ActDeleteNote,
    /// Adopt resource.
    ActAdopt,
    /// Change policy.
    ActChangePolicy,
    /// Set roles.
    ActSetRoles,
    /// Make service.
    ActMakeService,
    /// Set pool.
    ActSetPool,
    /// Chat about it.
    ChatAbout,
    /// Starting.
    TaskStarting,
    /// Subscribed topic (0).
    Topic,
    /// For (0) min.
    ElapsedSpan,
    /// Your reason.
    ReasonField,
    /// Loading phase details.
    LoadingPhase,
    /// Deadline (0).
    Deadline,
    /// Held: out of tries.
    TriesExhausted,
    /// Held: stalled on (0).
    StalledOn,
    /// Held: dependency (0) ended unsuccessfully.
    DependencyEnded,
    /// Held: the system changed.
    DriftHold,
    /// Held: the effect failed for good.
    EffectFailedHold,
    /// Held: the effect may or may not have been made; inspect the system before deciding.
    EffectUncertainHold,
    /// Held: spent (0), budget (1).
    BudgetHold,
    /// Held: past deadline (0).
    DeadlineHold,
    /// Held: stopped by (0) at (1).
    StoppedHold,
    /// Held: waiting for a resource past its bound.
    ResourceHold,
    /// Held: the pool lost its allocation.
    PoolHold,
    /// Held: the procedure needs a decision.
    ProcedureHold,
    /// Loading task (0) phase facts.
    LoadingDependency,
}
impl JigPhrase {
    /// Stable phrase identity within jig's owner.
    #[must_use]
    #[expect(clippy::too_many_lines, reason = "closed phrase-number data is kept in one exhaustive table")]
    pub const fn id(self) -> PhraseId {
        let number = match self {
            Self::LoadingNavigation => 0,
            Self::ByNamed => 1,
            Self::Refused => 2,
            Self::Leaving => 3,
            Self::Live => 4,
            Self::Offline => 5,
            Self::Starting => 6,
            Self::Behind => 7,
            Self::Needs => 8,
            Self::MoreNeeds => 9,
            Self::Updates => 10,
            Self::MoreUpdates => 11,
            Self::SignedIn => 12,
            Self::SignedOut => 13,
            Self::Frame => 14,
            Self::Navigation => 15,
            Self::Project => 16,
            Self::AllProjects => 17,
            Self::TaskNumber => 18,
            Self::Inbox => 19,
            Self::Conversations => 20,
            Self::ProjectPage => 21,
            Self::Deployment => 22,
            Self::TaskPage => 23,
            Self::Open => 24,
            Self::Confirming => 25,
            Self::InFlight => 26,
            Self::Retrying => 27,
            Self::Parked => 28,
            Self::Unknown => 29,
            Self::LookAgain => 30,
            Self::Looking => 31,
            Self::GoneUnknown => 32,
            Self::Changed => 33,
            Self::Proposed => 34,
            Self::Done => 35,
            Self::Failed => 36,
            Self::Cancelled => 37,
            Self::Withdrawn => 38,
            Self::ByParty => 39,
            Self::ByTask => 40,
            Self::ByDeployment => 41,
            Self::At => 42,
            Self::Accepted => 43,
            Self::Rejected => 44,
            Self::PassedUp => 45,
            Self::Released => 46,
            Self::ReleasedAgain => 47,
            Self::ReleasedWithout => 48,
            Self::LeftHeld => 49,
            Self::DecidedFirst => 50,
            Self::MadeChat => 51,
            Self::MadeWords => 52,
            Self::MadeGoal => 53,
            Self::MadeTask => 54,
            Self::MadeService => 55,
            Self::MadeResult => 56,
            Self::MadeNote => 57,
            Self::RoleMissing => 58,
            Self::BudgetMissing => 59,
            Self::GrantMissing => 60,
            Self::DepthMissing => 61,
            Self::DeadlineMissing => 62,
            Self::TasksMissing => 63,
            Self::Agent => 64,
            Self::Procedure => 65,
            Self::Person => 66,
            Self::RoleExecutor => 67,
            Self::KindMissing => 68,
            Self::LimitLive => 69,
            Self::LimitInbox => 70,
            Self::LimitWords => 71,
            Self::LimitConversation => 72,
            Self::LimitFields => 73,
            Self::LimitRecords => 74,
            Self::LimitRequests => 75,
            Self::LimitReads => 76,
            Self::LimitWatches => 77,
            Self::LimitObjects => 78,
            Self::MovedHold => 79,
            Self::MovedSpec => 80,
            Self::MovedPerson => 81,
            Self::MovedNote => 82,
            Self::MovedWaiting => 83,
            Self::Revision => 84,
            Self::NoObject => 85,
            Self::NotOffered => 86,
            Self::KeyConflict => 87,
            Self::Amount => 88,
            Self::WaitingSpan => 89,
            Self::Span => 90,
            Self::Object => 91,
            Self::Proposal => 92,
            Self::Escalation => 93,
            Self::Question => 94,
            Self::PersonTask => 95,
            Self::Result => 96,
            Self::Note => 97,
            Self::ContextProject => 98,
            Self::ContextTask => 99,
            Self::ChipNumber => 100,
            Self::Waiting => 101,
            Self::Idle => 102,
            Self::WakeAny => 103,
            Self::WakeAt => 104,
            Self::WakeResult => 105,
            Self::WakeTopic => 106,
            Self::WakeInbox => 107,
            Self::Running => 108,
            Self::Writing => 109,
            Self::Tool => 110,
            Self::Attempt => 111,
            Self::RetryAt => 112,
            Self::WaitingWords => 113,
            Self::HostLost => 114,
            Self::Temporary => 115,
            Self::Permanent => 116,
            Self::Stall => 117,
            Self::Reason => 118,
            Self::Closing => 119,
            Self::Held => 120,
            Self::Ended => 121,
            Self::WaitingParty => 122,
            Self::WaitingTask => 123,
            Self::WaitingRole => 124,
            Self::Since => 125,
            Self::Funds => 126,
            Self::Pool => 127,
            Self::ProposalWhy => 128,
            Self::OwnTasks => 129,
            Self::Batch => 130,
            Self::BatchTask => 131,
            Self::AfterTask => 132,
            Self::Budget => 133,
            Self::Before => 134,
            Self::After => 135,
            Self::Depth => 136,
            Self::TasksAllowed => 137,
            Self::Grant => 138,
            Self::ToolAllowed => 139,
            Self::PassedBy => 140,
            Self::Widen => 141,
            Self::AmendTask => 142,
            Self::Effect => 143,
            Self::AnswerField => 144,
            Self::ApprovalState => 145,
            Self::Options => 146,
            Self::CannotDo => 147,
            Self::TakenBy => 148,
            Self::StallSpan => 149,
            Self::Verdict => 150,
            Self::ChoiceResult => 151,
            Self::ApprovalYes => 152,
            Self::ApprovalNo => 153,
            Self::FirstUnread => 154,
            Self::Reply => 155,
            Self::LoadingPart => 156,
            Self::LoadingObject => 157,
            Self::ActWrite => 158,
            Self::ActAnswer => 159,
            Self::ActTake => 160,
            Self::ActHandBack => 161,
            Self::ActAccept => 162,
            Self::ActReject => 163,
            Self::ActPassUp => 164,
            Self::ActRelease => 165,
            Self::ActLeaveHeld => 166,
            Self::ActAmend => 167,
            Self::ActAmendRelease => 168,
            Self::ActStop => 169,
            Self::ActCancel => 170,
            Self::ActClose => 171,
            Self::ActTakeOver => 172,
            Self::ActSetGoal => 173,
            Self::ActPrioritise => 174,
            Self::ActMarkSeen => 175,
            Self::ActEditNote => 176,
            Self::ActDeleteNote => 177,
            Self::ActAdopt => 178,
            Self::ActChangePolicy => 179,
            Self::ActSetRoles => 180,
            Self::ActMakeService => 181,
            Self::ActSetPool => 182,
            Self::ChatAbout => 183,
            Self::TaskStarting => 184,
            Self::Topic => 185,
            Self::ElapsedSpan => 186,
            Self::ReasonField => 187,
            Self::LoadingPhase => 188,
            Self::Deadline => 189,
            Self::TriesExhausted => 190,
            Self::StalledOn => 191,
            Self::DependencyEnded => 192,
            Self::DriftHold => 193,
            Self::EffectFailedHold => 194,
            Self::EffectUncertainHold => 195,
            Self::BudgetHold => 196,
            Self::DeadlineHold => 197,
            Self::StoppedHold => 198,
            Self::ResourceHold => 199,
            Self::PoolHold => 200,
            Self::ProcedureHold => 201,
            Self::LoadingDependency => 202,
        };
        PhraseId { owner: Owner::Jig, number }
    }
    /// Hole contract checked for every supplied language.
    #[must_use]
    #[expect(clippy::too_many_lines, reason = "every phrase declares its holes in one exhaustive table")]
    pub fn shape(self, text: u32) -> PhraseShape {
        let holes: Box<[HoleShape]> = match self {
            Self::LoadingNavigation
            | Self::Refused
            | Self::Leaving
            | Self::Live
            | Self::Offline
            | Self::Starting
            | Self::Behind
            | Self::SignedOut
            | Self::Frame
            | Self::Navigation
            | Self::AllProjects
            | Self::TaskNumber
            | Self::Inbox
            | Self::Conversations
            | Self::ProjectPage
            | Self::Deployment
            | Self::Open
            | Self::Confirming
            | Self::InFlight
            | Self::Retrying
            | Self::Parked
            | Self::Unknown
            | Self::LookAgain
            | Self::Looking
            | Self::GoneUnknown
            | Self::Changed
            | Self::Done
            | Self::ByDeployment
            | Self::Accepted
            | Self::PassedUp
            | Self::Released
            | Self::ReleasedAgain
            | Self::ReleasedWithout
            | Self::Agent
            | Self::Procedure
            | Self::Person
            | Self::RoleExecutor
            | Self::KindMissing
            | Self::LimitFields
            | Self::LimitRecords
            | Self::LimitRequests
            | Self::LimitReads
            | Self::LimitWatches
            | Self::LimitObjects
            | Self::MovedHold
            | Self::MovedSpec
            | Self::MovedPerson
            | Self::MovedNote
            | Self::MovedWaiting
            | Self::NoObject
            | Self::NotOffered
            | Self::KeyConflict
            | Self::Object
            | Self::Proposal
            | Self::Escalation
            | Self::Question
            | Self::PersonTask
            | Self::Result
            | Self::Note
            | Self::Waiting
            | Self::Idle
            | Self::WakeAny
            | Self::WakeAt
            | Self::WakeResult
            | Self::WakeTopic
            | Self::WakeInbox
            | Self::Running
            | Self::Writing
            | Self::WaitingWords
            | Self::HostLost
            | Self::Temporary
            | Self::Permanent
            | Self::Stall
            | Self::Held
            | Self::Ended
            | Self::Pool
            | Self::ProposalWhy
            | Self::OwnTasks
            | Self::Batch
            | Self::Before
            | Self::After
            | Self::PassedBy
            | Self::Widen
            | Self::Effect
            | Self::AnswerField
            | Self::Options
            | Self::CannotDo
            | Self::Reply
            | Self::LoadingPart
            | Self::LoadingObject
            | Self::ActWrite
            | Self::ActAnswer
            | Self::ActTake
            | Self::ActHandBack
            | Self::ActAccept
            | Self::ActReject
            | Self::ActPassUp
            | Self::ActRelease
            | Self::ActLeaveHeld
            | Self::ActAmend
            | Self::ActAmendRelease
            | Self::ActStop
            | Self::ActCancel
            | Self::ActClose
            | Self::ActTakeOver
            | Self::ActSetGoal
            | Self::ActPrioritise
            | Self::ActMarkSeen
            | Self::ActEditNote
            | Self::ActDeleteNote
            | Self::ActAdopt
            | Self::ActChangePolicy
            | Self::ActSetRoles
            | Self::ActMakeService
            | Self::ActSetPool
            | Self::ChatAbout
            | Self::TaskStarting
            | Self::ReasonField
            | Self::LoadingPhase
            | Self::TriesExhausted
            | Self::DriftHold
            | Self::EffectFailedHold
            | Self::EffectUncertainHold
            | Self::ResourceHold
            | Self::PoolHold
            | Self::ProcedureHold => Box::new([]),
            Self::ByNamed
            | Self::SignedIn
            | Self::Project
            | Self::Failed
            | Self::Cancelled
            | Self::Withdrawn
            | Self::Rejected
            | Self::LeftHeld
            | Self::DecidedFirst
            | Self::Tool
            | Self::Reason
            | Self::ApprovalState
            | Self::TakenBy
            | Self::StalledOn => Box::new([HoleShape { kind: HoleKind::Name, bytes: text }]),
            Self::Needs
            | Self::MoreNeeds
            | Self::Updates
            | Self::MoreUpdates
            | Self::TaskPage
            | Self::ByParty
            | Self::ByTask
            | Self::MadeChat
            | Self::MadeGoal
            | Self::MadeTask
            | Self::MadeService
            | Self::MadeResult
            | Self::MadeNote
            | Self::LimitWords
            | Self::LimitConversation
            | Self::Revision
            | Self::ContextProject
            | Self::ContextTask
            | Self::Attempt
            | Self::WaitingParty
            | Self::WaitingTask
            | Self::BatchTask
            | Self::AfterTask
            | Self::Depth
            | Self::TasksAllowed
            | Self::ToolAllowed
            | Self::AmendTask
            | Self::Verdict
            | Self::FirstUnread
            | Self::Topic
            | Self::DependencyEnded
            | Self::LoadingDependency => Box::new([HoleShape { kind: HoleKind::Number, bytes: 0 }]),
            Self::Proposed
            | Self::MadeWords
            | Self::RoleMissing
            | Self::DepthMissing
            | Self::TasksMissing
            | Self::LimitLive
            | Self::LimitInbox
            | Self::Closing
            | Self::WaitingRole => Box::new([
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Number, bytes: 0 },
            ]),
            Self::At | Self::Since | Self::Deadline | Self::DeadlineHold => {
                Box::new([HoleShape { kind: HoleKind::Wall, bytes: 0 }])
            }
            Self::BudgetMissing | Self::BudgetHold => Box::new([
                HoleShape { kind: HoleKind::Amount, bytes: 0 },
                HoleShape { kind: HoleKind::Amount, bytes: 0 },
            ]),
            Self::GrantMissing | Self::Grant => Box::new([
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Name, bytes: text },
            ]),
            Self::DeadlineMissing => {
                Box::new([HoleShape { kind: HoleKind::Wall, bytes: 0 }, HoleShape { kind: HoleKind::Wall, bytes: 0 }])
            }
            Self::Amount | Self::Budget => Box::new([HoleShape { kind: HoleKind::Amount, bytes: 0 }]),
            Self::WaitingSpan | Self::Span | Self::StallSpan | Self::ElapsedSpan => {
                Box::new([HoleShape { kind: HoleKind::Span, bytes: 0 }])
            }
            Self::ChipNumber | Self::ChoiceResult => Box::new([
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Name, bytes: text },
            ]),
            Self::RetryAt => Box::new([
                HoleShape { kind: HoleKind::Wall, bytes: 0 },
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Number, bytes: 0 },
            ]),
            Self::Funds => Box::new([
                HoleShape { kind: HoleKind::Amount, bytes: 0 },
                HoleShape { kind: HoleKind::Amount, bytes: 0 },
                HoleShape { kind: HoleKind::Amount, bytes: 0 },
            ]),
            Self::ApprovalYes | Self::ApprovalNo => Box::new([
                HoleShape { kind: HoleKind::Name, bytes: text },
                HoleShape { kind: HoleKind::Name, bytes: text },
            ]),
            Self::StoppedHold => Box::new([
                HoleShape { kind: HoleKind::Name, bytes: text },
                HoleShape { kind: HoleKind::Wall, bytes: 0 },
            ]),
        };
        PhraseShape { id: self.id(), holes, plural: None }
    }
}
/// Every jig phrase; loading checks this whole contract.
pub const ALL: [JigPhrase; 203] = [
    JigPhrase::LoadingNavigation,
    JigPhrase::ByNamed,
    JigPhrase::Refused,
    JigPhrase::Leaving,
    JigPhrase::Live,
    JigPhrase::Offline,
    JigPhrase::Starting,
    JigPhrase::Behind,
    JigPhrase::Needs,
    JigPhrase::MoreNeeds,
    JigPhrase::Updates,
    JigPhrase::MoreUpdates,
    JigPhrase::SignedIn,
    JigPhrase::SignedOut,
    JigPhrase::Frame,
    JigPhrase::Navigation,
    JigPhrase::Project,
    JigPhrase::AllProjects,
    JigPhrase::TaskNumber,
    JigPhrase::Inbox,
    JigPhrase::Conversations,
    JigPhrase::ProjectPage,
    JigPhrase::Deployment,
    JigPhrase::TaskPage,
    JigPhrase::Open,
    JigPhrase::Confirming,
    JigPhrase::InFlight,
    JigPhrase::Retrying,
    JigPhrase::Parked,
    JigPhrase::Unknown,
    JigPhrase::LookAgain,
    JigPhrase::Looking,
    JigPhrase::GoneUnknown,
    JigPhrase::Changed,
    JigPhrase::Proposed,
    JigPhrase::Done,
    JigPhrase::Failed,
    JigPhrase::Cancelled,
    JigPhrase::Withdrawn,
    JigPhrase::ByParty,
    JigPhrase::ByTask,
    JigPhrase::ByDeployment,
    JigPhrase::At,
    JigPhrase::Accepted,
    JigPhrase::Rejected,
    JigPhrase::PassedUp,
    JigPhrase::Released,
    JigPhrase::ReleasedAgain,
    JigPhrase::ReleasedWithout,
    JigPhrase::LeftHeld,
    JigPhrase::DecidedFirst,
    JigPhrase::MadeChat,
    JigPhrase::MadeWords,
    JigPhrase::MadeGoal,
    JigPhrase::MadeTask,
    JigPhrase::MadeService,
    JigPhrase::MadeResult,
    JigPhrase::MadeNote,
    JigPhrase::RoleMissing,
    JigPhrase::BudgetMissing,
    JigPhrase::GrantMissing,
    JigPhrase::DepthMissing,
    JigPhrase::DeadlineMissing,
    JigPhrase::TasksMissing,
    JigPhrase::Agent,
    JigPhrase::Procedure,
    JigPhrase::Person,
    JigPhrase::RoleExecutor,
    JigPhrase::KindMissing,
    JigPhrase::LimitLive,
    JigPhrase::LimitInbox,
    JigPhrase::LimitWords,
    JigPhrase::LimitConversation,
    JigPhrase::LimitFields,
    JigPhrase::LimitRecords,
    JigPhrase::LimitRequests,
    JigPhrase::LimitReads,
    JigPhrase::LimitWatches,
    JigPhrase::LimitObjects,
    JigPhrase::MovedHold,
    JigPhrase::MovedSpec,
    JigPhrase::MovedPerson,
    JigPhrase::MovedNote,
    JigPhrase::MovedWaiting,
    JigPhrase::Revision,
    JigPhrase::NoObject,
    JigPhrase::NotOffered,
    JigPhrase::KeyConflict,
    JigPhrase::Amount,
    JigPhrase::WaitingSpan,
    JigPhrase::Span,
    JigPhrase::Object,
    JigPhrase::Proposal,
    JigPhrase::Escalation,
    JigPhrase::Question,
    JigPhrase::PersonTask,
    JigPhrase::Result,
    JigPhrase::Note,
    JigPhrase::ContextProject,
    JigPhrase::ContextTask,
    JigPhrase::ChipNumber,
    JigPhrase::Waiting,
    JigPhrase::Idle,
    JigPhrase::WakeAny,
    JigPhrase::WakeAt,
    JigPhrase::WakeResult,
    JigPhrase::WakeTopic,
    JigPhrase::WakeInbox,
    JigPhrase::Running,
    JigPhrase::Writing,
    JigPhrase::Tool,
    JigPhrase::Attempt,
    JigPhrase::RetryAt,
    JigPhrase::WaitingWords,
    JigPhrase::HostLost,
    JigPhrase::Temporary,
    JigPhrase::Permanent,
    JigPhrase::Stall,
    JigPhrase::Reason,
    JigPhrase::Closing,
    JigPhrase::Held,
    JigPhrase::Ended,
    JigPhrase::WaitingParty,
    JigPhrase::WaitingTask,
    JigPhrase::WaitingRole,
    JigPhrase::Since,
    JigPhrase::Funds,
    JigPhrase::Pool,
    JigPhrase::ProposalWhy,
    JigPhrase::OwnTasks,
    JigPhrase::Batch,
    JigPhrase::BatchTask,
    JigPhrase::AfterTask,
    JigPhrase::Budget,
    JigPhrase::Before,
    JigPhrase::After,
    JigPhrase::Depth,
    JigPhrase::TasksAllowed,
    JigPhrase::Grant,
    JigPhrase::ToolAllowed,
    JigPhrase::PassedBy,
    JigPhrase::Widen,
    JigPhrase::AmendTask,
    JigPhrase::Effect,
    JigPhrase::AnswerField,
    JigPhrase::ApprovalState,
    JigPhrase::Options,
    JigPhrase::CannotDo,
    JigPhrase::TakenBy,
    JigPhrase::StallSpan,
    JigPhrase::Verdict,
    JigPhrase::ChoiceResult,
    JigPhrase::ApprovalYes,
    JigPhrase::ApprovalNo,
    JigPhrase::FirstUnread,
    JigPhrase::Reply,
    JigPhrase::LoadingPart,
    JigPhrase::LoadingObject,
    JigPhrase::ActWrite,
    JigPhrase::ActAnswer,
    JigPhrase::ActTake,
    JigPhrase::ActHandBack,
    JigPhrase::ActAccept,
    JigPhrase::ActReject,
    JigPhrase::ActPassUp,
    JigPhrase::ActRelease,
    JigPhrase::ActLeaveHeld,
    JigPhrase::ActAmend,
    JigPhrase::ActAmendRelease,
    JigPhrase::ActStop,
    JigPhrase::ActCancel,
    JigPhrase::ActClose,
    JigPhrase::ActTakeOver,
    JigPhrase::ActSetGoal,
    JigPhrase::ActPrioritise,
    JigPhrase::ActMarkSeen,
    JigPhrase::ActEditNote,
    JigPhrase::ActDeleteNote,
    JigPhrase::ActAdopt,
    JigPhrase::ActChangePolicy,
    JigPhrase::ActSetRoles,
    JigPhrase::ActMakeService,
    JigPhrase::ActSetPool,
    JigPhrase::ChatAbout,
    JigPhrase::TaskStarting,
    JigPhrase::Topic,
    JigPhrase::ElapsedSpan,
    JigPhrase::ReasonField,
    JigPhrase::LoadingPhase,
    JigPhrase::Deadline,
    JigPhrase::TriesExhausted,
    JigPhrase::StalledOn,
    JigPhrase::DependencyEnded,
    JigPhrase::DriftHold,
    JigPhrase::EffectFailedHold,
    JigPhrase::EffectUncertainHold,
    JigPhrase::BudgetHold,
    JigPhrase::DeadlineHold,
    JigPhrase::StoppedHold,
    JigPhrase::ResourceHold,
    JigPhrase::PoolHold,
    JigPhrase::ProcedureHold,
    JigPhrase::LoadingDependency,
];
/// Shapes with names bounded by the client's text capacity.
#[must_use]
pub fn shapes(text: u32) -> Box<[PhraseShape]> {
    let mut rows = List::with_capacity(203);
    for phrase in ALL {
        rows.push(phrase.shape(text)).expect("all jig shapes fit");
    }
    rows.into_boxed()
}
