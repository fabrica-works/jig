//! Admission of live tracked tasks per project (domain/tasks.md, 11).
use crate::{Domain, Limits, Refusal};

pub(crate) fn room(domain: &Domain, project: u32, adding: u32, limits: &Limits) -> Result<(), Refusal> {
    let mut total = adding;
    for (_, id) in &domain.names {
        let row = &domain.tasks.get(*id).expect("name indexes live task").record;
        if row.project == project && row.tracked.is_some() {
            total = total.checked_add(1).ok_or(Refusal::Goals)?;
        }
    }
    if total > limits.project_goals {
        return Err(Refusal::Goals);
    }
    Ok(())
}
