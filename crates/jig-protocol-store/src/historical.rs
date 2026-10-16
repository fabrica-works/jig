//! The core's four archived reads translated by the kit (protocol/store.md, 3).
use alloc::boxed::Box;
use jig_domain_core::{self as core, CoreKey, CoreRecord, Key, Now, Record};
use jig_domain_tasks as tasks;

#[must_use]
pub fn key(now: &Now) -> Option<Key> {
    match now {
        Now::LoadEndedChat { task } | Now::SeenChat { task, .. } | Now::HistoricalPersonTask { task, .. } => {
            Some(Key::Tasks(tasks::Key::Ended(*task)))
        }
        Now::HistoricalProposal { proposer, proposal, .. } => {
            Some(Key::Core(CoreKey::ProposalDecision { proposer: *proposer, proposal: *proposal }))
        }
        Now::HistoricalEscalation { task, revision, .. } => {
            Some(Key::Core(CoreKey::EscalationDecision { task: *task, revision: *revision }))
        }
        Now::CallBusy { .. }
        | Now::Relayed { .. }
        | Now::EffectAnswer { .. }
        | Now::SettledCallRefused { .. }
        | Now::Call { .. }
        | Now::DropCall { .. }
        | Now::Undelivered { .. }
        | Now::SignInRefused { .. }
        | Now::WatchRefused { .. }
        | Now::Account(_)
        | Now::RunText { .. }
        | Now::NotesIndexed { .. }
        | Now::NotesRecalled { .. }
        | Now::NotesRefused { .. }
        | Now::NoteBusy { .. }
        | Now::DropPayload { .. }
        | Now::DropAssignment { .. }
        | Now::TurnPayload { .. }
        | Now::AnswerPayload { .. }
        | Now::AcceptedTurn { .. }
        | Now::RefusedPayload { .. }
        | Now::Activate { .. }
        | Now::PrepareAgent { .. }
        | Now::StartPreparation { .. }
        | Now::CompleteBrief { .. }
        | Now::BriefCorePlanned { .. }
        | Now::WorkspaceRequest { .. }
        | Now::RunPreparationFailed { .. }
        | Now::RunPrepared { .. }
        | Now::ProcedureDelegateOutcome { .. }
        | Now::RestoreRefused => None,
    }
}

#[must_use]
pub fn answer(now: Now, row: Option<Record>) -> Option<core::Event> {
    Some(match now {
        Now::LoadEndedChat { .. } => core::Event::ForgetChat { row: ended(row)? },
        Now::SeenChat { request, .. } => core::Event::SeenChatLoaded { request, row: ended(row), busy: false },
        Now::HistoricalPersonTask { request, person, project, task } => {
            core::Event::HistoricalPersonTask { request, person, project, task, row: ended(row), busy: false }
        }
        Now::HistoricalProposal { request, person, project, proposer, proposal } => core::Event::HistoricalProposal {
            request,
            person,
            project,
            proposer,
            proposal,
            row: proposal_row(row),
            busy: false,
        },
        Now::HistoricalEscalation { request, person, project, task, revision } => core::Event::HistoricalEscalation {
            request,
            person,
            project,
            task,
            revision,
            row: escalation_row(row),
            busy: false,
        },
        Now::CallBusy { .. }
        | Now::Relayed { .. }
        | Now::EffectAnswer { .. }
        | Now::SettledCallRefused { .. }
        | Now::Call { .. }
        | Now::DropCall { .. }
        | Now::Undelivered { .. }
        | Now::SignInRefused { .. }
        | Now::WatchRefused { .. }
        | Now::Account(_)
        | Now::RunText { .. }
        | Now::NotesIndexed { .. }
        | Now::NotesRecalled { .. }
        | Now::NotesRefused { .. }
        | Now::NoteBusy { .. }
        | Now::DropPayload { .. }
        | Now::DropAssignment { .. }
        | Now::TurnPayload { .. }
        | Now::AnswerPayload { .. }
        | Now::AcceptedTurn { .. }
        | Now::RefusedPayload { .. }
        | Now::Activate { .. }
        | Now::PrepareAgent { .. }
        | Now::StartPreparation { .. }
        | Now::CompleteBrief { .. }
        | Now::BriefCorePlanned { .. }
        | Now::WorkspaceRequest { .. }
        | Now::RunPreparationFailed { .. }
        | Now::RunPrepared { .. }
        | Now::ProcedureDelegateOutcome { .. }
        | Now::RestoreRefused => return None,
    })
}

fn ended(row: Option<Record>) -> Option<Box<tasks::TaskRecord>> {
    match row? {
        Record::Tasks(tasks::Stored::Ended(row)) => Some(row),
        Record::Tasks(
            tasks::Stored::Live(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Pool(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::ClosedLedger(_)
            | tasks::Stored::DecidedProposal(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::Activity(_)
            | tasks::Stored::Update(_)
            | tasks::Stored::Result(_)
            | tasks::Stored::ConversationNewest(_)
            | tasks::Stored::ConversationOldest(_)
            | tasks::Stored::Goal(_)
            | tasks::Stored::EndedByParty(_)
            | tasks::Stored::EndedByProject(_)
            | tasks::Stored::EndedDelegate(_)
            | tasks::Stored::Delegate(_),
        )
        | Record::Core(_)
        | Record::People(_)
        | Record::Notes(_) => None,
    }
}

fn proposal_row(row: Option<Record>) -> Option<core::ProposalDecisionRecord> {
    match row? {
        Record::Core(CoreRecord::ProposalDecision(row)) => Some(row),
        Record::Core(
            CoreRecord::Deployment(_)
            | CoreRecord::Projection(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Call(_)
            | CoreRecord::Turn(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::EscalationDecision(_),
        )
        | Record::Tasks(_)
        | Record::People(_)
        | Record::Notes(_) => None,
    }
}

fn escalation_row(row: Option<Record>) -> Option<core::EscalationDecisionRecord> {
    match row? {
        Record::Core(CoreRecord::EscalationDecision(row)) => Some(row),
        Record::Core(
            CoreRecord::Deployment(_)
            | CoreRecord::Projection(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Call(_)
            | CoreRecord::Turn(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::ProposalDecision(_),
        )
        | Record::Tasks(_)
        | Record::People(_)
        | Record::Notes(_) => None,
    }
}
