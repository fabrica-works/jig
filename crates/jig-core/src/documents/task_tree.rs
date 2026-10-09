//! Task pages, live trees and goal lines (`client/domain/pages.md`, 5 and 6.2).
//! Copies bounded rows from their owners. Trees fold every unshown live row into
//! phase counts; held counts cover the whole subtree. Goal progress uses retained
//! goal projections, including ended rows. Nothing here mutates a task or policy.

#![expect(clippy::manual_map, reason = "step code uses explicit matches instead of closures")]

use super::{Chip, Cursor, Missing, Proposer, Text, Window};
use crate::{Core, Limits};
use alloc::boxed::Box;
use jig_core_tasks as tasks;
use skein_lib::{List, Map, Wall};

/// Connector-owned value's kind, carried only as a reference.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PartKind {
    /// A named resource or its state.
    Resource,
    /// A counted resource pool.
    Pool,
    /// A task's connector result.
    Result,
    /// A procedure's panel or kind.
    Procedure,
    /// A proposed effect's description.
    Effect,
    /// An existing connector-owned outbox entry.
    Outbox,
}

/// The core's key for a connector-owned part; keys are never shortened.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PartKey {
    /// Literal resource path segments.
    Path(List<Box<[u8]>>),
    /// A numbered kind, resource, task or proposal.
    Number(u64),
}

/// A reference to application data; no application value is copied.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Part {
    pub connector: u16,
    pub kind: PartKind,
    pub key: PartKey,
}

/// Terminal coverage rule of a granted resource pattern.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PatternEnd {
    /// One literal terminal segment.
    Exact(Box<[u8]>),
    /// A prefix and its descendants.
    Open(Box<[u8]>),
}

/// One connector permission on an unchanged resource pattern.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GrantLine {
    pub connector: u16,
    pub kind: u16,
    pub segments: List<Box<[u8]>>,
    pub last: PatternEnd,
}

/// A connector resource scope where notes may be written.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct NoteScope {
    pub connector: u16,
    pub segments: List<Box<[u8]>>,
    pub last: PatternEnd,
}

/// An authority carrier in bounded document form.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AuthorityLine {
    pub tools: tasks::Tools,
    pub grants: List<GrantLine>,
    pub executors: List<tasks::AuthorityExecutor>,
    pub tasks: u32,
    pub depth: u32,
    pub ceiling: tasks::Budget,
    pub notes: tasks::Scopes,
    pub note_resources: List<NoteScope>,
    pub cut: bool,
}

/// Result form and its admitted bounds, usable as a person task's form.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ContractLine {
    /// A report in words.
    Report { words: u32 },
    /// A configured choice with an explanation and bounded follow-ups.
    Verdict { choices: List<tasks::Verdict>, cut: bool },
    /// A connector-owned change and its summary.
    Change { connector: u16, kind: u16, words: u32 },
}

/// Contract-shaped result words and an optional connector-owned value.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ResultValue {
    /// Successful report.
    Report(Text),
    /// A selected verdict code with its explanation.
    Verdict { code: u32, words: Text },
    /// A connector value with summary words.
    Change { kind: u16, part: Part, words: Text },
    /// A task that could not complete its contract.
    Failure(Text),
}

/// Ending carried while a live task closes, or in an ended result document.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ending {
    /// Completed its contract.
    Done(ResultValue),
    /// Failed with these words.
    Failed(Text),
    /// Cancelled, preserving any previously completed result.
    Cancelled { reason: Text, result: Option<ResultValue>, by: tasks::Party, at: Wall },
}

/// One task reference, including a historical result still named by live work.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TaskLink {
    /// An ended task whose stored owner row supplied its identity and wall time.
    Retained { chip: Chip, at: Option<Wall> },
    /// The referenced task is live and visible.
    Live(Chip),
    /// Its title and complete result belong to a stored task row.
    Stored { number: u64, status: Option<tasks::Status> },
}

/// A pending proposal's immutable identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ProposalRef {
    pub proposer: Proposer,
    pub number: u64,
}

/// Current held-decision recipient and any durable rejection words.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EscalationState {
    /// The hold is being routed to its eligible holder in this decision.
    Routing,
    /// An identified holder may decide it.
    Waiting { holder: tasks::EscalationHolder, entry: u64, since: Wall },
    /// Decided to leave it held.
    Rejected { by: u64, reason: Text },
}

