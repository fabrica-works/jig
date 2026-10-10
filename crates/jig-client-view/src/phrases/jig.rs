//! Jig's closed words and typed holes (client/domain/view.md, section 7).
use super::{HoleKind, HoleShape, PhraseId, PhraseShape};
use crate::Owner;
use alloc::boxed::Box;
use skein_lib::List;
/// Each sentence jig says, independent of language word order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JigPhrase {
    PolicyPeriodSpend,
    PolicyEditRole,
    PolicyRequests,
    PolicyDecides,
    PolicyRequirements,
    PolicyProjections,
    PolicyPermissions,
    PolicyEffect,
    PolicyJudge,
    PolicyGuarded,
    PolicyObserved,
    PolicyMustGuard,
    PolicyGuardNotRequired,
    PolicyPermission,

    /// A proposal made by an actual party.
    ProposedByParty,
    /// A task maker's proposal part.
    PartProposalTask,
    /// A party maker's proposal part.
    PartProposalParty,
    /// The executor stopped progressing.
    HoldStalled,
    /// Dependency task .
    HoldDependency,
    /// Task stopped.
    HoldStopped,
    /// Required resource drifted.
    HoldDrift,
    /// Effects need a decision.
    HoldEffects,
    /// An effect failed permanently.
    HoldEffectFailed,
    /// Uncertain outbox entry .
    HoldUncertainEntry,
    /// Insufficient activation budget.
    HoldBudget,
    /// Deadline passed.
    HoldDeadline,
    /// Resource holds waited too long.
    HoldWaited,
    /// Allocation lost from pool .
    HoldPoolLost,
    /// Procedure needs a decision.
    HoldProcedure,
    /// Executor reported failure.
    FailureRun,
    /// Agent failed.
    FailureAgent,
    /// Result broke its contract.
    FailureInvalid,

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
    /// Closing: (0) delegates.
    Closing,
    /// Conjunction before a connector name in a closing phase.
    ClosingConnector,
    /// The named connector owns the effects being settled.
    ClosingEffects,
    /// No policy recipient may accept this proposal.
    NobodyMayAccept,
    /// Missing proposal approval authority.
    ApprovalAuthorityMissing,
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
    /// Before the hold.
    BeforeHold,
    /// Task (0), attempt (1): spent (2).
    SpentRun,
    /// Task (0): spent (1).
    SpentDelegate,
    /// Review before confirming.
    Confirmation,
    /// Confirm.
    Confirm,
    /// Dismiss.
    Dismiss,
    /// Dangerous action: review its consequences.
    Dangerous,
    /// This will be proposed to its holder.
    WillPropose,
    /// The revision this action names changed; inspect the current facts and open it again.
    ChangedConfirmation,
    /// Required words are missing.
    MissingWords,
    /// Choose an option.
    MissingChoice,
    /// Choose whether to make the effect again.
    MissingAgain,
    /// Supply the amendment.
    MissingChange,
    /// Choose what becomes of every live delegate.
    MissingDelegates,
    /// Supply a priority.
    MissingPriority,
    /// Waiting for this object’s project context.
    MissingContext,
    /// Acceptance makes or changes the following, funded as shown.
    WillAccept,
    /// Reject this proposal and send its proposer your reason.
    WillReject,
    /// Pass this decision to the holder above.
    WillPassUp,
    /// Remove this escalation from your inbox; the task stays held with your reason.
    WillLeaveHeld,
    /// Stop the live run and hold its task.
    WillStop,
    /// Cancel this task and everything below it; already made effects remain.
    WillCancel,
    /// Close this chat after disposing of every live delegate; pending effects settle.
    WillClose,
    /// Become this task’s requester and move its funding.
    WillTakeOver,
    /// Take this role task so others cannot answer it at once.
    WillTake,
    /// Hand this task back to its role.
    WillHandBack,
    /// Send these words to the task.
    WillWrite,
    /// Send the answer matching the task’s contract.
    WillAnswer,
    /// Replace the named specification with these changes.
    WillAmend,
    /// Amend and release in one decision.
    WillAmendRelease,
    /// Change this goal’s priority.
    WillPrioritise,
    /// Replace the named note revision.
    WillEditNote,
    /// Delete this note revision.
    WillDeleteNote,
    /// Make a goal from these words and options.
    WillGoal,
    /// The next run resumes its conversation; tries count again from zero.
    ResumeRelease,
    /// Releasing carries on closing with the kept result.
    ContinueClosing,
    /// Release to wait again.
    ReleaseWait,
    /// Release for the executor to decide afresh.
    ReleaseFresh,
    /// Release with procedure counts reset.
    ReleaseProcedure,
    /// Make the effect again.
    RepeatEffect,
    /// Release without making the effect again; tell its executor so.
    SkipEffect,
    /// Inspect the connector’s system facts; the engine will do as you choose.
    EngineFollowsChoice,
    /// (0) waiting tasks.
    TasksWaiting,
    /// (0) running tasks.
    TasksRunning,
    /// (0) held tasks.
    TasksHeld,
    /// (0) closing tasks.
    TasksClosing,
    /// (0) completed tasks.
    TasksDone,
    /// (0) failed tasks.
    TasksFailed,
    /// (0) cancelled tasks.
    TasksCancelled,
    /// (0) live runs.
    LiveRuns,
    /// (0) proposals will be withdrawn.
    PendingProposals,
    /// (0) effects not yet begun will be cancelled.
    PendingEffects,
    /// (0) begun effects will settle.
    SettlingEffects,
    /// Takeover reserves (0) from your pool.
    TakeoverFunds,
    /// Cancel delegate (0) and its subtree.
    DelegateCancel,
    /// Take over delegate (0), reserving (1).
    DelegateTake,
    /// Choose each delegate’s outcome.
    DelegateChoices,
    /// Write and release together.
    WriteAndRelease,
    /// Write without releasing.
    WriteOnly,
    /// New specification.
    NewSpec,
    /// New instructions.
    NewInstructions,
    /// No instruction text was supplied by the document owner.
    InstructionNotSupplied,
    /// Supplied complete instructions contain no text.
    InstructionEmpty,
    /// New budget.
    NewBudget,
    /// New deadline (UTC, YYYY-MM-DD HH:MM).
    NewDeadline,
    /// Drop dependency (0).
    DropDependency,
    /// New goal priority.
    PriorityField,
    /// Note description.
    NoteDescription,
    /// Note body.
    NoteBody,
    /// This text value is incomplete.
    DocumentTextCut,
    /// The named independent person-task revision (0).
    PersonAddressRevision,
    /// No completion wall was supplied by the owner.
    ResultTimeUnknown,
    /// Exact owner change result kind (0).
    ChangeResultKind,
    /// This note reference list is incomplete.
    NoteReferencesCut,
    /// Charter (0).
    Charter,
    /// Connector (0), procedure code (1), supplied by the goal bounds.
    GoalProcedure,
    /// Priority (0).
    Priority,
    /// Notice.
    Notice,
    /// Outcome unknown for this action; inspect its current facts.
    UnknownAction,
    /// Results through (0).
    ResultsThrough,
    /// Chat (0), replies through (1).
    RepliesThrough,
    /// Failed.
    FailedPhase,
    /// Cancelled.
    CancelledPhase,
    /// Address to party number.
    AddressParty,
    /// Address to task number.
    AddressTask,
    /// Address to role number in this project.
    AddressRole,
    /// Due to run.
    DuePhase,
    /// Preparing a host.
    PreparingPhase,
    /// Claimed by a host.
    ClaimedPhase,
    /// Executor active.
    ActivePhase,
    /// Closing.
    ClosingPhase,
    /// Needs you.
    NeedsYou,
    /// Updates.
    InboxUpdates,
    /// All kinds.
    AllKinds,
    /// All goals.
    AllGoals,
    /// Goal (0).
    OnlyGoal,
    /// New.
    NewUpdate,
    /// For the person on call.
    OnCall,
    /// Passes up at (0).
    PassesUp,
    /// Read earlier.
    ReadEarlier,
    /// Read later.
    ReadLater,
    /// Start of the list.
    WindowStart,
    /// End of the list.
    WindowEnd,
    /// More entries are available.
    MoreRows,
    /// First words.
    NewChat,
    /// Start a conversation.
    StartChat,
    /// Live delegates: (0).
    Delegates,
    /// Sign in.
    SignInPage,
    /// Sign in with (0).
    SignInWith,
    /// Page missing.
    MissingPage,
    /// Client out of date: update or reload this client to continue; pending requests are kept.
    UpdateClient,
    /// Someone else's work: sign in as its owner to continue.
    SomeoneElse,
    /// Agent spend: (0).
    OwnSpend,
    /// Delegate spend: (0).
    DelegateSpend,
    /// Overview.
    Overview,
    /// Activity.
    Activity,
    /// Plan.
    Plan,
    /// History.
    History,
    /// Budget and authority.
    BudgetAuthority,
    /// Connectors.
    Panels,
    /// Goal.
    Goal,
    /// Resources.
    Resources,
    /// Policy.
    Policy,
    /// Roles and services.
    Roles,
    /// Spend.
    Spend,
    /// Recurring tasks.
    Recurring,
    RecurringKey,
    RecurringSkip,
    RecurringWait,
    RecurringLastPeriod,
    RecurringPendingPeriod,
    RecurringCut,
    /// A supplied next run wall time.
    NextRun,
    /// The source has not supplied the next period wall time.
    NextPeriod,
    /// A collective policy holder in project (0), followed by every supplied role.
    WaitingRoles,
    /// The independently named hold revision (0).
    HoldRevision,
    /// The engine's current period number.
    CurrentPeriod,
    /// When its retained period ledger opened.
    PeriodBegan,
    /// Settings.
    Settings,
    /// Goals.
    Goals,
    /// Tasks.
    Tasks,
    /// Notes.
    Notes,
    /// Reserved: (0).
    Reserved,
    /// Left: (0).
    Left,
    /// Total budget: (0).
    TotalBudget,
    /// Own spend: (0).
    OwnBudget,
    /// Ended delegate spend: (0).
    EndedBudget,
    /// Held below: (0).
    HeldBelow,
    /// Fold subtree.
    FoldSubtree,
    /// Open subtree.
    OpenSubtree,
    /// All plan tasks.
    AllPlan,
    /// Held only.
    HeldPlan,
    /// Live only.
    LivePlan,
    /// This task only.
    OwnHistory,
    /// This task and its subtree.
    SubtreeHistory,
    /// Made in batch (0).
    BatchMade,
    /// Requester changed.
    MovedWork,
    /// Goal specification.
    GoalWords,
    /// Goal budget.
    GoalBudget,
    /// Goal deadline (UTC, YYYY-MM-DD HH:MM).
    GoalDeadline,
    /// Goal priority.
    GoalPriority,
    /// Priority orders work competing for one resource.
    PriorityMeaning,
    /// Your pool has left: (0).
    PoolLeft,
    /// Beyond your pool, this will be proposed.
    GoalPropose,
    /// Search notes by description.
    NoteSearch,
    /// Tasks I asked for.
    AskedByMe,
    /// All tasks.
    AllTasks,
    /// Period (0).
    Period,
    /// Role (0).
    RoleNumber,
    /// Party (0).
    PartyNumber,
    /// Allotment: (0).
    Allotment,
    /// Decides kind (0).
    DecidesKind,
    /// Spend per period: (0).
    PolicySpend,
    /// Host (0).
    HostNumber,
    /// Connected.
    HostConnected,
    /// Lost.
    HostDisconnected,
    /// Slots used: (0).
    SlotsUsed,
    /// Slots free: (0).
    SlotsFree,
    /// LLM account (0).
    AccountNumber,
    /// Usable.
    AccountUsable,
    /// Unavailable.
    AccountUnavailable,
    /// Capacity.
    Capacity,
    /// Live tasks: (0).
    TasksLive,
    /// Task limit: (0).
    CapacityLimit,
    /// Grace holds until (0).
    Grace,
    /// Tasks waiting for this account.
    WaitingAccount,
    /// Parked; your words resume it.
    ParkedResume,
    /// Retrying.
    RetryingPhase,
    /// Resource scope: (0).
    ResourceScope,
    /// Result expected.
    ResultExpected,
    /// A report.
    ReportExpected,
    /// Words.
    WordsExpected,
    /// Approval of the named state.
    ApprovalExpected,
    /// Pending proposals: (0).
    ProposalsWaiting,
    /// Conversation.
    ConversationLog,
    /// Being sent.
    WordsBeingSent,
    /// Sent.
    WordsSent,
    /// Read.
    WordsRead,
    /// Resumed its conversation.
    RunResumed,
    /// Started fresh; may have lost earlier detail. Tail events: (0).
    RunFreshTail,
    /// Run finished.
    RunFinished,
    /// Run cancelled.
    RunCancelled,
    /// Input tokens: (0).
    TurnInput,
    /// Output tokens: (0).
    TurnOutput,
    /// Cache tokens read: (0).
    CacheRead,
    /// Cache tokens written: (0).
    CacheWritten,
    /// Turn spent: (0).
    TurnSpent,
    /// Provisional.
    Provisional,
    /// Some turn content did not fit.
    TurnCut,
    /// Uninterpreted block: (0).
    OpaqueDialect,
    /// Opaque bytes: (0).
    OpaqueBytes,
    /// Call (0).
    CallNumber,
    /// Open input and result for (0).
    OpenCall,
    /// Fold input and result for (0).
    FoldCall,
    /// Call failed.
    CallFailed,
    /// Call completed.
    CallCompleted,
    /// Call in progress.
    CallPending,
    /// News topic (0).
    NewsTopic,
    /// Turn (0).
    TurnNumber,
    /// Instructions.
    TurnInstructions,
    PanelPart,
    ResultPart,
    HoldPart,
    OutboxEntry,
    TakenByNumber,
    RunIdle,
    RunDue,
    RunPreparing,
    RunClaimed,
    RunOwnerActive,
    RunBackingOff,
    RunPhaseSince,
    RunPlacement,
    RunUnobserved,
    RunObservedSince,
    RunReportedAt,
    RunActualStart,
    RunStartUnknown,
    WorkPreparing,
    WorkStarting,
    WorkActive,
    WorkWriting,
    WorkTool,
    WorkWaiting,
    WorkEnding,
    RunRetryFailed,
    RunRetryRefused,
    ClosingRun,
    ClosingDelegates,
    ClosingStageEffects,
    ClosingReleases,
    ClosingSettled,
    ClosingStageUnknown,
    ClosingConnectorsUnknown,
    ClosingConnectorsEmpty,
    ClosingPendingEnding,
    HeldPriorLifecycle,
    HeldPriorWaiting,

    EngineLimitWords,
    EngineLimitIdentity,
    EngineLimitRoles,
    EngineLimitArithmetic,
    EngineLimitDuplicate,
    EngineLimitEmpty,
    EngineLimitBatch,
    EngineLimitProjectLiveTasks,
    EngineLimitLiveTasks,
    EngineLimitTree,
    EngineLimitDepth,
    EngineLimitDelegates,
    EngineLimitSubscription,
    EngineLimitDependencies,
    EngineLimitCycle,
    EngineLimitSpec,
    EngineLimitContract,
    EngineLimitAuthority,
    EngineLimitInputs,
    EngineLimitAttempts,
    EngineLimitLiveDelegates,
    EngineLimitRestore,
    EngineLimitRead,
    EngineLimitTurn,
    EngineLimitHoldKind,
    EngineLimitHoldTaken,
    EngineLimitHolds,
    EngineLimitInboxFull,
    EngineLimitNotes,
    EngineLimitRevision,
    EngineLimitQuestion,
    EngineLimitPriority,
    EngineLimitPolicy,
    EngineLimitResource,
    EngineLimitService,
    FailedBare,
    CancelledBare,
    DecidedBare,
    DeletedEnding,
    ByProjectDeployment,
    RoleNotAllowed,
    NoHeldRole,
    HeldOwner,
    HeldMaintainer,
    HeldMember,
    HeldObserver,
    HeldPolicy,
    DisposalMissing,
    DisposalUnknown,
    DisposalRepeated,
    RefusalTakenBy,
    NotWaitingForYou,
    NoFurtherHolder,
    GrantKindMissing,
    FundingMissing,
    DepthMaximum,
    DeadlineAuthorityMissing,
    TasksLeft,
    ToolMissing,
    ScopeMissing,
    ReferenceMissing,
    ResourceAccessMissing,
    ResourceUnavailable,
    AuthorityShapeMissing,
    AgainRequired,
    AgainUnexpected,
    AccountMissing,
    WriterMissing,
    RequiredJudgeMissing,
    FailedJudge,
    UnguardedJudge,
    PartConnector,
    PartResource,
    PartKind,
    PartTask,
    PartProject,
    PartProposal,
    /// Opaque numeric parameter name and value.
    SpecificationNumber,
    /// Opaque bytes parameter name.
    SpecificationBytes,
    /// Parameter name, connector and numbered resource.
    SpecificationResource,
    /// An input task identity.
    SpecificationInput,
    /// The owner supplied an incomplete specification.
    SpecificationCut,
}
impl JigPhrase {
    /// Stable phrase identity within jig's owner.
    #[must_use]
    #[expect(clippy::too_many_lines, reason = "closed phrase-number data is kept in one exhaustive table")]
    pub const fn id(self) -> PhraseId {
        let number = match self {
            Self::PolicyPeriodSpend => 590,
            Self::PolicyEditRole => 591,
            Self::PolicyRequests => 592,
            Self::PolicyDecides => 593,
            Self::PolicyRequirements => 594,
            Self::PolicyProjections => 595,
            Self::PolicyPermissions => 596,
            Self::PolicyEffect => 597,
            Self::PolicyJudge => 598,
            Self::PolicyGuarded => 599,
            Self::PolicyObserved => 600,
            Self::PolicyMustGuard => 601,
            Self::PolicyGuardNotRequired => 602,
            Self::PolicyPermission => 603,

            Self::ProposedByParty => 570,
            Self::PartProposalTask => 571,
            Self::PartProposalParty => 572,
            Self::RunIdle => 530,
            Self::RunDue => 531,
            Self::RunPreparing => 532,
            Self::RunClaimed => 533,
            Self::RunOwnerActive => 534,
            Self::RunBackingOff => 535,
            Self::RunPhaseSince => 536,
            Self::RunPlacement => 537,
            Self::RunUnobserved => 538,
            Self::RunObservedSince => 539,
            Self::RunReportedAt => 540,
            Self::RunActualStart => 541,
            Self::RunStartUnknown => 542,
            Self::WorkPreparing => 543,
            Self::WorkStarting => 544,
            Self::WorkActive => 545,
            Self::WorkWriting => 546,
            Self::WorkTool => 547,
            Self::WorkWaiting => 548,
            Self::WorkEnding => 549,
            Self::RunRetryFailed => 550,
            Self::RunRetryRefused => 551,
            Self::ClosingRun => 552,
            Self::ClosingDelegates => 553,
            Self::ClosingStageEffects => 554,
            Self::ClosingReleases => 555,
            Self::ClosingSettled => 556,
            Self::ClosingStageUnknown => 557,
            Self::ClosingConnectorsUnknown => 558,
            Self::ClosingConnectorsEmpty => 559,
            Self::ClosingPendingEnding => 560,
            Self::HeldPriorLifecycle => 561,
            Self::HeldPriorWaiting => 562,

            Self::SpecificationNumber => 3200,
            Self::SpecificationBytes => 3201,
            Self::SpecificationResource => 3202,
            Self::SpecificationInput => 3203,
            Self::SpecificationCut => 3204,
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
            Self::ClosingConnector => 403,
            Self::ClosingEffects => 404,
            Self::NobodyMayAccept => 405,
            Self::ApprovalAuthorityMissing => 406,
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
            Self::BeforeHold => 203,
            Self::SpentRun => 204,
            Self::SpentDelegate => 205,
            Self::Confirmation => 206,
            Self::Confirm => 207,
            Self::Dismiss => 208,
            Self::Dangerous => 209,
            Self::WillPropose => 210,
            Self::ChangedConfirmation => 211,
            Self::MissingWords => 212,
            Self::MissingChoice => 213,
            Self::MissingAgain => 214,
            Self::MissingChange => 215,
            Self::MissingDelegates => 216,
            Self::MissingPriority => 217,
            Self::MissingContext => 218,
            Self::WillAccept => 219,
            Self::WillReject => 220,
            Self::WillPassUp => 221,
            Self::WillLeaveHeld => 222,
            Self::WillStop => 223,
            Self::WillCancel => 224,
            Self::WillClose => 225,
            Self::WillTakeOver => 226,
            Self::WillTake => 227,
            Self::WillHandBack => 228,
            Self::WillWrite => 229,
            Self::WillAnswer => 230,
            Self::WillAmend => 231,
            Self::WillAmendRelease => 232,
            Self::WillPrioritise => 233,
            Self::WillEditNote => 234,
            Self::WillDeleteNote => 235,
            Self::WillGoal => 236,
            Self::ResumeRelease => 237,
            Self::ContinueClosing => 238,
            Self::ReleaseWait => 239,
            Self::ReleaseFresh => 240,
            Self::ReleaseProcedure => 241,
            Self::RepeatEffect => 242,
            Self::SkipEffect => 243,
            Self::EngineFollowsChoice => 244,
            Self::TasksWaiting => 245,
            Self::TasksRunning => 246,
            Self::TasksHeld => 247,
            Self::TasksClosing => 248,
            Self::TasksDone => 249,
            Self::TasksFailed => 250,
            Self::TasksCancelled => 251,
            Self::LiveRuns => 252,
            Self::PendingProposals => 253,
            Self::PendingEffects => 254,
            Self::SettlingEffects => 255,
            Self::TakeoverFunds => 256,
            Self::DelegateCancel => 257,
            Self::DelegateTake => 258,
            Self::DelegateChoices => 259,
            Self::WriteAndRelease => 260,
            Self::WriteOnly => 261,
            Self::NewSpec => 262,
            Self::NewInstructions => 263,
            Self::InstructionNotSupplied => 580,
            Self::InstructionEmpty => 581,
            Self::NewBudget => 264,
            Self::NewDeadline => 265,
            Self::DropDependency => 266,
            Self::PriorityField => 267,
            Self::NoteDescription => 268,
            Self::NoteBody => 269,
            Self::DocumentTextCut => 3500,
            Self::PersonAddressRevision => 3600,
            Self::ResultTimeUnknown => 3700,
            Self::ChangeResultKind => 3701,
            Self::NoteReferencesCut => 3300,
            Self::Charter => 270,
            Self::GoalProcedure => 510,
            Self::Priority => 271,
            Self::Notice => 272,
            Self::UnknownAction => 273,
            Self::ResultsThrough => 274,
            Self::RepliesThrough => 275,
            Self::FailedPhase => 276,
            Self::CancelledPhase => 277,
            Self::AddressParty => 278,
            Self::AddressTask => 279,
            Self::AddressRole => 280,
            Self::DuePhase => 281,
            Self::PreparingPhase => 282,
            Self::ClaimedPhase => 283,
            Self::ActivePhase => 284,
            Self::ClosingPhase => 285,
            Self::NeedsYou => 286,
            Self::InboxUpdates => 287,
            Self::AllKinds => 288,
            Self::AllGoals => 289,
            Self::OnlyGoal => 290,
            Self::NewUpdate => 291,
            Self::OnCall => 292,
            Self::PassesUp => 293,
            Self::ReadEarlier => 294,
            Self::ReadLater => 295,
            Self::WindowStart => 296,
            Self::WindowEnd => 297,
            Self::MoreRows => 298,
            Self::NewChat => 299,
            Self::StartChat => 300,
            Self::Delegates => 301,
            Self::SignInPage => 302,
            Self::SignInWith => 303,
            Self::MissingPage => 304,
            Self::UpdateClient => 305,
            Self::SomeoneElse => 306,
            Self::OwnSpend => 307,
            Self::DelegateSpend => 308,
            Self::Overview => 309,
            Self::Activity => 310,
            Self::Plan => 311,
            Self::History => 312,
            Self::BudgetAuthority => 313,
            Self::Panels => 314,
            Self::Goal => 315,
            Self::Resources => 316,
            Self::Policy => 317,
            Self::Roles => 318,
            Self::Spend => 319,
            Self::Recurring => 320,
            Self::NextRun => 407,
            Self::NextPeriod => 408,
            Self::WaitingRoles => 411,
            Self::HoldRevision => 412,
            Self::CurrentPeriod => 409,
            Self::PeriodBegan => 410,
            Self::Settings => 321,
            Self::Goals => 322,
            Self::Tasks => 323,
            Self::Notes => 324,
            Self::Reserved => 325,
            Self::Left => 326,
            Self::TotalBudget => 327,
            Self::OwnBudget => 328,
            Self::EndedBudget => 329,
            Self::HeldBelow => 330,
            Self::FoldSubtree => 331,
            Self::OpenSubtree => 332,
            Self::AllPlan => 333,
            Self::HeldPlan => 334,
            Self::LivePlan => 335,
            Self::OwnHistory => 336,
            Self::SubtreeHistory => 337,
            Self::BatchMade => 338,
            Self::MovedWork => 339,
            Self::GoalWords => 340,
            Self::GoalBudget => 341,
            Self::GoalDeadline => 342,
            Self::GoalPriority => 343,
            Self::PriorityMeaning => 344,
            Self::PoolLeft => 345,
            Self::GoalPropose => 346,
            Self::NoteSearch => 347,
            Self::AskedByMe => 348,
            Self::AllTasks => 349,
            Self::Period => 350,
            Self::RoleNumber => 351,
            Self::PartyNumber => 352,
            Self::Allotment => 353,
            Self::DecidesKind => 354,
            Self::PolicySpend => 355,
            Self::HostNumber => 356,
            Self::HostConnected => 357,
            Self::HostDisconnected => 358,
            Self::SlotsUsed => 359,
            Self::SlotsFree => 360,
            Self::AccountNumber => 361,
            Self::AccountUsable => 362,
            Self::AccountUnavailable => 363,
            Self::Capacity => 364,
            Self::TasksLive => 365,
            Self::CapacityLimit => 366,
            Self::Grace => 367,
            Self::WaitingAccount => 368,
            Self::ParkedResume => 369,
            Self::RetryingPhase => 370,
            Self::ResourceScope => 371,
            Self::ResultExpected => 372,
            Self::ReportExpected => 373,
            Self::WordsExpected => 374,
            Self::ApprovalExpected => 375,
            Self::ProposalsWaiting => 376,
            Self::ConversationLog => 377,
            Self::WordsBeingSent => 378,
            Self::WordsSent => 379,
            Self::WordsRead => 380,
            Self::RunResumed => 381,
            Self::RunFreshTail => 382,
            Self::RunFinished => 383,
            Self::RunCancelled => 384,
            Self::TurnInput => 385,
            Self::TurnOutput => 386,
            Self::CacheRead => 387,
            Self::CacheWritten => 388,
            Self::TurnSpent => 389,
            Self::Provisional => 390,
            Self::TurnCut => 391,
            Self::OpaqueDialect => 392,
            Self::OpaqueBytes => 393,
            Self::CallNumber => 394,
            Self::OpenCall => 395,
            Self::FoldCall => 396,
            Self::CallFailed => 397,
            Self::CallCompleted => 398,
            Self::CallPending => 399,
            Self::NewsTopic => 400,
            Self::TurnNumber => 401,
            Self::TurnInstructions => 402,
            Self::PanelPart => 490,
            Self::ResultPart => 491,
            Self::HoldPart => 492,
            Self::OutboxEntry => 508,
            Self::TakenByNumber => 509,

            Self::EngineLimitWords => 413,
            Self::EngineLimitIdentity => 414,
            Self::EngineLimitRoles => 415,
            Self::EngineLimitArithmetic => 416,
            Self::EngineLimitDuplicate => 417,
            Self::EngineLimitEmpty => 418,
            Self::EngineLimitBatch => 419,
            Self::EngineLimitProjectLiveTasks => 420,
            Self::EngineLimitLiveTasks => 421,
            Self::EngineLimitTree => 422,
            Self::EngineLimitDepth => 423,
            Self::EngineLimitDelegates => 424,
            Self::EngineLimitSubscription => 425,
            Self::EngineLimitDependencies => 426,
            Self::EngineLimitCycle => 427,
            Self::EngineLimitSpec => 428,
            Self::EngineLimitContract => 429,
            Self::EngineLimitAuthority => 430,
            Self::EngineLimitInputs => 431,
            Self::EngineLimitAttempts => 432,
            Self::EngineLimitLiveDelegates => 433,
            Self::EngineLimitRestore => 434,
            Self::EngineLimitRead => 435,
            Self::EngineLimitTurn => 436,
            Self::EngineLimitHoldKind => 437,
            Self::EngineLimitHoldTaken => 438,
            Self::EngineLimitHolds => 439,
            Self::EngineLimitInboxFull => 440,
            Self::EngineLimitNotes => 441,
            Self::EngineLimitRevision => 442,
            Self::EngineLimitQuestion => 443,
            Self::EngineLimitPriority => 444,
            Self::EngineLimitPolicy => 445,
            Self::EngineLimitResource => 446,
            Self::EngineLimitService => 447,
            Self::FailedBare => 448,
            Self::CancelledBare => 449,
            Self::DecidedBare => 450,
            Self::DeletedEnding => 451,
            Self::ByProjectDeployment => 452,
            Self::RoleNotAllowed => 453,
            Self::NoHeldRole => 454,
            Self::HeldOwner => 455,
            Self::HeldMaintainer => 456,
            Self::HeldMember => 457,
            Self::HeldObserver => 458,
            Self::HeldPolicy => 459,
            Self::DisposalMissing => 460,
            Self::DisposalUnknown => 461,
            Self::DisposalRepeated => 462,
            Self::RefusalTakenBy => 463,
            Self::NotWaitingForYou => 464,
            Self::NoFurtherHolder => 465,
            Self::GrantKindMissing => 466,
            Self::FundingMissing => 467,
            Self::DepthMaximum => 468,
            Self::DeadlineAuthorityMissing => 469,
            Self::TasksLeft => 470,
            Self::ToolMissing => 471,
            Self::ScopeMissing => 472,
            Self::ReferenceMissing => 473,
            Self::ResourceAccessMissing => 474,
            Self::ResourceUnavailable => 475,
            Self::AuthorityShapeMissing => 476,
            Self::AgainRequired => 477,
            Self::AgainUnexpected => 478,
            Self::AccountMissing => 479,
            Self::WriterMissing => 480,
            Self::RequiredJudgeMissing => 481,
            Self::FailedJudge => 482,
            Self::UnguardedJudge => 483,
            Self::PartConnector => 484,
            Self::PartResource => 485,
            Self::PartKind => 486,
            Self::PartTask => 487,
            Self::PartProject => 488,
            Self::PartProposal => 489,
            Self::HoldStalled => 493,
            Self::HoldDependency => 494,
            Self::HoldStopped => 495,
            Self::HoldDrift => 496,
            Self::HoldEffects => 497,
            Self::HoldEffectFailed => 498,
            Self::HoldUncertainEntry => 499,
            Self::HoldBudget => 500,
            Self::HoldDeadline => 501,
            Self::HoldWaited => 502,
            Self::HoldPoolLost => 503,
            Self::HoldProcedure => 504,
            Self::FailureRun => 505,
            Self::FailureAgent => 506,
            Self::FailureInvalid => 507,
            Self::RecurringKey => 800,
            Self::RecurringSkip => 801,
            Self::RecurringWait => 802,
            Self::RecurringLastPeriod => 803,
            Self::RecurringPendingPeriod => 804,
            Self::RecurringCut => 805,
        };
        PhraseId { owner: Owner::Jig, number }
    }
    /// Hole contract checked for every supplied language.
    #[must_use]
    #[expect(clippy::too_many_lines, reason = "every phrase declares its holes in one exhaustive table")]
    pub fn shape(self, text: u32) -> PhraseShape {
        let holes: Box<[HoleShape]> = match self {
            Self::PolicyJudge | Self::PolicyPermission | Self::SpecificationResource => Box::new([
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Number, bytes: 0 },
            ]),
            Self::PolicyRequirements
            | Self::PolicyProjections
            | Self::PolicyPermissions
            | Self::PolicyGuarded
            | Self::PolicyMustGuard
            | Self::PolicyGuardNotRequired
            | Self::RunIdle
            | Self::RunDue
            | Self::RunPreparing
            | Self::RunUnobserved
            | Self::RunStartUnknown
            | Self::WorkPreparing
            | Self::WorkStarting
            | Self::WorkActive
            | Self::WorkWriting
            | Self::WorkWaiting
            | Self::WorkEnding
            | Self::ClosingDelegates
            | Self::ClosingStageEffects
            | Self::ClosingReleases
            | Self::ClosingSettled
            | Self::ClosingStageUnknown
            | Self::ClosingConnectorsUnknown
            | Self::ClosingConnectorsEmpty
            | Self::ClosingPendingEnding
            | Self::HeldPriorLifecycle
            | Self::HeldPriorWaiting
            | Self::SpecificationCut
            | Self::HoldStalled
            | Self::HoldStopped
            | Self::HoldDrift
            | Self::HoldEffects
            | Self::HoldEffectFailed
            | Self::HoldBudget
            | Self::HoldDeadline
            | Self::HoldWaited
            | Self::HoldProcedure
            | Self::FailureRun
            | Self::FailureAgent
            | Self::FailureInvalid
            | Self::PanelPart
            | Self::ResultPart
            | Self::HoldPart
            | Self::EngineLimitWords
            | Self::EngineLimitIdentity
            | Self::EngineLimitRoles
            | Self::EngineLimitArithmetic
            | Self::EngineLimitDuplicate
            | Self::EngineLimitEmpty
            | Self::EngineLimitBatch
            | Self::EngineLimitProjectLiveTasks
            | Self::EngineLimitLiveTasks
            | Self::EngineLimitTree
            | Self::EngineLimitDepth
            | Self::EngineLimitDelegates
            | Self::EngineLimitSubscription
            | Self::EngineLimitDependencies
            | Self::EngineLimitCycle
            | Self::EngineLimitSpec
            | Self::EngineLimitContract
            | Self::EngineLimitAuthority
            | Self::EngineLimitInputs
            | Self::EngineLimitAttempts
            | Self::EngineLimitLiveDelegates
            | Self::EngineLimitRestore
            | Self::EngineLimitRead
            | Self::EngineLimitTurn
            | Self::EngineLimitHoldKind
            | Self::EngineLimitHoldTaken
            | Self::EngineLimitHolds
            | Self::EngineLimitInboxFull
            | Self::EngineLimitNotes
            | Self::EngineLimitRevision
            | Self::EngineLimitQuestion
            | Self::EngineLimitPriority
            | Self::EngineLimitPolicy
            | Self::EngineLimitResource
            | Self::EngineLimitService
            | Self::FailedBare
            | Self::CancelledBare
            | Self::DecidedBare
            | Self::DeletedEnding
            | Self::RoleNotAllowed
            | Self::NoHeldRole
            | Self::HeldOwner
            | Self::HeldMaintainer
            | Self::HeldMember
            | Self::HeldObserver
            | Self::NotWaitingForYou
            | Self::NoFurtherHolder
            | Self::FundingMissing
            | Self::DeadlineAuthorityMissing
            | Self::ToolMissing
            | Self::ScopeMissing
            | Self::ReferenceMissing
            | Self::ResourceAccessMissing
            | Self::ResourceUnavailable
            | Self::AuthorityShapeMissing
            | Self::AgainRequired
            | Self::AgainUnexpected
            | Self::AccountMissing
            | Self::WriterMissing
            | Self::Overview
            | Self::Activity
            | Self::Plan
            | Self::History
            | Self::BudgetAuthority
            | Self::Panels
            | Self::Goal
            | Self::Resources
            | Self::Policy
            | Self::Roles
            | Self::Spend
            | Self::Recurring
            | Self::NextPeriod
            | Self::RecurringSkip
            | Self::RecurringWait
            | Self::RecurringCut
            | Self::Settings
            | Self::Goals
            | Self::Tasks
            | Self::Notes
            | Self::FoldSubtree
            | Self::OpenSubtree
            | Self::AllPlan
            | Self::HeldPlan
            | Self::LivePlan
            | Self::OwnHistory
            | Self::SubtreeHistory
            | Self::MovedWork
            | Self::GoalWords
            | Self::GoalBudget
            | Self::GoalDeadline
            | Self::GoalPriority
            | Self::PriorityMeaning
            | Self::GoalPropose
            | Self::NoteSearch
            | Self::AskedByMe
            | Self::AllTasks
            | Self::HostConnected
            | Self::HostDisconnected
            | Self::AccountUsable
            | Self::AccountUnavailable
            | Self::Capacity
            | Self::WaitingAccount
            | Self::ParkedResume
            | Self::RetryingPhase
            | Self::ResultExpected
            | Self::ReportExpected
            | Self::WordsExpected
            | Self::ApprovalExpected
            | Self::ConversationLog
            | Self::WordsBeingSent
            | Self::WordsSent
            | Self::WordsRead
            | Self::RunResumed
            | Self::RunFinished
            | Self::RunCancelled
            | Self::Provisional
            | Self::TurnCut
            | Self::CallFailed
            | Self::CallCompleted
            | Self::CallPending
            | Self::TurnInstructions
            | Self::NeedsYou
            | Self::InboxUpdates
            | Self::AllKinds
            | Self::AllGoals
            | Self::NewUpdate
            | Self::OnCall
            | Self::ReadEarlier
            | Self::ReadLater
            | Self::WindowStart
            | Self::WindowEnd
            | Self::MoreRows
            | Self::NewChat
            | Self::StartChat
            | Self::SignInPage
            | Self::MissingPage
            | Self::UpdateClient
            | Self::SomeoneElse
            | Self::LoadingNavigation
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
            | Self::ClosingConnector
            | Self::ClosingEffects
            | Self::NobodyMayAccept
            | Self::ApprovalAuthorityMissing
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
            | Self::ProcedureHold
            | Self::BeforeHold
            | Self::Confirmation
            | Self::Confirm
            | Self::Dismiss
            | Self::Dangerous
            | Self::WillPropose
            | Self::ChangedConfirmation
            | Self::MissingWords
            | Self::MissingChoice
            | Self::MissingAgain
            | Self::MissingChange
            | Self::MissingDelegates
            | Self::MissingPriority
            | Self::MissingContext
            | Self::WillAccept
            | Self::WillReject
            | Self::WillPassUp
            | Self::WillLeaveHeld
            | Self::WillStop
            | Self::WillCancel
            | Self::WillClose
            | Self::WillTakeOver
            | Self::WillTake
            | Self::WillHandBack
            | Self::WillWrite
            | Self::WillAnswer
            | Self::WillAmend
            | Self::WillAmendRelease
            | Self::WillPrioritise
            | Self::WillEditNote
            | Self::WillDeleteNote
            | Self::WillGoal
            | Self::ResumeRelease
            | Self::ContinueClosing
            | Self::ReleaseWait
            | Self::ReleaseFresh
            | Self::ReleaseProcedure
            | Self::RepeatEffect
            | Self::SkipEffect
            | Self::EngineFollowsChoice
            | Self::DelegateChoices
            | Self::WriteAndRelease
            | Self::WriteOnly
            | Self::NewSpec
            | Self::NewInstructions
            | Self::InstructionNotSupplied
            | Self::InstructionEmpty
            | Self::NewBudget
            | Self::NewDeadline
            | Self::PriorityField
            | Self::NoteDescription
            | Self::NoteBody
            | Self::DocumentTextCut
            | Self::ResultTimeUnknown
            | Self::NoteReferencesCut
            | Self::Notice
            | Self::UnknownAction
            | Self::FailedPhase
            | Self::CancelledPhase
            | Self::AddressParty
            | Self::AddressTask
            | Self::AddressRole
            | Self::DuePhase
            | Self::PreparingPhase
            | Self::ClaimedPhase
            | Self::ActivePhase
            | Self::ClosingPhase => Box::new([]),
            Self::PartResource
            | Self::ResourceScope
            | Self::OpaqueDialect
            | Self::OpenCall
            | Self::FoldCall
            | Self::SignInWith
            | Self::ByNamed
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
            Self::PolicyEditRole
            | Self::PolicyRequests
            | Self::PolicyDecides
            | Self::PolicyObserved
            | Self::RunClaimed
            | Self::RunOwnerActive
            | Self::RunRetryRefused
            | Self::ClosingRun
            | Self::SpecificationBytes
            | Self::SpecificationInput
            | Self::HoldDependency
            | Self::HoldUncertainEntry
            | Self::HoldPoolLost
            | Self::ByProjectDeployment
            | Self::HeldPolicy
            | Self::DisposalMissing
            | Self::DisposalUnknown
            | Self::DisposalRepeated
            | Self::RefusalTakenBy
            | Self::GrantKindMissing
            | Self::DepthMaximum
            | Self::TasksLeft
            | Self::OutboxEntry
            | Self::TakenByNumber
            | Self::PartConnector
            | Self::PartKind
            | Self::PartTask
            | Self::PartProject
            | Self::HeldBelow
            | Self::BatchMade
            | Self::Period
            | Self::CurrentPeriod
            | Self::RecurringKey
            | Self::RecurringLastPeriod
            | Self::RecurringPendingPeriod
            | Self::WaitingRoles
            | Self::HoldRevision
            | Self::RoleNumber
            | Self::PartyNumber
            | Self::DecidesKind
            | Self::HostNumber
            | Self::SlotsUsed
            | Self::SlotsFree
            | Self::AccountNumber
            | Self::TasksLive
            | Self::CapacityLimit
            | Self::ProposalsWaiting
            | Self::RunFreshTail
            | Self::TurnInput
            | Self::TurnOutput
            | Self::CacheRead
            | Self::CacheWritten
            | Self::OpaqueBytes
            | Self::CallNumber
            | Self::NewsTopic
            | Self::TurnNumber
            | Self::OnlyGoal
            | Self::Delegates
            | Self::Needs
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
            | Self::PersonAddressRevision
            | Self::ChangeResultKind
            | Self::FirstUnread
            | Self::Topic
            | Self::DependencyEnded
            | Self::LoadingDependency
            | Self::TasksWaiting
            | Self::TasksRunning
            | Self::TasksHeld
            | Self::TasksClosing
            | Self::TasksDone
            | Self::TasksFailed
            | Self::TasksCancelled
            | Self::LiveRuns
            | Self::PendingProposals
            | Self::PendingEffects
            | Self::SettlingEffects
            | Self::DelegateCancel
            | Self::DropDependency
            | Self::Charter
            | Self::Priority
            | Self::Closing
            | Self::ResultsThrough => Box::new([HoleShape { kind: HoleKind::Number, bytes: 0 }]),
            Self::PolicyEffect
            | Self::RunRetryFailed
            | Self::SpecificationNumber
            | Self::GoalProcedure
            | Self::RequiredJudgeMissing
            | Self::FailedJudge
            | Self::UnguardedJudge
            | Self::PartProposal
            | Self::PartProposalTask
            | Self::PartProposalParty
            | Self::ProposedByParty
            | Self::Proposed
            | Self::MadeWords
            | Self::RoleMissing
            | Self::DepthMissing
            | Self::TasksMissing
            | Self::LimitLive
            | Self::LimitInbox
            | Self::WaitingRole
            | Self::RepliesThrough => Box::new([
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Number, bytes: 0 },
            ]),
            Self::RunBackingOff
            | Self::RunPhaseSince
            | Self::RunPlacement
            | Self::RunObservedSince
            | Self::RunReportedAt
            | Self::RunActualStart
            | Self::NextRun
            | Self::PeriodBegan
            | Self::Grace
            | Self::PassesUp
            | Self::At
            | Self::Since
            | Self::Deadline
            | Self::DeadlineHold => Box::new([HoleShape { kind: HoleKind::Wall, bytes: 0 }]),
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
            Self::PolicyPeriodSpend
            | Self::Reserved
            | Self::Left
            | Self::TotalBudget
            | Self::OwnBudget
            | Self::EndedBudget
            | Self::PoolLeft
            | Self::Allotment
            | Self::PolicySpend
            | Self::TurnSpent
            | Self::OwnSpend
            | Self::DelegateSpend
            | Self::Amount
            | Self::Budget
            | Self::TakeoverFunds => Box::new([HoleShape { kind: HoleKind::Amount, bytes: 0 }]),
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
            Self::WorkTool | Self::ApprovalYes | Self::ApprovalNo => Box::new([
                HoleShape { kind: HoleKind::Name, bytes: text },
                HoleShape { kind: HoleKind::Name, bytes: text },
            ]),
            Self::StoppedHold => Box::new([
                HoleShape { kind: HoleKind::Name, bytes: text },
                HoleShape { kind: HoleKind::Wall, bytes: 0 },
            ]),
            Self::SpentRun => Box::new([
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Amount, bytes: 0 },
            ]),
            Self::SpentDelegate | Self::DelegateTake => Box::new([
                HoleShape { kind: HoleKind::Number, bytes: 0 },
                HoleShape { kind: HoleKind::Amount, bytes: 0 },
            ]),
        };
        PhraseShape { id: self.id(), holes, plural: None }
    }
}
/// Every jig phrase; loading checks this whole contract.
pub const ALL: [JigPhrase; 579] = [
    JigPhrase::PolicyPeriodSpend,
    JigPhrase::PolicyEditRole,
    JigPhrase::PolicyRequests,
    JigPhrase::PolicyDecides,
    JigPhrase::PolicyRequirements,
    JigPhrase::PolicyProjections,
    JigPhrase::PolicyPermissions,
    JigPhrase::PolicyEffect,
    JigPhrase::PolicyJudge,
    JigPhrase::PolicyGuarded,
    JigPhrase::PolicyObserved,
    JigPhrase::PolicyMustGuard,
    JigPhrase::PolicyGuardNotRequired,
    JigPhrase::PolicyPermission,
    JigPhrase::ProposedByParty,
    JigPhrase::PartProposalTask,
    JigPhrase::PartProposalParty,
    JigPhrase::RunIdle,
    JigPhrase::RunDue,
    JigPhrase::RunPreparing,
    JigPhrase::RunClaimed,
    JigPhrase::RunOwnerActive,
    JigPhrase::RunBackingOff,
    JigPhrase::RunPhaseSince,
    JigPhrase::RunPlacement,
    JigPhrase::RunUnobserved,
    JigPhrase::RunObservedSince,
    JigPhrase::RunReportedAt,
    JigPhrase::RunActualStart,
    JigPhrase::RunStartUnknown,
    JigPhrase::WorkPreparing,
    JigPhrase::WorkStarting,
    JigPhrase::WorkActive,
    JigPhrase::WorkWriting,
    JigPhrase::WorkTool,
    JigPhrase::WorkWaiting,
    JigPhrase::WorkEnding,
    JigPhrase::RunRetryFailed,
    JigPhrase::RunRetryRefused,
    JigPhrase::ClosingRun,
    JigPhrase::ClosingDelegates,
    JigPhrase::ClosingStageEffects,
    JigPhrase::ClosingReleases,
    JigPhrase::ClosingSettled,
    JigPhrase::ClosingStageUnknown,
    JigPhrase::ClosingConnectorsUnknown,
    JigPhrase::ClosingConnectorsEmpty,
    JigPhrase::ClosingPendingEnding,
    JigPhrase::HeldPriorLifecycle,
    JigPhrase::HeldPriorWaiting,
    JigPhrase::SpecificationNumber,
    JigPhrase::SpecificationBytes,
    JigPhrase::SpecificationResource,
    JigPhrase::SpecificationInput,
    JigPhrase::SpecificationCut,
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
    JigPhrase::ClosingConnector,
    JigPhrase::ClosingEffects,
    JigPhrase::NobodyMayAccept,
    JigPhrase::ApprovalAuthorityMissing,
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
    JigPhrase::BeforeHold,
    JigPhrase::SpentRun,
    JigPhrase::SpentDelegate,
    JigPhrase::Confirmation,
    JigPhrase::Confirm,
    JigPhrase::Dismiss,
    JigPhrase::Dangerous,
    JigPhrase::WillPropose,
    JigPhrase::ChangedConfirmation,
    JigPhrase::MissingWords,
    JigPhrase::MissingChoice,
    JigPhrase::MissingAgain,
    JigPhrase::MissingChange,
    JigPhrase::MissingDelegates,
    JigPhrase::MissingPriority,
    JigPhrase::MissingContext,
    JigPhrase::WillAccept,
    JigPhrase::WillReject,
    JigPhrase::WillPassUp,
    JigPhrase::WillLeaveHeld,
    JigPhrase::WillStop,
    JigPhrase::WillCancel,
    JigPhrase::WillClose,
    JigPhrase::WillTakeOver,
    JigPhrase::WillTake,
    JigPhrase::WillHandBack,
    JigPhrase::WillWrite,
    JigPhrase::WillAnswer,
    JigPhrase::WillAmend,
    JigPhrase::WillAmendRelease,
    JigPhrase::WillPrioritise,
    JigPhrase::WillEditNote,
    JigPhrase::WillDeleteNote,
    JigPhrase::WillGoal,
    JigPhrase::ResumeRelease,
    JigPhrase::ContinueClosing,
    JigPhrase::ReleaseWait,
    JigPhrase::ReleaseFresh,
    JigPhrase::ReleaseProcedure,
    JigPhrase::RepeatEffect,
    JigPhrase::SkipEffect,
    JigPhrase::EngineFollowsChoice,
    JigPhrase::TasksWaiting,
    JigPhrase::TasksRunning,
    JigPhrase::TasksHeld,
    JigPhrase::TasksClosing,
    JigPhrase::TasksDone,
    JigPhrase::TasksFailed,
    JigPhrase::TasksCancelled,
    JigPhrase::LiveRuns,
    JigPhrase::PendingProposals,
    JigPhrase::PendingEffects,
    JigPhrase::SettlingEffects,
    JigPhrase::TakeoverFunds,
    JigPhrase::DelegateCancel,
    JigPhrase::DelegateTake,
    JigPhrase::DelegateChoices,
    JigPhrase::WriteAndRelease,
    JigPhrase::WriteOnly,
    JigPhrase::NewSpec,
    JigPhrase::NewInstructions,
    JigPhrase::InstructionNotSupplied,
    JigPhrase::InstructionEmpty,
    JigPhrase::NewBudget,
    JigPhrase::NewDeadline,
    JigPhrase::DropDependency,
    JigPhrase::PriorityField,
    JigPhrase::NoteDescription,
    JigPhrase::NoteBody,
    JigPhrase::DocumentTextCut,
    JigPhrase::PersonAddressRevision,
    JigPhrase::ResultTimeUnknown,
    JigPhrase::ChangeResultKind,
    JigPhrase::NoteReferencesCut,
    JigPhrase::Charter,
    JigPhrase::GoalProcedure,
    JigPhrase::Priority,
    JigPhrase::Notice,
    JigPhrase::UnknownAction,
    JigPhrase::ResultsThrough,
    JigPhrase::RepliesThrough,
    JigPhrase::FailedPhase,
    JigPhrase::CancelledPhase,
    JigPhrase::AddressParty,
    JigPhrase::AddressTask,
    JigPhrase::AddressRole,
    JigPhrase::DuePhase,
    JigPhrase::PreparingPhase,
    JigPhrase::ClaimedPhase,
    JigPhrase::ActivePhase,
    JigPhrase::ClosingPhase,
    JigPhrase::NeedsYou,
    JigPhrase::InboxUpdates,
    JigPhrase::AllKinds,
    JigPhrase::AllGoals,
    JigPhrase::OnlyGoal,
    JigPhrase::NewUpdate,
    JigPhrase::OnCall,
    JigPhrase::PassesUp,
    JigPhrase::ReadEarlier,
    JigPhrase::ReadLater,
    JigPhrase::WindowStart,
    JigPhrase::WindowEnd,
    JigPhrase::MoreRows,
    JigPhrase::NewChat,
    JigPhrase::StartChat,
    JigPhrase::Delegates,
    JigPhrase::SignInPage,
    JigPhrase::SignInWith,
    JigPhrase::MissingPage,
    JigPhrase::UpdateClient,
    JigPhrase::SomeoneElse,
    JigPhrase::OwnSpend,
    JigPhrase::DelegateSpend,
    JigPhrase::Overview,
    JigPhrase::Activity,
    JigPhrase::Plan,
    JigPhrase::History,
    JigPhrase::BudgetAuthority,
    JigPhrase::Panels,
    JigPhrase::Goal,
    JigPhrase::Resources,
    JigPhrase::Policy,
    JigPhrase::Roles,
    JigPhrase::Spend,
    JigPhrase::Recurring,
    JigPhrase::NextRun,
    JigPhrase::NextPeriod,
    JigPhrase::WaitingRoles,
    JigPhrase::HoldRevision,
    JigPhrase::CurrentPeriod,
    JigPhrase::PeriodBegan,
    JigPhrase::Settings,
    JigPhrase::Goals,
    JigPhrase::Tasks,
    JigPhrase::Notes,
    JigPhrase::Reserved,
    JigPhrase::Left,
    JigPhrase::TotalBudget,
    JigPhrase::OwnBudget,
    JigPhrase::EndedBudget,
    JigPhrase::HeldBelow,
    JigPhrase::FoldSubtree,
    JigPhrase::OpenSubtree,
    JigPhrase::AllPlan,
    JigPhrase::HeldPlan,
    JigPhrase::LivePlan,
    JigPhrase::OwnHistory,
    JigPhrase::SubtreeHistory,
    JigPhrase::BatchMade,
    JigPhrase::MovedWork,
    JigPhrase::GoalWords,
    JigPhrase::GoalBudget,
    JigPhrase::GoalDeadline,
    JigPhrase::GoalPriority,
    JigPhrase::PriorityMeaning,
    JigPhrase::PoolLeft,
    JigPhrase::GoalPropose,
    JigPhrase::NoteSearch,
    JigPhrase::AskedByMe,
    JigPhrase::AllTasks,
    JigPhrase::Period,
    JigPhrase::RoleNumber,
    JigPhrase::PartyNumber,
    JigPhrase::Allotment,
    JigPhrase::DecidesKind,
    JigPhrase::PolicySpend,
    JigPhrase::HostNumber,
    JigPhrase::HostConnected,
    JigPhrase::HostDisconnected,
    JigPhrase::SlotsUsed,
    JigPhrase::SlotsFree,
    JigPhrase::AccountNumber,
    JigPhrase::AccountUsable,
    JigPhrase::AccountUnavailable,
    JigPhrase::Capacity,
    JigPhrase::TasksLive,
    JigPhrase::CapacityLimit,
    JigPhrase::Grace,
    JigPhrase::WaitingAccount,
    JigPhrase::ParkedResume,
    JigPhrase::RetryingPhase,
    JigPhrase::ResourceScope,
    JigPhrase::ResultExpected,
    JigPhrase::ReportExpected,
    JigPhrase::WordsExpected,
    JigPhrase::ApprovalExpected,
    JigPhrase::ProposalsWaiting,
    JigPhrase::ConversationLog,
    JigPhrase::WordsBeingSent,
    JigPhrase::WordsSent,
    JigPhrase::WordsRead,
    JigPhrase::RunResumed,
    JigPhrase::RunFreshTail,
    JigPhrase::RunFinished,
    JigPhrase::RunCancelled,
    JigPhrase::TurnInput,
    JigPhrase::TurnOutput,
    JigPhrase::CacheRead,
    JigPhrase::CacheWritten,
    JigPhrase::TurnSpent,
    JigPhrase::Provisional,
    JigPhrase::TurnCut,
    JigPhrase::OpaqueDialect,
    JigPhrase::OpaqueBytes,
    JigPhrase::CallNumber,
    JigPhrase::OpenCall,
    JigPhrase::FoldCall,
    JigPhrase::CallFailed,
    JigPhrase::CallCompleted,
    JigPhrase::CallPending,
    JigPhrase::NewsTopic,
    JigPhrase::TurnNumber,
    JigPhrase::TurnInstructions,
    JigPhrase::PanelPart,
    JigPhrase::ResultPart,
    JigPhrase::HoldPart,
    JigPhrase::OutboxEntry,
    JigPhrase::TakenByNumber,
    JigPhrase::EngineLimitWords,
    JigPhrase::EngineLimitIdentity,
    JigPhrase::EngineLimitRoles,
    JigPhrase::EngineLimitArithmetic,
    JigPhrase::EngineLimitDuplicate,
    JigPhrase::EngineLimitEmpty,
    JigPhrase::EngineLimitBatch,
    JigPhrase::EngineLimitProjectLiveTasks,
    JigPhrase::EngineLimitLiveTasks,
    JigPhrase::EngineLimitTree,
    JigPhrase::EngineLimitDepth,
    JigPhrase::EngineLimitDelegates,
    JigPhrase::EngineLimitSubscription,
    JigPhrase::EngineLimitDependencies,
    JigPhrase::EngineLimitCycle,
    JigPhrase::EngineLimitSpec,
    JigPhrase::EngineLimitContract,
    JigPhrase::EngineLimitAuthority,
    JigPhrase::EngineLimitInputs,
    JigPhrase::EngineLimitAttempts,
    JigPhrase::EngineLimitLiveDelegates,
    JigPhrase::EngineLimitRestore,
    JigPhrase::EngineLimitRead,
    JigPhrase::EngineLimitTurn,
    JigPhrase::EngineLimitHoldKind,
    JigPhrase::EngineLimitHoldTaken,
    JigPhrase::EngineLimitHolds,
    JigPhrase::EngineLimitInboxFull,
    JigPhrase::EngineLimitNotes,
    JigPhrase::EngineLimitRevision,
    JigPhrase::EngineLimitQuestion,
    JigPhrase::EngineLimitPriority,
    JigPhrase::EngineLimitPolicy,
    JigPhrase::EngineLimitResource,
    JigPhrase::EngineLimitService,
    JigPhrase::FailedBare,
    JigPhrase::CancelledBare,
    JigPhrase::DecidedBare,
    JigPhrase::DeletedEnding,
    JigPhrase::ByProjectDeployment,
    JigPhrase::RoleNotAllowed,
    JigPhrase::NoHeldRole,
    JigPhrase::HeldOwner,
    JigPhrase::HeldMaintainer,
    JigPhrase::HeldMember,
    JigPhrase::HeldObserver,
    JigPhrase::HeldPolicy,
    JigPhrase::DisposalMissing,
    JigPhrase::DisposalUnknown,
    JigPhrase::DisposalRepeated,
    JigPhrase::RefusalTakenBy,
    JigPhrase::NotWaitingForYou,
    JigPhrase::NoFurtherHolder,
    JigPhrase::GrantKindMissing,
    JigPhrase::FundingMissing,
    JigPhrase::DepthMaximum,
    JigPhrase::DeadlineAuthorityMissing,
    JigPhrase::TasksLeft,
    JigPhrase::ToolMissing,
    JigPhrase::ScopeMissing,
    JigPhrase::ReferenceMissing,
    JigPhrase::ResourceAccessMissing,
    JigPhrase::ResourceUnavailable,
    JigPhrase::AuthorityShapeMissing,
    JigPhrase::AgainRequired,
    JigPhrase::AgainUnexpected,
    JigPhrase::AccountMissing,
    JigPhrase::WriterMissing,
    JigPhrase::RequiredJudgeMissing,
    JigPhrase::FailedJudge,
    JigPhrase::UnguardedJudge,
    JigPhrase::PartConnector,
    JigPhrase::PartResource,
    JigPhrase::PartKind,
    JigPhrase::PartTask,
    JigPhrase::PartProject,
    JigPhrase::PartProposal,
    JigPhrase::HoldStalled,
    JigPhrase::HoldDependency,
    JigPhrase::HoldStopped,
    JigPhrase::HoldDrift,
    JigPhrase::HoldEffects,
    JigPhrase::HoldEffectFailed,
    JigPhrase::HoldUncertainEntry,
    JigPhrase::HoldBudget,
    JigPhrase::HoldDeadline,
    JigPhrase::HoldWaited,
    JigPhrase::HoldPoolLost,
    JigPhrase::HoldProcedure,
    JigPhrase::FailureRun,
    JigPhrase::FailureAgent,
    JigPhrase::FailureInvalid,
    JigPhrase::RecurringKey,
    JigPhrase::RecurringSkip,
    JigPhrase::RecurringWait,
    JigPhrase::RecurringLastPeriod,
    JigPhrase::RecurringPendingPeriod,
    JigPhrase::RecurringCut,
];
/// Shapes with names bounded by the client's text capacity.
#[must_use]
pub fn shapes(text: u32) -> Box<[PhraseShape]> {
    let mut rows = List::with_capacity(u32::try_from(ALL.len()).expect("phrase count fits"));
    for phrase in ALL {
        rows.push(phrase.shape(text)).expect("all jig shapes fit");
    }
    crate::owner_words::shapes(rows.into_boxed())
}
