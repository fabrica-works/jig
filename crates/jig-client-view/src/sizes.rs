//! Renderer envelopes (client/domain/view.md, section 10).
//! Ordinary incoming lists, including nested lists, fit `rows` (contract.md, 5.3),
//! offered acts fit their closed vocabulary ceiling, and all text fits `text`.
//! The protocol also checks whole document bytes.
//! These counts deliberately do not spend that independent byte budget: full
//! lists and full texts together form an envelope even when admission would
//! reject their combination. Counts include loading slot fallbacks, but not
//! the enclosing page or the connector fragments that replace those fallbacks.
use crate::Limits;

fn markdown(limits: &Limits) -> Option<u32> {
    skein_ui::markdown::max_nodes(limits.text)
}
fn authority(limits: &Limits) -> Option<u32> {
    // Four scalars; each row may occur in kinds, grants (two slots), tools,
    // and note scopes (one slot).
    limits.rows.checked_mul(limits.segments.checked_mul(2)?.checked_add(14)?)?.checked_add(4)
}
fn trail(limits: &Limits) -> Option<u32> {
    // Introduction, actor, time, reason and a whole lacking-authority explanation.
    limits.rows.checked_mul(limits.segments.checked_add(9)?)
}
fn outcome(limits: &Limits) -> Option<u32> {
    // A moved hold includes its complete bounded attempt failures or exact
    // connector part identity; endings retain only supplied actor/decision facts.
    Some(limits.rows.checked_mul(limits.rows.checked_add(9)?)?.checked_add(12)?.max(limits.segments.checked_add(12)?))
}
/// Whole closed offer controls or refusals, independent of paged rows.
#[must_use]
pub fn offered_nodes(limits: &Limits) -> Option<u32> {
    // Each offer is a two-node control or its label plus a whole refusal.
    jig_client::contract::MAX_OFFERED_ACTS.checked_mul(outcome(limits)?.checked_add(1)?.max(2))
}
/// One chip, including each retained dependency at most once for this call.
#[must_use]
pub fn chip_nodes(limits: &Limits) -> Option<u32> {
    // A held prior closing can combine full failure rows and connector waits.
    // Observed activation includes only supplied work, timing and retry facts;
    // the bound also includes every optional pending-ending explanation.
    limits.rows.checked_mul(8)?.checked_add(26)?.checked_mul(limits.objects.checked_add(1)?)
}
fn frame_of_card(limits: &Limits) -> Option<u32> {
    // Item, named region, heading, project, state and answer; full inbox
    // lineage is additional chips. The page's other cards are not here.
    chip_nodes(limits)?
        .checked_mul(limits.rows)?
        .checked_add(markdown(limits)?.checked_add(6)?)?
        .checked_add(outcome(limits)?.checked_mul(2)?)?
        .checked_add(offered_nodes(limits)?)?
        .checked_add(13)
}
fn amendment(limits: &Limits) -> Option<u32> {
    // Recipient, spec, instructions, wake, dependencies, authority, separator.
    crate::specification::nodes(limits)?
        .checked_add(crate::instructions::nodes(limits)?)?
        .checked_add(crate::owner_wake::policy_nodes())?
        .checked_add(crate::owner_authority::document_nodes(limits.rows, limits.segments)?)?
        .checked_add(limits.rows.checked_mul(5)?)?
        .checked_add(5)
}
/// A whole proposed task, reusable by the recurring-template owner.
pub(crate) fn proposed_task_nodes(limits: &Limits) -> Option<u32> {
    crate::specification::nodes(limits)?
        .checked_add(crate::proposed_task::contract_nodes(limits)?)?
        .checked_add(crate::owner_authority::document_nodes(limits.rows, limits.segments)?)?
        .checked_add(crate::owner_wake::policy_nodes())?
        .checked_add(limits.rows)?
        .checked_add(crate::proposed_task::part_nodes(limits)?.checked_mul(limits.rows)?)?
        .checked_add(7)
}
fn proposal_action(limits: &Limits) -> Option<u32> {
    let batch = proposed_task_nodes(limits)?.checked_mul(limits.rows)?.checked_add(3)?;
    let widen = authority(limits)?.checked_mul(2)?.checked_add(3)?;
    let amend = amendment(limits)?.checked_add(markdown(limits)?)?.checked_add(3)?;
    Some(batch.max(widen).max(amend).max(3))
}
// The retained-result renderer projects only the original Proposal document.
fn proposal_document_nodes(limits: &Limits) -> Option<u32> {
    frame_of_card(limits)?
        .checked_add(markdown(limits)?)?
        .checked_add(proposal_action(limits)?)?
        .checked_add(authority(limits)?)?
        .checked_add(trail(limits)?)?
        .checked_add(outcome(limits)?)?
        .checked_add(limits.rows)?
        .checked_add(8)
}
/// Whole proposal card, including either exact document and shared context.
#[must_use]
pub fn proposal_nodes(limits: &Limits) -> Option<u32> {
    Some(
        proposal_document_nodes(limits)?
            .max(frame_of_card(limits)?.checked_add(crate::owner_proposal::max_nodes(limits)?)?),
    )
}
fn result_value(limits: &Limits) -> Option<u32> {
    // Verdict contains explanation words, each named field, follow-up chips and a retained proposal.
    let verdict = markdown(limits)?
        .checked_add(1)?
        .checked_add(chip_nodes(limits)?)?
        .checked_mul(limits.rows)?
        .checked_add(proposal_document_nodes(limits)?)?
        .checked_add(markdown(limits)?)?
        .checked_add(2)?;
    let change = markdown(limits)?.checked_add(crate::proposed_task::part_nodes(limits)?)?.checked_add(2)?;
    Some(verdict.max(markdown(limits)?.checked_add(3)?).max(change))
}
pub(crate) fn result_facts(limits: &Limits) -> Option<u32> {
    chip_nodes(limits)?
        .checked_add(result_value(limits)?)?
        .checked_add(crate::owner_result::nodes(limits)?)?
        .checked_add(10)
}
/// Whole result, including follow-up chips and a kept verdict proposal.
#[must_use]
pub fn result_nodes(limits: &Limits) -> Option<u32> {
    frame_of_card(limits)?.checked_add(result_facts(limits)?)
}
fn person_contract(limits: &Limits) -> Option<u32> {
    // Choice group, full named choices, cannot-do button and reason field;
    // approval's slot and words' field are smaller alternatives.
    limits.rows.checked_mul(2)?.checked_add(5)
}
/// Whole person task with every passive input result and active contract.
#[must_use]
pub fn person_task_nodes(limits: &Limits) -> Option<u32> {
    result_facts(limits)?
        .checked_add(1)?
        .checked_mul(limits.rows)?
        .checked_add(frame_of_card(limits)?)?
        .checked_add(chip_nodes(limits)?)?
        .checked_add(crate::specification::nodes(limits)?)?
        .checked_add(person_contract(limits)?)?
        .checked_add(outcome(limits)?)?
        .checked_add(limits.rows)?
        .checked_add(9)
}
/// Whole question, including the answer field and retained answer.
#[must_use]
pub fn question_nodes(limits: &Limits) -> Option<u32> {
    frame_of_card(limits)?.checked_add(chip_nodes(limits)?)?.checked_add(markdown(limits)?)?.checked_add(6)
}
/// Whole escalation with failed attempts, prior result, spend and trail.
#[must_use]
pub fn escalation_nodes(limits: &Limits) -> Option<u32> {
    frame_of_card(limits)?
        .checked_add(chip_nodes(limits)?)?
        .checked_add(result_value(limits)?)?
        .checked_add(limits.rows.checked_mul(5)?)?
        .checked_add(trail(limits)?)?
        .checked_add(outcome(limits)?)?
        .checked_add(limits.rows)?
        .checked_add(12)
}
/// Unread reply, preserving the whole task's chip and first words.
#[must_use]
pub fn reply_nodes(limits: &Limits) -> Option<u32> {
    frame_of_card(limits)?.checked_add(chip_nodes(limits)?)?.checked_add(3)
}
/// A refusal region without a current object.
#[must_use]
pub fn refusal_nodes(limits: &Limits) -> Option<u32> {
    outcome(limits)?.checked_add(1)
}
/// Largest object card, including task, note and loading alternatives.
#[must_use]
pub fn card_nodes(limits: &Limits) -> Option<u32> {
    let note = markdown(limits)?.checked_add(limits.rows.checked_mul(2)?)?.checked_add(6)?;
    let simple = frame_of_card(limits)?.checked_add(chip_nodes(limits)?.max(note))?;
    Some(
        simple
            .max(proposal_nodes(limits)?)
            .max(question_nodes(limits)?)
            .max(person_task_nodes(limits)?)
            .max(frame_of_card(limits)?.checked_add(crate::owner_person_task::max_nodes(limits)?)?)
            .max(result_nodes(limits)?)
            .max(reply_nodes(limits)?)
            .max(escalation_nodes(limits)?)
            .max(owner_escalation_nodes(limits)?)
            .max(frame_of_card(limits)?.checked_add(crate::owner_historical::max_nodes(limits)?)?)
            .max(refusal_nodes(limits)?)
            .max(
                frame_of_card(limits)?
                    .checked_add(chip_nodes(limits)?)?
                    .checked_add(crate::task_result::nodes(limits)?)?,
            ),
    )
}
/// Connection region, including a behind status.
#[must_use]
pub fn link_nodes(_limits: &Limits) -> Option<u32> {
    Some(5)
}
/// Needs, updates, exact counts and charter-list loss facts.
#[must_use]
pub fn needs_nodes(limits: &Limits) -> Option<u32> {
    limits.projects.max(limits.rows).checked_add(8)
}
/// Signed-in identity, independent text losses and sign out.
#[must_use]
pub fn party_nodes(_limits: &Limits) -> Option<u32> {
    Some(9)
}
/// Membership buttons, names, roles and whole retained ledgers.
#[must_use]
pub fn projects_nodes(limits: &Limits) -> Option<u32> {
    limits.projects.max(limits.rows).checked_mul(18)?.checked_add(1)
}
/// Task-number field in its region.
#[must_use]
pub fn go_to_task_nodes(_limits: &Limits) -> Option<u32> {
    Some(2)
}
/// Navigation links and the application's slot.
#[must_use]
pub fn navigation_nodes(limits: &Limits) -> Option<u32> {
    limits.navigation_entries.checked_mul(2)?.checked_add(3)
}
/// The banner landmark and its five parts.
#[must_use]
pub fn banner_nodes(limits: &Limits) -> Option<u32> {
    link_nodes(limits)?
        .checked_add(needs_nodes(limits)?)?
        .checked_add(party_nodes(limits)?)?
        .checked_add(projects_nodes(limits)?)?
        .checked_add(go_to_task_nodes(limits)?)?
        .checked_add(1)
}
/// One object or goal dialog, with every delegate and amendment field.
#[must_use]
pub fn confirmation_nodes(limits: &Limits) -> Option<u32> {
    let form = amendment(limits)?.checked_mul(2)?.checked_add(limits.rows.checked_mul(3)?)?.checked_add(15)?;
    let delegates = chip_nodes(limits)?.checked_add(4)?.checked_mul(limits.delegates.max(limits.rows))?;
    // The closed act table takes one alternative; using their sum also covers
    // live context, missing/changed statuses, impact, reason and review buttons.
    chip_nodes(limits)?
        .checked_add(
            form.max(proposal_action(limits)?)
                .max(crate::owner_proposal::max_nodes(limits)?)
                .max(person_contract(limits)?)
                .max(crate::owner_person_task::form_nodes(limits)?)
                .max(markdown(limits)?),
        )?
        .checked_add(delegates)?
        .checked_add(crate::owner_chip::max_nodes(limits)?)?
        .checked_add(limits.rows)?
        .checked_add(67)
}
/// Full visible notices and the oldest notice's dismissal control.
#[must_use]
pub fn notices_nodes(limits: &Limits) -> Option<u32> {
    outcome(limits)?.checked_add(4)?.checked_mul(limits.notices)?.checked_add(3)
}

mod pages;
pub use pages::{
    arranged_nodes, conversation_nodes, conversations_nodes, deployment_nodes, inbox_nodes, not_the_work_nodes,
    page_nodes, project_header_nodes, project_nodes, project_section_nodes, task_header_nodes, task_nodes,
    task_section_nodes,
};

/// Whole held-task escalation, including bounded input-result and person cards.
#[must_use]
pub fn owner_escalation_nodes(l: &Limits) -> Option<u32> {
    owner_escalation::nodes(l)
}

mod owner_escalation;
