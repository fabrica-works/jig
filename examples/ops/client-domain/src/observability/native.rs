//! Whole genuine observability facts (examples.md, 7.1 and 8; root.md, 6).
//! jig owns the lead's proposals; no connector revision is invented.
use super::documents::{Service, State, Subject};
use alloc::boxed::Box;
use skein_lib::{List, Wall};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadPoint {
    pub observed: u64,
    pub load_percent: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fact {
    pub observed: u64,
    pub healthy_replicas: u32,
    pub errors: u32,
    pub error_rate_percent: u8,
    pub load_percent: u8,
    pub load: Box<[LoadPoint]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Health {
    pub observed: u64,
    pub healthy_replicas: u32,
    pub errors: u32,
    pub error_rate_percent: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub at: Wall,
    pub health: Option<Health>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Incident {
    pub project: u32,
    pub number: u64,
    pub watch: u64,
    pub lead: Option<u64>,
    pub alerts: Box<[u64]>,
    pub raised: Wall,
    pub service: Service,
    pub severity: u8,
    pub state: State,
    pub timeline: List<Point>,
    pub cut: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceState {
    pub service: Service,
    pub fact: Option<Fact>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Alert {
    /// Number unique within the deployment.
    pub number: u64,
    /// Service that raised it.
    pub service: Service,
    /// Severity set by this connector's classification rules.
    pub severity: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Purpose {
    /// One activation-qualified agent call.
    Call { attempt: u64, completion: u32, position: u32 },
    /// One stable procedure purpose.
    Procedure { purpose: u64 },
    /// One stable goal projection purpose.
    Projection { purpose: u64 },
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Key {
    /// Deployment that owns the operation.
    pub deployment: [u8; 16],
    /// Asking task.
    pub task: u64,
    /// The call, procedure or projection that asked.
    pub origin: Purpose,
    /// Stable purpose within that owner.
    pub purpose: u64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Effect {
    /// Alert rule to silence.
    pub rule: Box<[u8]>,
    /// End of the silence in system seconds.
    pub until: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Saved but not yet sent.
    Kept,
    /// An attempt was committed for release.
    Sent,
    /// An answer was lost; lookup follows.
    Uncertain,
    /// The attempt bound was reached; a person must settle it.
    Held,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Entry {
    /// Admitted owner scope, retained with the outbox when known.
    pub project: Option<u32>,
    /// Core-owned outbox number, separate from the stable effect key.
    pub number: u64,
    /// Stable effect key.
    pub key: Key,
    /// The silence's value kept by this connector.
    pub effect: Effect,
    /// Last attempt number.
    pub attempt: u32,
    /// Absolute system time after which an uncertain attempt may be retried.
    pub deadline: u64,
    /// Current durable phase.
    pub phase: Phase,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Service { service: Service, fact: Fact },
    Alert { alert: Alert, raised: Wall, state: State, fact: Option<Fact> },
    Outbox { entry: Entry },
    Deployment { watches: u32, effects: List<Entry>, read_limit: u32, incident_limit: u32 },
    Incident { incident: Incident, fact: Option<Fact> },
    Project { project: u32, services: List<ServiceState>, incidents: List<Incident> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub subject: Subject,
    pub value: Value,
}
fn service_price(service: &Service, limits: &super::Limits) -> Option<u64> {
    if service.environment.len() > usize::try_from(limits.text).ok()?
        || service.name.len() > usize::try_from(limits.text).ok()?
    {
        return None;
    }
    u64::try_from(service.environment.len()).ok()?.checked_add(u64::try_from(service.name.len()).ok()?)
}
fn fact_price(fact: &Fact, limits: &super::Limits) -> Option<u64> {
    if fact.load.len() > usize::try_from(limits.rows).ok()? {
        return None;
    }
    u64::try_from(fact.load.len()).ok()?.checked_mul(u64::try_from(size_of::<LoadPoint>()).ok()?)
}
fn incident_price(incident: &Incident, limits: &super::Limits) -> Option<u64> {
    if incident.timeline.capacity() > limits.rows || incident.alerts.len() > usize::try_from(limits.rows).ok()? {
        return None;
    }
    service_price(&incident.service, limits)?
        .checked_add(List::<Point>::worst_case(incident.timeline.capacity())?)?
        .checked_add(u64::try_from(incident.alerts.len()).ok()?.checked_mul(8)?)
}
#[must_use]
pub fn price(document: &Document, limits: &super::Limits) -> Option<u64> {
    if !identity(document) {
        return None;
    }
    let bytes = match &document.value {
        Value::Service { service, fact } => service_price(service, limits)?.checked_add(fact_price(fact, limits)?)?,
        Value::Alert { alert, fact, .. } => {
            let mut bytes = service_price(&alert.service, limits)?;
            if let Some(fact) = fact {
                bytes = bytes.checked_add(fact_price(fact, limits)?)?;
            }
            bytes
        }
        Value::Outbox { entry } => text_price(&entry.effect.rule, limits)?,
        Value::Deployment { effects, .. } => {
            if effects.capacity() > limits.rows {
                return None;
            }
            let mut bytes = List::<Entry>::worst_case(effects.capacity())?;
            for entry in effects {
                bytes = bytes.checked_add(text_price(&entry.effect.rule, limits)?)?;
            }
            bytes
        }
        Value::Incident { incident, fact } => {
            let mut bytes = incident_price(incident, limits)?;
            if let Some(fact) = fact {
                bytes = bytes.checked_add(fact_price(fact, limits)?)?;
            }
            bytes
        }
        Value::Project { services, incidents, .. } => {
            if services.capacity() > limits.rows || incidents.capacity() > limits.rows {
                return None;
            }
            let mut bytes = List::<ServiceState>::worst_case(services.capacity())?
                .checked_add(List::<Incident>::worst_case(incidents.capacity())?)?;
            for service in services {
                bytes = bytes.checked_add(service_price(&service.service, limits)?)?;
                if let Some(fact) = &service.fact {
                    bytes = bytes.checked_add(fact_price(fact, limits)?)?;
                }
            }
            for incident in incidents {
                bytes = bytes.checked_add(incident_price(incident, limits)?)?;
            }
            bytes
        }
    };
    bytes.checked_add(u64::try_from(size_of::<Document>()).ok()?)
}

fn text_price(value: &[u8], limits: &super::Limits) -> Option<u64> {
    if value.len() > usize::try_from(limits.text).ok()? {
        return None;
    }
    u64::try_from(value.len()).ok()
}

fn identity(document: &Document) -> bool {
    use super::documents::{Kind, Scope};
    match &document.value {
        Value::Service { .. } | Value::Alert { .. } => false,
        Value::Incident { incident, .. } => {
            document.subject.scope == Scope::Project(incident.project)
                && document.subject.kind == Kind::Incident(incident.number)
        }
        Value::Outbox { entry } => {
            document.subject.kind == Kind::Outbox(entry.number)
                && match document.subject.scope {
                    Scope::Project(project) => entry.project == Some(project),
                    Scope::Deployment => false,
                }
        }
        Value::Project { project, incidents, .. } => {
            if document.subject.scope != Scope::Project(*project) {
                return false;
            }
            for incident in incidents {
                if incident.project != *project {
                    return false;
                }
            }
            match document.subject.kind {
                Kind::Incidents { .. } => true,
                Kind::Project(named) => named == *project,
                Kind::Incident(_)
                | Kind::Panel(_)
                | Kind::Result(_)
                | Kind::Effect(_)
                | Kind::Outbox(_)
                | Kind::Hold(_)
                | Kind::Deployment => false,
            }
        }
        Value::Deployment { .. } => {
            document.subject.scope == Scope::Deployment && document.subject.kind == Kind::Deployment
        }
    }
}
