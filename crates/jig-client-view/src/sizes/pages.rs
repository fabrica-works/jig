//! Full windows and nested documents bound every page (view.md, 10).
//! Counts include the shared cards and loading slot fallbacks. `slot_nodes`
//! is the sum of the owners' fragment bounds for the page, including navigation.
use super::{authority, card_nodes, chip_nodes, markdown};
use jig_client::Limits;
fn runtime(l: &Limits) -> Option<u32> {
    chip_nodes(l)?.checked_mul(l.rows)?.checked_add(l.rows.checked_mul(5)?)?.checked_add(24)
}
/// Inbox cards, lineage, filters, counts and both edges.
#[must_use]
pub fn inbox_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    card_nodes(l)?
        .checked_add(chip_nodes(l)?)?
        .checked_add(28)?
        .checked_mul(l.rows.checked_mul(4)?)?
        .checked_add(l.projects.checked_mul(3)?)?
        .checked_add(84)?
        .checked_add(slot_nodes)
}
/// Chat rows, every runtime alternative and all offered project forms.
#[must_use]
pub fn conversations_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    let row = card_nodes(l)?
        .checked_add(chip_nodes(l)?)?
        .checked_add(runtime(l)?)?
        .checked_add(crate::owner_budget::budget_nodes())?
        .checked_add(20)?
        .checked_mul(l.rows)?;
    let forms = l.charters.max(l.rows).checked_mul(3)?.checked_add(9)?.checked_mul(l.projects.max(l.rows))?;
    row.checked_add(forms)?.checked_add(66)?.checked_add(slot_nodes)
}
fn call(l: &Limits) -> Option<u32> {
    card_nodes(l)?.checked_mul(l.rows)?.checked_add(l.rows.checked_mul(5)?)?.checked_add(16)
}
fn turn(l: &Limits) -> Option<u32> {
    // Every message has a full block list; a call can name a full object list.
    call(l)?
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
    let owner = 100_u32.checked_add(l.rows.checked_mul(4)?)?;
    let rows = turn(l)?
        .checked_add(owner)?
        .checked_add(card_nodes(l)?.checked_mul(2)?)?
        .checked_add(20)?
        .checked_mul(l.rows)?;
    let live = call(l)?.max(10).checked_add(markdown(l)?)?.checked_add(8)?.checked_mul(l.rows)?;
    rows.checked_add(live)?.checked_add(l.conversation_words.checked_mul(9)?)?.checked_add(34)?.checked_add(slot_nodes)
}
fn plan(l: &Limits) -> Option<u32> {
    let older = card_nodes(l)?
        .checked_mul(l.rows.checked_add(1)?)?
        .checked_add(chip_nodes(l)?)?
        .checked_add(l.rows.checked_mul(2)?)?
        .checked_add(18)?
        .checked_mul(l.rows)?
        .checked_add(26)?;
    older.max(crate::owner_tree::max_nodes(l)?.checked_add(6)?).checked_add(crate::historical_pages::ended_nodes(l)?)
}
fn overview(l: &Limits) -> Option<u32> {
    crate::owner_chip::max_nodes(l)?
        .checked_add(l.rows.checked_add(20)?.checked_mul(l.rows)?)?
        .checked_add(crate::task_result::nodes(l)?)?
        .checked_add(crate::specification::nodes(l)?)?
        .checked_add(crate::instructions::nodes(l)?)?
        .checked_add(crate::proposed_task::contract_nodes(l)?)?
        .checked_add(crate::owner_wake::policy_nodes())?
        .checked_add(crate::owner_wake::subscription_nodes(l.rows)?)?
        .checked_add(chip_nodes(l)?.checked_add(4)?.checked_mul(l.rows)?.checked_mul(3)?)?
        .checked_add(card_nodes(l)?.checked_mul(l.rows.checked_add(2)?)?)?
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
    let budget = crate::owner_authority::document_nodes(l.rows)?
        .checked_add(crate::owner_budget::budget_nodes())?
        .checked_add(l.rows.checked_mul(4)?)?
        .checked_add(4)?;
    let activity = conversation_nodes(l, 0)?
        .checked_add(runtime(l)?)?
        .checked_add(card_nodes(l)?.checked_mul(2)?)?
        .checked_add(crate::owner_chip::observation_nodes())?;
    let panels = l
        .rows
        .checked_mul(4)?
        .checked_add(2)?
        .checked_mul(5)?
        .checked_add(l.rows.checked_add(20)?.checked_mul(l.rows)?)?;
    let section = plan(l)?.max(overview(l)?).max(history(l)?).max(budget).max(activity).max(panels);
    card_nodes(l)?
        .checked_add(chip_nodes(l)?.checked_mul(l.rows)?)?
        .checked_add(section)?
        .checked_add(35)?
        .checked_add(slot_nodes)
}
fn goals(l: &Limits) -> Option<u32> {
    let board = card_nodes(l)?
        .checked_add(chip_nodes(l)?)?
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
fn tasks(l: &Limits) -> Option<u32> {
    card_nodes(l)?
        .checked_add(chip_nodes(l)?)?
        .checked_add(6)?
        .checked_mul(l.rows)?
        .checked_add(l.rows.checked_mul(2)?)?
        .checked_add(40)
}
fn notes(l: &Limits) -> Option<u32> {
    card_nodes(l)?.checked_add(l.rows.checked_add(8)?)?.checked_mul(l.rows)?.checked_add(26)
}
fn spend(l: &Limits) -> Option<u32> {
    // Each past period has pools and task lines, each at the row bound.
    card_nodes(l)?
        .checked_add(chip_nodes(l)?)?
        .checked_add(8)?
        .checked_mul(l.rows)?
        .checked_add(8)?
        .checked_mul(l.rows.checked_add(1)?)?
        .checked_add(10)
}
fn settings(l: &Limits) -> Option<u32> {
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
        .checked_add(chip_nodes(l)?)?
        .checked_add(l.rows.checked_mul(2)?)?
        .checked_add(18)?
        .checked_mul(l.rows)?;
    policy
        .checked_add(spend(l)?)?
        .checked_add(crate::historical_pages::spend_nodes(l)?)?
        .checked_add(recurring)?
        .checked_add(l.rows.checked_mul(24)?)?
        .checked_add(32)
}
/// Project board, filtered windows and every settings subregion together.
#[must_use]
pub fn project_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    let overview = goals(l)?.checked_add(inbox_nodes(l, 0)?)?;
    overview
        .max(goals(l)?)
        .max(tasks(l)?)
        .max(notes(l)?)
        .max(settings(l)?)
        .max(crate::owner_project::max_nodes(l)?)
        .checked_add(24)?
        .checked_add(slot_nodes)
}
/// Hosts, accounts, project capacity, refusals and deployment parts.
#[must_use]
pub fn deployment_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    let legacy = chip_nodes(l)?
        .checked_mul(l.rows)?
        .checked_mul(l.rows)?
        .checked_mul(3)?
        .checked_add(l.rows.checked_mul(40)?)?
        .checked_add(64)?;
    legacy.max(crate::owner_deployment::max_nodes(l)?).checked_add(slot_nodes)
}
/// Starting, providers, missing, out of date and someone else's work.
#[must_use]
pub fn not_the_work_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    l.providers.checked_mul(2)?.checked_add(24)?.checked_add(slot_nodes)
}
/// The largest jig page with every window and text at its bound.
#[must_use]
pub fn page_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    inbox_nodes(l, 0)?
        .max(conversations_nodes(l, 0)?)
        .max(task_nodes(l, 0)?)
        .max(project_nodes(l, 0)?)
        .max(deployment_nodes(l, 0)?)
        .max(not_the_work_nodes(l, 0)?)
        .checked_add(slot_nodes)
}
