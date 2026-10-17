//! The same single-source funding transfer for adoption and accepted amendments.
use crate::domain::{publish, record, task_mut};
use crate::{Domain, Funder, Limits, Numbers, Refusal, Request};
use skein_lib::{Env, Queue};

pub(crate) fn check(
    domain: &Domain,
    staged: &crate::compound::Staged,
    number: u64,
    destination: Funder,
    budget: u64,
    unopened: Option<Numbers>,
) -> Result<(), Refusal> {
    let old = staged.record(domain, number).ok_or(Refusal::Funding)?;
    let covered = match destination {
        Funder::Task(parent) => {
            parent != number
                && match staged.record(domain, parent) {
                    Some(row) => row.project == old.project,
                    None => false,
                }
                && staged.descendant(domain, number, parent, domain.names.len())
        }
        Funder::Pool { project, .. } | Funder::Period { project, .. } | Funder::Recurring { project, .. } => {
            project == old.project
        }
    };
    if !covered {
        return Err(Refusal::Funding);
    }
    let spent = super::total(old.numbers).ok_or(Refusal::Funding)?;
    let left = budget.checked_sub(spent).ok_or(Refusal::Funding)?;
    let source = staged.numbers(domain, old.funder).ok_or(Refusal::Funding)?;
    source.reserved.checked_sub(old.numbers.budget).ok_or(Refusal::Funding)?;
    source.spent_below.checked_add(spent).ok_or(Refusal::Funding)?;
    let mut after = staged.numbers(domain, destination).or(unopened).ok_or(Refusal::Funding)?;
    after.reserved = after.reserved.checked_add(left).ok_or(Refusal::Funding)?;
    if super::available(after).is_none()
        || super::available(Numbers { budget: left, spent: 0, spent_below: 0, reserved: old.numbers.reserved })
            .is_none()
    {
        return Err(Refusal::Funding);
    }
    Ok(())
}

pub(crate) fn apply(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    destination: Funder,
    budget: u64,
    out: &mut Queue<Request>,
) {
    let old = record(domain, number).expect("funding transfer checked");
    let source = old.funder;
    let before = old.numbers.budget;
    let spent = super::total(old.numbers).expect("funding transfer checked");
    let left = budget.checked_sub(spent).expect("funding transfer checked");
    match source {
        Funder::Task(parent) => {
            let row = task_mut(domain, parent).expect("checked task source");
            row.record.numbers.reserved = row.record.numbers.reserved.checked_sub(before).expect("source preflight");
            row.record.numbers.spent_below =
                row.record.numbers.spent_below.checked_add(spent).expect("source preflight");
            publish(domain, env, parent, out);
        }
        Funder::Pool { .. } | Funder::Period { .. } | Funder::Recurring { .. } => {
            let ledger = domain.funding.get_mut(&source).expect("checked finite source");
            ledger.numbers.reserved = ledger.numbers.reserved.checked_sub(before).expect("source preflight");
            ledger.numbers.spent_below = ledger.numbers.spent_below.checked_add(spent).expect("source preflight");
            super::save_funding(domain, source, out);
        }
    }
    match destination {
        Funder::Task(parent) => {
            let row = task_mut(domain, parent).expect("checked task destination");
            row.record.numbers.reserved = row.record.numbers.reserved.checked_add(left).expect("destination preflight");
            publish(domain, env, parent, out);
        }
        Funder::Pool { .. } | Funder::Period { .. } | Funder::Recurring { .. } => {
            let ledger = domain.funding.get_mut(&destination).expect("checked finite destination");
            ledger.numbers.reserved = ledger.numbers.reserved.checked_add(left).expect("destination preflight");
            super::save_funding(domain, destination, out);
        }
    }
    let task = task_mut(domain, number).expect("target live");
    task.record.funder = destination;
    task.record.numbers.budget = left;
    task.record.numbers.spent = 0;
    task.record.numbers.spent_below = 0;
    task.record.authority.budget.spend = left;
}
