//! Keep: scoped whole documents and bounded owner facts; missing facts stay unknown.
//! Replace: incident/service vocabulary and the application objects its pages name.
//! Observability owns incident history and health (examples.md, 3 and 8).
use alloc::boxed::Box;
use jig_client::contract::{ObjectKey, Part, ProposalKey, Revision};
use skein_lib::{List, Wall};
/// Scope supplied with every read and watch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    Project(u32),
    Deployment,
}
/// Whole-document identity; an outbox entry is distinct from a proposal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Incidents { open: bool },
    Incident(u64),
    Panel(u64),
    Result(u64),
    Effect(ProposalKey),
    Outbox(u64),
    Hold(u64),
    Project(u32),
    Deployment,
}
/// A subject and its actual scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subject {
    pub scope: Scope,
    pub kind: Kind,
}
/// The service's names, copied from its owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Service {
    pub environment: Box<[u8]>,
    pub name: Box<[u8]>,
}
/// The owner's observations, without an inferred health threshold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Health {
    pub observed: Wall,
    pub healthy_replicas: u32,
    pub errors: u32,
    pub error_rate_percent: u8,
}
/// A retained incident observation; absent health denotes the alert arrival.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub at: Wall,
    pub health: Option<Health>,
}
/// The incident's committed lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Alerting,
    Resolved,
}
/// An incident, with the supplied lead identity and its bounded history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Incident {
    pub project: u32,
    pub number: u64,
    pub revision: Revision,
    pub watch: u64,
    pub raised: Wall,
    pub service: Service,
    pub severity: u8,
    pub state: State,
    pub timeline: List<Point>,
    pub cut: bool,
    pub health: Option<Health>,
    /// The watch is not the lead; absence stays unknown.
    pub lead: Option<u64>,
    pub proposals: List<ProposalKey>,
}
/// A qualified service and only the health its owner supplied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceState {
    pub service: Service,
    pub health: Option<Health>,
}
/// Current connector facts in a jig slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartDocument {
    pub subject: Subject,
    pub revision: Revision,
    pub service: Option<Service>,
    pub health: Option<Health>,
    pub incident: Option<Incident>,
    pub incidents: List<Incident>,
    pub services: List<ServiceState>,
    pub effects: List<Effect>,
    pub watches: Option<u32>,
    pub request_limit: Option<u32>,
    pub incident_limit: Option<u32>,
}
/// Recovery state of an observed silence effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectState {
    Kept,
    Sent,
    Uncertain,
    Held,
}
/// A retained outbox entry with its source and supplied attribution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effect {
    pub entry: u64,
    pub task: u64,
    pub project: Option<u32>,
    pub alert: u64,
    pub until: Wall,
    pub state: EffectState,
    pub attempt: u32,
    pub deadline: Wall,
    pub origin: crate::infrastructure::documents::Creation,
}
/// One authoritative document for a family page or connector part.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Document {
    Incidents { subject: Subject, revision: Revision, incidents: List<Incident> },
    Incident(Incident),
    Part(Box<PartDocument>),
}
impl Document {
    /// The identity echoed by this whole document.
    #[must_use]
    pub fn subject(&self) -> Subject {
        match self {
            Self::Incidents { subject, .. } => *subject,
            Self::Incident(incident) => {
                Subject { scope: Scope::Project(incident.project), kind: Kind::Incident(incident.number) }
            }
            Self::Part(part) => part.subject,
        }
    }
    /// The owner's current document revision.
    #[must_use]
    pub fn revision(&self) -> Revision {
        match self {
            Self::Incidents { revision, .. } => *revision,
            Self::Incident(incident) => incident.revision,
            Self::Part(part) => part.revision,
        }
    }
    /// Whole-value admission against the configured text, row and byte bounds.
    #[must_use]
    pub fn fits(&self, limits: &super::Limits) -> bool {
        match price(self, limits) {
            Some(bytes) => match bytes.checked_add(u64::try_from(size_of::<Document>()).expect("document layout")) {
                Some(bytes) => bytes <= limits.document_bytes,
                None => false,
            },
            None => false,
        }
    }
    /// jig objects supplied by the incident owner, never guessed from its watch.
    #[must_use]
    pub fn objects(&self, rows: u32) -> List<ObjectKey> {
        let mut objects = List::with_capacity(rows.checked_add(1).expect("incident object bound"));
        match self {
            Self::Incident(incident) => {
                if let Some(lead) = incident.lead {
                    objects.push(ObjectKey::Task(lead)).expect("lead capacity");
                }
                for proposal in &incident.proposals {
                    objects.push(ObjectKey::Proposal(*proposal)).expect("admitted proposal capacity");
                }
            }
            Self::Incidents { incidents, .. } => {
                for incident in incidents {
                    if let Some(lead) = incident.lead {
                        objects.push(ObjectKey::Task(lead)).expect("incident lead capacity");
                    }
                }
            }
            Self::Part(_) => {}
        }
        objects
    }
}
fn incident_price(incident: &Incident, limits: &super::Limits) -> Option<u64> {
    if incident.timeline.capacity() > limits.rows || incident.proposals.capacity() > limits.rows {
        return None;
    }
    let mut bytes = service_price(&incident.service, limits)?;
    bytes = bytes.checked_add(List::<Point>::worst_case(incident.timeline.capacity())?)?;
    bytes.checked_add(List::<ProposalKey>::worst_case(incident.proposals.capacity())?)
}
fn service_price(service: &Service, limits: &super::Limits) -> Option<u64> {
    if service.environment.len() > usize::try_from(limits.text).ok()?
        || service.name.len() > usize::try_from(limits.text).ok()?
    {
        return None;
    }
    u64::try_from(service.environment.len()).ok()?.checked_add(u64::try_from(service.name.len()).ok()?)
}
fn price(document: &Document, limits: &super::Limits) -> Option<u64> {
    match document {
        Document::Incident(incident) => incident_price(incident, limits),
        Document::Incidents { incidents, .. } => {
            if incidents.capacity() > limits.rows {
                return None;
            }
            let mut bytes = List::<Incident>::worst_case(incidents.capacity())?;
            for incident in incidents {
                bytes = bytes.checked_add(incident_price(incident, limits)?)?;
            }
            Some(bytes)
        }
        Document::Part(part) => {
            if part.effects.capacity() > limits.rows
                || part.incidents.capacity() > limits.rows
                || part.services.capacity() > limits.rows
            {
                return None;
            }
            let mut bytes = u64::try_from(size_of::<PartDocument>())
                .ok()?
                .checked_add(List::<Effect>::worst_case(part.effects.capacity())?)?
                .checked_add(List::<Incident>::worst_case(part.incidents.capacity())?)?
                .checked_add(List::<ServiceState>::worst_case(part.services.capacity())?)?;
            for incident in &part.incidents {
                bytes = bytes.checked_add(incident_price(incident, limits)?)?;
            }
            for service in &part.services {
                bytes = bytes.checked_add(service_price(&service.service, limits)?)?;
            }
            if let Some(service) = &part.service {
                bytes = bytes.checked_add(service_price(service, limits)?)?;
            }
            if let Some(incident) = &part.incident {
                bytes = bytes.checked_add(incident_price(incident, limits)?)?;
            }
            Some(bytes)
        }
    }
}
/// A scoped whole-read terminal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadResult {
    Value(Box<Document>),
    Unknown,
    Busy,
    SignedOut,
}
/// A level-triggered watch callback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WatchEvent {
    Opened,
    Alive,
    Delivered(Box<Document>),
}
/// One terminal for every opened watch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchEnd {
    Closed,
    Dropped,
    Unknown,
    SignedOut,
}
/// The subject for a part; resources and kinds need words only.
#[must_use]
pub fn subject(part: &Part, scope: Scope) -> Option<Subject> {
    let kind = match part {
        Part::Panel { task, .. } => Kind::Panel(*task),
        Part::Result { task, .. } => Kind::Result(*task),
        Part::Effect { proposal, .. } => Kind::Effect(*proposal),
        Part::Outbox { scope, entry, .. } => {
            let scope = match scope {
                jig_client::contract::PartScope::Project(project) => Scope::Project(*project),
                jig_client::contract::PartScope::Deployment => Scope::Deployment,
            };
            return Some(Subject { scope, kind: Kind::Outbox(*entry) });
        }
        Part::Hold { task, .. } => Kind::Hold(*task),
        Part::Project { project, .. } => {
            return Some(Subject { scope: Scope::Project(*project), kind: Kind::Project(*project) });
        }
        Part::Deployment { .. } => return Some(Subject { scope: Scope::Deployment, kind: Kind::Deployment }),
        Part::Connector { .. } | Part::Resource { .. } | Part::Kind { .. } => return None,
    };
    Some(Subject { scope, kind })
}
