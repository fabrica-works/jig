//! Goal admission from genuine whole Project offers; pages.md, 6.2.
//! The engine's bounded offer and Frame facts remain independent source values.
use crate::{Ui, page::project::goals::GoalForm};
use jig_client::contract::{ActKind, Admission, Bounds, Funder, GoalExecutor};
/// Preserve the original goal confirmation protocol without a fabricated Project.
pub(crate) fn prediction(
    domain: &Ui,
    form: GoalForm,
    scope: u64,
    lent: &jig_client::Client,
) -> Option<(Admission, u64)> {
    let doc = domain.project_page.owner.as_ref()?;
    if doc.number != form.project {
        return None;
    }
    let mut offer = None;
    for offered in &doc.acts {
        if offered.act == ActKind::SetGoal {
            offer = Some(&offered.admission);
        }
    }
    let offer = offer?;
    let bounded_pool = match offer {
        Admission::Bounded { within, .. } => match within {
            Bounds::Goal { pool_left, .. } => Some(*pool_left),
            Bounds::Amend { .. } => None,
        },
        Admission::Allowed | Admission::Proposed { .. } | Admission::Refused(_) => None,
    };
    if let Some(pool_left) = bounded_pool {
        return Some((
            jig_client::contract::admission::goal(offer, form.executor, form.budget, form.deadline),
            pool_left,
        ));
    }
    let mut pool = None;
    let mut listed = false;
    if let Some(frame) = &domain.frame(lent).document {
        for project in &frame.projects {
            if project.project == form.project
                && let Some(ledger) = &project.pool
            {
                pool = Some(ledger.left);
            }
        }
        for project in &frame.charters {
            if project.project == form.project {
                match form.executor {
                    GoalExecutor::Charter(number) => {
                        for charter in &project.goals {
                            if charter.number == number {
                                listed = true;
                            }
                        }
                    }
                    GoalExecutor::Procedure { .. } => {}
                }
            }
        }
    }
    if !doc.configured_pools_cut {
        for ledger in &doc.configured_pools {
            match ledger.source {
                Funder::Pool { project, person, period }
                    if project == doc.number && person == scope && period == doc.configured_period =>
                {
                    pool = Some(ledger.left);
                }
                Funder::Task(_) | Funder::Pool { .. } | Funder::Period { .. } | Funder::Recurring { .. } => {}
            }
        }
    }
    let pool = pool?;
    match offer {
        Admission::Allowed | Admission::Proposed { .. } if !listed => {
            Some((Admission::Refused(jig_client::contract::Refusal::NotOffered), pool))
        }
        Admission::Allowed | Admission::Proposed { .. } | Admission::Refused(_) | Admission::Bounded { .. } => {
            Some((jig_client::contract::admission::goal(offer, form.executor, form.budget, form.deadline), pool))
        }
    }
}
