//! Comparisons of delivered numerical/list facts, without authority rules.
use jig_core::{
    acts::{Admission, Beyond, Bounds},
    documents::{AuthorityLine, GrantLine, PatternEnd, Task},
};
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::Wall;

fn deadline(bound: Option<Wall>, draft: Option<Wall>) -> bool {
    match (bound, draft) {
        (None, _) => true,
        (Some(limit), Some(value)) => value <= limit,
        (Some(_), None) => false,
    }
}
// Literal pattern inclusion from supplied segments and terminals only. No connector lookup.
fn pattern(segments: &skein_lib::List<Box<[u8]>>, last: &PatternEnd, value: &people::Pattern) -> bool {
    match last {
        PatternEnd::Exact(bound) => match &value.last {
            people::Last::Exact(candidate) => segments.as_slice() == value.segments.as_ref() && candidate == bound,
            people::Last::Open(_) => false,
        },
        PatternEnd::Open(bound) => {
            let prefix = segments.as_slice();
            if value.segments.get(..prefix.len()) != Some(prefix) {
                return false;
            }
            match value.segments.get(prefix.len()) {
                Some(segment) => segment.starts_with(bound),
                None => match &value.last {
                    people::Last::Exact(segment) | people::Last::Open(segment) => segment.starts_with(bound),
                },
            }
        }
    }
}
fn grant(
    left: &GrantLine,
    right: &people::Grant,
    implications: &skein_lib::List<jig_core_authority::Implication>,
) -> bool {
    left.connector == right.connector
        && (left.kind == right.kind
            || implications
                .iter()
                .any(|pair| pair.connector == left.connector && pair.kind == left.kind && pair.implies == right.kind))
        && pattern(&left.segments, &left.last, &right.pattern)
}
fn covers(
    bound: &AuthorityLine,
    draft: &people::Authority,
    implications: &skein_lib::List<jig_core_authority::Implication>,
) -> bool {
    !bound.cut
        && draft.spend <= bound.ceiling.spend
        && deadline(bound.ceiling.deadline, draft.deadline)
        && draft.tools & !bound.tools.0 == 0
        && draft.notes & !bound.notes.0 == 0
        && draft.delegation.tasks <= bound.tasks
        && draft.delegation.depth <= bound.depth
        && draft.delegation.kinds.iter().all(|value| {
            bound.executors.iter().any(|listed| match (listed, value) {
            (tasks::AuthorityExecutor::Charter(a), people::Executor::Charter(b))
            | (tasks::AuthorityExecutor::Procedure(a), people::Executor::Procedure(b))
            | (tasks::AuthorityExecutor::Role(a), people::Executor::Role(b)) => a == b,
            (tasks::AuthorityExecutor::Charter(_), people::Executor::Procedure(_) | people::Executor::Role(_))
            | (tasks::AuthorityExecutor::Procedure(_), people::Executor::Charter(_) | people::Executor::Role(_))
            | (tasks::AuthorityExecutor::Role(_), people::Executor::Charter(_) | people::Executor::Procedure(_)) => {
                false
            }
        })
        })
        && draft.grants.iter().all(|value| bound.grants.iter().any(|listed| grant(listed, value, implications)))
        && draft.note_resources.iter().all(|value| {
            bound.note_resources.iter().any(|listed| {
                listed.connector == value.connector && pattern(&listed.segments, &listed.last, &value.pattern)
            })
        })
}
fn beyond(value: &Beyond) -> Admission {
    match value {
        Beyond::Proposed { to } => Admission::Proposed { to: to.clone() },
        Beyond::Refused(why) => Admission::Refused(*why),
    }
}
/// `None` means the draft is outside the supplied supported form or its old facts are incomplete.
pub(super) fn predict(value: &Admission, ask: &people::Ask, before: Option<&Task>) -> Option<Admission> {
    let Admission::Bounded { within, beyond: outcome } = value else {
        return Some(value.clone());
    };
    match (within, ask) {
        (
            Bounds::Goal { pool_left, charters, procedures, latest, budget, maximum_budget, maximum_deadline },
            people::Ask::SetGoal { executor, budget: amount, deadline: until, .. },
        ) => {
            let supported = match executor {
                people::GoalExecutor::Charter(charter) => charters.as_slice().contains(charter),
                people::GoalExecutor::Procedure { connector, code } => {
                    procedures.iter().any(|value| value.connector == *connector && value.code == *code)
                }
            };
            if !supported || amount > maximum_budget || !deadline(*maximum_deadline, *until) {
                return None;
            }
            Some(if amount <= pool_left && amount <= budget && deadline(*latest, *until) {
                Admission::Allowed
            } else {
                beyond(outcome)
            })
        }
        (
            Bounds::Amend { budget_left, grants, implications, authority, ceiling },
            people::Ask::Amend { amendment, revision, .. },
        ) => {
            let old = before?;
            if old.spec_revision != *revision || old.authority.cut {
                return None;
            }
            let Some(after) = &amendment.authority else {
                return Some(Admission::Allowed);
            };
            if covers(&old.authority, after, implications) {
                return Some(Admission::Allowed);
            }
            if !covers(ceiling, after, implications) {
                return None;
            }
            let direct = covers(authority, after, implications)
                && after.spend.saturating_sub(old.authority.ceiling.spend) <= *budget_left
                && after.grants.iter().all(|value| {
                    old.authority.grants.iter().any(|listed| grant(listed, value, implications))
                        || grants.iter().any(|listed| grant(listed, value, implications))
                });
            Some(if direct { Admission::Allowed } else { beyond(outcome) })
        }
        (Bounds::Goal { .. } | Bounds::Amend { .. }, _) => None,
    }
}

#[cfg(test)]
mod tests;