/// A hold's reason, revision and prior lifecycle, with its escalation.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct HoldLine {
    pub source: Option<Part>,
    pub reason: tasks::Hold,
    pub revision: u64,
    pub before: PriorPhase,
    pub escalation: Option<EscalationState>,
    pub tries: tasks::Tries,
    pub hold_wait_since: Option<Wall>,
}

/// The lifecycle preserved by a hold.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PriorPhase {
    /// Waiting for dependencies or resource holdings.
    Waiting,
    /// An activation's phase.
    Active(tasks::Active),
    /// Closing through this obligation.
    Closing(tasks::Stage),
}

/// A task's concrete allotment and its financial source.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BudgetLine {
    pub numbers: tasks::Numbers,
    pub left: u64,
    pub funder: tasks::Funder,
    pub deadline: Option<Wall>,
}

/// A resource the task holds or awaits, and its authentic writer if present.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ResourceLine {
    pub part: Part,
    pub kind: u16,
    pub taken: bool,
    pub holder: Option<u64>,
    pub writer: Option<tasks::Writer>,
    pub writer_lost: bool,
}

/// A bounded specification parameter; connector resources remain references.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Parameter {
    /// An opaque named numeric value.
    Number { name: u32, value: u64 },
    /// An opaque named text value.
    Bytes { name: u32, value: Text },
    /// A connector-owned numbered resource.
    Resource { name: u32, part: Part },
}

/// A task page's header, overview, funding, authority and current held decision.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Task {
    pub chip: Chip,
    pub acts: List<crate::acts::Offered>,
    pub project: u32,
    pub goal: Option<u64>,
    pub priority: Option<u32>,
    pub executor: tasks::Executor,
    pub requester: tasks::Party,
    pub lineage: List<Chip>,
    /// Every live direct delegate; identities are never omitted from a close decision.
    pub delegates: List<super::LiveDelegate>,
    pub changed: Wall,
    pub spec_revision: u64,
    /// Last committed immutable activity position.
    pub activity: u64,
    pub words: Text,
    pub parameters: List<Parameter>,
    pub instructions: Option<Text>,
    pub resources: List<ResourceLine>,
    pub inputs: List<TaskLink>,
    pub dependencies: List<TaskLink>,
    pub references: List<TaskLink>,
    pub subscriptions: List<tasks::Subscription>,
    pub wake: tasks::WakePolicy,
    pub contract: ContractLine,
    pub result: Option<Ending>,
    pub budget: BudgetLine,
    pub authority: AuthorityLine,
    pub hold: Option<HoldLine>,
    pub proposals: List<ProposalRef>,
    /// At least one list omitted rows; individual text values state their cuts.
    pub cut: bool,
}

/// Phase totals of every task counted, independent of visible rows.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PhaseCounts {
    pub waiting: u32,
    pub idle: u32,
    pub starting: u32,
    pub running: u32,
    pub backing_off: u32,
    pub closing: u32,
    pub held: u32,
    pub done: u32,
    pub failed: u32,
    pub cancelled: u32,
}

impl PhaseCounts {
    /// Begin a bounded phase census.
    pub const ZERO: Self = Self {
        waiting: 0,
        idle: 0,
        starting: 0,
        running: 0,
        backing_off: 0,
        closing: 0,
        held: 0,
        done: 0,
        failed: 0,
        cancelled: 0,
    };

    pub(super) fn add(&mut self, phase: &tasks::Phase) {
        let count = match phase {
            tasks::Phase::Waiting => &mut self.waiting,
            tasks::Phase::Active(active) => match active {
                tasks::Active::Idle => &mut self.idle,
                tasks::Active::Due | tasks::Active::Preparing | tasks::Active::Claimed { .. } => &mut self.starting,
                tasks::Active::Running { .. } => &mut self.running,
                tasks::Active::BackingOff { .. } => &mut self.backing_off,
            },
            tasks::Phase::Closing(_) => &mut self.closing,
            tasks::Phase::Held { .. } => &mut self.held,
            tasks::Phase::Ended(ending) => match ending {
                tasks::Ending::Done(_) => &mut self.done,
                tasks::Ending::Failed { .. } => &mut self.failed,
                tasks::Ending::Cancelled { .. } => &mut self.cancelled,
            },
        };
        *count = count.checked_add(1).expect("census bounded by task limits");
    }
}

