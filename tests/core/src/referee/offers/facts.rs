//! Comparisons of delivered numerical/list facts, without authority rules.
use jig_domain_core::{
    acts::{Admission, Beyond, Bounds},
    documents::{AuthorityLine, GrantLine, PatternEnd, Task},
};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
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
    implications: &skein_lib::List<jig_domain_authority::Implication>,
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
    implications: &skein_lib::List<jig_domain_authority::Implication>,
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
            Bounds::Create {
                authority: _,
                pool_left,
                charters,
                procedures,
                latest,
                budget,
                maximum_budget,
                maximum_deadline,
            },
            people::Ask::Create { batch, .. },
        ) => {
            let [member] = batch.as_ref() else { return None };
            let executor = &member.executor;
            let amount = &member.authority.spend;
            let until = &member.authority.deadline;
            let supported = match executor {
                people::TaskExecutor::Person(_) => false,
                people::TaskExecutor::Agent { charter } => charters.as_slice().contains(charter),
                people::TaskExecutor::Procedure { connector, code } => {
                    !member.flags.conversation
                        && procedures.iter().any(|value| value.connector == *connector && value.code == *code)
                }
            };
            if !supported || amount > maximum_budget || !deadline(*maximum_deadline, *until) {
                return None;
            }
            Some(if amount <= pool_left && amount <= budget && deadline(*latest, *until) {
                Admission::Allowed
            } else {
                let proposable = match outcome {
                    Beyond::Proposed { to: jig_domain_core::acts::Recipient::Roles { roles, .. } } => {
                        roles.iter().copied().min().map(people::Accepter::Role)
                    }
                    Beyond::Refused(people::Refusal::Authority { proposable, .. }) => *proposable,
                    Beyond::Proposed { .. } | Beyond::Refused(_) => return None,
                };
                Admission::Refused(people::Refusal::Authority {
                    proposable,
                    lacking: if amount > budget {
                        people::Lack::Budget { needs: *amount, left: *budget }
                    } else {
                        people::Lack::Deadline
                    },
                })
            })
        }
        (
            Bounds::Create { maximum_budget, maximum_deadline, .. },
            people::Ask::Propose { action: people::Action::Batch(batch), .. },
        ) => {
            let [member] = batch.as_ref() else { return None };
            if member.tracked.is_none()
                || member.authority.spend > *maximum_budget
                || !deadline(*maximum_deadline, member.authority.deadline)
            {
                return None;
            }
            Some(beyond(outcome))
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
        (Bounds::Create { .. } | Bounds::Amend { .. }, _) => None,
    }
}

#[cfg(test)]
mod tests;
