//! Whole typed payload admission, including nested boxes and list capacities.
//! The kit stands in for decoding; client/domain/contract.md, 5.3.
use crate::Bounds;
use core::mem::size_of;
use jig_client::contract::{
    Account, ActKind, ActiveRun, Activity, ActivityLine, ActivityPage, Actor, AddressChange, Admission, AdmissionLimit,
    Amendment, Answer, Ask, AttemptFailure, Authority, Beyond, Block, Budget, Call, Capacity, Charter, Chip, ChipPhase,
    Closing, ClosingStage, ConversationLine, ConversationPage, Conversations, Count, CurrentCall, CurrentHold, Cursor,
    Decider, Decision, Delivery, Deployment, Disposal, DisposalIssue, DocumentSpecification, DocumentText, Ending,
    Escalation, Executor, FailureClass, FieldRule, Frame, FrameParty, GoalExecutor, GoalLine, GoalPage, GoalProcedure,
    Goals, Grant, HeldRole, HeldRuntime, HistoryChange, HistoryLine, HistoryPage, Hold, HoldCategory, HoldFailure,
    HoldReason, Holding, Host, Impact, Inbox, InboxEntry, InboxGroup, InboxPage, Lacking, Limit, LiveCall,
    LiveDelegate, Made, Message, Moved, Needs, Note, NoteAuthor, NotePage, NoteScope, ObjectKey, ObjectPage, Offered,
    OfferedCharters, OptionLine, Outcome, OwnerChipPhase, Parameter, ParameterDocument, Part, PartScope, Party,
    PartyKind, Passed, PersonAnswer, PersonContract, PersonTask, Phase, PhaseCounts, Policy, Pool, Priority, Project,
    ProjectRole, Proposal, ProposalAction, ProposalKey, ProposedTask, Proposer, ProvisionalTurnName, Question, Read,
    ReadNone, ReadResult, ReadValue, Recipient, Recurring, Refusal, ReplyPosition, Result, ResultContract, ResultField,
    ResultValue, RetryNotice, Revision, RolePolicy, RunBeginning, RunEnding, RunObservation, RunStage, RunState,
    RunText, RunWork, Runtime, Seen, Speaker, Specification, SpecificationDocument, SpendLine, SpendPage, SpendPeriod,
    Steering, Subject, TakenBy, Task, TaskContext, TaskFilter, TaskHold, TaskLine, TaskSpend, Tasks, Text, Tree,
    TreeNode, Turn, TurnLimits, TurnName, Usage, VersionRange, WakePolicy, WatchEnd, WatchEvent, WriterSlot,
};
use skein_lib::{Duration, List, Token, Wall};

mod result_details;

pub(crate) trait Payload {
    fn heap(&self, bounds: &Bounds) -> Option<u64>;
}

