//! Retained page fragments (client/domain/pages.md, 5.2 and 6.5; view.md,
//! 7 and 10). The source owns row order, optional clocks and all counters.
//! Brief delegate identity does not manufacture a Result card; a funding
//! record does not supply pools, task rows or an amount left.
use crate::{
    Builder, JigBindings, Role, Run, historical_pages_words::HistoricalPhrase as P, phrases::Hole, words::Words,
};
use alloc::boxed::Box;
use jig_domain_client::{
    Domain, Limits,
    contract::{Chip, ChipPhase, Cursor, Funder, OwnerChipEnding, OwnerEndedPage, OwnerSpendPage},
};

/// Finite tree storage for one whole retained page fragment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoricalFragment {
    pub nodes: u32,
    pub text_bytes: u32,
    pub text_per_node: u32,
    pub depth: u32,
    pub bindings: u32,
    pub slots: u32,
}
/// Brief row identity and phase plus the supplied complete ending/result.
#[must_use]
pub fn ended_nodes(l: &Limits) -> Option<u32> {
    crate::chip_nodes(l)?
        .checked_add(crate::result_nodes(l)?)?
        .checked_add(crate::markdown::max_nodes(l.text)?)?
        .checked_add(32)?
        .checked_mul(l.rows)?
        .checked_add(2)
}
/// Every four-counter row, both source identities and supplied period clocks.
#[must_use]
pub fn spend_nodes(l: &Limits) -> Option<u32> {
    l.rows.checked_mul(18)?.checked_add(2)
}
fn envelope(nodes: u32, l: &Limits, words: &Words) -> Option<HistoricalFragment> {
    let phrase = words.book.jig_capacity()?;
    let text_per_node = phrase.max(crate::literal_segment::max_bytes(l.text)?).checked_mul(2)?;
    Some(HistoricalFragment {
        nodes,
        text_bytes: nodes.checked_mul(text_per_node)?,
        text_per_node,
        depth: l.rows.checked_add(32)?,
        bindings: nodes,
        slots: nodes,
    })
}
#[must_use]
pub fn ended_worst_case(l: &Limits, words: &Words) -> Option<HistoricalFragment> {
    envelope(ended_nodes(l)?, l, words)
}
#[must_use]
pub fn spend_worst_case(l: &Limits, words: &Words) -> Option<HistoricalFragment> {
    envelope(spend_nodes(l)?, l, words)
}
fn edges_fit(before: Option<&Cursor>, after: Option<&Cursor>, l: &Limits) -> Option<()> {
    if let Some(edge) = before
        && !edge.value.fits(l.text)
    {
        return None;
    }
    if let Some(edge) = after
        && !edge.value.fits(l.text)
    {
        return None;
    }
    Some(())
}
fn chip_fits(chip: &Chip, l: &Limits, words: &Words) -> Option<()> {
    if !chip.title.bytes.fits(l.text) {
        return None;
    }
    match chip.details.as_ref() {
        ChipPhase::Owner { details, .. } => {
            crate::owner_chip::required(details, l, words)?;
        }
        ChipPhase::Active(_)
        | ChipPhase::Waiting { .. }
        | ChipPhase::Idle { .. }
        | ChipPhase::Starting
        | ChipPhase::Running { .. }
        | ChipPhase::WaitingForWords { .. }
        | ChipPhase::BackingOff { .. }
        | ChipPhase::Procedure { .. }
        | ChipPhase::Person { .. }
        | ChipPhase::Closing(_)
        | ChipPhase::Held { .. }
        | ChipPhase::Ended { .. } => {}
    }
    Some(())
}
fn ending_fits(ending: &OwnerChipEnding, l: &Limits) -> Option<()> {
    match ending {
        OwnerChipEnding::Done(result) => {
            crate::owner_chip::result_nodes(result, l)?;
        }
        OwnerChipEnding::Failed(reason) => {
            if !reason.bytes.fits(l.text) {
                return None;
            }
        }
        OwnerChipEnding::Cancelled { reason, result, .. } => {
            if !reason.bytes.fits(l.text) {
                return None;
            }
            if let Some(result) = result {
                crate::owner_chip::result_nodes(result, l)?;
            }
        }
    }
    Some(())
}
#[must_use]
pub fn ended_required(page: &OwnerEndedPage, l: &Limits, words: &Words) -> Option<HistoricalFragment> {
    if page.rows.capacity() > l.rows {
        return None;
    }
    edges_fit(page.before.as_ref(), page.after.as_ref(), l)?;
    for row in &page.rows {
        chip_fits(&row.chip, l, words)?;
        ending_fits(&row.ended, l)?;
    }
    ended_worst_case(l, words)
}
#[must_use]
pub fn spend_required(page: &OwnerSpendPage, l: &Limits, words: &Words) -> Option<HistoricalFragment> {
    if page.rows.capacity() > l.rows {
        return None;
    }
    edges_fit(page.before.as_ref(), page.after.as_ref(), l)?;
    spend_worst_case(l, words)
}
fn saying(words: &Words, phrase: P, holes: &[Hole]) -> Box<[u8]> {
    words.book.say(words.language, phrase.id(), holes).expect("checked historical words and typed source holes").text
}
fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    b.text(Run::Plain, &saying(words, phrase, holes));
}
fn wall(b: &mut Builder, words: &Words, phrase: P, at: skein_lib::Wall) {
    text(b, words, phrase, &[Hole::Wall(words.adjusted_wall(at))]);
}
/// Display admitted rows as a companion to either existing Plan rendering path.
pub fn ended_render(
    b: &mut Builder,
    client: &Domain,
    page: &OwnerEndedPage,
    words: &Words,
    bindings: &mut JigBindings,
) {
    b.open(Role::Region);
    b.name(&saying(words, P::EndedDelegates, &[]));
    for row in &page.rows {
        b.open(Role::Region);
        b.name(&saying(words, P::EndedRow, &[Hole::Number(row.chip.task)]));
        match row.chip.details.as_ref() {
            ChipPhase::Owner { phase, .. } => {
                crate::cards::chip::head(b, &row.chip, words, bindings);
                crate::chip_phase::brief(b, phase, words);
            }
            ChipPhase::Active(_)
            | ChipPhase::Waiting { .. }
            | ChipPhase::Idle { .. }
            | ChipPhase::Starting
            | ChipPhase::Running { .. }
            | ChipPhase::WaitingForWords { .. }
            | ChipPhase::BackingOff { .. }
            | ChipPhase::Procedure { .. }
            | ChipPhase::Person { .. }
            | ChipPhase::Closing(_)
            | ChipPhase::Held { .. }
            | ChipPhase::Ended { .. } => crate::cards::chip::chip(b, client, &row.chip, words, bindings),
        }
        crate::owner_chip::ending(b, &row.ended, client, words, bindings);
        match row.at {
            Some(at) => wall(b, words, P::At, at),
            None => text(b, words, P::AtAbsent, &[]),
        }
        b.close();
    }
    b.close();
}
fn funder_name(value: Funder, words: &Words) -> Box<[u8]> {
    use crate::proposed_task_words::ProposedPhrase;
    let saying = match value {
        Funder::Task(task) => words.book.say(words.language, ProposedPhrase::FunderTask.id(), &[Hole::Number(task)]),
        Funder::Pool { project, person, period } => words.book.say(
            words.language,
            ProposedPhrase::FunderPool.id(),
            &[Hole::Number(u64::from(project)), Hole::Number(person), Hole::Number(period)],
        ),
        Funder::Recurring { project, task, period } => words.book.say(
            words.language,
            ProposedPhrase::FunderRecurring.id(),
            &[Hole::Number(u64::from(project)), Hole::Number(task), Hole::Number(period)],
        ),
        Funder::Period { project, period } => words.book.say(
            words.language,
            ProposedPhrase::FunderPeriod.id(),
            &[Hole::Number(u64::from(project)), Hole::Number(period)],
        ),
    };
    saying.expect("checked funder identity and supplied source coordinates").text
}
/// All supplied historical counters; no live balance or ledger projection.
pub fn spend_render(b: &mut Builder, page: &OwnerSpendPage, words: &Words) {
    b.open(Role::Region);
    b.name(&saying(words, P::FundingHistory, &[]));
    for row in &page.rows {
        b.open(Role::Region);
        b.name(&funder_name(row.funder, words));
        crate::proposed_task::funder(b, row.funder, words);
        match row.parent {
            Some(parent) => {
                text(b, words, P::Parent, &[]);
                crate::proposed_task::funder(b, parent, words);
            }
            None => text(b, words, P::ParentAbsent, &[]),
        }
        for (phrase, amount) in [
            (P::Budget, row.numbers.budget),
            (P::Spent, row.numbers.spent),
            (P::SpentBelow, row.numbers.spent_below),
            (P::Reserved, row.numbers.reserved),
        ] {
            text(b, words, phrase, &[Hole::Amount(amount)]);
        }
        text(b, words, P::Made, &[Hole::Number(u64::from(row.made))]);
        text(b, words, if row.closed { P::Closed } else { P::Open }, &[]);
        match row.period {
            Some(period) => {
                wall(b, words, P::Began, period.began);
                match period.next {
                    Some(next) => wall(b, words, P::Next, next),
                    None => text(b, words, P::NextAbsent, &[]),
                }
            }
            None => text(b, words, P::PeriodAbsent, &[]),
        }
        b.close();
    }
    b.close();
}
