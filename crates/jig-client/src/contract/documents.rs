//! Whole committed documents, made by the engine and owned by this client.
//! See client/domain/contract.md, 5, and client/domain/objects.md, 3–8.
//! List capacities and Text byte bounds are validated by the protocol before entry;
//! the working set keeps only what its own configured limits admit.
use super::{
    Actor, Amendment, Authority, Count, Decision, Ending, Executor, Holding, Lacking, NoteScope, ObjectKey, Part,
    Party, Phase, Policy, ProposalKey, Refusal, Revision, Text, Turn, TurnName, VersionRange, WakePolicy,
};
use skein_lib::{Duration, List, Wall};
/// Who is signed in and what the engine offers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Frame {
    pub party: Party,
    pub projects: List<ProjectRole>,
    pub needs: Needs,
    pub updates: Count,
    pub versions: VersionRange,
    pub retention: Duration,
    pub deployment_role: Option<u16>,
}

/// Exact counts of work needing this party.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Needs {
    pub questions: u32,
    pub proposals: u32,
    pub escalations: u32,
    pub person_tasks: u32,
}

/// A project role and the charters offered there.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProjectRole {
    pub project: u32,
    pub name: Text,
    pub roles: List<u16>,
    pub chats: List<Charter>,
    pub goals: List<Charter>,
    pub pool: Budget,
    /// The role default used by the project chat charter.
    pub chat_budget: u64,
}

/// An offered charter for a chat or goal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Charter {
    pub number: u32,
    pub name: Text,
}

/// Funding and spend, in the engine amounts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Budget {
    pub total: u64,
    pub own: u64,
    pub ended: u64,
    pub reserved: u64,
    pub left: u64,
    pub funder: Actor,
}

/// A named task wherever a page mentions it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chip {
    pub task: u64,
    pub title: Text,
    pub executor: Executor,
    pub phase: Phase,
}

/// A complete task document with independently moving revisions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Task {
    pub chip: Chip,
    pub project: u32,
    pub requester: Actor,
    pub lineage: List<u64>,
    pub spec: Text,
    pub spec_revision: Revision,
    pub instructions: Text,
    pub wake: WakePolicy,
    pub result_contract: ResultContract,
    pub dependencies: List<Chip>,
    pub references: List<Chip>,
    pub resources: List<Part>,
    pub budget: Budget,
    pub authority: Authority,
    pub hold: Option<Hold>,
    pub escalation: Option<Escalation>,
    pub proposals: List<ProposalKey>,
    pub activity: u64,
    pub deadline: Option<Wall>,
    pub changed: Wall,
    pub acts: List<Offered>,
    pub parts: List<Part>,
    pub closing: Option<Closing>,
    pub spend: List<SpendLine>,
    pub result: Option<Result>,
    pub runtime: Runtime,
    pub impact: Impact,
    pub delegates: List<LiveDelegate>,
    pub writer_slots: List<WriterSlot>,
}

/// What a task still waits for while closing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Closing {
    pub delegates: u32,
    pub effects: u32,
}

/// Spend attributed to a run or delegate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpendLine {
    pub task: u64,
    pub attempt: Option<u32>,
    pub spent: u64,
}

/// An engine offered act kind, without view words.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActKind {
    /// Write to an agent.
    Write,
    /// Answer a question or person task.
    Answer,
    /// Take a role task.
    Take,
    /// Hand a task back.
    HandBack,
    /// Accept a proposal.
    Accept,
    /// Reject a proposal.
    Reject,
    /// Pass a decision up.
    PassUp,
    /// Release a held task.
    Release,
    /// Leave a task held.
    LeaveHeld,
    /// Amend a specification.
    Amend,
    /// Amend and release together.
    AmendRelease,
    /// Stop a live run.
    Stop,
    /// Cancel a subtree.
    Cancel,
    /// Close a chat.
    Close,
    /// Become a requester.
    TakeOver,
    /// Set a goal.
    SetGoal,
    /// Reorder goals.
    Prioritise,
    /// Mark updates read.
    MarkSeen,
    /// Edit a note.
    EditNote,
    /// Delete a note.
    DeleteNote,
    /// Adopt a resource.
    Adopt,
    /// Change policy.
    ChangePolicy,
    /// Change roles.
    SetRoles,
    /// Make a service.
    MakeService,
    /// Set a pool.
    SetPool,
}

/// What the engine says would happen on an act.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Admission {
    /// It would be made.
    Allowed,
    /// It would be proposed to a holder.
    Proposed { to: Recipient },
    /// It is not admitted and says why.
    Refused(Refusal),
}

