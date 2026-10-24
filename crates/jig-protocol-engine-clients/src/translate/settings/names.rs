//! Translate stored choices and masks through the revision's own catalogue.
use super::{s, t, w};
use alloc::boxed::Box;
use jig_domain_authority::Models;
use skein_lib::List;

pub(super) fn live(models: &[crate::Model]) -> Box<[s::ModelName]> {
    let mut result = List::with_capacity(u32::try_from(models.len()).expect("configured model count"));
    for model in models {
        result.push(s::ModelName { name: model.name.clone(), efforts: model.efforts.clone() }).expect("model count");
    }
    result.into_boxed()
}
pub(super) fn authority(
    value: Models,
    models: &[s::ModelName],
    limits: &w::Limits,
) -> Result<Option<w::ModelSet>, w::Problem> {
    if value.is_all() {
        return Ok(Some(w::ModelSet::All));
    }
    let mut names = List::with_capacity(jig_domain_tasks::MODELS);
    for (at, model) in models.iter().enumerate() {
        if value.bits() & 1_u64.checked_shl(u32::try_from(at).expect("model bit")).unwrap_or(0) != 0 {
            names.push(model.name.clone()).expect("model count");
        }
    }
    let mut names = names.into_boxed();
    names.sort_unstable();
    let mut rows = List::with_capacity(u32::try_from(names.len()).expect("model count"));
    for name in names {
        rows.push(name).expect("model count");
    }
    Ok(Some(w::ModelSet::Names(w::ModelSetNames::new(limits, w::ModelSetNamesParts { names: rows })?)))
}
pub(super) fn choice(
    value: Option<&t::LlmChoice>,
    models: &[s::ModelName],
    limits: &w::Limits,
) -> Result<Option<w::LlmChoice>, w::Problem> {
    let Some(value) = value else { return Ok(None) };
    let mut subagents = List::with_capacity(u32::try_from(value.subagents.len()).expect("subagent count"));
    for row in &value.subagents {
        subagents.push(model_effort(*row, models, limits)?).expect("subagent count");
    }
    Ok(Some(w::LlmChoice::new(
        limits,
        w::LlmChoiceParts { account: value.account, main: model_effort(value.main, models, limits)?, subagents },
    )?))
}
fn model_effort(
    value: t::ModelEffort,
    models: &[s::ModelName],
    limits: &w::Limits,
) -> Result<w::ModelEffort, w::Problem> {
    let unknown = w::Problem { path: w::Path::ModelEffortModel, reason: skein_codec::Reason::Bound };
    let Some(model) = models.get(usize::from(value.model)) else { return Err(unknown) };
    let effort = match value.effort {
        Some(at) => match model.efforts.get(usize::from(at)) {
            Some(value) => Some(value.clone()),
            None => return Err(unknown),
        },
        None => None,
    };
    w::ModelEffort::new(limits, w::ModelEffortParts { model: model.name.clone(), effort })
}