/// A visible live descendant and its immediate requester relationship.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TreeRow {
    pub chip: Chip,
    pub parent: u64,
    pub depth: u32,
    pub proposals: List<ProposalRef>,
    pub spent: u64,
    pub dependencies: List<TaskLink>,
    pub cut: bool,
}

/// Unshown descendants folded under their nearest shown common root.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FoldedCount {
    pub parent: u64,
    pub phases: PhaseCounts,
    pub held_below: u32,
}

/// A window on a task's live subtree; counts include rows outside the window.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tree {
    pub root: Chip,
    pub rows: List<TreeRow>,
    pub folded: List<FoldedCount>,
    pub held_below: u32,
    pub next: Option<Cursor>,
}

/// One live goal and its whole plan's progress, allotment and held count.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GoalLine {
    pub chip: Chip,
    pub priority: u32,
    pub phases: PhaseCounts,
    pub budget: BudgetLine,
    pub held_below: u32,
}

/// A page of goals ordered by priority and stable task number.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Goals {
    pub lines: List<GoalLine>,
    pub next: Option<Cursor>,
}

pub(super) fn part_path(limits: &Limits, connector: u16, kind: PartKind, bytes: &[Box<[u8]>]) -> Result<Part, Missing> {
    Ok(Part { connector, kind, key: PartKey::Path(path(limits, bytes)?) })
}

fn path(limits: &Limits, bytes: &[Box<[u8]>]) -> Result<List<Box<[u8]>>, Missing> {
    if bytes.len() > usize::try_from(limits.documents.lines).expect("line count fits") {
        return Err(Missing::TooLarge);
    }
    let mut result = List::with_capacity(limits.documents.lines);
    for segment in bytes {
        if segment.len() > usize::try_from(limits.documents.text_bytes).expect("text bound fits") {
            return Err(Missing::TooLarge);
        }
        result.push(skein_lib::bytes::copy_of(segment)).expect("validated path length");
    }
    Ok(result)
}

fn pattern(limits: &Limits, value: &tasks::Pattern) -> Result<PatternEnd, Missing> {
    let bytes = match &value.last {
        tasks::Last::Exact(bytes) | tasks::Last::Open(bytes) => bytes,
    };
    if bytes.len() > usize::try_from(limits.documents.text_bytes).expect("text bound fits") {
        return Err(Missing::TooLarge);
    }
    Ok(match &value.last {
        tasks::Last::Exact(bytes) => PatternEnd::Exact(skein_lib::bytes::copy_of(bytes)),
        tasks::Last::Open(bytes) => PatternEnd::Open(skein_lib::bytes::copy_of(bytes)),
    })
}

pub(super) fn authority(limits: &Limits, value: &tasks::Authority) -> Result<AuthorityLine, Missing> {
    let mut grants = List::with_capacity(limits.documents.lines);
    for grant in value.grants.iter().take(bound(limits)) {
        grants
            .push(GrantLine {
                connector: grant.connector,
                kind: grant.kind,
                segments: path(limits, &grant.pattern.segments)?,
                last: pattern(limits, &grant.pattern)?,
            })
            .expect("bounded grants");
    }
    let mut executors = List::with_capacity(limits.documents.lines);
    for kind in value.delegation.kinds.iter().take(bound(limits)) {
        executors.push(*kind).expect("bounded executor list");
    }
    let mut note_resources = List::with_capacity(limits.documents.lines);
    for scope in value.note_resources.iter().take(bound(limits)) {
        note_resources
            .push(NoteScope {
                connector: scope.connector,
                segments: path(limits, &scope.pattern.segments)?,
                last: pattern(limits, &scope.pattern)?,
            })
            .expect("bounded note scopes");
    }
    Ok(AuthorityLine {
        tools: value.tools,
        cut: size(grants.len()) < value.grants.len()
            || size(executors.len()) < value.delegation.kinds.len()
            || size(note_resources.len()) < value.note_resources.len(),
        grants,
        executors,
        tasks: value.delegation.tasks,
        depth: value.delegation.depth,
        ceiling: value.budget,
        notes: value.notes,
        note_resources,
    })
}

