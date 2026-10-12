//! Whole genuine owner facts, with no inferred lead, revision, proposal or load.
use jig_ops_domain_client::observability::{documents, native};
use jig_ops_domain_observability as source;
use skein_lib::List;
#[must_use]
pub fn service(value: source::Service) -> documents::Service {
    documents::Service { environment: value.environment, name: value.name }
}
#[must_use]
pub fn fact(value: source::Fact) -> native::Fact {
    native::Fact {
        observed: value.observed,
        healthy_replicas: value.healthy_replicas,
        errors: value.errors,
        error_rate_percent: value.error_rate_percent,
        load_percent: value.load_percent,
        load: value
            .load
            .into_vec()
            .into_iter()
            .map(|value| native::LoadPoint { observed: value.at, load_percent: value.percent })
            .collect::<Vec<_>>()
            .into(),
    }
}
#[must_use]
pub fn state(value: source::IncidentState) -> documents::State {
    match value {
        source::IncidentState::Alerting => documents::State::Alerting,
        source::IncidentState::Resolved => documents::State::Resolved,
    }
}
#[must_use]
pub fn incident(value: source::Incident) -> native::Incident {
    let mut timeline = List::with_capacity(value.timeline.capacity());
    for value in &value.timeline {
        timeline
            .push(native::Point {
                at: value.at,
                health: value.health.map(|health| native::Health {
                    observed: health.observed,
                    healthy_replicas: health.healthy_replicas,
                    errors: health.errors,
                    error_rate_percent: health.error_rate_percent,
                }),
            })
            .expect("source timeline capacity");
    }
    native::Incident {
        project: value.project,
        number: value.number,
        watch: value.watch,
        lead: value.lead,
        alerts: value.alerts,
        raised: value.raised,
        service: service(value.service),
        severity: value.severity,
        state: state(value.state),
        timeline,
        cut: value.cut,
    }
}
#[must_use]
pub fn purpose(value: source::Purpose) -> native::Purpose {
    match value {
        source::Purpose::Call { attempt, completion, position } => {
            native::Purpose::Call { attempt, completion, position }
        }
        source::Purpose::Procedure { purpose } => native::Purpose::Procedure { purpose },
        source::Purpose::Projection { purpose } => native::Purpose::Projection { purpose },
    }
}
#[must_use]
pub fn entry(value: source::Entry) -> native::Entry {
    native::Entry {
        project: value.project,
        number: value.number,
        key: native::Key {
            deployment: value.key.deployment,
            task: value.key.task,
            origin: purpose(value.key.origin),
            purpose: value.key.purpose,
        },
        effect: native::Effect { rule: value.effect.rule, until: value.effect.until },
        attempt: value.attempt,
        deadline: value.deadline,
        phase: match value.phase {
            source::Phase::Kept => native::Phase::Kept,
            source::Phase::Sent => native::Phase::Sent,
            source::Phase::Uncertain => native::Phase::Uncertain,
            source::Phase::Held => native::Phase::Held,
        },
    }
}
#[must_use]
pub fn document(value: source::ClientDocument) -> native::Value {
    match value {
        source::ClientDocument::Service { service: value, fact: current } => {
            native::Value::Service { service: service(value), fact: fact(current) }
        }
        source::ClientDocument::Incident { incident: value, fact: current } => {
            native::Value::Incident { incident: incident(value), fact: current.map(fact) }
        }
        source::ClientDocument::Alert { alert, raised, state: current, fact: health } => native::Value::Alert {
            alert: native::Alert { number: alert.number, service: service(alert.service), severity: alert.severity },
            raised,
            state: state(current),
            fact: health.map(fact),
        },
        source::ClientDocument::Outbox { entry: value } => native::Value::Outbox { entry: entry(value) },
        source::ClientDocument::Project { project, services, incidents } => {
            let mut views = List::with_capacity(services.capacity());
            for value in &services {
                views
                    .push(native::ServiceState {
                        service: service(value.service.clone()),
                        fact: value.fact.clone().map(fact),
                    })
                    .expect("source service capacity");
            }
            let mut history = List::with_capacity(incidents.capacity());
            for value in &incidents {
                history.push(incident(value.clone())).expect("source incident capacity");
            }
            native::Value::Project { project, services: views, incidents: history }
        }
        source::ClientDocument::Deployment { watches, effects, read_limit, incident_limit } => {
            let mut entries = List::with_capacity(effects.capacity());
            for value in &effects {
                entries.push(entry(value.clone())).expect("source outbox capacity");
            }
            native::Value::Deployment { watches, effects: entries, read_limit, incident_limit }
        }
    }
}
