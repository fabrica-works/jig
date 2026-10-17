//! Whole-value mappings for infrastructure/documents.
use crate::infrastructure::documents as d;
use crate::wire::{Invalid, w};
pub(crate) fn procedure_encode(value: &d::Procedure) -> Result<w::InfrastructureProcedure, Invalid> {
    Ok(match value {
        d::Procedure::Remediate { service, operation, deadline } => {
            let parts = w::InfrastructureProcedureRemediateParts {
                service: crate::wire::infrastructure_documents::service_encode(service)?,
                operation: (*operation),
                deadline: (*deadline).as_nanos(),
            };
            let Ok(value) = w::InfrastructureProcedureRemediate::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureProcedure::Remediate(value)
        }
        d::Procedure::Scale { service, replicas, deadline } => {
            let parts = w::InfrastructureProcedureScaleParts {
                service: crate::wire::infrastructure_documents::service_encode(service)?,
                replicas: (*replicas),
                deadline: (*deadline).as_nanos(),
            };
            let Ok(value) = w::InfrastructureProcedureScale::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureProcedure::Scale(value)
        }
        d::Procedure::Provision { environment, until, price, deadline } => {
            let parts = w::InfrastructureProcedureProvisionParts {
                environment: (*environment),
                until: (*until).as_nanos(),
                price: (*price),
                deadline: (*deadline).as_nanos(),
            };
            let Ok(value) = w::InfrastructureProcedureProvision::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureProcedure::Provision(value)
        }
        d::Procedure::TearDown { environment, created_by, deadline } => {
            let parts = w::InfrastructureProcedureTearDownParts {
                environment: (*environment),
                created_by: crate::wire::infrastructure_documents::creation_encode(created_by)?,
                deadline: (*deadline).as_nanos(),
            };
            let Ok(value) = w::InfrastructureProcedureTearDown::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureProcedure::TearDown(value)
        }
    })
}
pub(crate) fn procedure_decode(value: w::InfrastructureProcedure) -> d::Procedure {
    match value {
        w::InfrastructureProcedure::Remediate(value) => {
            let value = value.into_parts();
            d::Procedure::Remediate {
                service: crate::wire::infrastructure_documents::service_decode(value.service),
                operation: value.operation,
                deadline: skein_lib::Wall::from_nanos(value.deadline),
            }
        }
        w::InfrastructureProcedure::Scale(value) => {
            let value = value.into_parts();
            d::Procedure::Scale {
                service: crate::wire::infrastructure_documents::service_decode(value.service),
                replicas: value.replicas,
                deadline: skein_lib::Wall::from_nanos(value.deadline),
            }
        }
        w::InfrastructureProcedure::Provision(value) => {
            let value = value.into_parts();
            d::Procedure::Provision {
                environment: value.environment,
                until: skein_lib::Wall::from_nanos(value.until),
                price: value.price,
                deadline: skein_lib::Wall::from_nanos(value.deadline),
            }
        }
        w::InfrastructureProcedure::TearDown(value) => {
            let value = value.into_parts();
            d::Procedure::TearDown {
                environment: value.environment,
                created_by: crate::wire::infrastructure_documents::creation_decode(value.created_by),
                deadline: skein_lib::Wall::from_nanos(value.deadline),
            }
        }
    }
}