/// An act and its admission judged by the engine.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Offered {
    pub act: ActKind,
    pub admission: Admission,
}

/// Where an object waits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Recipient {
    /// At a task above.
    Task(u64),
    /// At a named party.
    Party(u64),
    /// At a policy role.
    Role { project: u32, role: u16 },
}

/// A hold revision and the facts it decides.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Hold {
    pub revision: Revision,
    pub reason: HoldReason,
    pub before: Phase,
    pub since: Wall,
    pub result: Option<ResultValue>,
}

/// Why a task is held, with the facts its card needs.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HoldReason {
    /// Out of tries, with failures.
    Tries { failures: List<AttemptFailure> },
    /// Stalled on named work.
    Stalled { since: Wall, waiting: Text },
    /// A dependency ended unsuccessfully.
    Dependency { task: u64, how: Ending },
    /// A connector reported drift.
    Drift { part: Part },
    /// A connector effect failed for good.
    EffectFailed { part: Part },
    /// An unrecoverable effect may have happened.
    EffectUncertain { part: Part },
    /// Out of budget.
    Budget { spent: u64, limit: u64 },
    /// Past a deadline.
    Deadline { at: Wall },
    /// Stopped by a party.
    Stopped { by: Party, at: Wall },
    /// Waited too long for a hold.
    Resource { resource: Part, holder: Actor },
    /// A pool lost allocation.
    Pool { part: Part },
    /// A procedure needs a decision.
    Procedure { part: Part },
}

/// One failed attempt and its failure class.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AttemptFailure {
    pub attempt: u32,
    pub class: FailureClass,
    pub reason: Text,
    pub at: Wall,
}

/// The failure class, as a fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FailureClass {
    /// A host was lost.
    HostLost,
    /// A transient failure.
    Temporary,
    /// A permanent failure.
    Permanent,
    /// The run stalled.
    Stall,
}

/// A held task escalation and the hold it names.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Escalation {
    /// Its committed terminal decision when read after leaving the inbox.
    pub ending: Option<Ending>,
    pub task: Chip,
    pub hold: Hold,
    pub waiting: Recipient,
    pub trail: List<Passed>,
    pub acts: List<Offered>,
}

/// A holder an object passed and why.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Passed {
    pub by: Actor,
    pub at: Wall,
    pub reason: Text,
    pub lacking: Option<Lacking>,
}

/// An immutable proposal and its current waiting recipient.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Proposal {
    pub key: ProposalKey,
    pub project: u32,
    pub proposer: Actor,
    pub why: Text,
    pub action: ProposalAction,
    pub needs: Authority,
    pub funding: Budget,
    pub waiting: Recipient,
    pub trail: List<Passed>,
    pub since: Wall,
    pub acts: List<Offered>,
    pub ending: Option<Ending>,
}

/// What acceptance would make or change.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ProposalAction {
    /// A bounded batch, optionally made the accepter's.
    Batch { tasks: List<ProposedTask>, own: bool },
    /// A connector effect.
    Effect { part: Part },
    /// Wider authority.
    Widen { before: Authority, after: Authority },
    /// An amendment beyond the amender authority.
    Amend { task: u64, before: Text, change: Amendment },
}

/// One task in a proposal batch.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProposedTask {
    pub local: u16,
    pub spec: Text,
    pub executor: Executor,
    pub budget: u64,
    pub deadline: Option<Wall>,
    pub dependencies: List<u16>,
    pub grants: List<super::Grant>,
}

/// Words addressed by one task message.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Question {
    pub task: Chip,
    pub message: u64,
    pub words: Text,
    pub since: Wall,
    pub acts: List<Offered>,
    pub ending: Option<Ending>,
}

/// The revision, form and inputs a person answers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PersonTask {
    pub task: Chip,
    pub revision: Revision,
    pub requester: Actor,
    pub spec: Text,
    pub contract: PersonContract,
    pub inputs: List<Result>,
    pub addressed: Recipient,
    pub taken: Option<Party>,
    pub stall: Option<Wall>,
    pub acts: List<Offered>,
    pub ending: Option<Ending>,
}

/// What a person task answer must say.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PersonContract {
    /// Choose from the named options.
    Choice { options: List<OptionLine> },
    /// Approve the named state.
    Approval { state: Text, part: Part },
    /// Answer in words.
    Words,
}

/// One option in a person task choice.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OptionLine {
    pub number: u16,
    pub words: Text,
}

