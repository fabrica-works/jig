//! Compare concrete drafts with the engine's numbers and lists.
//! The engine owns authorization; no role or policy is interpreted here.
//! See client/domain/pages.md, 6.2, and objects.md, 11.
use super::{
    Admission, Amendment, Beyond, Bounds, GoalExecutor, OwnerAuthority, OwnerAuthorityDocument, OwnerGrant,
    OwnerPattern, OwnerPatternEnd, Refusal,
};
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
pub fn amendment(offer: &Admission, change: &Amendment, before: &OwnerAuthorityDocument) -> Admission {
    match offer {
        Admission::Bounded { within, beyond: outside } => match within {
            Bounds::Amend { budget_left, grants, authority, ceiling } => {
                let Some(after) = &change.authority else {
                    return Admission::Allowed;
                };
                if before.cut {
                    return Admission::Refused(Refusal::NotOffered);
                }
                let before = &before.authority;
                let narrowing = covers(before, after);
                if !narrowing && (authority.cut || ceiling.cut || !covers(&ceiling.authority, after)) {
                    return Admission::Refused(Refusal::NotOffered);
                }
                let added = after.spend.saturating_sub(before.spend);
                let mut listed = true;
                for grant in &after.grants {
                    if !grant_covered(before.grants.as_slice(), grant) {
                        listed &= grant_covered(grants.as_slice(), grant);
                    }
                }
                if narrowing || added <= *budget_left && listed && covers(&authority.authority, after) {
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

// Compare the supplied whole scalar/delegation facts and literal pattern order.
// Connector kinds currently compare equal; configured implications require an owner fact.
// The supported form changes spend/deadline while preserving each literal permission.
fn covers(facts: &OwnerAuthority, draft: &OwnerAuthority) -> bool {
    let mut fits = draft.spend <= facts.spend
        && draft.delegation.depth <= facts.delegation.depth
        && draft.delegation.tasks <= facts.delegation.tasks
        && draft.tools & !facts.tools == 0
        && draft.notes & !facts.notes == 0
        && deadline(draft.deadline, facts.deadline);
    for grant in &draft.grants {
        fits &= grant_covered(facts.grants.as_slice(), grant);
    }
    for kind in &draft.delegation.kinds {
        fits &= facts.delegation.kinds.as_slice().contains(kind);
    }
    for scope in &draft.note_resources {
        let mut listed = false;
        for given in &facts.note_resources {
            listed |= given.connector == scope.connector && pattern_at_most(&scope.pattern, &given.pattern);
        }
        fits &= listed;
    }
    fits
}

fn grant_covered(facts: &[OwnerGrant], draft: &OwnerGrant) -> bool {
    for given in facts {
        if given.connector == draft.connector
            && given.kind == draft.kind
            && pattern_at_most(&draft.pattern, &given.pattern)
        {
            return true;
        }
    }
    false
}
// domain/authority.md, sections4–5: literal segment inclusion, with no path joining.
fn pattern_at_most(draft: &OwnerPattern, given: &OwnerPattern) -> bool {
    match &given.last {
        OwnerPatternEnd::Exact(last) => match &draft.last {
            OwnerPatternEnd::Exact(candidate) => draft.segments == given.segments && candidate == last,
            OwnerPatternEnd::Open(_) => false,
        },
        OwnerPatternEnd::Open(last) => {
            let prefix = given.segments.as_slice();
            let segments = draft.segments.as_slice();
            if segments.get(..prefix.len()) != Some(prefix) {
                return false;
            }
            if let Some(segment) = segments.get(prefix.len()) {
                return segment.starts_with(last);
            }
            match &draft.last {
                OwnerPatternEnd::Exact(segment) | OwnerPatternEnd::Open(segment) => segment.starts_with(last),
            }
        }
    }
}
