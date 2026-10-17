//! Whole-value mappings for infrastructure/native.
use crate::infrastructure::native as d;
use crate::wire::{Invalid, w};
use skein_lib::List;
pub(crate) fn value_encode(value: &d::Value) -> Result<w::InfrastructureNativeValue, Invalid> {
    Ok(match value {
        d::Value::Environment { environment, end_revision, fact, end, pool } => {
            let parts = w::InfrastructureNativeValueEnvironmentParts {
                environment: crate::wire::infrastructure_native::environment_encode(environment)?,
                end_revision: (*end_revision),
                fact: crate::wire::infrastructure_native::environment_fact_encode(fact)?,
                end: match end {
                    Some(item) => Some(crate::wire::infrastructure_native::environment_end_encode(item)?),
                    None => None,
                },
                pool: match pool {
                    Some(item) => Some(crate::wire::infrastructure_native::client_pool_encode(*item)?),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureNativeValueEnvironment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeValue::Environment(value)
        }
        d::Value::Panel { task, state, resources, effects } => {
            let parts = w::InfrastructureNativeValuePanelParts {
                task: (*task),
                state: crate::wire::infrastructure_native::procedure_state_encode(state)?,
                resources: resources_encode(resources)?,
                effects: effects_encode(effects)?,
            };
            let Ok(value) = w::InfrastructureNativeValuePanel::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureNativeValue::Panel(value)
        }
        d::Value::Result { task, state, resources } => {
            let parts = w::InfrastructureNativeValueResultParts {
                task: (*task),
                state: crate::wire::infrastructure_native::procedure_state_encode(state)?,
                resources: resources_encode(resources)?,
            };
            let Ok(value) = w::InfrastructureNativeValueResult::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureNativeValue::Result(value)
        }
        d::Value::Effect { proposal, task, effect, description } => {
            let parts = w::InfrastructureNativeValueEffectParts {
                proposal: (*proposal),
                task: (*task),
                effect: crate::wire::infrastructure_native::effect_encode(effect)?,
                description: crate::wire::infrastructure_native::description_encode(description)?,
            };
            let Ok(value) = w::InfrastructureNativeValueEffect::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureNativeValue::Effect(value)
        }
        d::Value::Outbox { entry, description } => {
            let parts = w::InfrastructureNativeValueOutboxParts {
                entry: crate::wire::infrastructure_native::entry_encode(entry)?,
                description: crate::wire::infrastructure_native::description_encode(description)?,
            };
            let Ok(value) = w::InfrastructureNativeValueOutbox::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureNativeValue::Outbox(value)
        }
        d::Value::Hold { task, resources, drift, effects } => {
            let parts = w::InfrastructureNativeValueHoldParts {
                task: (*task),
                resources: resources_encode(resources)?,
                drift: drift_encode(drift)?,
                effects: effects_encode(effects)?,
            };
            let Ok(value) = w::InfrastructureNativeValueHold::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureNativeValue::Hold(value)
        }
        d::Value::Project { project, resources, environments } => {
            let parts = w::InfrastructureNativeValueProjectParts {
                project: (*project),
                resources: resources_encode(resources)?,
                environments: environments_encode(environments)?,
            };
            let Ok(value) = w::InfrastructureNativeValueProject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureNativeValue::Project(value)
        }
        d::Value::Deployment { backend, effects, made, proposal_limit, staged_limit } => {
            let parts = w::InfrastructureNativeValueDeploymentParts {
                backend: crate::wire::infrastructure_native::backend_encode(*backend),
                effects: effects_encode(effects)?,
                made: (*made),
                proposal_limit: (*proposal_limit),
                staged_limit: (*staged_limit),
            };
            let Ok(value) = w::InfrastructureNativeValueDeployment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeValue::Deployment(value)
        }
    })
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn value_decode(value: w::InfrastructureNativeValue) -> d::Value {
    match value {
        w::InfrastructureNativeValue::Environment(value) => {
            let value = value.into_parts();
            d::Value::Environment {
                environment: crate::wire::infrastructure_native::environment_decode(value.environment),
                end_revision: value.end_revision,
                fact: crate::wire::infrastructure_native::environment_fact_decode(value.fact),
                end: match value.end {
                    Some(item) => Some(crate::wire::infrastructure_native::environment_end_decode(item)),
                    None => None,
                },
                pool: match value.pool {
                    Some(item) => Some(crate::wire::infrastructure_native::client_pool_decode(item)),
                    None => None,
                },
            }
        }
        w::InfrastructureNativeValue::Panel(value) => {
            let value = value.into_parts();
            d::Value::Panel {
                task: value.task,
                state: crate::wire::infrastructure_native::procedure_state_decode(value.state),
                resources: resources_decode(value.resources),
                effects: effects_decode(value.effects),
            }
        }
        w::InfrastructureNativeValue::Result(value) => {
            let value = value.into_parts();
            d::Value::Result {
                task: value.task,
                state: crate::wire::infrastructure_native::procedure_state_decode(value.state),
                resources: resources_decode(value.resources),
            }
        }
        w::InfrastructureNativeValue::Effect(value) => {
            let value = value.into_parts();
            d::Value::Effect {
                proposal: value.proposal,
                task: value.task,
                effect: crate::wire::infrastructure_native::effect_decode(value.effect),
                description: crate::wire::infrastructure_native::description_decode(value.description),
            }
        }
        w::InfrastructureNativeValue::Outbox(value) => {
            let value = value.into_parts();
            d::Value::Outbox {
                entry: crate::wire::infrastructure_native::entry_decode(value.entry),
                description: crate::wire::infrastructure_native::description_decode(value.description),
            }
        }
        w::InfrastructureNativeValue::Hold(value) => {
            let value = value.into_parts();
            d::Value::Hold {
                task: value.task,
                resources: resources_decode(value.resources),
                drift: drift_decode(value.drift),
                effects: effects_decode(value.effects),
            }
        }
        w::InfrastructureNativeValue::Project(value) => {
            let value = value.into_parts();
            d::Value::Project {
                project: value.project,
                resources: resources_decode(value.resources),
                environments: environments_decode(value.environments),
            }
        }
        w::InfrastructureNativeValue::Deployment(value) => {
            let value = value.into_parts();
            d::Value::Deployment {
                backend: crate::wire::infrastructure_native::backend_decode(value.backend),
                effects: effects_decode(value.effects),
                made: value.made,
                proposal_limit: value.proposal_limit,
                staged_limit: value.staged_limit,
            }
        }
    }
}

pub(crate) fn document_encode(value: &d::Document) -> Result<w::InfrastructureNativeDocument, Invalid> {
    let parts = w::InfrastructureNativeDocumentParts {
        subject: crate::wire::infrastructure_documents::subject_encode(&(value.subject))?,
        value: value_encode(&(value.value))?,
    };
    let Ok(value) = w::InfrastructureNativeDocument::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn document_decode(value: w::InfrastructureNativeDocument) -> d::Document {
    let value = value.into_parts();
    d::Document {
        subject: crate::wire::infrastructure_documents::subject_decode(value.subject),
        value: value_decode(value.value),
    }
}

fn resources_encode(source: &List<d::ClientResource>) -> Result<List<w::InfrastructureNativeClientResource>, Invalid> {
    let mut values = List::with_capacity(source.capacity());
    for item in source {
        let value = crate::wire::infrastructure_native::client_resource_encode(item)?;
        if values.push(value).is_err() {
            return Err(Invalid);
        }
    }
    Ok(values)
}

fn resources_decode(source: List<w::InfrastructureNativeClientResource>) -> List<d::ClientResource> {
    let mut values = List::with_capacity(source.capacity());
    for item in source.into_boxed() {
        let value = crate::wire::infrastructure_native::client_resource_decode(item);
        assert!(values.push(value).is_ok(), "same list capacity");
    }
    values
}

fn effects_encode(source: &List<d::Entry>) -> Result<List<w::InfrastructureNativeEntry>, Invalid> {
    let mut values = List::with_capacity(source.capacity());
    for item in source {
        let value = crate::wire::infrastructure_native::entry_encode(item)?;
        if values.push(value).is_err() {
            return Err(Invalid);
        }
    }
    Ok(values)
}

fn effects_decode(source: List<w::InfrastructureNativeEntry>) -> List<d::Entry> {
    let mut values = List::with_capacity(source.capacity());
    for item in source.into_boxed() {
        let value = crate::wire::infrastructure_native::entry_decode(item);
        assert!(values.push(value).is_ok(), "same list capacity");
    }
    values
}

fn drift_encode(source: &List<d::DriftAccount>) -> Result<List<w::InfrastructureNativeDriftAccount>, Invalid> {
    let mut values = List::with_capacity(source.capacity());
    for item in source {
        let value = crate::wire::infrastructure_native::drift_account_encode(item)?;
        if values.push(value).is_err() {
            return Err(Invalid);
        }
    }
    Ok(values)
}

fn drift_decode(source: List<w::InfrastructureNativeDriftAccount>) -> List<d::DriftAccount> {
    let mut values = List::with_capacity(source.capacity());
    for item in source.into_boxed() {
        let value = crate::wire::infrastructure_native::drift_account_decode(item);
        assert!(values.push(value).is_ok(), "same list capacity");
    }
    values
}

fn environments_encode(
    source: &List<d::ClientEnvironment>,
) -> Result<List<w::InfrastructureNativeClientEnvironment>, Invalid> {
    let mut values = List::with_capacity(source.capacity());
    for item in source {
        let value = crate::wire::infrastructure_native::client_environment_encode(item)?;
        if values.push(value).is_err() {
            return Err(Invalid);
        }
    }
    Ok(values)
}

fn environments_decode(source: List<w::InfrastructureNativeClientEnvironment>) -> List<d::ClientEnvironment> {
    let mut values = List::with_capacity(source.capacity());
    for item in source.into_boxed() {
        let value = crate::wire::infrastructure_native::client_environment_decode(item);
        assert!(values.push(value).is_ok(), "same list capacity");
    }
    values
}