pub(crate) fn fits<T: Payload>(value: &T, bounds: &Bounds) -> bool {
    value
        .heap(bounds)
        .and_then(|heap| heap.checked_add(u64::try_from(size_of::<T>()).ok()?))
        .is_some_and(|size| size <= bounds.document_bytes)
}
macro_rules! plain {
    ($($t:ty),*) => { $(impl Payload for $t { fn heap(&self, _bounds: &Bounds) -> Option<u64> { Some(0) } })* };
}
plain!(
    bool,
    u16,
    u32,
    u64,
    Duration,
    Wall,
    Token,
    RunStage,
    ClosingStage,
    Proposer,
    jig_client::contract::RecurringOverlap
);
impl Payload for RetryNotice {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::OwnerFailed { .. } | Self::Failed { .. } | Self::Refused { .. } => Some(0),
        }
    }
}
impl Payload for Text {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        if self.fits(bounds.text) { u64::try_from(self.bytes().len()).ok() } else { None }
    }
}
impl Payload for Box<[u8]> {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        if self.len() > bounds.text as usize {
            return None;
        }
        u64::try_from(self.len()).ok()
    }
}
impl<T: Payload> Payload for List<T> {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        if self.capacity() > bounds.rows {
            return None;
        }
        let mut total = Self::worst_case(self.capacity())?;
        for item in self {
            total = total.checked_add(item.heap(bounds)?)?;
        }
        Some(total)
    }
}
impl<T: Payload> Payload for Option<T> {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.as_ref().map_or(Some(0), |value| value.heap(bounds))
    }
}
impl<T: Payload> Payload for Box<T> {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        u64::try_from(size_of::<T>()).ok()?.checked_add((**self).heap(bounds)?)
    }
}
macro_rules! fields {
    ($name:ty { $($field:ident),* ; acts }) => {
        impl Payload for $name {
            fn heap(&self, bounds: &Bounds) -> Option<u64> {
                let Self { $($field),*, acts } = self;
                let total = crate::offered::heap(acts, bounds)?;
                $(let total = total.checked_add($field.heap(bounds)?)?;)*
                Some(total)
            }
        }
    };
    ($name:ty { $($field:ident),* }) => {
        impl Payload for $name {
            fn heap(&self, bounds: &Bounds) -> Option<u64> {
                let Self { $($field),* } = self;
                let total = 0_u64;
                $(let total = total.checked_add($field.heap(bounds)?)?;)*
                Some(total)
            }
        }
    };
}
fields!(AddressChange { revision, address });
fields!(Amendment { addressed, spec, wake, dependencies, authority, instructions });
fields!(WakePolicy { words, results, topics, subscriptions });
impl Payload for PersonAnswer {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Choice { option, reason } => sum(&[option.heap(bounds), reason.heap(bounds)]),
            Self::Approved { state, words } => sum(&[state.heap(bounds), words.heap(bounds)]),
            Self::Words(value0) => sum(&[value0.heap(bounds)]),
            Self::Cannot { reason } => sum(&[reason.heap(bounds)]),
        }
    }
}
impl Payload for Disposal {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Cancel { task } | Self::TakeOver { task } => sum(&[task.heap(bounds)]),
        }
    }
}
fields!(Priority { task, priority });
impl Payload for Seen {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Results { through } => sum(&[through.heap(bounds)]),
            Self::Replies { chat, through } => sum(&[chat.heap(bounds), through.heap(bounds)]),
        }
    }
}
fields!(Holding { party, role });
fields!(Policy { ceiling, period, spend, roles, requirements, deciders });
fields!(RolePolicy { number, authority, allotment, decisions });
fields!(Decider { kind, role });
impl Payload for Ask {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::StartChat { project, words } => sum(&[project.heap(bounds), words.heap(bounds)]),
            Self::Write { project, task, words, release } => {
                sum(&[project.heap(bounds), task.heap(bounds), words.heap(bounds), release.heap(bounds)])
            }
            Self::AnswerQuestion { project, task, message, words } => {
                sum(&[project.heap(bounds), task.heap(bounds), message.heap(bounds), words.heap(bounds)])
            }
            Self::AnswerPerson { project, task, revision, answer } => {
                sum(&[project.heap(bounds), task.heap(bounds), revision.heap(bounds), answer.heap(bounds)])
            }
            Self::Take { project, task } | Self::HandBack { project, task } | Self::Stop { project, task } => {
                sum(&[project.heap(bounds), task.heap(bounds)])
            }
            Self::DecideProposal { project, proposal, decision } => {
                sum(&[project.heap(bounds), proposal.heap(bounds), decision.heap(bounds)])
            }
            Self::DecideEscalation { project, task, revision, decision } => {
                sum(&[project.heap(bounds), task.heap(bounds), revision.heap(bounds), decision.heap(bounds)])
            }
            Self::Release { project, task, revision, again } => {
                sum(&[project.heap(bounds), task.heap(bounds), revision.heap(bounds), again.heap(bounds)])
            }
            Self::Amend { project, task, revision, change, release, reason } => sum(&[
                project.heap(bounds),
                task.heap(bounds),
                revision.heap(bounds),
                change.heap(bounds),
                release.heap(bounds),
                reason.heap(bounds),
            ]),
            Self::Cancel { project, task, reason } | Self::TakeOver { project, task, reason } => {
                sum(&[project.heap(bounds), task.heap(bounds), reason.heap(bounds)])
            }
            Self::Close { project, chat, disposals } => {
                sum(&[project.heap(bounds), chat.heap(bounds), disposals.heap(bounds)])
            }
            Self::SetGoal { project, spec, executor, budget, priority, deadline } => sum(&[
                project.heap(bounds),
                spec.heap(bounds),
                executor.heap(bounds),
                budget.heap(bounds),
                priority.heap(bounds),
                deadline.heap(bounds),
            ]),
            Self::Prioritise { project, goals } => sum(&[project.heap(bounds), goals.heap(bounds)]),
            Self::MarkSeen { seen } => sum(&[seen.heap(bounds)]),
            Self::EditNote { project, note, scope, revision, description, body, references } => sum(&[
                project.heap(bounds),
                note.heap(bounds),
                revision.heap(bounds),
                scope.heap(bounds),
                description.heap(bounds),
                body.heap(bounds),
                references.heap(bounds),
            ]),
            Self::DeleteNote { project, note, scope, revision } => {
                sum(&[project.heap(bounds), note.heap(bounds), scope.heap(bounds), revision.heap(bounds)])
            }
            Self::ChangePolicy { project, edit } => sum(&[project.heap(bounds), edit.heap(bounds)]),
            Self::SetRoles { project, holdings } => sum(&[project.heap(bounds), holdings.heap(bounds)]),
            Self::MakeService { project, name, role } => {
                sum(&[project.heap(bounds), name.heap(bounds), role.heap(bounds)])
            }
            Self::SetPool { project, party, budget } => {
                sum(&[project.heap(bounds), party.heap(bounds), budget.heap(bounds)])
            }
        }
    }
}
impl Payload for Outcome {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Owner(value) => value.heap(bounds),
            Self::Made { what } => sum(&[what.heap(bounds)]),
            Self::Proposed { proposal } => sum(&[proposal.heap(bounds)]),
            Self::Decided { by, how, at } => sum(&[by.heap(bounds), how.heap(bounds), at.heap(bounds)]),
            Self::DecidedFirst { by, how, at } => sum(&[by.heap(bounds), how.heap(bounds), at.heap(bounds)]),
            Self::Changed => Some(0),
        }
    }
}
impl Payload for Made {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Chat { task } | Self::Goal { task } | Self::PersonResult { task } => sum(&[task.heap(bounds)]),
            Self::Words { task, message } => sum(&[task.heap(bounds), message.heap(bounds)]),
            Self::Tasks { tasks } => sum(&[tasks.heap(bounds)]),
            Self::Service { party } => sum(&[party.heap(bounds)]),
            Self::Note { note, scope } => sum(&[note.heap(bounds), scope.heap(bounds)]),
        }
    }
}
impl Payload for Answer {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Done(value0) => sum(&[value0.heap(bounds)]),
            Self::Refused(value0) => sum(&[value0.heap(bounds)]),
        }
    }
}
fields!(Revision { value });
fields!(ProposalKey { proposer, number });
impl Payload for ObjectKey {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Task(value0) | Self::Escalation(value0) | Self::PersonTask(value0) | Self::Result(value0) => {
                sum(&[value0.heap(bounds)])
            }
            Self::Proposal(value0) => sum(&[value0.heap(bounds)]),
            Self::Question { task, message } => sum(&[task.heap(bounds), message.heap(bounds)]),
            Self::Note { project, note } => sum(&[project.heap(bounds), note.heap(bounds)]),
        }
    }
}
impl Payload for PartyKind {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Person | Self::Service => Some(0),
        }
    }
}
fields!(Party { number, kind, name });
fields!(TakenBy { number, name });
impl Payload for Actor {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Party(value0) | Self::Task(value0) => sum(&[value0.heap(bounds)]),
            Self::Deployment => Some(0),
            Self::ProjectDeployment { project } => project.heap(bounds),
        }
    }
}
fields!(GoalProcedure { connector, code });
impl Payload for GoalExecutor {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Charter(charter) => charter.heap(bounds),
            Self::Procedure { connector, code } => sum(&[connector.heap(bounds), code.heap(bounds)]),
        }
    }
}
impl Payload for Executor {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Agent { charter } => sum(&[charter.heap(bounds)]),
            Self::Procedure { connector, kind } => sum(&[connector.heap(bounds), kind.heap(bounds)]),
            Self::Party { party } => sum(&[party.heap(bounds)]),
            Self::Role { project, role } => sum(&[project.heap(bounds), role.heap(bounds)]),
        }
    }
}
impl Payload for Phase {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Waiting
            | Self::Idle
            | Self::Due
            | Self::Preparing
            | Self::Claimed
            | Self::Running
            | Self::BackingOff
            | Self::Active
            | Self::Closing
            | Self::Held
            | Self::Done
            | Self::Failed
            | Self::Cancelled
            | Self::Ended => Some(0),
        }
    }
}
impl Payload for PartScope {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Project(project) => project.heap(bounds),
            Self::Deployment => Some(0),
        }
    }
}
impl Payload for Part {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Resource { connector, path } => sum(&[connector.heap(bounds), path.heap(bounds)]),
            Self::Kind { connector, kind } => sum(&[connector.heap(bounds), kind.heap(bounds)]),
            Self::Panel { connector, task } | Self::Result { connector, task } | Self::Hold { connector, task } => {
                sum(&[connector.heap(bounds), task.heap(bounds)])
            }
            Self::Effect { connector, proposal } => sum(&[connector.heap(bounds), proposal.heap(bounds)]),
            Self::Outbox { connector, scope, entry } => {
                sum(&[connector.heap(bounds), scope.heap(bounds), entry.heap(bounds)])
            }
            Self::Project { connector, project } => sum(&[connector.heap(bounds), project.heap(bounds)]),
            Self::Connector { connector } | Self::Deployment { connector } => sum(&[connector.heap(bounds)]),
        }
    }
}
fields!(Grant { connector, kind, pattern });
fields!(Authority { budget, grants, task_kinds, depth, tasks, deadline, tools, note_scopes });
impl Payload for NoteScope {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Deployment => Some(0),
            Self::Project(value0) => sum(&[value0.heap(bounds)]),
            Self::Resource { project, connector, pattern } => {
                sum(&[project.heap(bounds), connector.heap(bounds), pattern.heap(bounds)])
            }
            Self::Goal { project, goal } => sum(&[project.heap(bounds), goal.heap(bounds)]),
        }
    }
}
impl Payload for Ending {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Done | Self::Deleted => Some(0),
            Self::Task(value) => value.heap(bounds),
            Self::Failed { reason } => reason.heap(bounds),
            Self::Withdrawn { reason } => reason.heap(bounds),
            Self::Cancelled { by, reason, at } => sum(&[by.heap(bounds), reason.heap(bounds), at.heap(bounds)]),
            Self::Decided { by, party, how, at } => {
                sum(&[by.heap(bounds), party.heap(bounds), how.heap(bounds), at.heap(bounds)])
            }
        }
    }
}
impl Payload for Decision {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Accepted | Self::PassedUp => Some(0),
            Self::Rejected { reason } | Self::LeftHeld { reason } => sum(&[reason.heap(bounds)]),
            Self::Released { again } => sum(&[again.heap(bounds)]),
        }
    }
}
fields!(TurnName { attempt, number });
fields!(Count { value, limited });
fields!(Frame {
    party,
    projects,
    needs,
    needs_total,
    updates,
    results_seen,
    charters,
    cut,
    retention,
    deployment_role
});
fields!(FrameParty { number, kind, login, name });
fields!(Needs { questions, proposals, escalations, person_tasks });
fields!(ProjectRole { project, name, role, pool, chat_budget });
fields!(OfferedCharters { project, chats, goals, cut });
fields!(Charter { number, name });
fields!(Budget { total, own, ended, reserved, left, funder });
fields!(Chip { task, title, executor, phase, details });
fields!(ActiveRun { stage, phase_since, attempt, placement, retry, observation });
fields!(RunObservation { channel, work, since, reported_at, started });
impl Payload for RunWork {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Tool { call, tool } => sum(&[call.heap(bounds), tool.heap(bounds)]),
            Self::Preparing | Self::Starting | Self::Active | Self::Writing | Self::Waiting | Self::Ending => Some(0),
        }
    }
}
impl Payload for HeldRuntime {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Waiting => Some(0),
            Self::Active(value) => value.heap(bounds),
            Self::Closing(value) => value.heap(bounds),
        }
    }
}
impl Payload for OwnerChipPhase {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Waiting { dependencies, .. } => dependencies.heap(bounds),
            Self::Active(_) | Self::Person { .. } | Self::Closing(_) | Self::Held { .. } | Self::Ended => Some(0),
        }
    }
}
impl Payload for ChipPhase {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Owner { phase, details } => sum(&[phase.heap(bounds), details.heap(bounds)]),
            Self::Active(value) => value.heap(bounds),
            Self::Waiting { dependencies, hold } => sum(&[dependencies.heap(bounds), hold.heap(bounds)]),
            Self::Idle { wake } => wake.heap(bounds),
            Self::Starting => Some(0),
            Self::Running { attempt, since, tool } => {
                sum(&[attempt.heap(bounds), since.heap(bounds), tool.heap(bounds)])
            }
            Self::WaitingForWords { attempt, since } => sum(&[attempt.heap(bounds), since.heap(bounds)]),
            Self::BackingOff { at, attempt, tries, class, reason } => sum(&[
                at.heap(bounds),
                attempt.heap(bounds),
                tries.heap(bounds),
                class.heap(bounds),
                reason.heap(bounds),
            ]),
            Self::Procedure { panel } => panel.heap(bounds),
            Self::Person { addressed, taken, since } => {
                sum(&[addressed.heap(bounds), taken.heap(bounds), since.heap(bounds)])
            }
            Self::Closing(value) => value.heap(bounds),
            Self::Held { reason, since, revision, was } => {
                sum(&[reason.heap(bounds), since.heap(bounds), revision.heap(bounds), was.heap(bounds)])
            }
            Self::Ended { how, at } => sum(&[how.heap(bounds), at.heap(bounds)]),
        }
    }
}
fields!(Task {
    chip,
    goal,
    priority,
    lineage_chips,
    inputs,
    project,
    requester,
    lineage,
    spec,
    spec_revision,
    person_revision,
    taken_by,
    instructions,
    wake,
    subscriptions,
    result_contract,
    dependencies,
    references,
    resources,
    budget,
    authority,
    hold,
    proposals,
    activity,
    changed,
    result,
    delegates,
    context,
    cut; acts
});
fields!(TaskContext { escalation, parts, closing, spend, runtime, impact, writer_slots });
fields!(Closing { delegates, connectors, stage, ending });
fields!(SpendLine { task, attempt, spent });
impl Payload for ActKind {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Write
            | Self::Answer
            | Self::Take
            | Self::HandBack
            | Self::Accept
            | Self::Reject
            | Self::PassUp
            | Self::Release
            | Self::LeaveHeld
            | Self::Amend
            | Self::AmendRelease
            | Self::Stop
            | Self::Cancel
            | Self::Close
            | Self::TakeOver
            | Self::SetGoal
            | Self::Prioritise
            | Self::MarkSeen
            | Self::EditNote
            | Self::DeleteNote
            | Self::Adopt
            | Self::ChangePolicy
            | Self::SetRoles
            | Self::MakeService
            | Self::SetPool => Some(0),
        }
    }
}
impl Payload for Admission {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Allowed => Some(0),
            Self::Proposed { to } => sum(&[to.heap(bounds)]),
            Self::Refused(value0) => sum(&[value0.heap(bounds)]),
            Self::Bounded { within, beyond } => sum(&[within.heap(bounds), beyond.heap(bounds)]),
        }
    }
}
impl Payload for jig_client::contract::Bounds {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Goal { pool_left, charters, procedures, latest, budget, maximum_budget, maximum_deadline } => sum(&[
                pool_left.heap(bounds),
                charters.heap(bounds),
                procedures.heap(bounds),
                latest.heap(bounds),
                budget.heap(bounds),
                maximum_budget.heap(bounds),
                maximum_deadline.heap(bounds),
            ]),
            Self::Amend { budget_left, grants, implications, authority, ceiling } => sum(&[
                budget_left.heap(bounds),
                grants.heap(bounds),
                implications.heap(bounds),
                authority.heap(bounds),
                ceiling.heap(bounds),
            ]),
        }
    }
}
impl Payload for Beyond {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Proposed { to } => to.heap(bounds),
            Self::Refused(why) => why.heap(bounds),
        }
    }
}
fields!(Offered { act, admission });
impl Payload for Recipient {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Task(value0) | Self::Party(value0) => sum(&[value0.heap(bounds)]),
            Self::Role { project, role } => sum(&[project.heap(bounds), role.heap(bounds)]),
            Self::Roles { project, roles } => sum(&[project.heap(bounds), roles.heap(bounds)]),
        }
    }
}
fields!(Hold { revision, reason, before, since, result, spend });
impl Payload for HoldReason {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Tries { failures } => sum(&[failures.heap(bounds)]),
            Self::Stalled { since, waiting } => sum(&[since.heap(bounds), waiting.heap(bounds)]),
            Self::Dependency { task, how } => sum(&[task.heap(bounds), how.heap(bounds)]),
            Self::Drift { part }
            | Self::EffectFailed { part }
            | Self::EffectUncertain { part }
            | Self::Pool { part }
            | Self::Procedure { part } => sum(&[part.heap(bounds)]),
            Self::Budget { spent, limit } => sum(&[spent.heap(bounds), limit.heap(bounds)]),
            Self::Deadline { at } => sum(&[at.heap(bounds)]),
            Self::Stopped { by, at } => sum(&[by.heap(bounds), at.heap(bounds)]),
            Self::Resource { resource, holder } => sum(&[resource.heap(bounds), holder.heap(bounds)]),
        }
    }
}
fields!(AttemptFailure { attempt, class, reason, at });
impl Payload for FailureClass {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::HostLost | Self::Temporary | Self::Permanent | Self::Stall => Some(0),
        }
    }
}
fields!(Escalation { ending, task, hold, steering, impact, waiting, trail; acts });
fields!(Steering {
    addressed,
    person_revision,
    taken_by,
    revision,
    spec,
    instructions,
    wake,
    dependencies,
    authority,
    budget
});
fields!(Passed { by, at, reason, lacking });
fields!(Proposal { key, project, proposer, why, action, needs, funding, waiting, trail, since, ending; acts });
impl Payload for ProposalAction {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Batch { tasks, own } => sum(&[tasks.heap(bounds), own.heap(bounds)]),
            Self::Effect { part } => sum(&[part.heap(bounds)]),
            Self::Widen { before, after } => sum(&[before.heap(bounds), after.heap(bounds)]),
            Self::Amend { task, before, change } => sum(&[task.heap(bounds), before.heap(bounds), change.heap(bounds)]),
        }
    }
}
fields!(ProposedTask {
    number,
    spec,
    executor,
    contract,
    authority,
    budget,
    funder,
    dependencies,
    resources,
    wake,
    priority,
    cut
});
plain!(
    jig_client::contract::OwnerExecutor,
    jig_client::contract::PersonAddress,
    jig_client::contract::Funder,
    jig_client::contract::OwnerPartKind
);
fields!(jig_client::contract::OwnerPart { connector, kind, key });
impl Payload for jig_client::contract::OwnerPartKey {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Path(segments) => segments.heap(bounds),
            Self::Number(_) => Some(0),
        }
    }
}
fields!(jig_client::contract::VerdictChoice { code, words, followups, fields });
impl Payload for jig_client::contract::TaskContractDocument {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Approval { state, part } => sum(&[state.heap(bounds), part.heap(bounds)]),
            Self::Report { words } => words.heap(bounds),
            Self::Verdict { choices, cut } => sum(&[choices.heap(bounds), cut.heap(bounds)]),
            Self::Change { connector, kind, words } => {
                sum(&[connector.heap(bounds), kind.heap(bounds), words.heap(bounds)])
            }
        }
    }
}
fields!(Question { task, from, message, words, since, ending; acts });
fields!(PersonTask {
    task,
    revision,
    requester,
    spec,
    contract,
    inputs,
    addressed,
    taken,
    since,
    stall,
    deadline,
    ending; acts
});
impl Payload for PersonContract {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Choice { options } => sum(&[options.heap(bounds)]),
            Self::Approval { state, part } => sum(&[state.heap(bounds), part.heap(bounds)]),
            Self::Words => Some(0),
        }
    }
}
fields!(FieldRule { name, max });
fields!(ResultField { name, value });
fields!(OptionLine { number, words, words_max, followups, fields });
impl Payload for ResultContract {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Report | Self::Approval | Self::Words => Some(0),
            Self::Verdict { choices } => sum(&[choices.heap(bounds)]),
            Self::Connector { connector, kind } => sum(&[connector.heap(bounds), kind.heap(bounds)]),
            Self::Choice { options } => sum(&[options.heap(bounds)]),
        }
    }
}
impl Payload for jig_client::contract::TaskResource {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Part(part) => part.heap(bounds),
            Self::Owner(value) => value.heap(bounds),
        }
    }
}
impl Payload for jig_client::contract::TaskResult {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Document(value) => value.heap(bounds),
            Self::Ending(value) => value.heap(bounds),
        }
    }
}
impl Payload for ResultValue {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Report(value0) | Self::Words(value0) | Self::Failure(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerVerdict { verdict, words, fields } => {
                sum(&[verdict.heap(bounds), words.heap(bounds), fields.heap(bounds)])
            }
            Self::Verdict { verdict, words, fields, tasks, proposal } => sum(&[
                verdict.heap(bounds),
                words.heap(bounds),
                fields.heap(bounds),
                tasks.heap(bounds),
                proposal.heap(bounds),
            ]),
            Self::Connector { summary, part } => sum(&[summary.heap(bounds), part.heap(bounds)]),
            Self::Change { kind, part, words } => sum(&[kind.heap(bounds), part.heap(bounds), words.heap(bounds)]),
            Self::Choice { option, by, reason } => sum(&[option.heap(bounds), by.heap(bounds), reason.heap(bounds)]),
            Self::Approved { state, words } => sum(&[state.heap(bounds), words.heap(bounds)]),
        }
    }
}
fields!(Result { task, ending, value, at, details; acts });
fields!(PhaseCounts { ended, waiting, running, held, closing, done, failed, cancelled });
fields!(TreeNode { chip, parent, dependencies, proposals, spend, folded, held_below });
fields!(Tree { task, nodes, folded, held_below });
impl Payload for InboxGroup {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Questions
            | Self::Proposals
            | Self::Escalations
            | Self::PersonTasks
            | Self::Results
            | Self::Replies => Some(0),
        }
    }
}
fields!(InboxEntry { reply, lineage, goal, recipient, stall, object, project, group, position, at });
fields!(Inbox { entries, needs, updates, results_seen, replies_seen, more });
fields!(ReplyPosition { chat, through });
fields!(ConversationLine {
    cached_replies,
    runtime,
    delegates,
    budget,
    chip,
    project,
    first_words,
    unread,
    seen,
    last_reply,
    changed
});
fields!(Conversations { rows, more, after });
fields!(GoalLine { changed, chip, priority, progress, budget, held_below });
fields!(Goals { project, rows, more });
fields!(Project {
    period,
    spend_lines,
    number,
    name,
    policy,
    roles,
    spend,
    pools,
    resources,
    services,
    recurring,
    parts; acts
});
fields!(Pool { party, period, budget });
fields!(Recurring {
    chip,
    key,
    batch,
    budget,
    overlap,
    current_period,
    began,
    next,
    last_period,
    pending_period,
    last_batch,
    cut
});
impl Payload for NoteAuthor {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Party { .. } | Self::Task { .. } => Some(0),
        }
    }
}
fields!(Note { number, revision, scope, description, body, writer, tasks, changed, cut; acts });
fields!(Deployment { hosts, accounts, capacity, proposals, refusals, parts });
fields!(Host { number, connected, grace, used, free, runs, refused });
fields!(Account { number, usable, reason, runs, waiting });
fields!(Capacity { project, live, limit });
fields!(HistoryLine { position, task, by, at, change });
impl Payload for jig_client::contract::HistoryRow {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Line(value) => value.heap(bounds),
            Self::Change(value) => value.heap(bounds),
            Self::Milestone(value) => value.heap(bounds),
        }
    }
}
impl Payload for HistoryChange {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Made { batch } => sum(&[batch.heap(bounds)]),
            Self::Amended { before, after } => sum(&[before.heap(bounds), after.heap(bounds)]),
            Self::Held(value0) => sum(&[value0.heap(bounds)]),
            Self::Released => Some(0),
            Self::Cancelled { reason } => sum(&[reason.heap(bounds)]),
            Self::Moved { from, to, reason } => sum(&[from.heap(bounds), to.heap(bounds), reason.heap(bounds)]),
        }
    }
}
impl Payload for ActivityLine {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        if let Activity::Owner { source, .. } = &self.event
            && (source.position != self.position || source.at != self.at)
        {
            return None;
        }
        self.event.heap(bounds)
    }
}
impl Payload for Activity {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Owner { source, decoded } => {
                if let Some(archive) = decoded {
                    let jig_client::contract::OwnerActivity::Turn { attempt, turn, .. } = source.line else {
                        return None;
                    };
                    if archive.task != source.task || archive.name != (TurnName { attempt, number: turn }) {
                        return None;
                    }
                }
                sum(&[source.heap(bounds), decoded.heap(bounds)])
            }
            Self::Words { by, message, words } => sum(&[by.heap(bounds), message.heap(bounds), words.heap(bounds)]),
            Self::RunBegan { attempt, how, why } => sum(&[attempt.heap(bounds), how.heap(bounds), why.heap(bounds)]),
            Self::RunEnded { attempt, how, spent } => {
                sum(&[attempt.heap(bounds), how.heap(bounds), spent.heap(bounds)])
            }
            Self::Turn { name, turn } => sum(&[name.heap(bounds), turn.heap(bounds)]),
            Self::Result { result } => sum(&[result.heap(bounds)]),
            Self::News { topic, summary, part } => sum(&[topic.heap(bounds), summary.heap(bounds), part.heap(bounds)]),
            Self::Decision { object, by, how } => sum(&[object.heap(bounds), by.heap(bounds), how.heap(bounds)]),
        }
    }
}
impl Payload for RunBeginning {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Resumed => Some(0),
            Self::Fresh { tail } => sum(&[tail.heap(bounds)]),
        }
    }
}
impl Payload for RunEnding {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Parked | Self::Finished | Self::Cancelled => Some(0),
            Self::Failed { class, reason } => sum(&[class.heap(bounds), reason.heap(bounds)]),
        }
    }
}
impl Payload for Runtime {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Active(value) => value.heap(bounds),
            Self::Waiting { dependencies, hold } => sum(&[dependencies.heap(bounds), hold.heap(bounds)]),
            Self::Idle | Self::Starting | Self::Ended => Some(0),
            Self::Held { was } => was.heap(bounds),
            Self::Running { attempt, since, tool } => {
                sum(&[attempt.heap(bounds), since.heap(bounds), tool.heap(bounds)])
            }
            Self::WaitingForWords { attempt, since } => sum(&[attempt.heap(bounds), since.heap(bounds)]),
            Self::BackingOff { at, attempt, tries, class, reason } => sum(&[
                at.heap(bounds),
                attempt.heap(bounds),
                tries.heap(bounds),
                class.heap(bounds),
                reason.heap(bounds),
            ]),
            Self::Procedure { panel } => sum(&[panel.heap(bounds)]),
            Self::Person { addressed, since } => sum(&[addressed.heap(bounds), since.heap(bounds)]),
            Self::Closing(value0) => sum(&[value0.heap(bounds)]),
        }
    }
}
fields!(Impact { tasks, live_runs, proposals, effects_pending, effects_settling, takeover_budget });
fields!(WriterSlot { resource, task, attempt });
fields!(LiveDelegate { chip, takeover_budget });
fields!(TaskFilter { phase, executor, goal, asked_by_me });
fields!(TaskLine { asked_by_me, chip, goal, requester, changed });
fields!(Tasks { project, filter, rows, more });
fields!(TaskSpend { chip, spent });
fields!(SpendPeriod { period, budget, pools, tasks });
fields!(Cursor { value });
impl Payload for Read {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Frame | Self::Deployment => Some(0),
            Self::Task { task } | Self::Escalation { task } | Self::PersonTask { task } | Self::Result { task } => {
                sum(&[task.heap(bounds)])
            }
            Self::Proposal { proposal } => sum(&[proposal.heap(bounds)]),
            Self::Question { task, message } => sum(&[task.heap(bounds), message.heap(bounds)]),
            Self::Ended { task, after } => sum(&[task.heap(bounds), after.heap(bounds)]),
            Self::History { task, subtree, after } => {
                sum(&[task.heap(bounds), subtree.heap(bounds), after.heap(bounds)])
            }
            Self::Activity { task, before } => sum(&[task.heap(bounds), before.heap(bounds)]),
            Self::Inbox { group: _, project, kind, goal, after } => {
                sum(&[project.heap(bounds), kind.heap(bounds), goal.heap(bounds), after.heap(bounds)])
            }
            Self::Conversations { after } => sum(&[after.heap(bounds)]),
            Self::Goals { project, after } | Self::Spend { project, after } => {
                sum(&[project.heap(bounds), after.heap(bounds)])
            }
            Self::Project { project } => sum(&[project.heap(bounds)]),
            Self::Notes { scope, description, after } => {
                sum(&[scope.heap(bounds), description.heap(bounds), after.heap(bounds)])
            }
        }
    }
}
fields!(ObjectPage { cut, rows, before, after });
fields!(ConversationPage { cut, rows, before, after });
fields!(ActivityPage { cut, rows, before, after });
fields!(HistoryPage { cut, rows, before, after });
fields!(InboxPage { cut, rows, before, after });
fields!(GoalPage { cut, rows, before, after });
fields!(NotePage { cut, rows, before, after });
impl Payload for ReadValue {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Frame(value0) => sum(&[value0.heap(bounds)]),
            Self::Task(value0) => sum(&[value0.heap(bounds)]),
            Self::Proposal(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerProject(value0) => sum(&[value0.heap(bounds)]),
            Self::Historical(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerProposal(value0) => sum(&[value0.heap(bounds)]),
            Self::Escalation(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerEscalation(value0) => sum(&[value0.heap(bounds)]),
            Self::Question(value0) => sum(&[value0.heap(bounds)]),
            Self::PersonTask(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerPersonTask(value) => value.heap(bounds),
            Self::Result(value0) => sum(&[value0.heap(bounds)]),
            Self::Ended(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerEnded(value) => value.heap(bounds),
            Self::History(value0) => sum(&[value0.heap(bounds)]),
            Self::Activity(value0) => sum(&[value0.heap(bounds)]),
            Self::Inbox(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerInbox(value0) => sum(&[value0.heap(bounds)]),
            Self::Conversations(value0) => sum(&[value0.heap(bounds)]),
            Self::Goals(value0) => sum(&[value0.heap(bounds)]),
            Self::Spend(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerSpend(value) => value.heap(bounds),
            Self::Project(value0) => sum(&[value0.heap(bounds)]),
            Self::Notes(value0) => sum(&[value0.heap(bounds)]),
            Self::Deployment(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerDeployment(value0) => value0.heap(bounds),
        }
    }
}
impl Payload for ReadNone {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Unknown | Self::Refused | Self::Gone | Self::Unreachable | Self::Busy | Self::SignedOut => Some(0),
        }
    }
}
impl Payload for ReadResult {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Value(value0) => sum(&[value0.heap(bounds)]),
            Self::None(value0) => sum(&[value0.heap(bounds)]),
        }
    }
}
fields!(SpendPage { cut, rows, before, after });
impl Payload for Lacking {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Budget { needed, left } => sum(&[needed.heap(bounds), left.heap(bounds)]),
            Self::Grant(value0) => sum(&[value0.heap(bounds)]),
            Self::TaskKind { executor } => executor.heap(bounds),
            Self::GrantKind { kind } => kind.heap(bounds),
            Self::Approver
            | Self::Funding
            | Self::Tool
            | Self::Scope
            | Self::Reference
            | Self::ResourceAccess
            | Self::ResourceUnavailable
            | Self::Shape
            | Self::Account
            | Self::Writer => Some(0),
            Self::Again { required } => required.heap(bounds),
            Self::Required { connector, kind }
            | Self::Failed { connector, kind }
            | Self::Unguarded { connector, kind } => sum(&[connector.heap(bounds), kind.heap(bounds)]),
            Self::Depth { needed, allowed } => sum(&[needed.heap(bounds), allowed.heap(bounds)]),
            Self::Deadline { asked, latest } => sum(&[asked.heap(bounds), latest.heap(bounds)]),
            Self::Tasks { needed, left } => sum(&[needed.heap(bounds), left.heap(bounds)]),
        }
    }
}
impl Payload for Limit {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Admission { which } => which.heap(bounds),
            Self::LiveTasks { project, limit } => sum(&[project.heap(bounds), limit.heap(bounds)]),
            Self::Inbox { task, limit } => sum(&[task.heap(bounds), limit.heap(bounds)]),
            Self::Words { limit } | Self::ConversationWords { limit } => sum(&[limit.heap(bounds)]),
            Self::Fields | Self::Records | Self::Requests | Self::Reads | Self::Watches | Self::Objects => Some(0),
        }
    }
}
impl Payload for Moved {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Hold { reason } => reason.heap(bounds),
            Self::Spec | Self::PersonTask | Self::Note | Self::Waiting => Some(0),
        }
    }
}
impl Payload for Refusal {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Role { held } => held.heap(bounds),
            Self::Disposal { task, issue } => sum(&[task.heap(bounds), issue.heap(bounds)]),
            Self::Taken { by } => by.heap(bounds),
            Self::Authority { lacking, proposable } => sum(&[lacking.heap(bounds), proposable.heap(bounds)]),
            Self::Limit { which } => sum(&[which.heap(bounds)]),
            Self::Moved { now, what } => sum(&[now.heap(bounds), what.heap(bounds)]),
            Self::Ended { how } => sum(&[how.heap(bounds)]),
            Self::NotWaiting | Self::NoFurther | Self::Unknown | Self::NotOffered | Self::KeyConflict => Some(0),
        }
    }
}
fields!(Turn { messages, used, spent, read_through, cut });
fields!(jig_client::contract::ArchivedTurn { task, name, cumulative, read, at, turn });
fields!(Usage { input, output, cached, cache_written });
fields!(Message { speaker, blocks });
impl Payload for Speaker {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Person | Self::Assistant | Self::Tool | Self::System => Some(0),
        }
    }
}
impl Payload for Block {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Text(value0) => sum(&[value0.heap(bounds)]),
            Self::Opaque { dialect, size } => sum(&[dialect.heap(bounds), size.heap(bounds)]),
            Self::Call(value0) => sum(&[value0.heap(bounds)]),
            Self::Result { call, summary, failed } => {
                sum(&[call.heap(bounds), summary.heap(bounds), failed.heap(bounds)])
            }
        }
    }
}
fields!(Call { number, tool, input, outcome, failed, objects, parts });
fields!(TurnLimits { messages, blocks, text });
fields!(VersionRange { first, last });
impl Payload for Subject {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Frame | Self::Conversations | Self::Deployment => Some(0),
            Self::Task { task } => sum(&[task.heap(bounds)]),
            Self::Tree { task, window } => sum(&[task.heap(bounds), window.heap(bounds)]),
            Self::Chips { tasks } => sum(&[tasks.heap(bounds)]),
            Self::Inbox { window } => sum(&[window.heap(bounds)]),
            Self::Goals { project } => sum(&[project.heap(bounds)]),
            Self::Tasks { project, filter, window } => {
                sum(&[project.heap(bounds), filter.heap(bounds), window.heap(bounds)])
            }
            Self::Run { task, turn, offset } => sum(&[task.heap(bounds), turn.heap(bounds), offset.heap(bounds)]),
        }
    }
}
impl Payload for Delivery {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Frame(value0) => sum(&[value0.heap(bounds)]),
            Self::Task(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerProject(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerProposal(value0) => sum(&[value0.heap(bounds)]),
            Self::Tree(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerTree(value0) => sum(&[value0.heap(bounds)]),
            Self::Chips(value0) => sum(&[value0.heap(bounds)]),
            Self::Inbox(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerInbox(value0) => sum(&[value0.heap(bounds)]),
            Self::Goals(value0) => sum(&[value0.heap(bounds)]),
            Self::Tasks(value0) => sum(&[value0.heap(bounds)]),
            Self::Conversations(value0) => sum(&[value0.heap(bounds)]),
            Self::Run(value0) => sum(&[value0.heap(bounds)]),
            Self::RunState(value0) => sum(&[value0.heap(bounds)]),
            Self::Deployment(value0) => sum(&[value0.heap(bounds)]),
            Self::OwnerDeployment(value0) => value0.heap(bounds),
        }
    }
}
fields!(jig_client::contract::CallIdentity { owner, sequence, retry, id });
fields!(CurrentCall { identity, tool, input, cut, began });
impl Payload for LiveCall {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Decoded(call) => call.heap(bounds),
            Self::Raw(call) => call.heap(bounds),
        }
    }
}
fields!(ProvisionalTurnName { attempt, number, retry });
fields!(RunText { attempt, began, turn, from, through, text, cut, call });
fields!(RunState { task, attempt, committed_turn, writing, call });
impl Payload for WatchEvent {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Opened | Self::Alive => Some(0),
            Self::Delivered(value0) => sum(&[value0.heap(bounds)]),
        }
    }
}
impl Payload for WatchEnd {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Closed
            | Self::Dropped
            | Self::Unknown
            | Self::NotAllowed
            | Self::Busy
            | Self::SubjectEnded
            | Self::SignedOut
            | Self::Stopping => Some(0),
        }
    }
}

