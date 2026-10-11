//! Held-task card projection (client/domain/view.md, section 10).
//! Typed occupied object keys share one residual Task-chip budget. Repeated
//! references still render repeatedly; embedded Person results remain whole.
use super::{Limits, chip_nodes, frame_of_card, markdown};

fn value(l: &Limits, proposals: u32) -> Option<u32> {
    // Kept legacy proposals appear in Conversation, never with Inbox lineage.
    let proposal = if proposals == 0 {
        0
    } else {
        super::proposal_document_nodes(l)?.checked_sub(chip_nodes(l)?.checked_mul(l.rows)?)?
    };
    let verdict = markdown(l)?
        .checked_add(1)?
        .checked_add(chip_nodes(l)?)?
        .checked_mul(l.rows)?
        .checked_add(proposal)?
        .checked_add(markdown(l)?)?
        .checked_add(2)?;
    let change = markdown(l)?.checked_add(crate::proposed_task::part_nodes(l)?)?.checked_add(2)?;
    Some(verdict.max(markdown(l)?.checked_add(3)?).max(change))
}

fn person(l: &Limits, facts: u32, frame: u32) -> Option<u32> {
    // Both source contracts retain all embedded results, links and controls.
    let old_facts = super::result_facts(l)?.checked_mul(l.rows)?;
    let new_facts = facts.checked_mul(l.rows)?;
    let original = super::person_task_nodes(l)?
        .checked_sub(chip_nodes(l)?.checked_mul(l.rows)?)?
        .checked_sub(old_facts)?
        .checked_add(new_facts)?;
    let owner = crate::owner_person_task::max_nodes(l)?
        .checked_sub(old_facts)?
        .checked_add(new_facts)?
        .checked_add(frame)?
        .checked_add(crate::owner_person_task::controls_nodes(l)?)?;
    Some(original.max(owner))
}

fn case(l: &Limits, results: u32, persons: u32, proposals: u32) -> Option<u32> {
    // The escalation itself and each present distinct typed key cannot also
    // be a stored Task. Other non-Task keys only reduce the actual capacity.
    let occupied = 1_u32.checked_add(results)?.checked_add(persons)?.checked_add(proposals)?;
    let Some(objects) = l.objects.checked_sub(occupied) else { return Some(0) };
    let l = &Limits { objects, ..*l };
    let value = value(l, proposals)?;
    let ending = value.checked_add(markdown(l)?)?.checked_add(8)?;
    let facts = chip_nodes(l)?.checked_add(value)?.checked_add(crate::owner_result::nodes(l)?)?.checked_add(10)?;
    // Nested Result and Person cards use TaskPage, without Inbox lineage.
    // Their whole frames also cover absent-body loading alternatives.
    let frame = frame_of_card(l)?.checked_sub(chip_nodes(l)?.checked_mul(l.rows)?)?;
    let result = if results == 0 { 0 } else { frame.checked_add(facts)?.checked_mul(l.rows)? };
    let person = if persons == 0 { 0 } else { person(l, facts, frame)? };
    // Owner-chip and Task endings are passive values/facts, without a Result
    // card's outer acts. Keep all other supplied metadata and ending reasons.
    let owner_chip = crate::owner_chip::max_nodes(l)?.checked_sub(super::result_nodes(l)?)?.checked_add(ending)?;
    frame_of_card(l)?
        .checked_add(chip_nodes(l)?)?
        .checked_add(owner_chip)?
        .checked_add(l.rows.checked_add(20)?.checked_mul(l.rows)?)?
        .checked_add(facts.max(ending))?
        .checked_add(crate::specification::nodes(l)?)?
        .checked_add(crate::instructions::nodes(l)?)?
        .checked_add(crate::proposed_task::contract_nodes(l)?)?
        .checked_add(crate::owner_wake::policy_nodes())?
        .checked_add(crate::owner_wake::subscription_nodes(l.rows)?)?
        .checked_add(chip_nodes(l)?.checked_add(4)?.checked_mul(l.rows)?.checked_mul(3)?)?
        .checked_add(result)?
        .checked_add(person)?
        .checked_add(l.rows.checked_mul(36)?)?
        .checked_add(crate::owner_authority::document_nodes(l.rows)?)?
        .checked_add(crate::owner_budget::budget_nodes())?
        .checked_add(28)
}

pub(super) fn nodes(l: &Limits) -> Option<u32> {
    // Presence is existential: one Result/Proposal can appear on every row.
    // All eight feasible combinations use the same residual budget per case.
    Some(
        case(l, 0, 0, 0)?
            .max(case(l, 0, 0, 1)?)
            .max(case(l, 0, 1, 0)?)
            .max(case(l, 0, 1, 1)?)
            .max(case(l, 1, 0, 0)?)
            .max(case(l, 1, 0, 1)?)
            .max(case(l, 1, 1, 0)?)
            .max(case(l, 1, 1, 1)?),
    )
}