pub(crate) fn panel_encode(value: &d::Panel) -> Result<w::InfrastructurePanel, Invalid> {
    let parts = w::InfrastructurePanelParts {
        task: (value.task),
        procedure: procedure_encode(&(value.procedure))?,
        phase: crate::wire::infrastructure_documents::phase_encode(value.phase),
    };
    let Ok(value) = w::InfrastructurePanel::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn panel_decode(value: w::InfrastructurePanel) -> d::Panel {
    let value = value.into_parts();
    d::Panel {
        task: value.task,
        procedure: procedure_decode(value.procedure),
        phase: crate::wire::infrastructure_documents::phase_decode(value.phase),
    }
}

pub(crate) fn resource_encode(value: &d::Resource) -> Result<w::InfrastructureResource, Invalid> {
    let parts = w::InfrastructureResourceParts {
        name: (value.name).clone(),
        role: resource_role_encode(value.role),
        fact: resource_fact_encode(&(value.fact))?,
    };
    let Ok(value) = w::InfrastructureResource::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn resource_decode(value: w::InfrastructureResource) -> d::Resource {
    let value = value.into_parts();
    d::Resource { name: value.name, role: resource_role_decode(value.role), fact: resource_fact_decode(value.fact) }
}

pub(crate) fn resource_role_encode(value: d::ResourceRole) -> w::InfrastructureResourceRole {
    match value {
        d::ResourceRole::Owned => w::InfrastructureResourceRole::Owned,
        d::ResourceRole::Participating => w::InfrastructureResourceRole::Participating,
        d::ResourceRole::Context => w::InfrastructureResourceRole::Context,
    }
}
pub(crate) fn resource_role_decode(value: w::InfrastructureResourceRole) -> d::ResourceRole {
    match value {
        w::InfrastructureResourceRole::Owned => d::ResourceRole::Owned,
        w::InfrastructureResourceRole::Participating => d::ResourceRole::Participating,
        w::InfrastructureResourceRole::Context => d::ResourceRole::Context,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn resource_fact_encode(value: &d::ResourceFact) -> Result<w::InfrastructureResourceFact, Invalid> {
    Ok(match value {
        d::ResourceFact::Service { version, replicas, healthy, revision, observed } => {
            let parts = w::InfrastructureResourceFactServiceParts {
                version: match version {
                    Some(item) => Some((*item).clone()),
                    None => None,
                },
                replicas: match replicas {
                    Some(item) => Some(*item),
                    None => None,
                },
                healthy: match healthy {
                    Some(item) => Some(*item),
                    None => None,
                },
                revision: match revision {
                    Some(item) => Some(*item),
                    None => None,
                },
                observed: match observed {
                    Some(item) => Some((*item).as_nanos()),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureResourceFactService::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureResourceFact::Service(value)
        }
        d::ResourceFact::Environment(value) => {
            let parts = w::InfrastructureResourceFactEnvironmentParts {
                value: match value {
                    Some(item) => Some(crate::wire::infrastructure_documents::environment_encode(item)?),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureResourceFactEnvironment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureResourceFact::Environment(value)
        }
        d::ResourceFact::Pool(value) => {
            let parts = w::InfrastructureResourceFactPoolParts {
                value: match value {
                    Some(item) => Some(crate::wire::infrastructure_documents::pool_encode(item)?),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureResourceFactPool::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureResourceFact::Pool(value)
        }
        d::ResourceFact::Quota(value) => {
            let parts = w::InfrastructureResourceFactQuotaParts {
                value: match value {
                    Some(item) => Some(crate::wire::infrastructure_documents::pool_encode(item)?),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureResourceFactQuota::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureResourceFact::Quota(value)
        }
    })
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn resource_fact_decode(value: w::InfrastructureResourceFact) -> d::ResourceFact {
    match value {
        w::InfrastructureResourceFact::Service(value) => {
            let value = value.into_parts();
            d::ResourceFact::Service {
                version: match value.version {
                    Some(item) => Some(item),
                    None => None,
                },
                replicas: match value.replicas {
                    Some(item) => Some(item),
                    None => None,
                },
                healthy: match value.healthy {
                    Some(item) => Some(item),
                    None => None,
                },
                revision: match value.revision {
                    Some(item) => Some(item),
                    None => None,
                },
                observed: match value.observed {
                    Some(item) => Some(skein_lib::Wall::from_nanos(item)),
                    None => None,
                },
            }
        }
        w::InfrastructureResourceFact::Environment(value) => {
            let value = value.into_parts();
            d::ResourceFact::Environment(match value.value {
                Some(item) => Some(crate::wire::infrastructure_documents::environment_decode(item)),
                None => None,
            })
        }
        w::InfrastructureResourceFact::Pool(value) => {
            let value = value.into_parts();
            d::ResourceFact::Pool(match value.value {
                Some(item) => Some(crate::wire::infrastructure_documents::pool_decode(item)),
                None => None,
            })
        }
        w::InfrastructureResourceFact::Quota(value) => {
            let value = value.into_parts();
            d::ResourceFact::Quota(match value.value {
                Some(item) => Some(crate::wire::infrastructure_documents::pool_decode(item)),
                None => None,
            })
        }
    }
}

pub(crate) fn recovery_encode(value: d::Recovery) -> w::InfrastructureRecovery {
    match value {
        d::Recovery::Keyed => w::InfrastructureRecovery::Keyed,
        d::Recovery::Conditional => w::InfrastructureRecovery::Conditional,
        d::Recovery::Unrecoverable => w::InfrastructureRecovery::Unrecoverable,
    }
}
pub(crate) fn recovery_decode(value: w::InfrastructureRecovery) -> d::Recovery {
    match value {
        w::InfrastructureRecovery::Keyed => d::Recovery::Keyed,
        w::InfrastructureRecovery::Conditional => d::Recovery::Conditional,
        w::InfrastructureRecovery::Unrecoverable => d::Recovery::Unrecoverable,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn description_encode(value: &d::Description) -> Result<w::InfrastructureDescription, Invalid> {
    let parts = w::InfrastructureDescriptionParts {
        words: (value.words).clone(),
        resource: (value.resource).clone(),
        kind: (value.kind),
        recovery: recovery_encode(value.recovery),
        price: match &(value.price) {
            Some(item) => Some(*item),
            None => None,
        },
        condition: (value.condition),
        form: form_encode(value.form),
        state: (value.state),
    };
    let Ok(value) = w::InfrastructureDescription::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn description_decode(value: w::InfrastructureDescription) -> d::Description {
    let value = value.into_parts();
    d::Description {
        words: value.words,
        resource: value.resource,
        kind: value.kind,
        recovery: recovery_decode(value.recovery),
        price: match value.price {
            Some(item) => Some(item),
            None => None,
        },
        condition: value.condition,
        form: form_decode(value.form),
        state: value.state,
    }
}

pub(crate) fn form_encode(value: d::Form) -> w::InfrastructureForm {
    match value {
        d::Form::Creation => w::InfrastructureForm::Creation,
        d::Form::Transition => w::InfrastructureForm::Transition,
        d::Form::Set => w::InfrastructureForm::Set,
    }
}
pub(crate) fn form_decode(value: w::InfrastructureForm) -> d::Form {
    match value {
        w::InfrastructureForm::Creation => d::Form::Creation,
        w::InfrastructureForm::Transition => d::Form::Transition,
        w::InfrastructureForm::Set => d::Form::Set,
    }
}

pub(crate) fn effect_action_encode(value: &d::EffectAction) -> Result<w::InfrastructureEffectAction, Invalid> {
    Ok(match value {
        d::EffectAction::Restart { service, operation } => {
            let parts = w::InfrastructureEffectActionRestartParts {
                service: crate::wire::infrastructure_documents::service_encode(service)?,
                operation: (*operation),
            };
            let Ok(value) = w::InfrastructureEffectActionRestart::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureEffectAction::Restart(value)
        }
        d::EffectAction::Scale { service, from, to } => {
            let parts = w::InfrastructureEffectActionScaleParts {
                service: crate::wire::infrastructure_documents::service_encode(service)?,
                from: (*from),
                to: (*to),
            };
            let Ok(value) = w::InfrastructureEffectActionScale::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureEffectAction::Scale(value)
        }
        d::EffectAction::Rollback { service, from, to } => {
            let parts = w::InfrastructureEffectActionRollbackParts {
                service: crate::wire::infrastructure_documents::service_encode(service)?,
                from: (*from).clone(),
                to: (*to).clone(),
            };
            let Ok(value) = w::InfrastructureEffectActionRollback::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureEffectAction::Rollback(value)
        }
        d::EffectAction::CreateEnvironment { environment, until, price } => {
            let parts = w::InfrastructureEffectActionCreateEnvironmentParts {
                environment: (*environment),
                until: (*until).as_nanos(),
                price: (*price),
            };
            let Ok(value) = w::InfrastructureEffectActionCreateEnvironment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureEffectAction::CreateEnvironment(value)
        }
        d::EffectAction::TearDown { environment, created_by } => {
            let parts = w::InfrastructureEffectActionTearDownParts {
                environment: (*environment),
                created_by: crate::wire::infrastructure_documents::creation_encode(created_by)?,
            };
            let Ok(value) = w::InfrastructureEffectActionTearDown::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureEffectAction::TearDown(value)
        }
    })
}
pub(crate) fn effect_action_decode(value: w::InfrastructureEffectAction) -> d::EffectAction {
    match value {
        w::InfrastructureEffectAction::Restart(value) => {
            let value = value.into_parts();
            d::EffectAction::Restart {
                service: crate::wire::infrastructure_documents::service_decode(value.service),
                operation: value.operation,
            }
        }
        w::InfrastructureEffectAction::Scale(value) => {
            let value = value.into_parts();
            d::EffectAction::Scale {
                service: crate::wire::infrastructure_documents::service_decode(value.service),
                from: value.from,
                to: value.to,
            }
        }
        w::InfrastructureEffectAction::Rollback(value) => {
            let value = value.into_parts();
            d::EffectAction::Rollback {
                service: crate::wire::infrastructure_documents::service_decode(value.service),
                from: value.from,
                to: value.to,
            }
        }
        w::InfrastructureEffectAction::CreateEnvironment(value) => {
            let value = value.into_parts();
            d::EffectAction::CreateEnvironment {
                environment: value.environment,
                until: skein_lib::Wall::from_nanos(value.until),
                price: value.price,
            }
        }
        w::InfrastructureEffectAction::TearDown(value) => {
            let value = value.into_parts();
            d::EffectAction::TearDown {
                environment: value.environment,
                created_by: crate::wire::infrastructure_documents::creation_decode(value.created_by),
            }
        }
    }
}

pub(crate) fn effect_state_encode(value: d::EffectState) -> w::InfrastructureEffectState {
    match value {
        d::EffectState::Kept => w::InfrastructureEffectState::Kept,
        d::EffectState::Sent => w::InfrastructureEffectState::Sent,
        d::EffectState::Uncertain => w::InfrastructureEffectState::Uncertain,
        d::EffectState::Held => w::InfrastructureEffectState::Held,
    }
}
pub(crate) fn effect_state_decode(value: w::InfrastructureEffectState) -> d::EffectState {
    match value {
        w::InfrastructureEffectState::Kept => d::EffectState::Kept,
        w::InfrastructureEffectState::Sent => d::EffectState::Sent,
        w::InfrastructureEffectState::Uncertain => d::EffectState::Uncertain,
        w::InfrastructureEffectState::Held => d::EffectState::Held,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn effect_encode(value: &d::Effect) -> Result<w::InfrastructureEffect, Invalid> {
    let parts = w::InfrastructureEffectParts {
        entry: (value.entry),
        task: (value.task),
        project: match &(value.project) {
            Some(item) => Some(*item),
            None => None,
        },
        origin: crate::wire::infrastructure_documents::creation_encode(&(value.origin))?,
        description: match &(value.description) {
            Some(item) => Some(description_encode(item)?),
            None => None,
        },
        state: effect_state_encode(value.state),
        attempt: (value.attempt),
        deadline: (value.deadline).as_nanos(),
        action: effect_action_encode(&(value.action))?,
    };
    let Ok(value) = w::InfrastructureEffect::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn effect_decode(value: w::InfrastructureEffect) -> d::Effect {
    let value = value.into_parts();
    d::Effect {
        entry: value.entry,
        task: value.task,
        project: match value.project {
            Some(item) => Some(item),
            None => None,
        },
        origin: crate::wire::infrastructure_documents::creation_decode(value.origin),
        description: match value.description {
            Some(item) => Some(description_decode(item)),
            None => None,
        },
        state: effect_state_decode(value.state),
        attempt: value.attempt,
        deadline: skein_lib::Wall::from_nanos(value.deadline),
        action: effect_action_decode(value.action),
    }
}
