//! Immutable semantic history (client/domain/objects.md, 11; domain/tasks.md, 7;
//! view.md, 7 and 10). Stored overrides are not historical effective defaults.
use crate::JigBindings;
use crate::owner_history_words::HistoryPhrase as P;
use crate::words::Words;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use crate::Limits;
use jig_client::contract::{
    DocumentText, OwnerHistoryChange, OwnerHistoryHolder, OwnerHistoryLine, OwnerMessageProposalKind,
    OwnerProposalHistory, OwnerProposalHistoryState, OwnerRetainedAction, OwnerRetainedAmendment, ParameterDocument,
    PersonAddress, ProposedTask, SpecificationDocument, TaskContractDocument,
};

/// Checked storage for the whole history and every nested retained fragment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryFragment {
    pub nodes: u32,
    pub text_bytes: u32,
    pub text_per_node: u32,
    pub depth: u32,
    pub bindings: u32,
    pub slots: u32,
}
fn text_fits(value: &DocumentText, limits: &Limits) -> Option<()> {
    if value.bytes.fits(limits.text) { Some(()) } else { None }
}
fn spec_fits(value: &SpecificationDocument, limits: &Limits) -> Option<()> {
    let value = &value.specification;
    text_fits(&value.words, limits)?;
    if value.parameters.capacity() > limits.rows || value.inputs.capacity() > limits.rows {
        return None;
    }
    for parameter in &value.parameters {
        match parameter {
            ParameterDocument::Bytes { value, .. } => text_fits(value, limits)?,
            ParameterDocument::Number { .. } | ParameterDocument::Resource { .. } => {}
        }
    }
    Some(())
}
fn contract_fits(value: &TaskContractDocument, limits: &Limits) -> Option<()> {
    match value {
        TaskContractDocument::Approval { state, part } => {
            if u32::try_from(state.len()).ok()? > limits.text {
                return None;
            }
            crate::owner_chip::part_fits(part, limits)?;
        }
        TaskContractDocument::Verdict { choices, .. } => {
            if choices.capacity() > limits.rows {
                return None;
            }
            for choice in choices {
                if choice.fields.capacity() > limits.rows {
                    return None;
                }
                for field in &choice.fields {
                    if u32::try_from(field.name.len()).ok()? > limits.text {
                        return None;
                    }
                }
            }
        }
        TaskContractDocument::Report { .. } | TaskContractDocument::Change { .. } => {}
    }
    Some(())
}
pub(crate) fn proposed_fits(value: &ProposedTask, limits: &Limits) -> Option<()> {
    spec_fits(&value.spec, limits)?;
    contract_fits(&value.contract, limits)?;
    crate::owner_amendment::authority_fits(&value.authority, limits)?;
    if value.resources.capacity() > limits.rows || value.dependencies.capacity() > limits.rows {
        return None;
    }
    for part in &value.resources {
        crate::owner_chip::part_fits(part, limits)?;
    }
    Some(())
}
fn amendment_fits(value: &OwnerRetainedAmendment, limits: &Limits) -> Option<()> {
    if let Some(instructions) = &value.instructions {
        text_fits(instructions, limits)?;
    }
    if let Some(spec) = &value.spec {
        spec_fits(spec, limits)?;
    }
    if let Some(authority) = &value.authority {
        crate::owner_amendment::authority_fits(authority, limits)?;
    }
    if let Some(dependencies) = &value.dependencies
        && dependencies.capacity() > limits.rows
    {
        return None;
    }
    text_fits(&value.reason, limits)
}
fn proposal_fits(proposal: &OwnerProposalHistory, limits: &Limits) -> Option<()> {
    match &proposal.state {
        OwnerProposalHistoryState::Rejected { reason, .. } => text_fits(reason, limits)?,
        OwnerProposalHistoryState::Pending { .. }
        | OwnerProposalHistoryState::Accepted { .. }
        | OwnerProposalHistoryState::Withdrawn => {}
    }
    match &proposal.action {
        OwnerRetainedAction::Batch(tasks) => {
            if tasks.capacity() > limits.rows {
                return None;
            }
            for task in tasks {
                proposed_fits(task, limits)?;
            }
        }
        OwnerRetainedAction::Effect { part, authority, .. }
        | OwnerRetainedAction::ProcedureEffect { part, authority, .. } => {
            crate::owner_chip::part_fits(part, limits)?;
            crate::owner_amendment::authority_fits(authority, limits)?;
        }
        OwnerRetainedAction::Amend { change, .. } => amendment_fits(change, limits)?,
        OwnerRetainedAction::Widen { authority, .. } => crate::owner_amendment::authority_fits(authority, limits)?,
        OwnerRetainedAction::Release { .. } => {}
    }
    Some(())
}
/// Validate every supplied nested text and allocated capacity before rendering.
#[must_use]
pub fn required(value: &OwnerHistoryLine, limits: &Limits, words: &Words) -> Option<HistoryFragment> {
    text_fits(&value.reason, limits)?;
    if let Some((before, after)) = &value.amendment {
        crate::owner_amendment::required(before, limits, words)?;
        crate::owner_amendment::required(after, limits, words)?;
    }
    if let Some(proposal) = &value.proposal {
        proposal_fits(proposal, limits)?;
    }
    worst_case(limits, words)
}
/// Validate a retained proposal without requiring a surrounding history line.
#[must_use]
pub fn proposal_required(proposal: &OwnerProposalHistory, limits: &Limits, words: &Words) -> Option<HistoryFragment> {
    proposal_fits(proposal, limits)?;
    proposal_worst_case(limits, words)
}
/// Conservative retained-proposal bound, including the whole history envelope's spare allowance.
#[must_use]
pub fn proposal_max_nodes(limits: &Limits) -> Option<u32> {
    max_nodes(limits)
}
/// Conservative whole-history storage envelope for the passive proposal subset.
#[must_use]
pub fn proposal_worst_case(limits: &Limits, words: &Words) -> Option<HistoryFragment> {
    worst_case(limits, words)
}
/// Finite whole-node envelope reused by bounded history pages.
#[must_use]
pub fn max_nodes(limits: &Limits) -> Option<u32> {
    let amendment = crate::owner_amendment::max_nodes(limits)?;
    let authority = crate::owner_authority::document_nodes(limits.rows, limits.segments)?;
    let batch = limits.rows.checked_mul(crate::sizes::proposed_task_nodes(limits)?)?;
    let delta = crate::specification::nodes(limits)?
        .checked_add(authority)?
        .checked_add(skein_ui::markdown::max_nodes(limits.text)?.checked_mul(2)?)?
        .checked_add(limits.rows)?
        .checked_add(crate::owner_wake::policy_nodes())?
        .checked_add(30)?;
    let effect = crate::proposed_task::part_nodes(limits)?.checked_add(authority)?.checked_add(8)?;
    100_u32.checked_add(amendment.checked_mul(2)?)?.checked_add(batch.max(delta).max(effect))
}
/// Finite nested envelope; raw reference bindings never acquire an application scope.
#[must_use]
pub fn worst_case(limits: &Limits, words: &Words) -> Option<HistoryFragment> {
    let nodes = max_nodes(limits)?;
    let phrase = crate::phrases::jig_capacity(&words.book)?;
    let text_per_node = crate::literal_segment::max_bytes(limits.text)?.max(phrase).checked_mul(2)?;
    let bindings = limits.rows.checked_mul(limits.rows.checked_add(1)?)?.max(1);
    Some(HistoryFragment {
        nodes,
        text_bytes: nodes.checked_mul(text_per_node)?,
        text_per_node,
        depth: 16,
        bindings,
        slots: bindings,
    })
}
fn saying(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    let value =
        words.book.say(words.language, phrase.id(), holes).expect("checked immutable history sayings and holes");
    b.text(Run::Plain, &value.text);
}
fn change(b: &mut Builder, value: OwnerHistoryChange, words: &Words) {
    let phrase = match value {
        OwnerHistoryChange::Cancelled => P::Cancelled,
        OwnerHistoryChange::Released => P::Released,
        OwnerHistoryChange::Amended => P::Amended,
        OwnerHistoryChange::Moved => P::Moved,
        OwnerHistoryChange::Proposed => P::Proposed,
        OwnerHistoryChange::ProposalPassed => P::Passed,
        OwnerHistoryChange::ProposalAccepted => P::Accepted,
        OwnerHistoryChange::ProposalRejected => P::Rejected,
        OwnerHistoryChange::ProposalWithdrawn => P::Withdrawn,
    };
    saying(b, words, phrase, &[]);
}
pub(crate) fn holder(b: &mut Builder, value: OwnerHistoryHolder, words: &Words) {
    match value {
        OwnerHistoryHolder::Task(number) => saying(b, words, P::HolderTask, &[Hole::Number(number)]),
        OwnerHistoryHolder::Person(number) => saying(b, words, P::HolderPerson, &[Hole::Number(number)]),
        OwnerHistoryHolder::Policy { project, kind } => {
            saying(b, words, P::HolderPolicy, &[Hole::Number(u64::from(project))]);
            let phrase = match kind {
                OwnerMessageProposalKind::Batch => P::Batch,
                OwnerMessageProposalKind::Effect => P::Effect,
                OwnerMessageProposalKind::Amend => P::Amend,
                OwnerMessageProposalKind::Widen => P::Widen,
                OwnerMessageProposalKind::Release => P::Release,
            };
            saying(b, words, phrase, &[]);
        }
    }
}
fn retained_amendment(b: &mut Builder, value: &OwnerRetainedAmendment, words: &Words, text_limit: u32) {
    match value.addressed {
        Some(address) => {
            saying(b, words, P::AddressRevision, &[Hole::Number(address.revision.value)]);
            match address.address {
                PersonAddress::Party(number) => saying(b, words, P::Party, &[Hole::Number(number)]),
                PersonAddress::Role(number) => saying(b, words, P::Role, &[Hole::Number(u64::from(number))]),
            }
        }
        None => saying(b, words, P::AddressAbsent, &[]),
    }
    match &value.instructions {
        Some(value) => {
            saying(b, words, P::Instructions, &[]);
            crate::instructions::document(b, Some(value), words);
        }
        None => saying(b, words, P::InstructionsAbsent, &[]),
    }
    match value.release {
        Some(revision) => saying(b, words, P::HoldRevision, &[Hole::Number(revision)]),
        None => saying(b, words, P::ReleaseAbsent, &[]),
    }
    match &value.spec {
        Some(value) => {
            saying(b, words, P::Spec, &[]);
            crate::specification::document(b, value, words);
        }
        None => saying(b, words, P::SpecAbsent, &[]),
    }
    match &value.authority {
        Some(value) => {
            saying(b, words, P::Authority, &[]);
            crate::owner_authority::document_words(b, value, words, text_limit);
        }
        None => saying(b, words, P::AuthorityAbsent, &[]),
    }
    match &value.wake {
        Some(value) => {
            saying(b, words, P::Wake, &[]);
            crate::owner_wake::policy(b, value, &words.book, words.language);
        }
        None => saying(b, words, P::WakeAbsent, &[]),
    }
    match &value.dependencies {
        Some(values) => {
            saying(b, words, P::Dependencies, &[]);
            for value in values {
                saying(b, words, P::Dependency, &[Hole::Number(*value)]);
            }
        }
        None => saying(b, words, P::DependenciesAbsent, &[]),
    }
    if value.cut {
        saying(b, words, P::DependenciesCut, &[]);
    }
    saying(b, words, P::Reason, &[]);
    crate::document_text::markdown(b, &value.reason, words);
}
fn action(b: &mut Builder, value: &OwnerRetainedAction, words: &Words, bindings: &mut JigBindings) {
    let text_limit = bindings.text_limit();
    match value {
        OwnerRetainedAction::Batch(tasks) => {
            saying(b, words, P::Batch, &[]);
            for task in tasks {
                crate::cards::proposal::proposed_task(b, task, words, bindings);
            }
        }
        OwnerRetainedAction::Effect { part, authority, attempt, completion, position, purpose } => {
            saying(b, words, P::Effect, &[]);
            crate::proposed_task::part(b, part, words, bindings);
            crate::owner_authority::document_words(b, authority, words, text_limit);
            for (phrase, number) in [
                (P::Attempt, *attempt),
                (P::Completion, u64::from(*completion)),
                (P::Position, u64::from(*position)),
                (P::Purpose, *purpose),
            ] {
                saying(b, words, phrase, &[Hole::Number(number)]);
            }
        }
        OwnerRetainedAction::ProcedureEffect { part, authority, step, purpose } => {
            saying(b, words, P::Effect, &[]);
            crate::proposed_task::part(b, part, words, bindings);
            crate::owner_authority::document_words(b, authority, words, text_limit);
            saying(b, words, P::ProcedureStep, &[Hole::Number(*step)]);
            saying(b, words, P::Purpose, &[Hole::Number(*purpose)]);
        }
        OwnerRetainedAction::Amend { task, change } => {
            saying(b, words, P::Amend, &[]);
            saying(b, words, P::Task, &[Hole::Number(*task)]);
            retained_amendment(b, change, words, text_limit);
        }
        OwnerRetainedAction::Widen { task, authority } => {
            saying(b, words, P::Widen, &[]);
            saying(b, words, P::Task, &[Hole::Number(*task)]);
            crate::owner_authority::document_words(b, authority, words, text_limit);
        }
        OwnerRetainedAction::Release { task } => {
            saying(b, words, P::Release, &[]);
            saying(b, words, P::Task, &[Hole::Number(*task)]);
        }
    }
}
/// Render only supplied immutable facts, borrowing the normal checked language.
pub fn render(b: &mut Builder, value: &OwnerHistoryLine, words: &Words, bindings: &mut JigBindings) {
    let text_limit = bindings.text_limit();
    b.open(Role::Region);
    let name = words.book.say(words.language, P::History.id(), &[]).expect("history heading checked");
    b.name(&name.text);
    saying(b, words, P::Task, &[Hole::Number(value.task)]);
    saying(b, words, P::Revision, &[Hole::Number(value.revision)]);
    crate::owner_result::requester(b, value.by, words);
    change(b, value.change, words);
    saying(b, words, P::Reason, &[]);
    crate::document_text::markdown(b, &value.reason, words);
    match value.at {
        Some(at) => saying(b, words, P::At, &[Hole::Wall(words.adjusted_wall(at))]),
        None => saying(b, words, P::AtAbsent, &[]),
    }
    match &value.amendment {
        Some((before, after)) => {
            saying(b, words, P::Stored, &[]);
            saying(b, words, P::Before, &[]);
            crate::owner_amendment::render(b, before, words, text_limit);
            saying(b, words, P::After, &[]);
            crate::owner_amendment::render(b, after, words, text_limit);
        }
        None => saying(b, words, P::SnapshotsAbsent, &[]),
    }
    if let Some(proposal) = &value.proposal {
        self::proposal(b, proposal, words, bindings);
    }
    b.close();
}

