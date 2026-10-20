//! A task's LLM choice and `me`'s models: the core names a model and its effort by index in
//! configuration's order, the client by name (contract.md, 3.2 and 5.1; domain/tasks.md, 3).
use super::w;
use crate::Model;
use jig_domain_authority::Models;
use jig_domain_tasks as t;
use skein_lib::{List, bytes};

/// Whether the configured models fit the core's indices and the wire's bounds.
pub(crate) fn fits(models: &[Model], limits: &w::Limits) -> bool {
    if models.len() > usize::try_from(t::MODELS).expect("u32 fits usize")
        || models.len() > usize::try_from(limits.me_project_models).expect("u32 fits usize")
    {
        return false;
    }
    for (at, model) in models.iter().enumerate() {
        let name = model.name.len();
        if model.name.is_empty()
            || !skein_codec::text_is_valid(&model.name)
            || name > usize::try_from(jig_domain_core::MODEL_NAME_BYTES).expect("u32 fits usize")
            || name > usize::try_from(limits.me_model_name).expect("u32 fits usize")
            || name > usize::try_from(limits.model_effort_model).expect("u32 fits usize")
            || name > usize::try_from(limits.refusal_llm_model).expect("u32 fits usize")
            || model.efforts.len() > usize::try_from(jig_domain_core::EFFORTS).expect("u32 fits usize")
            || model.efforts.len() > usize::try_from(limits.me_model_efforts).expect("u32 fits usize")
        {
            return false;
        }
        for earlier in models.iter().take(at) {
            if earlier.name == model.name {
                return false;
            }
        }
        for effort in &model.efforts {
            if effort.is_empty()
                || !skein_codec::text_is_valid(effort)
                || effort.len() > usize::try_from(jig_domain_core::EFFORT_BYTES).expect("u32 fits usize")
                || effort.len() > usize::try_from(limits.me_model_efforts_item).expect("u32 fits usize")
                || effort.len() > usize::try_from(limits.model_effort_effort_some).expect("u32 fits usize")
            {
                return false;
            }
        }
    }
    true
}

/// The models `allowed` names, in configuration's order, as `me` shows them.
pub fn models(allowed: Models, models: &[Model], limits: &w::Limits) -> Result<List<w::MeModel>, w::Problem> {
    let mut rows = List::with_capacity(u32::try_from(models.len()).expect("checked models"));
    for (at, model) in models.iter().enumerate() {
        if allowed.0 & (1_u64 << at) == 0 {
            continue;
        }
        let mut efforts = List::with_capacity(u32::try_from(model.efforts.len()).expect("checked efforts"));
        for effort in &model.efforts {
            efforts.push(bytes::copy_of(effort)).expect("same capacity");
        }
        rows.push(w::MeModel::new(
            limits,
            w::MeModelParts {
                name: bytes::copy_of(&model.name),
                provider: model.provider,
                window: model.window,
                output: model.output,
                efforts,
            },
        )?)
        .expect("same capacity");
    }
    Ok(rows)
}

/// A kept choice by its names; a model or effort past configuration is a problem, as the core
/// keeps only configured ones.
pub fn choice(
    value: Option<&t::LlmChoice>,
    models: &[Model],
    limits: &w::Limits,
) -> Result<Option<w::LlmChoice>, w::Problem> {
    let Some(value) = value else {
        return Ok(None);
    };
    let mut subagents = List::with_capacity(u32::try_from(value.subagents.len()).expect("bounded sub-agents"));
    for item in &value.subagents {
        subagents.push(model_effort(*item, models, limits)?).expect("same capacity");
    }
    Ok(Some(w::LlmChoice::new(
        limits,
        w::LlmChoiceParts { main: model_effort(value.main, models, limits)?, subagents },
    )?))
}
fn model_effort(value: t::ModelEffort, models: &[Model], limits: &w::Limits) -> Result<w::ModelEffort, w::Problem> {
    let unknown = w::Problem { path: w::Path::ModelEffortModel, reason: skein_codec::Reason::Bound };
    let Some(model) = models.get(usize::from(value.model)) else {
        return Err(unknown);
    };
    let effort = match value.effort {
        Some(effort) => match model.efforts.get(usize::from(effort)) {
            Some(name) => Some(bytes::copy_of(name)),
            None => return Err(unknown),
        },
        None => None,
    };
    w::ModelEffort::new(limits, w::ModelEffortParts { model: bytes::copy_of(&model.name), effort })
}