fn sum(values: &[Option<u64>]) -> Option<u64> {
    values.iter().try_fold(0_u64, |total, value| total.checked_add((*value)?))
}

impl Payload for AdmissionLimit {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Words
            | Self::Identity
            | Self::Roles
            | Self::Arithmetic
            | Self::Duplicate
            | Self::Empty
            | Self::Batch
            | Self::ProjectLiveTasks
            | Self::LiveTasks
            | Self::Tree
            | Self::Depth
            | Self::Delegates
            | Self::Subscription
            | Self::Dependencies
            | Self::Cycle
            | Self::Spec
            | Self::Contract
            | Self::Authority
            | Self::Inputs
            | Self::Attempts
            | Self::LiveDelegates
            | Self::Restore
            | Self::Read
            | Self::Turn
            | Self::HoldKind
            | Self::HoldTaken
            | Self::Holds
            | Self::InboxFull
            | Self::Notes
            | Self::Revision
            | Self::Question
            | Self::Priority
            | Self::Policy
            | Self::Resource
            | Self::Service => Some(0),
        }
    }
}
impl Payload for HeldRole {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Owner | Self::Maintainer | Self::Member | Self::Observer => Some(0),
            Self::Policy { role } => role.heap(bounds),
        }
    }
}
impl Payload for DisposalIssue {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Missing | Self::Unknown | Self::Repeated => Some(0),
        }
    }
}