/// Render supplied retained proposal facts inline, without current controls or outer history metadata.
pub fn proposal(b: &mut Builder, proposal: &OwnerProposalHistory, words: &Words, bindings: &mut JigBindings) {
    saying(b, words, P::Proposal, &[Hole::Number(proposal.number)]);
    saying(b, words, P::Proposer, &[Hole::Number(proposal.proposer)]);
    saying(b, words, if proposal.as_holder { P::AsHolder } else { P::NotAsHolder }, &[]);
    action(b, &proposal.action, words, bindings);
    match &proposal.state {
        OwnerProposalHistoryState::Pending { holder: value, since } => {
            saying(b, words, P::Pending, &[]);
            holder(b, *value, words);
            saying(b, words, P::Since, &[Hole::Wall(words.adjusted_wall(*since))]);
        }
        OwnerProposalHistoryState::Accepted { by } => {
            saying(b, words, P::Accepted, &[]);
            crate::owner_result::requester(b, *by, words);
        }
        OwnerProposalHistoryState::Rejected { by, reason } => {
            saying(b, words, P::Rejected, &[]);
            crate::owner_result::requester(b, *by, words);
            crate::document_text::markdown(b, reason, words);
        }
        OwnerProposalHistoryState::Withdrawn => saying(b, words, P::Withdrawn, &[]),
    }
}
