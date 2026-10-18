//! A board move is one confirmed compound of priority-only amendments (pages.md, 6.2).
use crate::{Ui, objects::Object};
use alloc::boxed::Box;
use jig_client::contract::{self as c, ActKind, Admission, Ask, ObjectKey};
use skein_lib::{Env, List};

pub(crate) fn admission(domain: &Ui) -> Admission {
    let rows = match (&domain.project_page.owner, &domain.project_page.document) {
        (Some(project), _) => &project.acts,
        (_, Some(project)) => &project.acts,
        _ => return Admission::Refused(c::Refusal::NotOffered),
    };
    jig_client::facts::offer(rows.as_slice(), ActKind::Prioritise)
        .cloned()
        .unwrap_or(Admission::Refused(c::Refusal::NotOffered))
}
pub(crate) fn index(rows: &List<c::TaskLine>, task: u64) -> Option<u32> {
    for (index, row) in rows.as_slice().iter().enumerate() {
        if row.chip.task == task {
            return u32::try_from(index).ok();
        }
    }
    None
}
pub(crate) fn open(domain: &mut Ui, env: &Env<crate::Limits>, task: u64, up: bool, lent: &jig_client::Client) {
    let rows = domain.borrow(lent).project_page().goals;
    let Some(index) = index(&rows, task) else { return };
    let target = if up { index.checked_sub(1) } else { index.checked_add(1) };
    let Some(target) = target else { return };
    if target >= rows.len() {
        return;
    }
    let Some(id) = domain.objects.find(ObjectKey::Task(task)) else { return };
    crate::confirm::open(domain, env, id, ActKind::Prioritise, lent);
    crate::confirm::priority(domain, target);
    crate::confirm::sync(domain, lent);
}
pub(crate) fn ask(domain: &Ui, object: &Object, target: u32, lent: &jig_client::Client) -> Option<Ask> {
    let ObjectKey::Task(task) = object.key else { return None };
    let project = domain.shown_project()?;
    let rows = domain.borrow(lent).project_page().goals;
    let source = index(&rows, task)?;
    if target >= rows.len() {
        return None;
    }
    let mut operations = List::with_capacity(rows.len());
    for (index, row) in rows.as_slice().iter().enumerate() {
        let index = u32::try_from(index).ok()?;
        let place = if index == source {
            target
        } else if source < target && index > source && index <= target {
            index.checked_sub(1)?
        } else if source > target && index >= target && index < source {
            index.checked_add(1)?
        } else {
            index
        };
        let priority = place;
        if row.tracked != Some(priority) {
            operations
                .push(c::Operation::Amend {
                    project,
                    task: row.chip.task,
                    revision: None,
                    change: Box::new(c::Amendment {
                        addressed: None,
                        spec: None,
                        wake: None,
                        dependencies: None,
                        authority: None,
                        instructions: None,
                        tracked: Some(c::Tracking::Tracked { priority }),
                    }),
                    reason: c::Text::new(Box::from([]), 0).expect("empty reason fits"),
                })
                .ok()?;
        }
    }
    Some(Ask::Compound { project, operations })
}
