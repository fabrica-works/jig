//! Whole-value mappings for infrastructure/documents.
use crate::infrastructure::documents as d;
use crate::wire::{Invalid, w};
pub(crate) fn scope_encode(value: d::Scope) -> Result<w::InfrastructureScope, Invalid> {
    Ok(match value {
        d::Scope::Project(value) => {
            let parts = w::InfrastructureScopeProjectParts { value };
            let Ok(value) = w::InfrastructureScopeProject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureScope::Project(value)
        }
        d::Scope::Deployment => w::InfrastructureScope::Deployment,
    })
}
pub(crate) fn scope_decode(value: w::InfrastructureScope) -> d::Scope {
    match value {
        w::InfrastructureScope::Project(value) => {
            let value = value.into_parts();
            d::Scope::Project(value.value)
        }
        w::InfrastructureScope::Deployment => d::Scope::Deployment,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn kind_encode(value: &d::Kind) -> Result<w::InfrastructureKind, Invalid> {
    Ok(match value {
        d::Kind::Environments { pool } => {
            let parts = w::InfrastructureKindEnvironmentsParts {
                pool: match pool {
                    Some(item) => Some(*item),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureKindEnvironments::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Environments(value)
        }
        d::Kind::Environment(value) => {
            let parts = w::InfrastructureKindEnvironmentParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindEnvironment::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Environment(value)
        }
        d::Kind::NativeEnvironment(value) => {
            let parts = w::InfrastructureKindNativeEnvironmentParts {
                value: crate::wire::infrastructure_native::environment_encode(value)?,
            };
            let Ok(value) = w::InfrastructureKindNativeEnvironment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureKind::NativeEnvironment(value)
        }
        d::Kind::NativeEnvironments { pool } => {
            let parts = w::InfrastructureKindNativeEnvironmentsParts {
                pool: match pool {
                    Some(item) => Some(crate::wire::infrastructure_native::pool_encode(item)?),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureKindNativeEnvironments::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureKind::NativeEnvironments(value)
        }
        d::Kind::Panel(value) => {
            let parts = w::InfrastructureKindPanelParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindPanel::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Panel(value)
        }
        d::Kind::Result(value) => {
            let parts = w::InfrastructureKindResultParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindResult::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Result(value)
        }
        d::Kind::Effect(value) => {
            let parts = w::InfrastructureKindEffectParts { value: crate::wire::proposal_encode(value)? };
            let Ok(value) = w::InfrastructureKindEffect::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Effect(value)
        }
        d::Kind::OwnerEffect(value) => {
            let parts = w::InfrastructureKindOwnerEffectParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindOwnerEffect::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::OwnerEffect(value)
        }
        d::Kind::Outbox(value) => {
            let parts = w::InfrastructureKindOutboxParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindOutbox::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Outbox(value)
        }
        d::Kind::Hold(value) => {
            let parts = w::InfrastructureKindHoldParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindHold::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Hold(value)
        }
        d::Kind::Project(value) => {
            let parts = w::InfrastructureKindProjectParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindProject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Project(value)
        }
        d::Kind::Deployment => w::InfrastructureKind::Deployment,
    })
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn kind_decode(value: w::InfrastructureKind) -> d::Kind {
    match value {
        w::InfrastructureKind::Environments(value) => {
            let value = value.into_parts();
            d::Kind::Environments {
                pool: match value.pool {
                    Some(item) => Some(item),
                    None => None,
                },
            }
        }
        w::InfrastructureKind::Environment(value) => {
            let value = value.into_parts();
            d::Kind::Environment(value.value)
        }
        w::InfrastructureKind::NativeEnvironment(value) => {
            let value = value.into_parts();
            d::Kind::NativeEnvironment(crate::wire::infrastructure_native::environment_decode(value.value))
        }
        w::InfrastructureKind::NativeEnvironments(value) => {
            let value = value.into_parts();
            d::Kind::NativeEnvironments {
                pool: match value.pool {
                    Some(item) => Some(crate::wire::infrastructure_native::pool_decode(item)),
                    None => None,
                },
            }
        }
        w::InfrastructureKind::Panel(value) => {
            let value = value.into_parts();
            d::Kind::Panel(value.value)
        }
        w::InfrastructureKind::Result(value) => {
            let value = value.into_parts();
            d::Kind::Result(value.value)
        }
        w::InfrastructureKind::Effect(value) => {
            let value = value.into_parts();
            d::Kind::Effect(crate::wire::proposal_decode(value.value))
        }
        w::InfrastructureKind::OwnerEffect(value) => {
            let value = value.into_parts();
            d::Kind::OwnerEffect(value.value)
        }
        w::InfrastructureKind::Outbox(value) => {
            let value = value.into_parts();
            d::Kind::Outbox(value.value)
        }
        w::InfrastructureKind::Hold(value) => {
            let value = value.into_parts();
            d::Kind::Hold(value.value)
        }
        w::InfrastructureKind::Project(value) => {
            let value = value.into_parts();
            d::Kind::Project(value.value)
        }
        w::InfrastructureKind::Deployment => d::Kind::Deployment,
    }
}

pub(crate) fn subject_encode(value: &d::Subject) -> Result<w::InfrastructureSubject, Invalid> {
    let parts = w::InfrastructureSubjectParts { scope: scope_encode(value.scope)?, kind: kind_encode(&(value.kind))? };
    let Ok(value) = w::InfrastructureSubject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn subject_decode(value: w::InfrastructureSubject) -> d::Subject {
    let value = value.into_parts();
    d::Subject { scope: scope_decode(value.scope), kind: kind_decode(value.kind) }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn pool_encode(value: &d::Pool) -> Result<w::InfrastructurePool, Invalid> {
    let parts = w::InfrastructurePoolParts {
        number: (value.number),
        name: (value.name).clone(),
        quota: match &(value.quota) {
            Some(item) => Some(*item),
            None => None,
        },
        used: match &(value.used) {
            Some(item) => Some(*item),
            None => None,
        },
    };
    let Ok(value) = w::InfrastructurePool::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn pool_decode(value: w::InfrastructurePool) -> d::Pool {
    let value = value.into_parts();
    d::Pool {
        number: value.number,
        name: value.name,
        quota: match value.quota {
            Some(item) => Some(item),
            None => None,
        },
        used: match value.used {
            Some(item) => Some(item),
            None => None,
        },
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn environment_encode(value: &d::Environment) -> Result<w::InfrastructureEnvironment, Invalid> {
    let parts = w::InfrastructureEnvironmentParts {
        project: (value.project),
        number: (value.number),
        pool: (value.pool),
        name: (value.name).clone(),
        revision: (value.revision).value,
        created_by: creation_encode(&(value.created_by))?,
        ready_at: match &(value.ready_at) {
            Some(item) => Some((*item).as_nanos()),
            None => None,
        },
        ready: match &(value.ready) {
            Some(item) => Some(*item),
            None => None,
        },
        cost: match &(value.cost) {
            Some(item) => Some(*item),
            None => None,
        },
        observed_until: match &(value.observed_until) {
            Some(item) => Some((*item).as_nanos()),
            None => None,
        },
        end: match &(value.end) {
            Some(item) => Some(end_encode(item)?),
            None => None,
        },
    };
    let Ok(value) = w::InfrastructureEnvironment::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn environment_decode(value: w::InfrastructureEnvironment) -> d::Environment {
    let value = value.into_parts();
    d::Environment {
        project: value.project,
        number: value.number,
        pool: value.pool,
        name: value.name,
        revision: jig_client::contract::Revision { value: value.revision },
        created_by: creation_decode(value.created_by),
        ready_at: match value.ready_at {
            Some(item) => Some(skein_lib::Wall::from_nanos(item)),
            None => None,
        },
        ready: match value.ready {
            Some(item) => Some(item),
            None => None,
        },
        cost: match value.cost {
            Some(item) => Some(item),
            None => None,
        },
        observed_until: match value.observed_until {
            Some(item) => Some(skein_lib::Wall::from_nanos(item)),
            None => None,
        },
        end: match value.end {
            Some(item) => Some(end_decode(item)),
            None => None,
        },
    }
}

pub(crate) fn end_encode(value: &d::End) -> Result<w::InfrastructureEnd, Invalid> {
    let parts = w::InfrastructureEndParts { revision: (value.revision).value, until: (value.until).as_nanos() };
    let Ok(value) = w::InfrastructureEnd::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn end_decode(value: w::InfrastructureEnd) -> d::End {
    let value = value.into_parts();
    d::End {
        revision: jig_client::contract::Revision { value: value.revision },
        until: skein_lib::Wall::from_nanos(value.until),
    }
}

pub(crate) fn service_encode(value: &d::Service) -> Result<w::InfrastructureService, Invalid> {
    let parts = w::InfrastructureServiceParts { environment: (value.environment).clone(), name: (value.name).clone() };
    let Ok(value) = w::InfrastructureService::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn service_decode(value: w::InfrastructureService) -> d::Service {
    let value = value.into_parts();
    d::Service { environment: value.environment, name: value.name }
}

pub(crate) fn backend_encode(value: d::Backend) -> w::InfrastructureBackend {
    match value {
        d::Backend::OperationIds => w::InfrastructureBackend::OperationIds,
        d::Backend::NoOperationIds => w::InfrastructureBackend::NoOperationIds,
    }
}
pub(crate) fn backend_decode(value: w::InfrastructureBackend) -> d::Backend {
    match value {
        w::InfrastructureBackend::OperationIds => d::Backend::OperationIds,
        w::InfrastructureBackend::NoOperationIds => d::Backend::NoOperationIds,
    }
}

pub(crate) fn creation_encode(value: &d::Creation) -> Result<w::InfrastructureCreation, Invalid> {
    let parts = w::InfrastructureCreationParts {
        deployment: (value.deployment),
        task: (value.task),
        origin: purpose_encode(&(value.origin))?,
        purpose: (value.purpose),
    };
    let Ok(value) = w::InfrastructureCreation::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn creation_decode(value: w::InfrastructureCreation) -> d::Creation {
    let value = value.into_parts();
    d::Creation {
        deployment: value.deployment,
        task: value.task,
        origin: purpose_decode(value.origin),
        purpose: value.purpose,
    }
}

pub(crate) fn purpose_encode(value: &d::Purpose) -> Result<w::InfrastructurePurpose, Invalid> {
    Ok(match value {
        d::Purpose::Call { attempt, completion, position } => {
            let parts = w::InfrastructurePurposeCallParts {
                attempt: (*attempt),
                completion: (*completion),
                position: (*position),
            };
            let Ok(value) = w::InfrastructurePurposeCall::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructurePurpose::Call(value)
        }
        d::Purpose::Procedure { purpose } => {
            let parts = w::InfrastructurePurposeProcedureParts { purpose: (*purpose) };
            let Ok(value) = w::InfrastructurePurposeProcedure::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructurePurpose::Procedure(value)
        }
        d::Purpose::Projection { purpose } => {
            let parts = w::InfrastructurePurposeProjectionParts { purpose: (*purpose) };
            let Ok(value) = w::InfrastructurePurposeProjection::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructurePurpose::Projection(value)
        }
    })
}
pub(crate) fn purpose_decode(value: w::InfrastructurePurpose) -> d::Purpose {
    match value {
        w::InfrastructurePurpose::Call(value) => {
            let value = value.into_parts();
            d::Purpose::Call { attempt: value.attempt, completion: value.completion, position: value.position }
        }
        w::InfrastructurePurpose::Procedure(value) => {
            let value = value.into_parts();
            d::Purpose::Procedure { purpose: value.purpose }
        }
        w::InfrastructurePurpose::Projection(value) => {
            let value = value.into_parts();
            d::Purpose::Projection { purpose: value.purpose }
        }
    }
}

pub(crate) fn phase_encode(value: d::Phase) -> w::InfrastructurePhase {
    match value {
        d::Phase::Active => w::InfrastructurePhase::Active,
        d::Phase::ReadingFacts => w::InfrastructurePhase::ReadingFacts,
        d::Phase::FactsReady => w::InfrastructurePhase::FactsReady,
        d::Phase::EffectOutstanding => w::InfrastructurePhase::EffectOutstanding,
        d::Phase::WaitingCondition => w::InfrastructurePhase::WaitingCondition,
        d::Phase::Holding => w::InfrastructurePhase::Holding,
        d::Phase::Done => w::InfrastructurePhase::Done,
        d::Phase::Held => w::InfrastructurePhase::Held,
    }
}
pub(crate) fn phase_decode(value: w::InfrastructurePhase) -> d::Phase {
    match value {
        w::InfrastructurePhase::Active => d::Phase::Active,
        w::InfrastructurePhase::ReadingFacts => d::Phase::ReadingFacts,
        w::InfrastructurePhase::FactsReady => d::Phase::FactsReady,
        w::InfrastructurePhase::EffectOutstanding => d::Phase::EffectOutstanding,
        w::InfrastructurePhase::WaitingCondition => d::Phase::WaitingCondition,
        w::InfrastructurePhase::Holding => d::Phase::Holding,
        w::InfrastructurePhase::Done => d::Phase::Done,
        w::InfrastructurePhase::Held => d::Phase::Held,
    }
}
