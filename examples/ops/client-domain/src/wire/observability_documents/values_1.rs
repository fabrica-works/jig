//! Whole-value mappings for observability/documents.
use crate::observability::documents as d;
use crate::wire::{Invalid, w};
use skein_lib::List;
pub(crate) fn scope_encode(value: d::Scope) -> Result<w::ObservabilityScope, Invalid> {
    Ok(match value {
        d::Scope::Project(value) => {
            let parts = w::ObservabilityScopeProjectParts { value };
            let Ok(value) = w::ObservabilityScopeProject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityScope::Project(value)
        }
        d::Scope::Deployment => w::ObservabilityScope::Deployment,
    })
}
pub(crate) fn scope_decode(value: w::ObservabilityScope) -> d::Scope {
    match value {
        w::ObservabilityScope::Project(value) => {
            let value = value.into_parts();
            d::Scope::Project(value.value)
        }
        w::ObservabilityScope::Deployment => d::Scope::Deployment,
    }
}

pub(crate) fn kind_encode(value: &d::Kind) -> Result<w::ObservabilityKind, Invalid> {
    Ok(match value {
        d::Kind::Incidents { open } => {
            let parts = w::ObservabilityKindIncidentsParts { open: (*open) };
            let Ok(value) = w::ObservabilityKindIncidents::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Incidents(value)
        }
        d::Kind::Incident(value) => {
            let parts = w::ObservabilityKindIncidentParts { value: (*value) };
            let Ok(value) = w::ObservabilityKindIncident::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Incident(value)
        }
        d::Kind::Panel(value) => {
            let parts = w::ObservabilityKindPanelParts { value: (*value) };
            let Ok(value) = w::ObservabilityKindPanel::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Panel(value)
        }
        d::Kind::Result(value) => {
            let parts = w::ObservabilityKindResultParts { value: (*value) };
            let Ok(value) = w::ObservabilityKindResult::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Result(value)
        }
        d::Kind::Effect(value) => {
            let parts = w::ObservabilityKindEffectParts { value: crate::wire::proposal_encode(value)? };
            let Ok(value) = w::ObservabilityKindEffect::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Effect(value)
        }
        d::Kind::Outbox(value) => {
            let parts = w::ObservabilityKindOutboxParts { value: (*value) };
            let Ok(value) = w::ObservabilityKindOutbox::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Outbox(value)
        }
        d::Kind::Hold(value) => {
            let parts = w::ObservabilityKindHoldParts { value: (*value) };
            let Ok(value) = w::ObservabilityKindHold::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Hold(value)
        }
        d::Kind::Project(value) => {
            let parts = w::ObservabilityKindProjectParts { value: (*value) };
            let Ok(value) = w::ObservabilityKindProject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Project(value)
        }
        d::Kind::Deployment => w::ObservabilityKind::Deployment,
    })
}
pub(crate) fn kind_decode(value: w::ObservabilityKind) -> d::Kind {
    match value {
        w::ObservabilityKind::Incidents(value) => {
            let value = value.into_parts();
            d::Kind::Incidents { open: value.open }
        }
        w::ObservabilityKind::Incident(value) => {
            let value = value.into_parts();
            d::Kind::Incident(value.value)
        }
        w::ObservabilityKind::Panel(value) => {
            let value = value.into_parts();
            d::Kind::Panel(value.value)
        }
        w::ObservabilityKind::Result(value) => {
            let value = value.into_parts();
            d::Kind::Result(value.value)
        }
        w::ObservabilityKind::Effect(value) => {
            let value = value.into_parts();
            d::Kind::Effect(crate::wire::proposal_decode(value.value))
        }
        w::ObservabilityKind::Outbox(value) => {
            let value = value.into_parts();
            d::Kind::Outbox(value.value)
        }
        w::ObservabilityKind::Hold(value) => {
            let value = value.into_parts();
            d::Kind::Hold(value.value)
        }
        w::ObservabilityKind::Project(value) => {
            let value = value.into_parts();
            d::Kind::Project(value.value)
        }
        w::ObservabilityKind::Deployment => d::Kind::Deployment,
    }
}

