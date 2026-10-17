//! Full windows and nested documents bound every page (view.md, 10).
//! Counts include the shared cards and loading slot fallbacks. `slot_nodes`
//! is the sum of the owners' fragment bounds for the page, including navigation.
use super::{authority, card_nodes, chip_nodes, markdown};
use crate::{
    Limits,
    address::{ProjectSection, TaskSection},
    page::Arrangement,
};
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
        .checked_add(85)?
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
    row.checked_add(forms)?.checked_add(70)?.checked_add(slot_nodes)
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
fn task_in(l: &Limits, _card: u32, _chip: u32, slot_nodes: u32) -> Option<u32> {
    arranged_task(l, slot_nodes, &Arrangement::none())
}
fn arranged_task(l: &Limits, slot_nodes: u32, arrangement: &Arrangement) -> Option<u32> {
    let mut sum = 0_u32;
    let mut largest = 0_u32;
    for section in [
        TaskSection::Overview,
        TaskSection::Activity,
        TaskSection::Plan,
        TaskSection::History,
        TaskSection::Budget,
        TaskSection::Panels,
    ] {
        let nodes = task_section_nodes(l, section)?;
        if arrangement.task.as_slice().contains(&section) {
            sum = sum.checked_add(nodes)?;
        } else {
            largest = largest.max(nodes);
        }
    }
    task_header_nodes(l)?.checked_add(sum)?.checked_add(largest)?.checked_add(slot_nodes)
}
/// Heading, task card, lineage and tabs, shared by every task section.
#[must_use]
pub fn task_header_nodes(l: &Limits) -> Option<u32> {
    card_nodes(l)?.checked_add(chip_nodes(l)?.checked_mul(l.rows)?)?.checked_add(36)
}
/// One whole task section, without its shared header or application fragments.
#[must_use]
pub fn task_section_nodes(l: &Limits, section: TaskSection) -> Option<u32> {
    let card = card_nodes(l)?;
    let chip = chip_nodes(l)?;
    match section {
        TaskSection::Overview => overview_in(l, card, chip),
        TaskSection::Plan => plan_in(l, card, chip),
        TaskSection::History => history(l),
        TaskSection::Budget => crate::owner_authority::document_nodes(l.rows)?
            .checked_add(crate::owner_budget::budget_nodes())?
            .checked_add(l.rows.checked_mul(4)?)?
            .checked_add(4),
        TaskSection::Activity => conversation_in(l, card, 0)?
            .checked_add(runtime_in(l, chip)?)?
            .checked_add(card.checked_mul(2)?)?
            .checked_add(super::proposal_nodes(l)?.checked_mul(l.objects)?)?
            .checked_add(crate::owner_chip::observation_nodes()),
        TaskSection::Panels => l
            .rows
            .checked_mul(4)?
            .checked_add(2)?
            .checked_mul(5)?
            .checked_add(l.rows.checked_add(20)?.checked_mul(l.rows)?),
    }
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
    board.checked_add(form)?.checked_add(35)
}
fn tasks_in(l: &Limits, card: u32, chip: u32) -> Option<u32> {
    card.checked_add(chip)?.checked_add(6)?.checked_mul(l.rows)?.checked_add(l.rows.checked_mul(2)?)?.checked_add(40)
}
fn notes_in(l: &Limits, card: u32) -> Option<u32> {
    card.checked_add(l.rows.checked_add(8)?)?.checked_mul(l.rows)?.checked_add(27)
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
fn project_in(l: &Limits, _card: u32, _chip: u32, slot_nodes: u32) -> Option<u32> {
    arranged_project(l, slot_nodes, &Arrangement::none())
}
/// Heading, project identity and tabs shared by every project section.
#[must_use]
pub const fn project_header_nodes(_l: &Limits) -> Option<u32> {
    Some(35)
}
/// One whole project section, including either document vocabulary.
#[must_use]
pub fn project_section_nodes(l: &Limits, section: ProjectSection) -> Option<u32> {
    let card = card_nodes(l)?;
    let chip = chip_nodes(l)?;
    let nodes = match section {
        ProjectSection::Overview => goals_in(l, card, chip)?.checked_add(inbox_in(l, card, chip, 0)?)?,
        ProjectSection::Goals => goals_in(l, card, chip)?,
        ProjectSection::Tasks => tasks_in(l, card, chip)?,
        ProjectSection::Notes => notes_in(l, card)?,
        ProjectSection::Settings
        | ProjectSection::Resources
        | ProjectSection::Policy
        | ProjectSection::Roles
        | ProjectSection::Spend
        | ProjectSection::Recurring => settings_in(l, card, chip)?,
    };
    Some(nodes.max(crate::owner_project::max_nodes(l)?))
}
fn arranged_project(l: &Limits, slot_nodes: u32, arrangement: &Arrangement) -> Option<u32> {
    let mut sum = 0_u32;
    let mut largest = 0_u32;
    for section in [
        ProjectSection::Overview,
        ProjectSection::Goals,
        ProjectSection::Tasks,
        ProjectSection::Notes,
        ProjectSection::Settings,
        ProjectSection::Resources,
        ProjectSection::Policy,
        ProjectSection::Roles,
        ProjectSection::Spend,
        ProjectSection::Recurring,
    ] {
        let nodes = project_section_nodes(l, section)?;
        if arrangement.project.as_slice().contains(&section) {
            sum = sum.checked_add(nodes)?;
        } else if !arrangement.project.as_slice().contains(&ProjectSection::Settings) || outside_settings(section) {
            largest = largest.max(nodes);
        }
    }
    project_header_nodes(l)?.checked_add(sum)?.checked_add(largest)?.checked_add(slot_nodes)
}
const fn outside_settings(section: ProjectSection) -> bool {
    match section {
        ProjectSection::Overview | ProjectSection::Goals | ProjectSection::Tasks | ProjectSection::Notes => true,
        ProjectSection::Settings
        | ProjectSection::Resources
        | ProjectSection::Policy
        | ProjectSection::Roles
        | ProjectSection::Spend
        | ProjectSection::Recurring => false,
    }
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
        .checked_add(70)?;
    synthetic.max(crate::owner_deployment::max_nodes(l)?.checked_add(3)?).checked_add(slot_nodes)
}
/// Starting, providers, missing, out of date and someone else's work.
#[must_use]
pub fn not_the_work_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    l.providers.checked_mul(2)?.checked_add(25)?.checked_add(slot_nodes)
}
/// The largest jig page with every window and text at its bound.
#[must_use]
pub fn page_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
    arranged_nodes(l, slot_nodes, &Arrangement::none())
}
/// The largest page with the arrangement's sections beside its address's own.
#[must_use]
pub fn arranged_nodes(l: &Limits, slot_nodes: u32, arrangement: &Arrangement) -> Option<u32> {
    let card = card_nodes(l)?;
    let chip = chip_nodes(l)?;
    inbox_in(l, card, chip, 0)?
        .max(conversations_in(l, card, chip, 0)?)
        .max(arranged_task(l, 0, arrangement)?)
        .max(arranged_project(l, 0, arrangement)?)
        .max(deployment_in(l, chip, 0)?)
        .max(not_the_work_nodes(l, 0)?)
        .checked_add(slot_nodes)
}
