//! Compare concrete drafts with the engine's numbers and lists.
//! The engine owns authorization; no role or policy is interpreted here.
//! See client/domain/pages.md, 6.2, and objects.md, 11.
use super::{Admission, Amendment, Authority, Beyond, Bounds, GoalExecutor, Refusal};
use skein_lib::Wall;

fn beyond(value: &Beyond) -> Admission {
    match value {
        Beyond::Proposed { to } => Admission::Proposed { to: to.clone() },
        Beyond::Refused(why) => Admission::Refused(why.clone()),
    }
}

fn deadline(draft: Option<Wall>, latest: Option<Wall>) -> bool {
    match latest {
        None => true,
        Some(latest) => match draft {
            Some(draft) => draft <= latest,
            None => false,
        },
    }
}

/// Explain the selected goal using only its current offer's factual bounds.
#[must_use]
pub fn goal(offer: &Admission, executor: GoalExecutor, budget: u64, at: Option<Wall>) -> Admission {
    match offer {
        Admission::Bounded { within, beyond: outside } => match within {
            Bounds::Goal {
                pool_left: _,
                charters,
                procedures,
                latest,
                budget: allowed,
                maximum_budget,
                maximum_deadline,
            } => {
                let mut listed = false;
                match executor {
                    GoalExecutor::Charter(charter) => {
                        for number in charters {
                            if *number == charter {
                                listed = true;
                            }
                        }
                    }
                    GoalExecutor::Procedure { connector, code } => {
                        for procedure in procedures {
                            if procedure.connector == connector && procedure.code == code {
                                listed = true;
                            }
                        }
                    }
                }
                if !listed || budget > *maximum_budget || !deadline(at, *maximum_deadline) {
                    return Admission::Refused(Refusal::NotOffered);
                }
                if budget <= *allowed && deadline(at, *latest) { Admission::Allowed } else { beyond(outside) }
            }
            Bounds::Amend { .. } => Admission::Refused(Refusal::NotOffered),
        },
        Admission::Allowed | Admission::Proposed { .. } | Admission::Refused(_) => offer.clone(),
    }
}

/// Explain an amendment against the current task facts and its supplied additions.
#[must_use]
pub fn amendment(offer: &Admission, change: &Amendment, before: &Authority) -> Admission {
    match offer {
        Admission::Bounded { within, beyond: outside } => match within {
            Bounds::Amend { budget_left, grants, authority, ceiling } => {
                let Some(after) = &change.authority else {
                    return Admission::Allowed;
                };
                let narrowing = covers(before, after);
                if !narrowing && !covers(ceiling, after) {
                    return Admission::Refused(Refusal::NotOffered);
                }
                let added = after.budget.saturating_sub(before.budget);
                let mut listed = true;
                for grant in &after.grants {
                    if !before.grants.as_slice().contains(grant) {
                        listed &= grants.as_slice().contains(grant);
                    }
                }
                if narrowing || added <= *budget_left && listed && covers(authority, after) {
                    Admission::Allowed
                } else {
                    beyond(outside)
                }
            }
            Bounds::Goal { .. } => Admission::Refused(Refusal::NotOffered),
        },
        Admission::Allowed | Admission::Proposed { .. } | Admission::Refused(_) => offer.clone(),
    }
}

// Membership compares complete engine-listed choices; it interprets no resource pattern.
fn covers(facts: &Authority, draft: &Authority) -> bool {
    let mut fits = draft.budget <= facts.budget
        && draft.depth <= facts.depth
        && draft.tasks <= facts.tasks
        && deadline(draft.deadline, facts.deadline);
    for grant in &draft.grants {
        fits &= facts.grants.as_slice().contains(grant);
    }
    for kind in &draft.task_kinds {
        fits &= facts.task_kinds.as_slice().contains(kind);
    }
    for tool in &draft.tools {
        fits &= facts.tools.as_slice().contains(tool);
    }
    for scope in &draft.note_scopes {
        fits &= facts.note_scopes.as_slice().contains(scope);
    }
    fits
}
