//! Full windows and nested documents bound every page (view.md, 10).
//! Counts include the shared cards and loading slot fallbacks. `slot_nodes`
//! is the sum of the owners' fragment bounds for the page, including navigation.
use super::{authority, card_nodes, chip_nodes, markdown};
use jig_domain_client::Limits;
fn runtime_in(l: &Limits, chip: u32) -> Option<u32> {
    chip.checked_mul(l.rows)?.checked_add(l.rows.checked_mul(5)?)?.checked_add(24)
}
/// Inbox cards, lineage, filters, counts and both edges.
#[must_use]
pub fn inbox_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    inbox_in(l, card_nodes(l)?, chip_nodes(l)?, slot_nodes)
}
fn inbox_in(l: &Limits, card: u32, chip: u32, slot_nodes: u32) -> Option<u32> {
    card.checked_add(chip)?
        .checked_add(28)?
        .checked_mul(l.rows.checked_mul(4)?)?
        .checked_add(l.projects.checked_mul(3)?)?
        .checked_add(84)?
        .checked_add(slot_nodes)
}
/// Chat rows, every runtime alternative and all offered project forms.
#[must_use]
pub fn conversations_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    conversations_in(l, card_nodes(l)?, chip_nodes(l)?, slot_nodes)
}
fn conversations_in(l: &Limits, card: u32, chip: u32, slot_nodes: u32) -> Option<u32> {
    let row = card
        .checked_add(chip)?
        .checked_add(runtime_in(l, chip)?)?
        .checked_add(crate::owner_budget::budget_nodes())?
        .checked_add(20)?
        .checked_mul(l.rows)?;
    let forms = l.charters.max(l.rows).checked_mul(3)?.checked_add(9)?.checked_mul(l.projects.max(l.rows))?;
    row.checked_add(forms)?.checked_add(66)?.checked_add(slot_nodes)
}
fn call_in(l: &Limits, card: u32) -> Option<u32> {
    card.checked_mul(l.rows)?.checked_add(l.rows.checked_mul(5)?)?.checked_add(16)
}
fn turn_in(l: &Limits, card: u32) -> Option<u32> {
    // Every message has a full block list; a call can name a full object list.
    call_in(l, card)?
        .checked_add(markdown(l)?)?
        .checked_add(6)?
        .checked_mul(l.rows)?
        .checked_add(2)?
        .checked_mul(l.rows)?
        .checked_add(10)
}
/// Committed log, pending words, all provisional turns, one independent current call and both edges.
#[must_use]
pub fn conversation_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    conversation_in(l, card_nodes(l)?, slot_nodes)
}
fn conversation_in(l: &Limits, card: u32, slot_nodes: u32) -> Option<u32> {
    let owner = 100_u32.checked_add(l.rows.checked_mul(4)?)?;
    let rows = turn_in(l, card)?
        .checked_add(owner)?
        .checked_add(card.checked_mul(2)?)?
        .checked_add(20)?
        .checked_mul(l.rows)?;
    let live = call_in(l, card)?.max(10).checked_add(markdown(l)?)?.checked_add(8)?.checked_mul(l.rows)?;
    rows.checked_add(live)?.checked_add(l.conversation_words.checked_mul(9)?)?.checked_add(34)?.checked_add(slot_nodes)
}
fn plan_in(l: &Limits, card: u32, chip: u32) -> Option<u32> {
    let older = card
        .checked_mul(l.rows.checked_add(1)?)?
        .checked_add(chip)?
        .checked_add(l.rows.checked_mul(2)?)?
        .checked_add(18)?
        .checked_mul(l.rows)?
        .checked_add(26)?;
    older.max(crate::owner_tree::max_nodes(l)?.checked_add(6)?).checked_add(crate::historical_pages::ended_nodes(l)?)
}
fn overview_in(l: &Limits, card: u32, chip: u32) -> Option<u32> {
    crate::owner_chip::max_nodes(l)?
        .checked_add(l.rows.checked_add(20)?.checked_mul(l.rows)?)?
        .checked_add(crate::task_result::nodes(l)?)?
        .checked_add(crate::specification::nodes(l)?)?
        .checked_add(crate::instructions::nodes(l)?)?
        .checked_add(crate::proposed_task::contract_nodes(l)?)?
        .checked_add(crate::owner_wake::policy_nodes())?
        .checked_add(crate::owner_wake::subscription_nodes(l.rows)?)?
        .checked_add(chip.checked_add(4)?.checked_mul(l.rows)?.checked_mul(3)?)?
        .checked_add(card.checked_mul(l.rows.checked_add(2)?)?)?
        .checked_add(l.rows.checked_mul(36)?)?
        .checked_add(24)
}
fn history(l: &Limits) -> Option<u32> {
    let line = markdown(l)?.checked_mul(2)?.checked_add(authority(l)?)?.checked_add(24)?;
    let row = line.max(crate::owner_history::max_nodes(l)?).max(crate::owner_milestone::max_nodes(l)?);
    row.checked_add(3)?.checked_mul(l.rows)?.checked_add(18)
}
/// Task header, lineage and the largest available executor section.
#[must_use]
pub fn task_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    task_in(l, card_nodes(l)?, chip_nodes(l)?, slot_nodes)
}
fn task_in(l: &Limits, card: u32, chip: u32, slot_nodes: u32) -> Option<u32> {
    let budget = crate::owner_authority::document_nodes(l.rows)?
        .checked_add(crate::owner_budget::budget_nodes())?
        .checked_add(l.rows.checked_mul(4)?)?
        .checked_add(4)?;
    let activity = conversation_in(l, card, 0)?
        .checked_add(runtime_in(l, chip)?)?
        .checked_add(card.checked_mul(2)?)?
        // The Task owner keeps departing cards alongside each fresh bounded
        // source, so the shared working-set capacity bounds their union.
        .checked_add(super::proposal_nodes(l)?.checked_mul(l.objects)?)?
        .checked_add(crate::owner_chip::observation_nodes())?;
    let panels = l
        .rows
        .checked_mul(4)?
        .checked_add(2)?
        .checked_mul(5)?
        .checked_add(l.rows.checked_add(20)?.checked_mul(l.rows)?)?;
    let section =
        plan_in(l, card, chip)?.max(overview_in(l, card, chip)?).max(history(l)?).max(budget).max(activity).max(panels);
    card.checked_add(chip.checked_mul(l.rows)?)?.checked_add(section)?.checked_add(35)?.checked_add(slot_nodes)
}
fn goals_in(l: &Limits, card: u32, chip: u32) -> Option<u32> {
    let board = card
        .checked_add(chip)?
        .checked_add(crate::owner_budget::budget_nodes())?
        .checked_add(15)?
        .checked_mul(l.rows)?
        .checked_mul(2)?;
    // A bounded goal offers whole charter and procedure lists independently.
    let choices = l.charters.max(l.rows).checked_add(l.rows)?;
    let form = choices
        .checked_mul(4)?
        .checked_add(l.rows.checked_mul(3)?)?
        .checked_add(32)?
        .checked_mul(l.projects.max(l.rows))?;
    board.checked_add(form)?.checked_add(34)
}
fn tasks_in(l: &Limits, card: u32, chip: u32) -> Option<u32> {
    card.checked_add(chip)?.checked_add(6)?.checked_mul(l.rows)?.checked_add(l.rows.checked_mul(2)?)?.checked_add(40)
}
fn notes_in(l: &Limits, card: u32) -> Option<u32> {
    card.checked_add(l.rows.checked_add(8)?)?.checked_mul(l.rows)?.checked_add(26)
}
fn spend_in(l: &Limits, card: u32, chip: u32) -> Option<u32> {
    // Each past period has pools and task lines, each at the row bound.
    card.checked_add(chip)?
        .checked_add(8)?
        .checked_mul(l.rows)?
        .checked_add(8)?
        .checked_mul(l.rows.checked_add(1)?)?
        .checked_add(10)
}
fn settings_in(l: &Limits, card: u32, chip: u32) -> Option<u32> {
    let policy = authority(l)?
        .checked_add(l.rows)?
        .checked_add(4)?
        .checked_mul(l.rows)?
        .checked_add(authority(l)?)?
        .checked_add(l.rows.checked_mul(2)?)?;
    // Each recurring line carries a chip, a whole template batch, its retained
    // last batch's links and only the period boundaries the source supplied.
    let recurring = super::proposed_task_nodes(l)?
        .checked_mul(l.rows)?
        .checked_add(chip)?
        .checked_add(l.rows.checked_mul(2)?)?
        .checked_add(18)?
        .checked_mul(l.rows)?;
    policy
        .checked_add(spend_in(l, card, chip)?)?
        .checked_add(crate::historical_pages::spend_nodes(l)?)?
        .checked_add(recurring)?
        .checked_add(l.rows.checked_mul(24)?)?
        .checked_add(32)
}
/// Project board, filtered windows and every settings subregion together.
#[must_use]
pub fn project_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    project_in(l, card_nodes(l)?, chip_nodes(l)?, slot_nodes)
}
fn project_in(l: &Limits, card: u32, chip: u32, slot_nodes: u32) -> Option<u32> {
    let overview = goals_in(l, card, chip)?.checked_add(inbox_in(l, card, chip, 0)?)?;
    overview
        .max(goals_in(l, card, chip)?)
        .max(tasks_in(l, card, chip)?)
        .max(notes_in(l, card)?)
        .max(settings_in(l, card, chip)?)
        .max(crate::owner_project::max_nodes(l)?)
        .checked_add(24)?
        .checked_add(slot_nodes)
}
/// Hosts, accounts, project capacity, refusals and deployment parts.
#[must_use]
pub fn deployment_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    deployment_in(l, chip_nodes(l)?, slot_nodes)
}
fn deployment_in(l: &Limits, chip: u32, slot_nodes: u32) -> Option<u32> {
    let synthetic = chip
        .checked_mul(l.rows)?
        .checked_mul(l.rows)?
        .checked_mul(3)?
        .checked_add(l.rows.checked_mul(40)?)?
        .checked_add(64)?;
    synthetic.max(crate::owner_deployment::max_nodes(l)?).checked_add(slot_nodes)
}
/// Starting, providers, missing, out of date and someone else's work.
#[must_use]
pub fn not_the_work_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    l.providers.checked_mul(2)?.checked_add(24)?.checked_add(slot_nodes)
}
/// The largest jig page with every window and text at its bound.
#[must_use]
pub fn page_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    let card = card_nodes(l)?;
    let chip = chip_nodes(l)?;
    inbox_in(l, card, chip, 0)?
        .max(conversations_in(l, card, chip, 0)?)
        .max(task_in(l, card, chip, 0)?)
        .max(project_in(l, card, chip, 0)?)
        .max(deployment_in(l, chip, 0)?)
        .max(not_the_work_nodes(l, 0)?)
        .checked_add(slot_nodes)
}
