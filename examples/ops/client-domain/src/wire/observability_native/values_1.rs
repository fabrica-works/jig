//! Whole-value mappings for observability/native.
use crate::observability::native as d;
use crate::wire::{Invalid, w};
use skein_lib::List;
pub(crate) fn load_point_encode(value: &d::LoadPoint) -> Result<w::ObservabilityNativeLoadPoint, Invalid> {
    let parts = w::ObservabilityNativeLoadPointParts { observed: (value.observed), load_percent: (value.load_percent) };
    let Ok(value) = w::ObservabilityNativeLoadPoint::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn load_point_decode(value: w::ObservabilityNativeLoadPoint) -> d::LoadPoint {
    let value = value.into_parts();
    d::LoadPoint { observed: value.observed, load_percent: value.load_percent }
}

pub(crate) fn fact_encode(value: &d::Fact) -> Result<w::ObservabilityNativeFact, Invalid> {
    let parts = w::ObservabilityNativeFactParts {
        observed: (value.observed),
        healthy_replicas: (value.healthy_replicas),
        errors: (value.errors),
        error_rate_percent: (value.error_rate_percent),
        load_percent: (value.load_percent),
        load: {
            let source = &(value.load);
            let Ok(capacity) = u32::try_from(source.len()) else { return Err(Invalid) };
            let mut values = List::with_capacity(capacity);
            for item in source {
                let value = load_point_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
    };
    let Ok(value) = w::ObservabilityNativeFact::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn fact_decode(value: w::ObservabilityNativeFact) -> d::Fact {
    let value = value.into_parts();
    d::Fact {
        observed: value.observed,
        healthy_replicas: value.healthy_replicas,
        errors: value.errors,
        error_rate_percent: value.error_rate_percent,
        load_percent: value.load_percent,
        load: {
            let source = value.load;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = load_point_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values.into_boxed()
        },
    }
}

pub(crate) fn health_encode(value: &d::Health) -> Result<w::ObservabilityNativeHealth, Invalid> {
    let parts = w::ObservabilityNativeHealthParts {
        observed: (value.observed),
        healthy_replicas: (value.healthy_replicas),
        errors: (value.errors),
        error_rate_percent: (value.error_rate_percent),
    };
    let Ok(value) = w::ObservabilityNativeHealth::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn health_decode(value: w::ObservabilityNativeHealth) -> d::Health {
    let value = value.into_parts();
    d::Health {
        observed: value.observed,
        healthy_replicas: value.healthy_replicas,
        errors: value.errors,
        error_rate_percent: value.error_rate_percent,
    }
}

pub(crate) fn point_encode(value: &d::Point) -> Result<w::ObservabilityNativePoint, Invalid> {
    let parts = w::ObservabilityNativePointParts {
        at: (value.at).as_nanos(),
        health: match &(value.health) {
            Some(item) => Some(health_encode(item)?),
            None => None,
        },
    };
    let Ok(value) = w::ObservabilityNativePoint::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn point_decode(value: w::ObservabilityNativePoint) -> d::Point {
    let value = value.into_parts();
    d::Point {
        at: skein_lib::Wall::from_nanos(value.at),
        health: match value.health {
            Some(item) => Some(health_decode(item)),
            None => None,
        },
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn incident_encode(value: &d::Incident) -> Result<w::ObservabilityNativeIncident, Invalid> {
    let parts = w::ObservabilityNativeIncidentParts {
        project: (value.project),
        number: (value.number),
        watch: (value.watch),
        lead: match &(value.lead) {
            Some(item) => Some(*item),
            None => None,
        },
        alerts: {
            let source = &(value.alerts);
            let Ok(capacity) = u32::try_from(source.len()) else { return Err(Invalid) };
            let mut values = List::with_capacity(capacity);
            for item in source {
                let value = *item;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        raised: (value.raised).as_nanos(),
        service: crate::wire::observability_documents::service_encode(&(value.service))?,
        severity: (value.severity),
        state: crate::wire::observability_documents::state_encode(value.state),
        timeline: {
            let source = &(value.timeline);
            let mut values = List::with_capacity(source.capacity());
            for item in source {
                let value = point_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        cut: (value.cut),
    };
    let Ok(value) = w::ObservabilityNativeIncident::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn incident_decode(value: w::ObservabilityNativeIncident) -> d::Incident {
    let value = value.into_parts();
    d::Incident {
        project: value.project,
        number: value.number,
        watch: value.watch,
        lead: match value.lead {
            Some(item) => Some(item),
            None => None,
        },
        alerts: {
            let source = value.alerts;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = item;
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values.into_boxed()
        },
        raised: skein_lib::Wall::from_nanos(value.raised),
        service: crate::wire::observability_documents::service_decode(value.service),
        severity: value.severity,
        state: crate::wire::observability_documents::state_decode(value.state),
        timeline: {
            let source = value.timeline;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = point_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values
        },
        cut: value.cut,
    }
}

pub(crate) fn service_state_encode(value: &d::ServiceState) -> Result<w::ObservabilityNativeServiceState, Invalid> {
    let parts = w::ObservabilityNativeServiceStateParts {
        service: crate::wire::observability_documents::service_encode(&(value.service))?,
        fact: match &(value.fact) {
            Some(item) => Some(fact_encode(item)?),
            None => None,
        },
    };
    let Ok(value) = w::ObservabilityNativeServiceState::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn service_state_decode(value: w::ObservabilityNativeServiceState) -> d::ServiceState {
    let value = value.into_parts();
    d::ServiceState {
        service: crate::wire::observability_documents::service_decode(value.service),
        fact: match value.fact {
            Some(item) => Some(fact_decode(item)),
            None => None,
        },
    }
}

pub(crate) fn alert_encode(value: &d::Alert) -> Result<w::ObservabilityNativeAlert, Invalid> {
    let parts = w::ObservabilityNativeAlertParts {
        number: (value.number),
        service: crate::wire::observability_documents::service_encode(&(value.service))?,
        severity: (value.severity),
    };
    let Ok(value) = w::ObservabilityNativeAlert::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn alert_decode(value: w::ObservabilityNativeAlert) -> d::Alert {
    let value = value.into_parts();
    d::Alert {
        number: value.number,
        service: crate::wire::observability_documents::service_decode(value.service),
        severity: value.severity,
    }
}

pub(crate) fn purpose_encode(value: &d::Purpose) -> Result<w::ObservabilityNativePurpose, Invalid> {
    Ok(match value {
        d::Purpose::Call { attempt, completion, position } => {
            let parts = w::ObservabilityNativePurposeCallParts {
                attempt: (*attempt),
                completion: (*completion),
                position: (*position),
            };
            let Ok(value) = w::ObservabilityNativePurposeCall::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityNativePurpose::Call(value)
        }
        d::Purpose::Procedure { purpose } => {
            let parts = w::ObservabilityNativePurposeProcedureParts { purpose: (*purpose) };
            let Ok(value) = w::ObservabilityNativePurposeProcedure::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::ObservabilityNativePurpose::Procedure(value)
        }
        d::Purpose::Projection { purpose } => {
            let parts = w::ObservabilityNativePurposeProjectionParts { purpose: (*purpose) };
            let Ok(value) = w::ObservabilityNativePurposeProjection::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::ObservabilityNativePurpose::Projection(value)
        }
    })
}
pub(crate) fn purpose_decode(value: w::ObservabilityNativePurpose) -> d::Purpose {
    match value {
        w::ObservabilityNativePurpose::Call(value) => {
            let value = value.into_parts();
            d::Purpose::Call { attempt: value.attempt, completion: value.completion, position: value.position }
        }
        w::ObservabilityNativePurpose::Procedure(value) => {
            let value = value.into_parts();
            d::Purpose::Procedure { purpose: value.purpose }
        }
        w::ObservabilityNativePurpose::Projection(value) => {
            let value = value.into_parts();
            d::Purpose::Projection { purpose: value.purpose }
        }
    }
}

pub(crate) fn key_encode(value: &d::Key) -> Result<w::ObservabilityNativeKey, Invalid> {
    let parts = w::ObservabilityNativeKeyParts {
        deployment: (value.deployment),
        task: (value.task),
        origin: purpose_encode(&(value.origin))?,
        purpose: (value.purpose),
    };
    let Ok(value) = w::ObservabilityNativeKey::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn key_decode(value: w::ObservabilityNativeKey) -> d::Key {
    let value = value.into_parts();
    d::Key {
        deployment: value.deployment,
        task: value.task,
        origin: purpose_decode(value.origin),
        purpose: value.purpose,
    }
}

pub(crate) fn effect_encode(value: &d::Effect) -> Result<w::ObservabilityNativeEffect, Invalid> {
    let parts = w::ObservabilityNativeEffectParts { rule: (value.rule).clone(), until: (value.until) };
    let Ok(value) = w::ObservabilityNativeEffect::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn effect_decode(value: w::ObservabilityNativeEffect) -> d::Effect {
    let value = value.into_parts();
    d::Effect { rule: value.rule, until: value.until }
}

pub(crate) fn phase_encode(value: d::Phase) -> w::ObservabilityNativePhase {
    match value {
        d::Phase::Kept => w::ObservabilityNativePhase::Kept,
        d::Phase::Sent => w::ObservabilityNativePhase::Sent,
        d::Phase::Uncertain => w::ObservabilityNativePhase::Uncertain,
        d::Phase::Held => w::ObservabilityNativePhase::Held,
    }
}
pub(crate) fn phase_decode(value: w::ObservabilityNativePhase) -> d::Phase {
    match value {
        w::ObservabilityNativePhase::Kept => d::Phase::Kept,
        w::ObservabilityNativePhase::Sent => d::Phase::Sent,
        w::ObservabilityNativePhase::Uncertain => d::Phase::Uncertain,
        w::ObservabilityNativePhase::Held => d::Phase::Held,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn entry_encode(value: &d::Entry) -> Result<w::ObservabilityNativeEntry, Invalid> {
    let parts = w::ObservabilityNativeEntryParts {
        project: match &(value.project) {
            Some(item) => Some(*item),
            None => None,
        },
        number: (value.number),
        key: key_encode(&(value.key))?,
        effect: effect_encode(&(value.effect))?,
        attempt: (value.attempt),
        deadline: (value.deadline),
        phase: phase_encode(value.phase),
    };
    let Ok(value) = w::ObservabilityNativeEntry::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn entry_decode(value: w::ObservabilityNativeEntry) -> d::Entry {
    let value = value.into_parts();
    d::Entry {
        project: match value.project {
            Some(item) => Some(item),
            None => None,
        },
        number: value.number,
        key: key_decode(value.key),
        effect: effect_decode(value.effect),
        attempt: value.attempt,
        deadline: value.deadline,
        phase: phase_decode(value.phase),
    }
}
