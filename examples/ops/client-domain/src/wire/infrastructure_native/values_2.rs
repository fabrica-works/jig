//! Whole-value mappings for infrastructure/native.
use crate::infrastructure::native as d;
use crate::wire::{Invalid, w};
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn entry_encode(value: &d::Entry) -> Result<w::InfrastructureNativeEntry, Invalid> {
    let parts = w::InfrastructureNativeEntryParts {
        project: match &(value.project) {
            Some(item) => Some(*item),
            None => None,
        },
        number: (value.number),
        key: crate::wire::infrastructure_native::key_encode(&(value.key))?,
        effect: crate::wire::infrastructure_native::effect_encode(&(value.effect))?,
        attempt: (value.attempt),
        deadline: (value.deadline),
        phase: crate::wire::infrastructure_native::phase_encode(value.phase),
    };
    let Ok(value) = w::InfrastructureNativeEntry::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn entry_decode(value: w::InfrastructureNativeEntry) -> d::Entry {
    let value = value.into_parts();
    d::Entry {
        project: match value.project {
            Some(item) => Some(item),
            None => None,
        },
        number: value.number,
        key: crate::wire::infrastructure_native::key_decode(value.key),
        effect: crate::wire::infrastructure_native::effect_decode(value.effect),
        attempt: value.attempt,
        deadline: value.deadline,
        phase: crate::wire::infrastructure_native::phase_decode(value.phase),
    }
}

pub(crate) fn service_fact_encode(value: &d::ServiceFact) -> Result<w::InfrastructureNativeServiceFact, Invalid> {
    let parts = w::InfrastructureNativeServiceFactParts {
        version: (value.version).clone(),
        replicas: (value.replicas),
        healthy: (value.healthy),
        revision: (value.revision),
        observed: (value.observed),
    };
    let Ok(value) = w::InfrastructureNativeServiceFact::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn service_fact_decode(value: w::InfrastructureNativeServiceFact) -> d::ServiceFact {
    let value = value.into_parts();
    d::ServiceFact {
        version: value.version,
        replicas: value.replicas,
        healthy: value.healthy,
        revision: value.revision,
        observed: value.observed,
    }
}

pub(crate) fn environment_fact_encode(
    value: &d::EnvironmentFact,
) -> Result<w::InfrastructureNativeEnvironmentFact, Invalid> {
    let parts = w::InfrastructureNativeEnvironmentFactParts {
        created_by: crate::wire::infrastructure_native::key_encode(&(value.created_by))?,
        ready_at: (value.ready_at),
        until: (value.until),
        price: (value.price),
    };
    let Ok(value) = w::InfrastructureNativeEnvironmentFact::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn environment_fact_decode(value: w::InfrastructureNativeEnvironmentFact) -> d::EnvironmentFact {
    let value = value.into_parts();
    d::EnvironmentFact {
        created_by: crate::wire::infrastructure_native::key_decode(value.created_by),
        ready_at: value.ready_at,
        until: value.until,
        price: value.price,
    }
}

pub(crate) fn procedure_encode(value: &d::Procedure) -> Result<w::InfrastructureNativeProcedure, Invalid> {
    Ok(match value {
        d::Procedure::Remediate { service, operation, deadline } => {
            let parts = w::InfrastructureNativeProcedureRemediateParts {
                service: crate::wire::infrastructure_native::service_encode(service)?,
                operation: (*operation),
                deadline: (*deadline),
            };
            let Ok(value) = w::InfrastructureNativeProcedureRemediate::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeProcedure::Remediate(value)
        }
        d::Procedure::Scale { service, replicas, deadline } => {
            let parts = w::InfrastructureNativeProcedureScaleParts {
                service: crate::wire::infrastructure_native::service_encode(service)?,
                replicas: (*replicas),
                deadline: (*deadline),
            };
            let Ok(value) = w::InfrastructureNativeProcedureScale::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeProcedure::Scale(value)
        }
        d::Procedure::Provision { environment, until, price, deadline } => {
            let parts = w::InfrastructureNativeProcedureProvisionParts {
                environment: crate::wire::infrastructure_native::environment_encode(environment)?,
                until: (*until),
                price: (*price),
                deadline: (*deadline),
            };
            let Ok(value) = w::InfrastructureNativeProcedureProvision::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeProcedure::Provision(value)
        }
        d::Procedure::TearDown { environment, created_by, deadline } => {
            let parts = w::InfrastructureNativeProcedureTearDownParts {
                environment: crate::wire::infrastructure_native::environment_encode(environment)?,
                created_by: crate::wire::infrastructure_native::key_encode(created_by)?,
                deadline: (*deadline),
            };
            let Ok(value) = w::InfrastructureNativeProcedureTearDown::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeProcedure::TearDown(value)
        }
    })
}
pub(crate) fn procedure_decode(value: w::InfrastructureNativeProcedure) -> d::Procedure {
    match value {
        w::InfrastructureNativeProcedure::Remediate(value) => {
            let value = value.into_parts();
            d::Procedure::Remediate {
                service: crate::wire::infrastructure_native::service_decode(value.service),
                operation: value.operation,
                deadline: value.deadline,
            }
        }
        w::InfrastructureNativeProcedure::Scale(value) => {
            let value = value.into_parts();
            d::Procedure::Scale {
                service: crate::wire::infrastructure_native::service_decode(value.service),
                replicas: value.replicas,
                deadline: value.deadline,
            }
        }
        w::InfrastructureNativeProcedure::Provision(value) => {
            let value = value.into_parts();
            d::Procedure::Provision {
                environment: crate::wire::infrastructure_native::environment_decode(value.environment),
                until: value.until,
                price: value.price,
                deadline: value.deadline,
            }
        }
        w::InfrastructureNativeProcedure::TearDown(value) => {
            let value = value.into_parts();
            d::Procedure::TearDown {
                environment: crate::wire::infrastructure_native::environment_decode(value.environment),
                created_by: crate::wire::infrastructure_native::key_decode(value.created_by),
                deadline: value.deadline,
            }
        }
    }
}

pub(crate) fn procedure_phase_encode(value: d::ProcedurePhase) -> w::InfrastructureNativeProcedurePhase {
    match value {
        d::ProcedurePhase::Active => w::InfrastructureNativeProcedurePhase::Active,
        d::ProcedurePhase::ReadingFacts => w::InfrastructureNativeProcedurePhase::ReadingFacts,
        d::ProcedurePhase::FactsReady => w::InfrastructureNativeProcedurePhase::FactsReady,
        d::ProcedurePhase::EffectOutstanding => w::InfrastructureNativeProcedurePhase::EffectOutstanding,
        d::ProcedurePhase::WaitingCondition => w::InfrastructureNativeProcedurePhase::WaitingCondition,
        d::ProcedurePhase::Holding => w::InfrastructureNativeProcedurePhase::Holding,
        d::ProcedurePhase::Done => w::InfrastructureNativeProcedurePhase::Done,
        d::ProcedurePhase::Held => w::InfrastructureNativeProcedurePhase::Held,
    }
}
pub(crate) fn procedure_phase_decode(value: w::InfrastructureNativeProcedurePhase) -> d::ProcedurePhase {
    match value {
        w::InfrastructureNativeProcedurePhase::Active => d::ProcedurePhase::Active,
        w::InfrastructureNativeProcedurePhase::ReadingFacts => d::ProcedurePhase::ReadingFacts,
        w::InfrastructureNativeProcedurePhase::FactsReady => d::ProcedurePhase::FactsReady,
        w::InfrastructureNativeProcedurePhase::EffectOutstanding => d::ProcedurePhase::EffectOutstanding,
        w::InfrastructureNativeProcedurePhase::WaitingCondition => d::ProcedurePhase::WaitingCondition,
        w::InfrastructureNativeProcedurePhase::Holding => d::ProcedurePhase::Holding,
        w::InfrastructureNativeProcedurePhase::Done => d::ProcedurePhase::Done,
        w::InfrastructureNativeProcedurePhase::Held => d::ProcedurePhase::Held,
    }
}

pub(crate) fn procedure_state_encode(
    value: &d::ProcedureState,
) -> Result<w::InfrastructureNativeProcedureState, Invalid> {
    let parts = w::InfrastructureNativeProcedureStateParts {
        task: (value.task),
        procedure: procedure_encode(&(value.procedure))?,
        phase: procedure_phase_encode(value.phase),
    };
    let Ok(value) = w::InfrastructureNativeProcedureState::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn procedure_state_decode(value: w::InfrastructureNativeProcedureState) -> d::ProcedureState {
    let value = value.into_parts();
    d::ProcedureState {
        task: value.task,
        procedure: procedure_decode(value.procedure),
        phase: procedure_phase_decode(value.phase),
    }
}

pub(crate) fn drift_kind_encode(value: d::DriftKind) -> w::InfrastructureNativeDriftKind {
    match value {
        d::DriftKind::Changed => w::InfrastructureNativeDriftKind::Changed,
        d::DriftKind::Deleted => w::InfrastructureNativeDriftKind::Deleted,
    }
}
pub(crate) fn drift_kind_decode(value: w::InfrastructureNativeDriftKind) -> d::DriftKind {
    match value {
        w::InfrastructureNativeDriftKind::Changed => d::DriftKind::Changed,
        w::InfrastructureNativeDriftKind::Deleted => d::DriftKind::Deleted,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn drift_account_encode(value: &d::DriftAccount) -> Result<w::InfrastructureNativeDriftAccount, Invalid> {
    let parts = w::InfrastructureNativeDriftAccountParts {
        resource: crate::wire::infrastructure_native::resource_encode(&(value.resource))?,
        kind: drift_kind_encode(value.kind),
        at: (value.at).as_nanos(),
        by: match &(value.by) {
            Some(item) => Some((*item).clone()),
            None => None,
        },
    };
    let Ok(value) = w::InfrastructureNativeDriftAccount::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn drift_account_decode(value: w::InfrastructureNativeDriftAccount) -> d::DriftAccount {
    let value = value.into_parts();
    d::DriftAccount {
        resource: crate::wire::infrastructure_native::resource_decode(value.resource),
        kind: drift_kind_decode(value.kind),
        at: skein_lib::Wall::from_nanos(value.at),
        by: match value.by {
            Some(item) => Some(item),
            None => None,
        },
    }
}

pub(crate) fn environment_end_encode(
    value: &d::EnvironmentEnd,
) -> Result<w::InfrastructureNativeEnvironmentEnd, Invalid> {
    let parts = w::InfrastructureNativeEnvironmentEndParts { revision: (value.revision), until: (value.until) };
    let Ok(value) = w::InfrastructureNativeEnvironmentEnd::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn environment_end_decode(value: w::InfrastructureNativeEnvironmentEnd) -> d::EnvironmentEnd {
    let value = value.into_parts();
    d::EnvironmentEnd { revision: value.revision, until: value.until }
}

pub(crate) fn client_pool_encode(value: d::ClientPool) -> Result<w::InfrastructureNativeClientPool, Invalid> {
    let parts = w::InfrastructureNativeClientPoolParts { quota: (value.quota), used: (value.used) };
    let Ok(value) = w::InfrastructureNativeClientPool::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn client_pool_decode(value: w::InfrastructureNativeClientPool) -> d::ClientPool {
    let value = value.into_parts();
    d::ClientPool { quota: value.quota, used: value.used }
}

pub(crate) fn resource_fact_encode(value: &d::ResourceFact) -> Result<w::InfrastructureNativeResourceFact, Invalid> {
    Ok(match value {
        d::ResourceFact::Service(value) => {
            let parts = w::InfrastructureNativeResourceFactServiceParts {
                value: match value {
                    Some(item) => Some(service_fact_encode(item)?),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureNativeResourceFactService::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeResourceFact::Service(value)
        }
        d::ResourceFact::Environment { fact, end } => {
            let parts = w::InfrastructureNativeResourceFactEnvironmentParts {
                fact: match fact {
                    Some(item) => Some(environment_fact_encode(item)?),
                    None => None,
                },
                end: match end {
                    Some(item) => Some(environment_end_encode(item)?),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureNativeResourceFactEnvironment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeResourceFact::Environment(value)
        }
        d::ResourceFact::Pool(value) => {
            let parts = w::InfrastructureNativeResourceFactPoolParts {
                value: match value {
                    Some(item) => Some(client_pool_encode(*item)?),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureNativeResourceFactPool::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeResourceFact::Pool(value)
        }
        d::ResourceFact::Quota(value) => {
            let parts = w::InfrastructureNativeResourceFactQuotaParts {
                value: match value {
                    Some(item) => Some(client_pool_encode(*item)?),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureNativeResourceFactQuota::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureNativeResourceFact::Quota(value)
        }
    })
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn resource_fact_decode(value: w::InfrastructureNativeResourceFact) -> d::ResourceFact {
    match value {
        w::InfrastructureNativeResourceFact::Service(value) => {
            let value = value.into_parts();
            d::ResourceFact::Service(match value.value {
                Some(item) => Some(service_fact_decode(item)),
                None => None,
            })
        }
        w::InfrastructureNativeResourceFact::Environment(value) => {
            let value = value.into_parts();
            d::ResourceFact::Environment {
                fact: match value.fact {
                    Some(item) => Some(environment_fact_decode(item)),
                    None => None,
                },
                end: match value.end {
                    Some(item) => Some(environment_end_decode(item)),
                    None => None,
                },
            }
        }
        w::InfrastructureNativeResourceFact::Pool(value) => {
            let value = value.into_parts();
            d::ResourceFact::Pool(match value.value {
                Some(item) => Some(client_pool_decode(item)),
                None => None,
            })
        }
        w::InfrastructureNativeResourceFact::Quota(value) => {
            let value = value.into_parts();
            d::ResourceFact::Quota(match value.value {
                Some(item) => Some(client_pool_decode(item)),
                None => None,
            })
        }
    }
}

pub(crate) fn client_resource_encode(
    value: &d::ClientResource,
) -> Result<w::InfrastructureNativeClientResource, Invalid> {
    let parts = w::InfrastructureNativeClientResourceParts {
        named: crate::wire::infrastructure_native::named_encode(&(value.named))?,
        fact: resource_fact_encode(&(value.fact))?,
    };
    let Ok(value) = w::InfrastructureNativeClientResource::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn client_resource_decode(value: w::InfrastructureNativeClientResource) -> d::ClientResource {
    let value = value.into_parts();
    d::ClientResource {
        named: crate::wire::infrastructure_native::named_decode(value.named),
        fact: resource_fact_decode(value.fact),
    }
}

pub(crate) fn client_environment_encode(
    value: &d::ClientEnvironment,
) -> Result<w::InfrastructureNativeClientEnvironment, Invalid> {
    let parts = w::InfrastructureNativeClientEnvironmentParts {
        environment: crate::wire::infrastructure_native::environment_encode(&(value.environment))?,
        fact: environment_fact_encode(&(value.fact))?,
        end: match &(value.end) {
            Some(item) => Some(environment_end_encode(item)?),
            None => None,
        },
        pool: match &(value.pool) {
            Some(item) => Some(client_pool_encode(*item)?),
            None => None,
        },
    };
    let Ok(value) = w::InfrastructureNativeClientEnvironment::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn client_environment_decode(value: w::InfrastructureNativeClientEnvironment) -> d::ClientEnvironment {
    let value = value.into_parts();
    d::ClientEnvironment {
        environment: crate::wire::infrastructure_native::environment_decode(value.environment),
        fact: environment_fact_decode(value.fact),
        end: match value.end {
            Some(item) => Some(environment_end_decode(item)),
            None => None,
        },
        pool: match value.pool {
            Some(item) => Some(client_pool_decode(item)),
            None => None,
        },
    }
}