pub(super) fn contract(limits: &Limits, value: &tasks::Contract) -> ContractLine {
    match value {
        tasks::Contract::Report { words } => ContractLine::Report { words: *words },
        tasks::Contract::Change { connector, kind, words } => {
            ContractLine::Change { connector: *connector, kind: *kind, words: *words }
        }
        tasks::Contract::Verdict { choices } => {
            let mut result = List::with_capacity(limits.documents.lines);
            for choice in choices.iter().take(bound(limits)) {
                result.push(*choice).expect("bounded form choices");
            }
            ContractLine::Verdict { cut: size(result.len()) < choices.len(), choices: result }
        }
    }
}

pub(super) fn result_value(limits: &Limits, value: &tasks::TaskResult) -> ResultValue {
    match value {
        tasks::TaskResult::Report { words } => {
            ResultValue::Report(super::small::text(words, limits.documents.text_bytes))
        }
        tasks::TaskResult::Verdict { code, words } => {
            ResultValue::Verdict { code: *code, words: super::small::text(words, limits.documents.text_bytes) }
        }
        tasks::TaskResult::Change { connector, kind, resource, words } => ResultValue::Change {
            kind: *kind,
            part: Part { connector: *connector, kind: PartKind::Result, key: PartKey::Number(*resource) },
            words: super::small::text(words, limits.documents.text_bytes),
        },
        tasks::TaskResult::Failure { reason } => {
            ResultValue::Failure(super::small::text(reason, limits.documents.text_bytes))
        }
    }
}

pub(super) fn ending(limits: &Limits, value: &tasks::Ending) -> Ending {
    match value {
        tasks::Ending::Done(result) => Ending::Done(result_value(limits, result)),
        tasks::Ending::Failed { reason } => Ending::Failed(super::small::text(reason, limits.documents.text_bytes)),
        tasks::Ending::Cancelled { reason, result, by, at } => Ending::Cancelled {
            by: *by,
            at: *at,
            reason: super::small::text(reason, limits.documents.text_bytes),
            result: match result {
                Some(result) => Some(result_value(limits, result)),
                None => None,
            },
        },
    }
}

pub(super) fn hold_source(limits: &Limits, source: Option<&tasks::HoldSource>) -> Result<Option<Part>, Missing> {
    match source {
        None => Ok(None),
        Some(tasks::HoldSource::Resource(name)) => {
            Ok(Some(part_path(limits, name.connector, PartKind::Resource, &name.path)?))
        }
        Some(tasks::HoldSource::Effect { connector, entry }) => {
            Ok(Some(Part { connector: *connector, kind: PartKind::Outbox, key: PartKey::Number(*entry) }))
        }
    }
}

pub(super) fn hold(limits: &Limits, row: &tasks::TaskRecord) -> Result<Option<HoldLine>, Missing> {
    match &row.phase {
        tasks::Phase::Held { was, why, source } => Ok(Some(HoldLine {
            source: hold_source(limits, source.as_deref())?,
            reason: *why,
            revision: row.revisions.hold,
            before: match was {
                tasks::Was::Waiting => PriorPhase::Waiting,
                tasks::Was::Active(active) => PriorPhase::Active(*active),
                tasks::Was::Closing(closing) => PriorPhase::Closing(closing.stage),
            },
            escalation: match &row.escalation {
                tasks::Escalation::Unheld { .. } => None,
                tasks::Escalation::Routing { .. } => Some(EscalationState::Routing),
                tasks::Escalation::Waiting { holder, entry, since, .. } => {
                    Some(EscalationState::Waiting { holder: *holder, entry: *entry, since: *since })
                }
                tasks::Escalation::Rejected { by, reason, .. } => Some(EscalationState::Rejected {
                    by: *by,
                    reason: super::small::text(reason, limits.documents.text_bytes),
                }),
            },
            tries: row.tries,
            hold_wait_since: row.hold_wait_since,
        })),
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => Ok(None),
    }
}