/// The kind of result a task owes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResultContract {
    /// A report in words.
    Report,
    /// A verdict and its fields.
    Verdict { choices: List<OptionLine> },
    /// A connector result.
    Connector { connector: u16, kind: u16 },
    /// One choice.
    Choice { options: List<OptionLine> },
    /// A named state approval.
    Approval,
    /// Words as a result.
    Words,
}

/// The result payload, with connector values as parts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResultValue {
    /// A report.
    Report(Text),
    /// A verdict and follow ups.
    Verdict { verdict: u16, fields: List<Text>, tasks: List<Chip>, proposal: Option<ProposalKey> },
    /// A summary and connector result.
    Connector { summary: Text, part: Part },
    /// A party choice.
    Choice { option: u16, by: Party, reason: Text },
    /// Approval of a named state.
    Approval { state: Text, approved: bool, by: Party },
    /// An answer in words.
    Words(Text),
}

/// A task ending and its result, as a card.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Result {
    pub task: Chip,
    pub ending: Ending,
    pub value: Option<ResultValue>,
    pub at: Wall,
    pub acts: List<Offered>,
}

/// Counts of tasks folded beyond a tree window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PhaseCounts {
    pub waiting: u32,
    pub running: u32,
    pub held: u32,
    pub closing: u32,
    pub done: u32,
    pub failed: u32,
    pub cancelled: u32,
}

/// One visible task and what is folded beneath it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TreeNode {
    pub chip: Chip,
    pub parent: Option<u64>,
    pub dependencies: List<u64>,
    pub proposals: List<ProposalKey>,
    pub spend: u64,
    pub folded: PhaseCounts,
    pub held_below: u32,
}

/// A live subtree within its watch window.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tree {
    pub task: u64,
    pub nodes: List<TreeNode>,
    pub folded: PhaseCounts,
    pub held_below: u32,
}

/// A group of inbox entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InboxGroup {
    /// Questions.
    Questions,
    /// Proposals.
    Proposals,
    /// Held tasks.
    Escalations,
    /// Tasks for parties.
    PersonTasks,
    /// Unread results.
    Results,
    /// Unread chat replies.
    Replies,
}

/// An entry names an object, project and read position.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InboxEntry {
    pub lineage: List<Chip>,
    pub goal: Option<u64>,
    pub recipient: Option<Recipient>,
    pub stall: Option<Wall>,
    pub object: ObjectKey,
    pub project: u32,
    pub group: InboxGroup,
    pub position: u64,
    pub at: Wall,
}

/// The bounded inbox across projects and its counts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Inbox {
    pub entries: List<InboxEntry>,
    pub needs: Needs,
    pub updates: Count,
    pub results_seen: u64,
    pub replies_seen: List<ReplyPosition>,
    pub more: bool,
}

/// The read position for one chat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReplyPosition {
    pub chat: u64,
    pub through: u64,
}

/// A chat in the party conversations list.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConversationLine {
    pub runtime: Runtime,
    pub delegates: u32,
    pub budget: Budget,
    pub chip: Chip,
    pub project: u32,
    pub first_words: Text,
    pub unread: Count,
    pub seen: u64,
    pub last_reply: u64,
    pub changed: Wall,
}

/// Live chats in a bounded watch window.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Conversations {
    pub rows: List<ConversationLine>,
    pub more: bool,
}

/// A goal on the project board.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GoalLine {
    pub chip: Chip,
    pub priority: u32,
    pub progress: PhaseCounts,
    pub budget: Budget,
    pub deadline: Option<Wall>,
    pub held_below: u32,
}

/// Live goals in priority order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Goals {
    pub project: u32,
    pub rows: List<GoalLine>,
    pub more: bool,
}

/// A project, its policy, roles, spend and parts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Project {
    pub number: u32,
    pub name: Text,
    pub policy: Policy,
    pub roles: List<Holding>,
    pub spend: Budget,
    pub pools: List<Pool>,
    pub resources: List<Part>,
    pub services: List<Party>,
    pub recurring: List<Recurring>,
    pub parts: List<Part>,
    pub acts: List<Offered>,
}

/// A party pool in the current period.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pool {
    pub party: u64,
    pub period: u64,
    pub budget: Budget,
}

/// A recurring template and its next occurrence.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Recurring {
    pub number: u64,
    pub spec: Text,
    pub period: u64,
    pub budget: u64,
    pub last: List<u64>,
    pub next: Wall,
}

