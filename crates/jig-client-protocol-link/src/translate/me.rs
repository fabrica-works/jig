//! Nested served me records, bounded by the receiving document.
use super::{Limits, List, Reader, c, piece_problem, project_role_fields, task_contract_document_fields, too_large, w};
pub(crate) fn projects(bytes: &[u8], limits: &Limits) -> Result<List<c::MeProject>, w::Problem> {
    let bound = limits.client.projects;
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::MeProject::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = project_role_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn templates(bytes: &[u8], limits: &Limits) -> Result<List<c::Template>, w::Problem> {
    let bound = limits.client.rows;
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::Template::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = template(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn template(value: &w::Template, limits: &Limits) -> Result<c::Template, w::Problem> {
    Ok(c::Template {
        number: value.number(),
        name: crate::translate::facts::document_text(value.name(), limits)?,
        executor: crate::translate::operations::restore_executor(value.executor()),
        contract: task_contract_document_fields(value.contract(), limits)?,
        shallower: value.shallower(),
        budget: crate::translate::operations::restore_budget(value.budget()),
        deadline: crate::translate::operations::restore_deadline(value.deadline()),
        tracked: *value.tracked(),
        flags: c::TaskFlags {
            conversation: value.flags().conversation(),
            listed: value.flags().listed(),
            retention: match value.flags().retention() {
                w::Retention::Kept => c::Retention::Kept,
                w::Retention::ForgottenOnceRead => c::Retention::ForgottenOnceRead,
            },
        },
        label: value.label(),
        grace: *value.grace(),
        llm: crate::translate::llm::choice(value.llm().as_ref(), limits)?,
    })
}
pub(crate) fn overrides(
    values: &List<w::TemplateOverride>,
    limits: &Limits,
) -> Result<List<c::TemplateOverride>, w::Problem> {
    if values.len() > limits.client.rows {
        return Err(too_large());
    }
    Ok(crate::translate::operations::restore_template_overrides(values))
}
