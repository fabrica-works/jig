//! A task's LLM choice and `me`'s models, by configured names (client/domain/contract.md, 3.2 and 5.1).
use super::{too_large, w};
use crate::Limits;
use jig_client_domain::contract as c;
use skein_lib::{List, bytes};

pub(crate) fn wire_choice(
    value: Option<&c::LlmChoice>,
    limits: &w::Limits,
) -> Result<Option<w::LlmChoice>, w::Problem> {
    let Some(value) = value else {
        return Ok(None);
    };
    let mut subagents = List::with_capacity(value.subagents.len());
    for item in &value.subagents {
        subagents.push(wire_model_effort(item, limits)?).expect("same capacity");
    }
    Ok(Some(w::LlmChoice::new(
        limits,
        w::LlmChoiceParts { account: value.account, main: wire_model_effort(&value.main, limits)?, subagents },
    )?))
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
fn wire_model_effort(value: &c::ModelEffort, limits: &w::Limits) -> Result<w::ModelEffort, w::Problem> {
    w::ModelEffort::new(
        limits,
        w::ModelEffortParts {
            model: bytes::copy_of(value.model.bytes()),
            effort: match &value.effort {
                Some(effort) => Some(bytes::copy_of(effort.bytes())),
                None => None,
            },
        },
    )
}
pub(crate) fn choice(value: Option<&w::LlmChoice>, limits: &Limits) -> Result<Option<c::LlmChoice>, w::Problem> {
    let Some(value) = value else {
        return Ok(None);
    };
    let mut subagents = List::with_capacity(value.subagents().len());
    for item in value.subagents() {
        subagents.push(model_effort(item, limits)?).expect("same capacity");
    }
    Ok(Some(c::LlmChoice { account: *value.account(), main: model_effort(value.main(), limits)?, subagents }))
}
fn model_effort(value: &w::ModelEffort, limits: &Limits) -> Result<c::ModelEffort, w::Problem> {
    Ok(c::ModelEffort {
        model: name(value.model(), limits.wire.model_effort_model)?,
        effort: match value.effort() {
            Some(effort) => Some(name(effort, limits.wire.model_effort_effort_some)?),
            None => None,
        },
    })
}
pub(crate) fn models(values: &List<w::MeModel>, limits: &Limits) -> Result<List<c::MeModel>, w::Problem> {
    let mut models = List::with_capacity(values.len());
    for value in values {
        let mut efforts = List::with_capacity(value.efforts().len());
        for effort in value.efforts() {
            efforts.push(name(effort, limits.wire.me_model_efforts_item)?).expect("same capacity");
        }
        models
            .push(c::MeModel {
                name: name(value.name(), limits.wire.me_model_name)?,
                provider: *value.provider(),
                window: value.window(),
                output: value.output(),
                efforts,
            })
            .expect("same capacity");
    }
    Ok(models)
}
pub(crate) fn refusal(value: &w::RefusalLlm, limits: &Limits) -> Result<c::Refusal, w::Problem> {
    Ok(c::Refusal::Llm {
        model: name(value.model(), limits.wire.refusal_llm_model)?,
        why: match value.why() {
            w::LlmWhy::Unconfigured => c::LlmWhy::Unconfigured,
            w::LlmWhy::Effort => c::LlmWhy::Effort,
            w::LlmWhy::Charter => c::LlmWhy::Charter,
            w::LlmWhy::Authority => c::LlmWhy::Authority,
            w::LlmWhy::Procedure => c::LlmWhy::Procedure,
            w::LlmWhy::Repeated => c::LlmWhy::Repeated,
        },
    })
}
fn name(value: &[u8], limit: u32) -> Result<c::Text, w::Problem> {
    match c::Text::new(bytes::copy_of(value), limit) {
        Ok(text) => Ok(text),
        Err(_) => Err(too_large()),
    }
}