/// A note with its independently moving revision.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Note {
    pub number: u64,
    pub revision: Revision,
    pub scope: NoteScope,
    pub description: Text,
    pub body: Text,
    pub by: Actor,
    pub tasks: List<u64>,
    pub changed: Wall,
    pub acts: List<Offered>,
}

/// Hosts, accounts and deployment capacity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Deployment {
    pub hosts: List<Host>,
    pub accounts: List<Account>,
    pub capacity: List<Capacity>,
    pub proposals: u32,
    pub refusals: List<Refusal>,
    pub parts: List<Part>,
}

/// Host connectivity, grace and run slots.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Host {
    pub number: u64,
    pub connected: bool,
    pub grace: Option<Wall>,
    pub used: u32,
    pub free: u32,
    pub runs: List<Chip>,
    pub refused: Option<Text>,
}

/// An LLM account availability and the work using it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Account {
    pub number: u32,
    pub usable: bool,
    pub reason: Option<Text>,
    pub runs: List<Chip>,
    pub waiting: List<Chip>,
}

/// Live task admission by project or deployment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Capacity {
    pub project: Option<u32>,
    pub live: u32,
    pub limit: u32,
}

/// One durable history change in commit order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HistoryLine {
    pub position: u64,
    pub task: u64,
    pub by: Actor,
    pub at: Wall,
    pub change: HistoryChange,
}

/// The change one history line describes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HistoryChange {
    /// Made in a batch.
    Made { batch: u64 },
    /// Specification changed.
    Amended { before: Text, after: Text },
    /// Held with its cause.
    Held(Hold),
    /// A hold lifted.
    Released,
    /// Cancelled with a reason.
    Cancelled { reason: Text },
    /// Requester changed.
    Moved { from: Actor, to: Actor, reason: Text },
}

/// One activity item in the order the engine committed it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActivityLine {
    pub position: u64,
    pub at: Wall,
    pub event: Activity,
}

/// A task activity event, turns decoded below this domain.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Activity {
    /// Words as the party wrote them.
    Words { by: Party, message: u64, words: Text },
    /// A run began.
    RunBegan { attempt: u32, how: RunBeginning, why: Text },
    /// A run ended.
    RunEnded { attempt: u32, how: RunEnding, spent: u64 },
    /// A committed decoded turn.
    Turn { name: TurnName, turn: Turn },
    /// A result heard.
    Result { result: Result },
    /// News heard.
    News { topic: u16, summary: Text, part: Option<Part> },
    /// A recorded decision.
    Decision { object: ObjectKey, by: Actor, how: Decision },
}

/// How an agent run begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RunBeginning {
    /// Resumed its transcript.
    Resumed,
    /// Started from a brief with a conversation tail.
    Fresh { tail: u32 },
}

/// How a run ended.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RunEnding {
    /// Parked for later.
    Parked,
    /// Finished its task.
    Finished,
    /// Cancelled.
    Cancelled,
    /// Failed with a class and reason.
    Failed { class: FailureClass, reason: Text },
}

/// Facts about a task phase that a chip alone does not carry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Runtime {
    /// Waiting on dependencies or a named resource hold.
    Waiting { dependencies: List<u64>, hold: Option<Part> },
    /// Idle and waiting on its wake policy.
    Idle,
    /// A run is due, preparing or claimed.
    Starting,
    /// A run is writing or calling a tool.
    Running { attempt: u32, since: Wall, tool: Option<Text> },
    /// A live run keeps its slot while waiting for the person.
    WaitingForWords { attempt: u32, since: Wall },
    /// A failed attempt waits until its retry time.
    BackingOff { at: Wall, attempt: u32, tries: u32, class: FailureClass, reason: Text },
    /// A connector procedure is active, shown in its panel.
    Procedure { panel: Part },
    /// A person task is waiting for its named recipient.
    Person { addressed: Recipient, since: Wall },
    /// Closing while its children and effects settle.
    Closing(Closing),
    /// Held, with its cause in the task hold.
    Held,
    /// Ended, with its outcome in the task result.
    Ended,
}

/// What a steering confirmation says a cancel or close would affect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Impact {
    pub tasks: PhaseCounts,
    pub live_runs: u32,
    pub proposals: u32,
    pub effects_pending: u32,
    pub effects_settling: u32,
    pub takeover_budget: u64,
}

/// A resource held for writing and the run holding its writer slot.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WriterSlot {
    pub resource: Part,
    pub task: u64,
    pub attempt: Option<u32>,
}

/// A live chat delegate and the funding a takeover would reserve.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LiveDelegate {
    pub chip: Chip,
    pub takeover_budget: u64,
}
