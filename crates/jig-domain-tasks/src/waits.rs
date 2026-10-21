//! Circles of waits: no task may come to wait, through others, for itself (domain/tasks.md, 6.7).
//!
//! A waiting or active task waits for each pending dependency, and a task not held or ended
//! for each live delegate. This walk judges the dependencies an amendment adds over those edges
//! alone, from each new dependency forward to the task; the wait index, the queue edges and the
//! walk back from the new waiter take its place.
#![expect(clippy::result_large_err, reason = "bounded owner refusal carries exact capacity diagnostics inline")]

use crate::domain::{Domain, record};
use crate::{Limits, Phase, Problem, Refusal};
use skein_lib::{List, Set};

/// Refuse dependencies that `task` would newly wait for when one of them already waits, through
/// others, for `task`, naming that circle from `task` (domain/tasks.md, 6.7 and 7).
#[expect(dead_code, reason = "the amendment's check calls it once a dependency may be added")]
pub(crate) fn check_added(domain: &Domain, limits: &Limits, task: u64, added: &[u64]) -> Result<(), Problem> {
    let mut seen = Set::with_capacity(limits.tasks);
    let mut path: List<(u64, usize)> = List::with_capacity(limits.tasks);
    for first in added {
        if *first == task {
            return Err(circle(limits, task, &path));
        }
        if record(domain, *first).is_none() || !seen.insert(*first).expect("one mark per live task") {
            continue;
        }
        path.clear();
        path.push((*first, 0)).expect("one step per live task");
        while let Some(&(at, edge)) = path.last() {
            let (dependencies, delegates) = waits(domain, at);
            let next = match edge.checked_sub(dependencies.len()) {
                None => dependencies.get(edge),
                Some(delegate) => delegates.get(delegate),
            };
            let top = path.len().saturating_sub(1);
            let Some(next) = next else {
                path.truncate(top);
                continue;
            };
            if let Some(step) = path.get_mut(top) {
                step.1 = edge.saturating_add(1);
            }
            if *next == task {
                return Err(circle(limits, task, &path));
            }
            if seen.insert(*next).expect("one mark per live task") {
                path.push((*next, 0)).expect("one step per live task");
            }
        }
    }
    Ok(())
}

/// The tasks `task` waits for: its pending dependencies, then its live delegates.
fn waits(domain: &Domain, task: u64) -> (&[u64], &[u64]) {
    let Some(row) = record(domain, task) else { return (&[], &[]) };
    match &row.phase {
        Phase::Waiting | Phase::Active(_) => (&row.waiting_on, &row.delegates),
        Phase::Closing(_) => (&[], &row.delegates),
        Phase::Held { .. } | Phase::Ended(_) => (&[], &[]),
    }
}

/// The circle `task` would close, from `task` along `path`, cut at `Limits::circle`.
fn circle(limits: &Limits, task: u64, path: &List<(u64, usize)>) -> Problem {
    let mut tasks = List::with_capacity(limits.circle);
    let mut cut = false;
    if tasks.push(task).is_err() {
        cut = true;
    }
    for (at, _) in path.as_slice() {
        if tasks.push(*at).is_err() {
            cut = true;
            break;
        }
    }
    Problem::naming(Some(task), Refusal::Circle { cut }, tasks.as_slice())
}