impl Payload for CurrentHold {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Document(reason) => reason.heap(bounds),
            Self::Reason(reason) => reason.heap(bounds),
        }
    }
}
impl Payload for HoldCategory {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Failures { .. }
            | Self::Stalled
            | Self::Dependency { .. }
            | Self::Stopped
            | Self::StoppedBy { .. }
            | Self::Drift
            | Self::Effects
            | Self::EffectFailed
            | Self::Uncertain { .. }
            | Self::Budget
            | Self::Deadline
            | Self::HoldsWaited
            | Self::ResourceUnavailable
            | Self::ResourceAccess
            | Self::PoolLost { .. }
            | Self::Procedure => Some(0),
        }
    }
}
impl Payload for HoldFailure {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Transient | Self::Permanent | Self::Run | Self::Agent | Self::Lost | Self::Invalid => Some(0),
        }
    }
}

fields!(Specification { words, parameters, inputs });
fields!(SpecificationDocument { specification, cut });
impl Payload for Parameter {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Number { name, value } => sum(&[name.heap(bounds), value.heap(bounds)]),
            Self::Bytes { name, value } => sum(&[name.heap(bounds), value.heap(bounds)]),
            Self::Resource { name, connector, resource } => {
                sum(&[name.heap(bounds), connector.heap(bounds), resource.heap(bounds)])
            }
        }
    }
}

fields!(DocumentText { bytes, cut });
fields!(DocumentSpecification { words, parameters, inputs });
impl Payload for ParameterDocument {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Number { name, value } => sum(&[name.heap(bounds), value.heap(bounds)]),
            Self::Bytes { name, value } => sum(&[name.heap(bounds), value.heap(bounds)]),
            Self::Resource { name, connector, resource } => {
                sum(&[name.heap(bounds), connector.heap(bounds), resource.heap(bounds)])
            }
        }
    }
}

pub mod owner_outcome;

mod owner_kind_order;

fields!(jig_client::contract::OwnerPhaseCounts {
    waiting,
    idle,
    starting,
    running,
    backing_off,
    closing,
    held,
    done,
    failed,
    cancelled
});

#[cfg(test)]
mod question;

impl Payload for TaskHold {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Document(value) => value.heap(bounds),
            Self::Owner(value) => value.heap(bounds),
        }
    }
}

fields!(jig_client::contract::OwnerEscalation { task; acts });