pub(crate) fn subject_encode(value: &d::Subject) -> Result<w::ObservabilitySubject, Invalid> {
    let parts = w::ObservabilitySubjectParts { scope: scope_encode(value.scope)?, kind: kind_encode(&(value.kind))? };
    let Ok(value) = w::ObservabilitySubject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn subject_decode(value: w::ObservabilitySubject) -> d::Subject {
    let value = value.into_parts();
    d::Subject { scope: scope_decode(value.scope), kind: kind_decode(value.kind) }
}

pub(crate) fn service_encode(value: &d::Service) -> Result<w::ObservabilityService, Invalid> {
    let parts = w::ObservabilityServiceParts { environment: (value.environment).clone(), name: (value.name).clone() };
    let Ok(value) = w::ObservabilityService::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn service_decode(value: w::ObservabilityService) -> d::Service {
    let value = value.into_parts();
    d::Service { environment: value.environment, name: value.name }
}

pub(crate) fn health_encode(value: &d::Health) -> Result<w::ObservabilityHealth, Invalid> {
    let parts = w::ObservabilityHealthParts {
        observed: (value.observed).as_nanos(),
        healthy_replicas: (value.healthy_replicas),
        errors: (value.errors),
        error_rate_percent: (value.error_rate_percent),
    };
    let Ok(value) = w::ObservabilityHealth::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn health_decode(value: w::ObservabilityHealth) -> d::Health {
    let value = value.into_parts();
    d::Health {
        observed: skein_lib::Wall::from_nanos(value.observed),
        healthy_replicas: value.healthy_replicas,
        errors: value.errors,
        error_rate_percent: value.error_rate_percent,
    }
}

pub(crate) fn point_encode(value: &d::Point) -> Result<w::ObservabilityPoint, Invalid> {
    let parts = w::ObservabilityPointParts {
        at: (value.at).as_nanos(),
        health: match &(value.health) {
            Some(item) => Some(health_encode(item)?),
            None => None,
        },
    };
    let Ok(value) = w::ObservabilityPoint::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn point_decode(value: w::ObservabilityPoint) -> d::Point {
    let value = value.into_parts();
    d::Point {
        at: skein_lib::Wall::from_nanos(value.at),
        health: match value.health {
            Some(item) => Some(health_decode(item)),
            None => None,
        },
    }
}

pub(crate) fn state_encode(value: d::State) -> w::ObservabilityState {
    match value {
        d::State::Alerting => w::ObservabilityState::Alerting,
        d::State::Resolved => w::ObservabilityState::Resolved,
    }
}
pub(crate) fn state_decode(value: w::ObservabilityState) -> d::State {
    match value {
        w::ObservabilityState::Alerting => d::State::Alerting,
        w::ObservabilityState::Resolved => d::State::Resolved,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn incident_encode(value: &d::Incident) -> Result<w::ObservabilityIncident, Invalid> {
    let parts = w::ObservabilityIncidentParts {
        project: (value.project),
        number: (value.number),
        revision: (value.revision).value,
        watch: (value.watch),
        raised: (value.raised).as_nanos(),
        service: service_encode(&(value.service))?,
        severity: (value.severity),
        state: state_encode(value.state),
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
        health: match &(value.health) {
            Some(item) => Some(health_encode(item)?),
            None => None,
        },
        lead: match &(value.lead) {
            Some(item) => Some(*item),
            None => None,
        },
        proposals: {
            let source = &(value.proposals);
            let mut values = List::with_capacity(source.capacity());
            for item in source {
                let value = crate::wire::proposal_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
    };
    let Ok(value) = w::ObservabilityIncident::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn incident_decode(value: w::ObservabilityIncident) -> d::Incident {
    let value = value.into_parts();
    d::Incident {
        project: value.project,
        number: value.number,
        revision: jig_client::contract::Revision { value: value.revision },
        watch: value.watch,
        raised: skein_lib::Wall::from_nanos(value.raised),
        service: service_decode(value.service),
        severity: value.severity,
        state: state_decode(value.state),
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
        health: match value.health {
            Some(item) => Some(health_decode(item)),
            None => None,
        },
        lead: match value.lead {
            Some(item) => Some(item),
            None => None,
        },
        proposals: {
            let source = value.proposals;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = crate::wire::proposal_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values
        },
    }
}

pub(crate) fn service_state_encode(value: &d::ServiceState) -> Result<w::ObservabilityServiceState, Invalid> {
    let parts = w::ObservabilityServiceStateParts {
        service: service_encode(&(value.service))?,
        health: match &(value.health) {
            Some(item) => Some(health_encode(item)?),
            None => None,
        },
    };
    let Ok(value) = w::ObservabilityServiceState::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn service_state_decode(value: w::ObservabilityServiceState) -> d::ServiceState {
    let value = value.into_parts();
    d::ServiceState {
        service: service_decode(value.service),
        health: match value.health {
            Some(item) => Some(health_decode(item)),
            None => None,
        },
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn part_document_encode(value: &d::PartDocument) -> Result<w::ObservabilityPartDocument, Invalid> {
    let parts = w::ObservabilityPartDocumentParts {
        subject: subject_encode(&(value.subject))?,
        revision: (value.revision).value,
        service: match &(value.service) {
            Some(item) => Some(service_encode(item)?),
            None => None,
        },
        health: match &(value.health) {
            Some(item) => Some(health_encode(item)?),
            None => None,
        },
        incident: match &(value.incident) {
            Some(item) => Some(incident_encode(item)?),
            None => None,
        },
        incidents: {
            let source = &(value.incidents);
            let mut values = List::with_capacity(source.capacity());
            for item in source {
                let value = incident_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        services: {
            let source = &(value.services);
            let mut values = List::with_capacity(source.capacity());
            for item in source {
                let value = service_state_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        effects: {
            let source = &(value.effects);
            let mut values = List::with_capacity(source.capacity());
            for item in source {
                let value = effect_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        watches: match &(value.watches) {
            Some(item) => Some(*item),
            None => None,
        },
        read_limit: match &(value.read_limit) {
            Some(item) => Some(*item),
            None => None,
        },
        incident_limit: match &(value.incident_limit) {
            Some(item) => Some(*item),
            None => None,
        },
    };
    let Ok(value) = w::ObservabilityPartDocument::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn part_document_decode(value: w::ObservabilityPartDocument) -> d::PartDocument {
    let value = value.into_parts();
    d::PartDocument {
        subject: subject_decode(value.subject),
        revision: jig_client::contract::Revision { value: value.revision },
        service: match value.service {
            Some(item) => Some(service_decode(item)),
            None => None,
        },
        health: match value.health {
            Some(item) => Some(health_decode(item)),
            None => None,
        },
        incident: match value.incident {
            Some(item) => Some(incident_decode(item)),
            None => None,
        },
        incidents: {
            let source = value.incidents;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = incident_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values
        },
        services: {
            let source = value.services;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = service_state_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values
        },
        effects: {
            let source = value.effects;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = effect_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values
        },
        watches: match value.watches {
            Some(item) => Some(item),
            None => None,
        },
        read_limit: match value.read_limit {
            Some(item) => Some(item),
            None => None,
        },
        incident_limit: match value.incident_limit {
            Some(item) => Some(item),
            None => None,
        },
    }
}

pub(crate) fn effect_state_encode(value: d::EffectState) -> w::ObservabilityEffectState {
    match value {
        d::EffectState::Kept => w::ObservabilityEffectState::Kept,
        d::EffectState::Sent => w::ObservabilityEffectState::Sent,
        d::EffectState::Uncertain => w::ObservabilityEffectState::Uncertain,
        d::EffectState::Held => w::ObservabilityEffectState::Held,
    }
}
pub(crate) fn effect_state_decode(value: w::ObservabilityEffectState) -> d::EffectState {
    match value {
        w::ObservabilityEffectState::Kept => d::EffectState::Kept,
        w::ObservabilityEffectState::Sent => d::EffectState::Sent,
        w::ObservabilityEffectState::Uncertain => d::EffectState::Uncertain,
        w::ObservabilityEffectState::Held => d::EffectState::Held,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn effect_encode(value: &d::Effect) -> Result<w::ObservabilityEffect, Invalid> {
    let parts = w::ObservabilityEffectParts {
        entry: (value.entry),
        task: (value.task),
        project: match &(value.project) {
            Some(item) => Some(*item),
            None => None,
        },
        alert: (value.alert),
        until: (value.until).as_nanos(),
        state: effect_state_encode(value.state),
        attempt: (value.attempt),
        deadline: (value.deadline).as_nanos(),
        origin: crate::wire::infrastructure_documents::creation_encode(&(value.origin))?,
    };
    let Ok(value) = w::ObservabilityEffect::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn effect_decode(value: w::ObservabilityEffect) -> d::Effect {
    let value = value.into_parts();
    d::Effect {
        entry: value.entry,
        task: value.task,
        project: match value.project {
            Some(item) => Some(item),
            None => None,
        },
        alert: value.alert,
        until: skein_lib::Wall::from_nanos(value.until),
        state: effect_state_decode(value.state),
        attempt: value.attempt,
        deadline: skein_lib::Wall::from_nanos(value.deadline),
        origin: crate::wire::infrastructure_documents::creation_decode(value.origin),
    }
}