pub(super) fn budget(row: &tasks::TaskRecord) -> BudgetLine {
    let left = row
        .numbers
        .budget
        .checked_sub(row.numbers.spent)
        .expect("spend within allotment")
        .checked_sub(row.numbers.spent_below)
        .expect("settled spend within allotment")
        .checked_sub(row.numbers.reserved)
        .expect("reservations within remaining allotment");
    BudgetLine { numbers: row.numbers, left, funder: row.funder, deadline: row.authority.budget.deadline }
}

fn bound(limits: &Limits) -> usize {
    size(limits.documents.lines)
}
fn size(value: u32) -> usize {
    usize::try_from(value).expect("bounded count fits usize")
}

pub(super) fn link(core: &Core, limits: &Limits, party: u64, number: u64) -> Result<TaskLink, Missing> {
    match core.tasks.task(number) {
        Some(_) => Ok(TaskLink::Live(super::small::chip(core, limits, party, number)?)),
        None => Ok(TaskLink::Stored {
            number,
            status: match core.tasks.stub(number) {
                Some(stub) => Some(stub.phase),
                None => None,
            },
        }),
    }
}

fn links(core: &Core, limits: &Limits, party: u64, numbers: &[u64]) -> Result<List<TaskLink>, Missing> {
    let mut result = List::with_capacity(limits.documents.lines);
    for number in numbers.iter().take(bound(limits)) {
        result.push(link(core, limits, party, *number)?).expect("bounded task links");
    }
    Ok(result)
}

pub(super) fn descendant(core: &Core, number: u64, root: u64, depth: u32) -> bool {
    let mut current = number;
    for _ in 0..=depth {
        if current == root {
            return true;
        }
        let Some(row) = core.tasks.task(current) else { return false };
        match row.requester {
            tasks::Party::Task(parent) => current = parent,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => return false,
        }
    }
    false
}

fn proposal(row: &tasks::TaskRecord) -> Option<ProposalRef> {
    let value = row.proposal.as_ref()?;
    match value.state {
        tasks::ProposalState::Pending { .. } => {
            Some(ProposalRef { proposer: Proposer::Task(value.proposer), number: value.number })
        }
        tasks::ProposalState::Accepted { .. }
        | tasks::ProposalState::Rejected { .. }
        | tasks::ProposalState::Withdrawn => None,
    }
}

pub(super) fn task(core: &Core, limits: &Limits, party: u64, number: u64) -> Result<Task, Missing> {
    let row = super::small::visible(core, party, number)?;
    task_from_row(core, limits, party, row)
}

