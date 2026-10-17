//! Whole-value mappings for observability/native.
use crate::observability::native as d;
use crate::wire::{Invalid, w};
use skein_lib::List;
pub(crate) fn value_encode(value: &d::Value) -> Result<w::ObservabilityNativeValue, Invalid> {
    Ok(match value {
        d::Value::Service { service, fact } => {
            let parts = w::ObservabilityNativeValueServiceParts {
                service: crate::wire::observability_documents::service_encode(service)?,
                fact: crate::wire::observability_native::fact_encode(fact)?,
            };
            let Ok(value) = w::ObservabilityNativeValueService::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityNativeValue::Service(value)
        }
        d::Value::Alert { alert, raised, state, fact } => {
            let parts = w::ObservabilityNativeValueAlertParts {
                alert: crate::wire::observability_native::alert_encode(alert)?,
                raised: (*raised).as_nanos(),
                state: crate::wire::observability_documents::state_encode(*state),
                fact: match fact {
                    Some(item) => Some(crate::wire::observability_native::fact_encode(item)?),
                    None => None,
                },
            };
            let Ok(value) = w::ObservabilityNativeValueAlert::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityNativeValue::Alert(value)
        }
        d::Value::Outbox { entry } => {
            let parts = w::ObservabilityNativeValueOutboxParts {
                entry: crate::wire::observability_native::entry_encode(entry)?,
            };
            let Ok(value) = w::ObservabilityNativeValueOutbox::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityNativeValue::Outbox(value)
        }
        d::Value::Deployment { watches, effects, read_limit, incident_limit } => {
            let parts = w::ObservabilityNativeValueDeploymentParts {
                watches: (*watches),
                effects: {
                    let source = effects;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source {
                        let value = crate::wire::observability_native::entry_encode(item)?;
                        if values.push(value).is_err() {
                            return Err(Invalid);
                        }
                    }
                    values
                },
                read_limit: (*read_limit),
                incident_limit: (*incident_limit),
            };
            let Ok(value) = w::ObservabilityNativeValueDeployment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::ObservabilityNativeValue::Deployment(value)
        }
        d::Value::Incident { incident, fact } => {
            let parts = w::ObservabilityNativeValueIncidentParts {
                incident: crate::wire::observability_native::incident_encode(incident)?,
                fact: match fact {
                    Some(item) => Some(crate::wire::observability_native::fact_encode(item)?),
                    None => None,
                },
            };
            let Ok(value) = w::ObservabilityNativeValueIncident::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityNativeValue::Incident(value)
        }
        d::Value::Project { project, services, incidents } => {
            let parts = w::ObservabilityNativeValueProjectParts {
                project: (*project),
                services: {
                    let source = services;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source {
                        let value = crate::wire::observability_native::service_state_encode(item)?;
                        if values.push(value).is_err() {
                            return Err(Invalid);
                        }
                    }
                    values
                },
                incidents: {
                    let source = incidents;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source {
                        let value = crate::wire::observability_native::incident_encode(item)?;
                        if values.push(value).is_err() {
                            return Err(Invalid);
                        }
                    }
                    values
                },
            };
            let Ok(value) = w::ObservabilityNativeValueProject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityNativeValue::Project(value)
        }
    })
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn value_decode(value: w::ObservabilityNativeValue) -> d::Value {
    match value {
        w::ObservabilityNativeValue::Service(value) => {
            let value = value.into_parts();
            d::Value::Service {
                service: crate::wire::observability_documents::service_decode(value.service),
                fact: crate::wire::observability_native::fact_decode(value.fact),
            }
        }
        w::ObservabilityNativeValue::Alert(value) => {
            let value = value.into_parts();
            d::Value::Alert {
                alert: crate::wire::observability_native::alert_decode(value.alert),
                raised: skein_lib::Wall::from_nanos(value.raised),
                state: crate::wire::observability_documents::state_decode(value.state),
                fact: match value.fact {
                    Some(item) => Some(crate::wire::observability_native::fact_decode(item)),
                    None => None,
                },
            }
        }
        w::ObservabilityNativeValue::Outbox(value) => {
            let value = value.into_parts();
            d::Value::Outbox { entry: crate::wire::observability_native::entry_decode(value.entry) }
        }
        w::ObservabilityNativeValue::Deployment(value) => {
            let value = value.into_parts();
            d::Value::Deployment {
                watches: value.watches,
                effects: {
                    let source = value.effects;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source.into_boxed() {
                        let value = crate::wire::observability_native::entry_decode(item);
                        assert!(values.push(value).is_ok(), "same list capacity");
                    }
                    values
                },
                read_limit: value.read_limit,
                incident_limit: value.incident_limit,
            }
        }
        w::ObservabilityNativeValue::Incident(value) => {
            let value = value.into_parts();
            d::Value::Incident {
                incident: crate::wire::observability_native::incident_decode(value.incident),
                fact: match value.fact {
                    Some(item) => Some(crate::wire::observability_native::fact_decode(item)),
                    None => None,
                },
            }
        }
        w::ObservabilityNativeValue::Project(value) => {
            let value = value.into_parts();
            d::Value::Project {
                project: value.project,
                services: {
                    let source = value.services;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source.into_boxed() {
                        let value = crate::wire::observability_native::service_state_decode(item);
                        assert!(values.push(value).is_ok(), "same list capacity");
                    }
                    values
                },
                incidents: {
                    let source = value.incidents;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source.into_boxed() {
                        let value = crate::wire::observability_native::incident_decode(item);
                        assert!(values.push(value).is_ok(), "same list capacity");
                    }
                    values
                },
            }
        }
    }
}

pub(crate) fn document_encode(value: &d::Document) -> Result<w::ObservabilityNativeDocument, Invalid> {
    let parts = w::ObservabilityNativeDocumentParts {
        subject: crate::wire::observability_documents::subject_encode(&(value.subject))?,
        value: value_encode(&(value.value))?,
    };
    let Ok(value) = w::ObservabilityNativeDocument::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn document_decode(value: w::ObservabilityNativeDocument) -> d::Document {
    let value = value.into_parts();
    d::Document {
        subject: crate::wire::observability_documents::subject_decode(value.subject),
        value: value_decode(value.value),
    }
}
