//! Supplied recurring templates and period facts (pages.md, section 6).
use crate::{
    Builder, JigBindings, Role, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::{
    Domain,
    action::Act,
    address::Address,
    contract::{Recurring, RecurringOverlap},
};

pub(super) fn recurring(b: &mut Builder, client: &Domain, row: &Recurring, words: &Words, bindings: &mut JigBindings) {
    b.open(Role::Region);
    b.name(row.chip.title.bytes());
    cards::chip::chip(b, client, &row.chip, words, bindings);
    words.text(b, JigPhrase::RecurringKey, &[Hole::Number(u64::from(row.key))]);
    b.open(Role::List);
    for task in &row.batch {
        cards::proposal::proposed_task(b, task, words, bindings);
    }
    b.close();
    words.text(b, JigPhrase::Budget, &[Hole::Amount(row.budget)]);
    let overlap = match row.overlap {
        RecurringOverlap::Skip => JigPhrase::RecurringSkip,
        RecurringOverlap::Wait => JigPhrase::RecurringWait,
    };
    words.text(b, overlap, &[]);
    words.text(b, JigPhrase::CurrentPeriod, &[Hole::Number(row.current_period)]);
    if let Some(began) = row.began {
        b.open(Role::Time);
        words.text(b, JigPhrase::PeriodBegan, &[Hole::Wall(words.adjusted_wall(began))]);
        b.close();
    }
    match row.next {
        Some(next) => {
            b.open(Role::Time);
            words.text(b, JigPhrase::NextRun, &[Hole::Wall(words.adjusted_wall(next))]);
            b.close();
        }
        None => words.text(b, JigPhrase::NextPeriod, &[]),
    }
    words.text(b, JigPhrase::RecurringLastPeriod, &[Hole::Number(row.last_period)]);
    if let Some(period) = row.pending_period {
        words.text(b, JigPhrase::RecurringPendingPeriod, &[Hole::Number(period)]);
    }
    for task in &row.last_batch {
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Link,
            JigPhrase::TaskPage,
            &[Hole::Number(*task)],
            Act::Go { address: Address::task(*task) },
        );
    }
    if row.cut {
        words.status(b, JigPhrase::RecurringCut, &[]);
    }
    b.close();
}