/// Construct a visible task from its retained owner row, without restoring it.
#[expect(clippy::too_many_lines, reason = "the task page gathers one bounded overview without changing child state")]
pub fn task_from_row(core: &Core, limits: &Limits, party: u64, row: &tasks::TaskRecord) -> Result<Task, Missing> {
    if core.people.role(party, row.project).is_none() {
        return Err(Missing::NotAllowed);
    }
    let number = row.number;
    let mut lineage = List::with_capacity(limits.documents.lines);
    let mut ancestor = row.requester;
    let mut cut = false;
    let mut goal = if row.tracked.is_some() { Some(row.number) } else { None };
    for _ in 0..limits.tasks.depth {
        match ancestor {
            tasks::Party::Task(parent) => {
                let parent_row = match super::small::visible(core, party, parent) {
                    Ok(row) => row,
                    Err(Missing::Unknown | Missing::Ended) => {
                        cut = true;
                        break;
                    }
                    Err(why @ (Missing::NotAllowed | Missing::TooLarge)) => return Err(why),
                };
                if parent_row.tracked.is_some() {
                    goal = Some(parent);
                }
                if lineage.room() > 0 {
                    lineage.push(super::small::chip(core, limits, party, parent)?).expect("lineage room checked");
                } else {
                    cut = true;
                }
                ancestor = parent_row.requester;
            }
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => break,
        }
    }
    let mut ordered = List::with_capacity(limits.documents.lines);
    for index in (0..lineage.len()).rev() {
        ordered.push(lineage.get(index).expect("lineage index").clone()).expect("same bounded lineage");
    }
    let mut parameters = List::with_capacity(limits.documents.lines);
    for value in row.spec.parameters.iter().take(bound(limits)) {
        let parameter = match value {
            tasks::Parameter::Number { name, value } => Parameter::Number { name: *name, value: *value },
            tasks::Parameter::Bytes { name, value } => {
                Parameter::Bytes { name: *name, value: super::small::text(value, limits.documents.text_bytes) }
            }
            tasks::Parameter::Resource { name, connector, resource } => Parameter::Resource {
                name: *name,
                part: Part { connector: *connector, kind: PartKind::Resource, key: PartKey::Number(*resource) },
            },
        };
        parameters.push(parameter).expect("bounded parameters");
    }
    let mut resources = List::with_capacity(limits.documents.lines);
    for value in row.holdings.iter().take(bound(limits)) {
        let (name, kind, part_kind) = match value {
            tasks::Holding::Write { resource, kind } => (resource, *kind, PartKind::Resource),
            tasks::Holding::Slot { pool, kind } => (pool, *kind, PartKind::Pool),
        };
        let slot = core.tasks.writer(name);
        resources
            .push(ResourceLine {
                part: part_path(limits, name.connector, part_kind, &name.path)?,
                kind,
                taken: row.holds_taken,
                holder: core.tasks.holder(name),
                writer: match slot {
                    Some(slot) => Some(slot.writer),
                    None => None,
                },
                writer_lost: match slot {
                    Some(slot) => slot.lost,
                    None => false,
                },
            })
            .expect("bounded resource lines");
    }
    let mut proposals = List::with_capacity(limits.documents.lines);
    for candidate in core.tasks.view_tasks().as_ref() {
        if descendant(core, candidate.number, number, limits.tasks.depth)
            && let Some(reference) = proposal(core.tasks.task(candidate.number).expect("live indexed task"))
        {
            if proposals.room() == 0 {
                cut = true;
            } else {
                proposals.push(reference).expect("proposal room checked");
            }
        }
    }
    let mut subscriptions = List::with_capacity(limits.documents.lines);
    for value in row.subscriptions.iter().take(bound(limits)) {
        subscriptions.push(*value).expect("bounded subscriptions");
    }
    cut |= size(parameters.len()) < row.spec.parameters.len()
        || size(resources.len()) < row.holdings.len()
        || size(subscriptions.len()) < row.subscriptions.len()
        || row.spec.inputs.len() > bound(limits)
        || row.dependencies.len() > bound(limits)
        || row.references.len() > bound(limits);
    let result = match &row.phase {
        tasks::Phase::Closing(closing) => Some(ending(limits, &closing.ending)),
        tasks::Phase::Held { was, .. } => match was {
            tasks::Was::Closing(closing) => Some(ending(limits, &closing.ending)),
            tasks::Was::Waiting | tasks::Was::Active(_) => None,
        },
        tasks::Phase::Ended(value) => Some(ending(limits, value)),
        tasks::Phase::Waiting | tasks::Phase::Active(_) => None,
    };
    let acts = super::offers::task(core, limits, party, row)?;
    let authority = authority(limits, &row.authority)?;
    if authority.cut {
        for offer in &acts {
            if let crate::acts::Admission::Bounded { within: crate::acts::Bounds::Amend { .. }, .. } = offer.admission {
                // The before facts used to compare a draft must also be whole.
                return Err(Missing::TooLarge);
            }
        }
    }
    Ok(Task {
        acts,
        chip: super::small::chip_with_core(core, limits, row)?,
        project: row.project,
        goal,
        priority: row.tracked,
        executor: row.executor,
        requester: row.requester,
        lineage: ordered,
        delegates: super::live_delegates::from_row(core, limits, party, row)?,
        changed: row.changed,
        spec_revision: row.revisions.spec,
        activity: row.activity,
        words: super::small::text(&row.spec.words, limits.documents.text_bytes),
        parameters,
        instructions: match row.executor {
            tasks::Executor::Agent { .. } => {
                Some(super::small::text(&core.settings.run.instructions, limits.documents.text_bytes))
            }
            tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => None,
        },
        resources,
        inputs: links(core, limits, party, &row.spec.inputs)?,
        dependencies: links(core, limits, party, &row.dependencies)?,
        references: links(core, limits, party, &row.references)?,
        subscriptions,
        wake: row.wake,
        contract: contract(limits, &row.contract),
        result,
        budget: budget(row),
        authority,
        hold: hold(limits, row)?,
        proposals,
        cut,
    })
}

