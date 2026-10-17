//! Whole-value mappings for infrastructure/native.
use crate::infrastructure::native as d;
use crate::wire::{Invalid, w};
use skein_lib::List;
pub(crate) fn service_encode(value: &d::Service) -> Result<w::InfrastructureNativeService, Invalid> {
    let parts =
        w::InfrastructureNativeServiceParts { environment: (value.environment).clone(), name: (value.name).clone() };
    let Ok(value) = w::InfrastructureNativeService::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn service_decode(value: w::InfrastructureNativeService) -> d::Service {
    let value = value.into_parts();
    d::Service { environment: value.environment, name: value.name }
}

pub(crate) fn environment_encode(value: &d::Environment) -> Result<w::InfrastructureNativeEnvironment, Invalid> {
    let parts = w::InfrastructureNativeEnvironmentParts { pool: (value.pool).clone(), name: (value.name).clone() };
    let Ok(value) = w::InfrastructureNativeEnvironment::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn environment_decode(value: w::InfrastructureNativeEnvironment) -> d::Environment {
    let value = value.into_parts();
    d::Environment { pool: value.pool, name: value.name }
}

pub(crate) fn pool_encode(value: &d::Pool) -> Result<w::InfrastructureNativePool, Invalid> {
    let parts = w::InfrastructureNativePoolParts { value: (value.0).clone() };
    let Ok(value) = w::InfrastructureNativePool::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn pool_decode(value: w::InfrastructureNativePool) -> d::Pool {
    let value = value.into_parts();
    d::Pool(value.value)
}

pub(crate) fn resource_encode(value: &d::Resource) -> Result<w::InfrastructureNativeResource, Invalid> {
    Ok(match value {
        d::Resource::Service(value) => {
            let parts = w::InfrastructureNativeResourceServiceParts { value: service_encode(value)? };
            let Ok(value) = w::InfrastructureNativeResourceService::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeResource::Service(value)
        }
        d::Resource::Environment(value) => {
            let parts = w::InfrastructureNativeResourceEnvironmentParts { value: environment_encode(value)? };
            let Ok(value) = w::InfrastructureNativeResourceEnvironment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeResource::Environment(value)
        }
        d::Resource::Pool(value) => {
            let parts = w::InfrastructureNativeResourcePoolParts { value: pool_encode(value)? };
            let Ok(value) = w::InfrastructureNativeResourcePool::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureNativeResource::Pool(value)
        }
        d::Resource::Quota(value) => {
            let parts = w::InfrastructureNativeResourceQuotaParts { value: pool_encode(value)? };
            let Ok(value) = w::InfrastructureNativeResourceQuota::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureNativeResource::Quota(value)
        }
    })
}
pub(crate) fn resource_decode(value: w::InfrastructureNativeResource) -> d::Resource {
    match value {
        w::InfrastructureNativeResource::Service(value) => {
            let value = value.into_parts();
            d::Resource::Service(service_decode(value.value))
        }
        w::InfrastructureNativeResource::Environment(value) => {
            let value = value.into_parts();
            d::Resource::Environment(environment_decode(value.value))
        }
        w::InfrastructureNativeResource::Pool(value) => {
            let value = value.into_parts();
            d::Resource::Pool(pool_decode(value.value))
        }
        w::InfrastructureNativeResource::Quota(value) => {
            let value = value.into_parts();
            d::Resource::Quota(pool_decode(value.value))
        }
    }
}

pub(crate) fn hold_encode(value: d::Hold) -> w::InfrastructureNativeHold {
    match value {
        d::Hold::ExclusiveWait => w::InfrastructureNativeHold::ExclusiveWait,
        d::Hold::PooledWait => w::InfrastructureNativeHold::PooledWait,
        d::Hold::None => w::InfrastructureNativeHold::None,
    }
}
pub(crate) fn hold_decode(value: w::InfrastructureNativeHold) -> d::Hold {
    match value {
        w::InfrastructureNativeHold::ExclusiveWait => d::Hold::ExclusiveWait,
        w::InfrastructureNativeHold::PooledWait => d::Hold::PooledWait,
        w::InfrastructureNativeHold::None => d::Hold::None,
    }
}

pub(crate) fn resource_role_encode(value: d::ResourceRole) -> w::InfrastructureNativeResourceRole {
    match value {
        d::ResourceRole::Owned => w::InfrastructureNativeResourceRole::Owned,
        d::ResourceRole::Participant => w::InfrastructureNativeResourceRole::Participant,
        d::ResourceRole::Context => w::InfrastructureNativeResourceRole::Context,
        d::ResourceRole::Unavailable => w::InfrastructureNativeResourceRole::Unavailable,
    }
}
pub(crate) fn resource_role_decode(value: w::InfrastructureNativeResourceRole) -> d::ResourceRole {
    match value {
        w::InfrastructureNativeResourceRole::Owned => d::ResourceRole::Owned,
        w::InfrastructureNativeResourceRole::Participant => d::ResourceRole::Participant,
        w::InfrastructureNativeResourceRole::Context => d::ResourceRole::Context,
        w::InfrastructureNativeResourceRole::Unavailable => d::ResourceRole::Unavailable,
    }
}

pub(crate) fn named_encode(value: &d::Named) -> Result<w::InfrastructureNativeNamed, Invalid> {
    let parts = w::InfrastructureNativeNamedParts {
        resource: resource_encode(&(value.resource))?,
        role: resource_role_encode(value.role),
        hold: hold_encode(value.hold),
    };
    let Ok(value) = w::InfrastructureNativeNamed::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn named_decode(value: w::InfrastructureNativeNamed) -> d::Named {
    let value = value.into_parts();
    d::Named {
        resource: resource_decode(value.resource),
        role: resource_role_decode(value.role),
        hold: hold_decode(value.hold),
    }
}

pub(crate) fn purpose_encode(value: &d::Purpose) -> Result<w::InfrastructureNativePurpose, Invalid> {
    Ok(match value {
        d::Purpose::Call { attempt, completion, position } => {
            let parts = w::InfrastructureNativePurposeCallParts {
                attempt: (*attempt),
                completion: (*completion),
                position: (*position),
            };
            let Ok(value) = w::InfrastructureNativePurposeCall::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureNativePurpose::Call(value)
        }
        d::Purpose::Procedure { purpose } => {
            let parts = w::InfrastructureNativePurposeProcedureParts { purpose: (*purpose) };
            let Ok(value) = w::InfrastructureNativePurposeProcedure::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativePurpose::Procedure(value)
        }
        d::Purpose::Projection { purpose } => {
            let parts = w::InfrastructureNativePurposeProjectionParts { purpose: (*purpose) };
            let Ok(value) = w::InfrastructureNativePurposeProjection::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativePurpose::Projection(value)
        }
    })
}
pub(crate) fn purpose_decode(value: w::InfrastructureNativePurpose) -> d::Purpose {
    match value {
        w::InfrastructureNativePurpose::Call(value) => {
            let value = value.into_parts();
            d::Purpose::Call { attempt: value.attempt, completion: value.completion, position: value.position }
        }
        w::InfrastructureNativePurpose::Procedure(value) => {
            let value = value.into_parts();
            d::Purpose::Procedure { purpose: value.purpose }
        }
        w::InfrastructureNativePurpose::Projection(value) => {
            let value = value.into_parts();
            d::Purpose::Projection { purpose: value.purpose }
        }
    }
}

pub(crate) fn key_encode(value: &d::Key) -> Result<w::InfrastructureNativeKey, Invalid> {
    let parts = w::InfrastructureNativeKeyParts {
        deployment: (value.deployment),
        task: (value.task),
        origin: purpose_encode(&(value.origin))?,
        purpose: (value.purpose),
    };
    let Ok(value) = w::InfrastructureNativeKey::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn key_decode(value: w::InfrastructureNativeKey) -> d::Key {
    let value = value.into_parts();
    d::Key {
        deployment: value.deployment,
        task: value.task,
        origin: purpose_decode(value.origin),
        purpose: value.purpose,
    }
}

pub(crate) fn backend_encode(value: d::Backend) -> w::InfrastructureNativeBackend {
    match value {
        d::Backend::OperationIds => w::InfrastructureNativeBackend::OperationIds,
        d::Backend::NoOperationIds => w::InfrastructureNativeBackend::NoOperationIds,
    }
}
pub(crate) fn backend_decode(value: w::InfrastructureNativeBackend) -> d::Backend {
    match value {
        w::InfrastructureNativeBackend::OperationIds => d::Backend::OperationIds,
        w::InfrastructureNativeBackend::NoOperationIds => d::Backend::NoOperationIds,
    }
}

pub(crate) fn effect_encode(value: &d::Effect) -> Result<w::InfrastructureNativeEffect, Invalid> {
    Ok(match value {
        d::Effect::Restart { service, operation } => {
            let parts = w::InfrastructureNativeEffectRestartParts {
                service: service_encode(service)?,
                operation: (*operation),
            };
            let Ok(value) = w::InfrastructureNativeEffectRestart::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureNativeEffect::Restart(value)
        }
        d::Effect::Scale { service, from, to } => {
            let parts =
                w::InfrastructureNativeEffectScaleParts { service: service_encode(service)?, from: (*from), to: (*to) };
            let Ok(value) = w::InfrastructureNativeEffectScale::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureNativeEffect::Scale(value)
        }
        d::Effect::Rollback { service, from, to } => {
            let parts = w::InfrastructureNativeEffectRollbackParts {
                service: service_encode(service)?,
                from: (*from).clone(),
                to: (*to).clone(),
            };
            let Ok(value) = w::InfrastructureNativeEffectRollback::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeEffect::Rollback(value)
        }
        d::Effect::CreateEnvironment { environment, until, price } => {
            let parts = w::InfrastructureNativeEffectCreateEnvironmentParts {
                environment: environment_encode(environment)?,
                until: (*until),
                price: (*price),
            };
            let Ok(value) = w::InfrastructureNativeEffectCreateEnvironment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeEffect::CreateEnvironment(value)
        }
        d::Effect::TearDown { environment, created_by } => {
            let parts = w::InfrastructureNativeEffectTearDownParts {
                environment: environment_encode(environment)?,
                created_by: key_encode(created_by)?,
            };
            let Ok(value) = w::InfrastructureNativeEffectTearDown::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeEffect::TearDown(value)
        }
    })
}
pub(crate) fn effect_decode(value: w::InfrastructureNativeEffect) -> d::Effect {
    match value {
        w::InfrastructureNativeEffect::Restart(value) => {
            let value = value.into_parts();
            d::Effect::Restart { service: service_decode(value.service), operation: value.operation }
        }
        w::InfrastructureNativeEffect::Scale(value) => {
            let value = value.into_parts();
            d::Effect::Scale { service: service_decode(value.service), from: value.from, to: value.to }
        }
        w::InfrastructureNativeEffect::Rollback(value) => {
            let value = value.into_parts();
            d::Effect::Rollback { service: service_decode(value.service), from: value.from, to: value.to }
        }
        w::InfrastructureNativeEffect::CreateEnvironment(value) => {
            let value = value.into_parts();
            d::Effect::CreateEnvironment {
                environment: environment_decode(value.environment),
                until: value.until,
                price: value.price,
            }
        }
        w::InfrastructureNativeEffect::TearDown(value) => {
            let value = value.into_parts();
            d::Effect::TearDown {
                environment: environment_decode(value.environment),
                created_by: key_decode(value.created_by),
            }
        }
    }
}

pub(crate) fn form_encode(value: d::Form) -> w::InfrastructureNativeForm {
    match value {
        d::Form::Creation => w::InfrastructureNativeForm::Creation,
        d::Form::Transition => w::InfrastructureNativeForm::Transition,
        d::Form::Set => w::InfrastructureNativeForm::Set,
    }
}
pub(crate) fn form_decode(value: w::InfrastructureNativeForm) -> d::Form {
    match value {
        w::InfrastructureNativeForm::Creation => d::Form::Creation,
        w::InfrastructureNativeForm::Transition => d::Form::Transition,
        w::InfrastructureNativeForm::Set => d::Form::Set,
    }
}

pub(crate) fn recovery_encode(value: d::Recovery) -> w::InfrastructureNativeRecovery {
    match value {
        d::Recovery::Keyed => w::InfrastructureNativeRecovery::Keyed,
        d::Recovery::Conditional => w::InfrastructureNativeRecovery::Conditional,
        d::Recovery::Unrecoverable => w::InfrastructureNativeRecovery::Unrecoverable,
    }
}
pub(crate) fn recovery_decode(value: w::InfrastructureNativeRecovery) -> d::Recovery {
    match value {
        w::InfrastructureNativeRecovery::Keyed => d::Recovery::Keyed,
        w::InfrastructureNativeRecovery::Conditional => d::Recovery::Conditional,
        w::InfrastructureNativeRecovery::Unrecoverable => d::Recovery::Unrecoverable,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn description_encode(value: &d::Description) -> Result<w::InfrastructureNativeDescription, Invalid> {
    let parts = w::InfrastructureNativeDescriptionParts {
        kind: (value.kind),
        resources: {
            let source = &(value.resources);
            let Ok(capacity) = u32::try_from(source.len()) else { return Err(Invalid) };
            let mut values = List::with_capacity(capacity);
            for item in source {
                let value = resource_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        condition: (value.condition),
        form: form_encode(value.form),
        recovery: recovery_encode(value.recovery),
        price: match &(value.price) {
            Some(item) => Some(*item),
            None => None,
        },
        state: (value.state),
    };
    let Ok(value) = w::InfrastructureNativeDescription::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn description_decode(value: w::InfrastructureNativeDescription) -> d::Description {
    let value = value.into_parts();
    d::Description {
        kind: value.kind,
        resources: {
            let source = value.resources;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = resource_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values.into_boxed()
        },
        condition: value.condition,
        form: form_decode(value.form),
        recovery: recovery_decode(value.recovery),
        price: match value.price {
            Some(item) => Some(item),
            None => None,
        },
        state: value.state,
    }
}

pub(crate) fn phase_encode(value: d::Phase) -> w::InfrastructureNativePhase {
    match value {
        d::Phase::Kept => w::InfrastructureNativePhase::Kept,
        d::Phase::Sent => w::InfrastructureNativePhase::Sent,
        d::Phase::Uncertain => w::InfrastructureNativePhase::Uncertain,
        d::Phase::Held => w::InfrastructureNativePhase::Held,
    }
}
pub(crate) fn phase_decode(value: w::InfrastructureNativePhase) -> d::Phase {
    match value {
        w::InfrastructureNativePhase::Kept => d::Phase::Kept,
        w::InfrastructureNativePhase::Sent => d::Phase::Sent,
        w::InfrastructureNativePhase::Uncertain => d::Phase::Uncertain,
        w::InfrastructureNativePhase::Held => d::Phase::Held,
    }
}
