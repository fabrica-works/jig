//! Resolve a task's batch against its own remaining allotment (authority.md, 7).
use crate::{Core, Delegate};
use alloc::boxed::Box;
use jig_domain_people::Budget;
use jig_domain_tasks as tasks;

impl Core {
    pub(crate) fn task_as_allowed(&self, task: u64, batch: &mut [Delegate]) -> Result<(), tasks::Refusal> {
        let mut allowed = false;
        for member in batch.iter() {
            if let Budget::AsAllowed { .. } = member.budget {
                if allowed {
                    return Err(tasks::Refusal::AuthorityShape);
                }
                allowed = true;
            }
        }
        if !allowed {
            return Ok(());
        }
        let numbers = self.tasks.delegation(task).ok_or(tasks::Refusal::Unknown)?.numbers;
        let left = numbers.budget.checked_sub(numbers.spent).ok_or(tasks::Refusal::Funding)?;
        let left = left.checked_sub(numbers.spent_below).ok_or(tasks::Refusal::Funding)?;
        let mut left = left.checked_sub(numbers.reserved).ok_or(tasks::Refusal::Funding)?;
        for member in batch.iter() {
            if member.budget == Budget::Named {
                left = left.checked_sub(member.authority.budget.spend).ok_or(tasks::Refusal::Funding)?;
            }
        }
        for member in batch {
            if let Budget::AsAllowed { cap } = member.budget {
                let given = left.min(cap.unwrap_or(u64::MAX));
                if given == 0 {
                    return Err(tasks::Refusal::Funding);
                }
                member.authority.budget.spend = given;
                left = left.checked_sub(given).expect("given from remainder");
            }
        }
        Ok(())
    }
}

/// Build the bounded durable answer after the task child made the batch.
pub(crate) fn made(numbers: &[u64], given: Option<jig_domain_people::Given>, room: u32) -> Box<[crate::Made]> {
    let mut made = skein_lib::List::with_capacity(room);
    for &task in numbers {
        let amount = match given {
            Some(given) if given.task == task => Some(given.amount),
            Some(_) | None => None,
        };
        made.push(crate::Made { task, revision: 0, given: amount }).expect("one reply per admitted member");
    }
    made.into_boxed()
}