pub(super) fn tree(core: &Core, limits: &Limits, party: u64, number: u64, window: &Window) -> Result<Tree, Missing> {
    let root = super::small::visible(core, party, number)?;
    let mut rows = List::with_capacity(window.rows);
    let mut phases = PhaseCounts::ZERO;
    let mut held_below = 0_u32;
    let mut folded_total = 0_u32;
    let mut last = None;
    let after = match window.after {
        Some(Cursor::Number { number }) => number,
        Some(Cursor::Inbox { .. }) => return Err(Missing::Unknown),
        None => 0,
    };
    let mut more = false;
    for candidate in core.tasks.view_tasks().as_ref() {
        if candidate.number == number || !descendant(core, candidate.number, number, limits.tasks.depth) {
            continue;
        }
        let row = super::small::visible(core, party, candidate.number)?;
        let held = match row.phase {
            tasks::Phase::Held { .. } => true,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => {
                false
            }
        };
        if held {
            held_below = held_below.checked_add(1).expect("bounded held count");
        }
        if rows.room() == 0 || candidate.number <= after {
            phases.add(&row.phase);
            folded_total = folded_total.checked_add(1).expect("bounded folded count");
            more |= candidate.number > after;
            continue;
        }
        let parent = match row.requester {
            tasks::Party::Task(parent) => parent,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => {
                unreachable!("descendant has a requester task")
            }
        };
        let mut proposals = List::with_capacity(1);
        if let Some(value) = proposal(row) {
            proposals.push(value).expect("one proposal per task");
        }
        rows.push(TreeRow {
            chip: super::small::chip(core, limits, party, row.number)?,
            parent,
            depth: row.depth.checked_sub(root.depth).expect("subtree depth"),
            proposals,
            spent: row.numbers.spent.checked_add(row.numbers.spent_below).expect("bounded spent"),
            dependencies: links(core, limits, party, &row.dependencies)?,
            cut: row.dependencies.len() > bound(limits),
        })
        .expect("tree window room checked");
        last = Some(Cursor::Number { number: row.number });
    }
    let mut folded = List::with_capacity(1);
    if folded_total > 0 {
        folded.push(FoldedCount { parent: number, held_below: phases.held, phases }).expect("one fold at window root");
    }
    Ok(Tree {
        root: super::small::chip(core, limits, party, number)?,
        rows,
        folded,
        held_below,
        next: if more { last } else { None },
    })
}

pub(super) fn goals(core: &Core, limits: &Limits, party: u64, project: u32, window: &Window) -> Result<Goals, Missing> {
    if core.people.role(party, project).is_none() {
        return Err(Missing::NotAllowed);
    }
    let mut order = Map::with_capacity(limits.tasks.tasks);
    for candidate in core.tasks.view_tasks().as_ref() {
        if candidate.project == project
            && let Some(priority) = candidate.tracked
        {
            order.insert((priority, candidate.number), candidate.number).expect("bounded live goals");
        }
    }
    let mut lines = List::with_capacity(window.rows);
    let mut taking = window.after.is_none();
    let mut last = None;
    let mut more = false;
    for (_, number) in &order {
        if !taking {
            if window.after == Some(Cursor::Number { number: *number }) {
                taking = true;
            }
            continue;
        }
        if lines.room() == 0 {
            more = true;
            break;
        }
        let row = super::small::visible(core, party, *number)?;
        let mut phases = PhaseCounts::ZERO;
        let mut held_below = 0_u32;
        match core.projections.get(number) {
            Some(projection) => {
                for task in &projection.plan {
                    phases.add(&task.phase);
                    if task.number != *number {
                        match task.phase {
                            tasks::Phase::Held { .. } => {
                                held_below = held_below.checked_add(1).expect("bounded held census");
                            }
                            tasks::Phase::Waiting
                            | tasks::Phase::Active(_)
                            | tasks::Phase::Closing(_)
                            | tasks::Phase::Ended(_) => {}
                        }
                    }
                }
            }
            None => {
                // During an uncompleted root decision, only live state exists.
                for candidate in core.tasks.view_tasks().as_ref() {
                    if descendant(core, candidate.number, *number, limits.tasks.depth) {
                        let task = core.tasks.task(candidate.number).expect("indexed live task");
                        phases.add(&task.phase);
                        if candidate.number != *number {
                            match task.phase {
                                tasks::Phase::Held { .. } => {
                                    held_below = held_below.checked_add(1).expect("bounded held census");
                                }
                                tasks::Phase::Waiting
                                | tasks::Phase::Active(_)
                                | tasks::Phase::Closing(_)
                                | tasks::Phase::Ended(_) => {}
                            }
                        }
                    }
                }
            }
        }
        lines
            .push(GoalLine {
                chip: super::small::chip(core, limits, party, *number)?,
                priority: row.tracked.expect("ordered goal"),
                phases,
                budget: budget(row),
                held_below,
            })
            .expect("goal window room checked");
        last = Some(Cursor::Number { number: *number });
    }
    if !taking {
        return Err(Missing::Unknown);
    }
    Ok(Goals { lines, next: if more { last } else { None } })
}

pub(super) fn overview_worst_case(limits: &super::DocumentLimits) -> Option<u64> {
    let text = u64::from(limits.text_bytes);
    let lines = u64::from(limits.lines);
    let rows = u64::from(limits.rows);
    let chip = super::chip_phase::heap_worst_case(limits)?;
    let path = List::<Box<[u8]>>::worst_case(limits.lines)?.checked_add(lines.checked_mul(text)?)?;
    let authority = authority_worst_case(limits)?;
    let references = List::<TaskLink>::worst_case(limits.lines)?.checked_add(lines.checked_mul(chip)?)?;
    let task = u64::try_from(size_of::<Task>())
        .ok()?
        .checked_add(text.checked_mul(5)?)?
        .checked_add(path)?
        .checked_add(chip)?
        .checked_add(List::<super::LiveDelegate>::worst_case(limits.lines)?)?
        .checked_add(lines.checked_mul(chip)?)?
        .checked_add(List::<Chip>::worst_case(limits.lines)?.checked_add(lines.checked_mul(chip)?)?.checked_mul(2)?)?
        .checked_add(List::<Parameter>::worst_case(limits.lines)?.checked_add(lines.checked_mul(text)?)?)?
        .checked_add(List::<ResourceLine>::worst_case(limits.lines)?.checked_add(lines.checked_mul(path)?)?)?
        .checked_add(references.checked_mul(3)?)?
        .checked_add(List::<tasks::Subscription>::worst_case(limits.lines)?)?
        .checked_add(List::<tasks::Verdict>::worst_case(limits.lines)?)?
        .checked_add(List::<ProposalRef>::worst_case(limits.lines)?)?
        .checked_add(authority)?;
    let tree = chip
        .checked_add(List::<TreeRow>::worst_case(limits.rows)?)?
        .checked_add(
            rows.checked_mul(chip.checked_add(references)?.checked_add(List::<ProposalRef>::worst_case(1)?)?)?,
        )?
        .checked_add(List::<FoldedCount>::worst_case(1)?)?;
    let goals = List::<GoalLine>::worst_case(limits.rows)?.checked_add(rows.checked_mul(chip)?)?;
    Some(task.max(tree).max(goals))
}

pub(super) fn grant_worst_case(limits: &super::DocumentLimits) -> Option<u64> {
    List::<Box<[u8]>>::worst_case(limits.lines)?
        .checked_add(u64::from(limits.lines).checked_add(1)?.checked_mul(u64::from(limits.text_bytes))?)
}
pub(super) fn authority_worst_case(limits: &super::DocumentLimits) -> Option<u64> {
    List::<GrantLine>::worst_case(limits.lines)?
        .checked_add(List::<NoteScope>::worst_case(limits.lines)?)?
        .checked_add(u64::from(limits.lines).checked_mul(grant_worst_case(limits)?)?.checked_mul(2)?)?
        .checked_add(List::<tasks::AuthorityExecutor>::worst_case(limits.lines)?)
}

pub(super) fn worst_case(limits: &super::DocumentLimits) -> Option<u64> {
    overview_worst_case(limits)?.checked_add(super::offers::heap_worst_case(limits)?)
}
