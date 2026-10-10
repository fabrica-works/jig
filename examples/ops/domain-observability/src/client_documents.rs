//! Connector-owned subjects and parts (`domain/connectors.md`, 5.5;
//! `client/domain/root.md`, section 6; `examples.md`, section 8).
use crate::{Domain, Incident, IncidentState, Limits, Request, Service};
use skein_lib::{Env, List, Queue, Token, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ClientScope {
    Project(u32),
    Deployment,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ClientSubject {
    Service { project: u32, service: Service },
    Incident { project: u32, incident: u64 },
    Alert { project: u32, alert: u64 },
    Outbox { project: u32, entry: u64 },
    Project { project: u32 },
    Deployment,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DocumentMissing {
    Unknown,
    NotAllowed,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClientService {
    pub service: Service,
    pub fact: Option<crate::Fact>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ClientDocument {
    Service { service: Service, fact: crate::Fact },
    Incident { incident: Incident, fact: Option<crate::Fact> },
    Alert { alert: crate::Alert, raised: Wall, state: IncidentState, fact: Option<crate::Fact> },
    Outbox { entry: crate::Entry },
    Project { project: u32, services: List<ClientService>, incidents: List<Incident> },
    Deployment { watches: u32, effects: List<crate::Entry>, request_limit: u32, incident_limit: u32 },
}

pub(crate) fn prepare(domain: &mut Domain, limits: &Limits, token: Token, subject: ClientSubject) {
    let valid = match &subject {
        ClientSubject::Service { service, .. } => crate::domain::service_valid(service, limits),
        ClientSubject::Incident { .. }
        | ClientSubject::Alert { .. }
        | ClientSubject::Outbox { .. }
        | ClientSubject::Project { .. }
        | ClientSubject::Deployment => true,
    };
    if valid {
        let _prepared = domain.client_subjects.insert(token, subject);
    }
}

fn belongs(domain: &Domain, project: u32, service: &Service) -> bool {
    for (_, watch) in &domain.watches {
        if watch.project == project && watch.services.contains(service) {
            return true;
        }
    }
    for (_, alert) in &domain.alerts {
        if alert.project == project && alert.alert.service == *service {
            return true;
        }
    }
    for (_, incident) in &domain.incidents {
        if incident.project == project && incident.service == *service {
            return true;
        }
    }
    false
}

fn scoped(scope: ClientScope, project: u32) -> Result<(), DocumentMissing> {
    if scope == ClientScope::Project(project) { Ok(()) } else { Err(DocumentMissing::Unknown) }
}

fn make(
    domain: &Domain,
    limits: &Limits,
    scope: ClientScope,
    token: Token,
    _party: u64,
) -> Result<ClientDocument, DocumentMissing> {
    let subject = domain.client_subjects.get(&token).ok_or(DocumentMissing::Unknown)?;
    match subject {
        ClientSubject::Service { project, service } => {
            scoped(scope, *project)?;
            if !belongs(domain, *project, service) {
                return Err(DocumentMissing::Unknown);
            }
            let fact = domain.facts.get(service).ok_or(DocumentMissing::Unknown)?.clone();
            Ok(ClientDocument::Service { service: service.clone(), fact })
        }
        ClientSubject::Incident { project, incident } => {
            scoped(scope, *project)?;
            let incident = domain.incidents.get(&(*project, *incident)).ok_or(DocumentMissing::Unknown)?;
            Ok(ClientDocument::Incident {
                incident: incident.clone(),
                fact: domain.facts.get(&incident.service).cloned(),
            })
        }
        ClientSubject::Alert { project, alert } => {
            scoped(scope, *project)?;
            let account = domain.alerts.get(&(*project, *alert)).ok_or(DocumentMissing::Unknown)?;
            let raised = account.raised;
            Ok(ClientDocument::Alert {
                alert: account.alert.clone(),
                raised,
                state: account.state,
                fact: domain.facts.get(&account.alert.service).cloned(),
            })
        }
        ClientSubject::Outbox { project, entry: number } => {
            scoped(scope, *project)?;
            for (_, entry) in &domain.effects {
                if entry.number == *number && entry.project == Some(*project) {
                    return Ok(ClientDocument::Outbox { entry: entry.clone() });
                }
            }
            Err(DocumentMissing::Unknown)
        }
        ClientSubject::Project { project } => {
            scoped(scope, *project)?;
            let mut services = List::with_capacity(limits.services);
            for (service, fact) in &domain.facts {
                if belongs(domain, *project, service) {
                    services
                        .push(ClientService { service: service.clone(), fact: Some(fact.clone()) })
                        .expect("bounded facts");
                }
            }
            let mut incidents = List::with_capacity(crate::incidents::capacity(limits).expect("incident bound"));
            for (_, incident) in &domain.incidents {
                if incident.project == *project {
                    incidents.push(incident.clone()).expect("bounded incidents");
                }
            }
            Ok(ClientDocument::Project { project: *project, services, incidents })
        }
        ClientSubject::Deployment => {
            if scope != ClientScope::Deployment {
                return Err(DocumentMissing::Unknown);
            }
            let mut effects = List::with_capacity(limits.effects);
            for (_, entry) in &domain.effects {
                effects.push(entry.clone()).expect("bounded outbox");
            }
            Ok(ClientDocument::Deployment {
                watches: domain.watches.len(),
                effects,
                request_limit: limits.reads,
                incident_limit: crate::incidents::capacity(limits).expect("incident bound"),
            })
        }
    }
}

pub(crate) fn document(
    domain: &Domain,
    limits: &Limits,
    watcher: Token,
    scope: ClientScope,
    subject: Token,
    party: u64,
    out: &mut Queue<Request>,
) {
    let document = make(domain, limits, scope, subject, party);
    out.push(Request::Document { watcher, subject, document });
}

pub(crate) fn read(
    domain: &mut Domain,
    limits: &Limits,
    scope: ClientScope,
    read: Token,
    party: u64,
    out: &mut Queue<Request>,
) {
    let answer = make(domain, limits, scope, read, party);
    domain.client_subjects.remove(&read);
    out.push(Request::ReadAnswered { read, answer });
}

pub(crate) fn changed(domain: &Domain, env: &Env<Limits>, service: &Service, out: &mut Queue<Request>) {
    let mut subjects = List::with_capacity(env.limits.reads);
    for (token, subject) in &domain.client_subjects {
        let touched = match subject {
            ClientSubject::Service { service: named, .. } => named == service,
            ClientSubject::Alert { project, alert } => match domain.alerts.get(&(*project, *alert)) {
                Some(account) => account.alert.service == *service,
                None => false,
            },
            ClientSubject::Incident { project, incident } => match domain.incidents.get(&(*project, *incident)) {
                Some(incident) => incident.service == *service,
                None => false,
            },
            ClientSubject::Project { project } => belongs(domain, *project, service),
            ClientSubject::Deployment | ClientSubject::Outbox { .. } => false,
        };
        if touched {
            subjects.push(*token).expect("bounded prepared client subjects");
        }
    }
    if !subjects.is_empty() {
        out.push(Request::Marked { subjects });
    }
}

/// Allocation bound for the largest owned document moved to the root's journal.
#[must_use]
pub fn document_worst_case(limits: &Limits) -> Option<u64> {
    let name = u64::from(limits.name_bytes).checked_mul(2)?;
    let fact = u64::from(limits.samples_per_service).checked_mul(u64::try_from(size_of::<crate::LoadPoint>()).ok()?)?;
    let timeline = List::<crate::IncidentPoint>::worst_case(limits.samples_per_service.max(1))?;
    let incidents = List::<Incident>::worst_case(crate::incidents::capacity(limits)?)?.checked_add(
        u64::from(crate::incidents::capacity(limits)?).checked_mul(
            name.checked_add(u64::from(limits.alerts_per_batch).checked_mul(8)?)?.checked_add(timeline)?,
        )?,
    )?;
    let services = List::<ClientService>::worst_case(limits.services)?
        .checked_add(u64::from(limits.services).checked_mul(name.checked_add(fact)?)?)?;
    let project = incidents.checked_add(services)?;
    let alert_set = u64::from(limits.alerts_per_batch).checked_mul(8)?;
    let incident = name.checked_add(alert_set)?.checked_add(timeline)?.checked_add(fact)?;
    let deployment = List::<crate::Entry>::worst_case(limits.effects)?
        .checked_add(u64::from(limits.effects).checked_mul(u64::from(limits.name_bytes))?)?;
    u64::try_from(size_of::<ClientDocument>()).ok()?.checked_add(project.max(incident).max(deployment))
}

/// Watch/outbox mutations affect the deployment part even without service facts.
pub(crate) fn records_changed(domain: &Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if domain.client_subjects.is_empty() {
        return;
    }
    let mut changed = false;
    for _ in 0..out.len() {
        let request = out.pop().expect("fixed output count");
        changed |= match &request {
            Request::Save { record } => match record {
                crate::Record::Watch(_)
                | crate::Record::Outbox(_)
                | crate::Record::Incident(_)
                | crate::Record::Alert(_)
                | crate::Record::Delegation(_) => true,
                crate::Record::Subscription { .. } => false,
            },
            Request::Erase { key } => match key {
                crate::RecordKey::Watch(_) | crate::RecordKey::Outbox(_) => true,
                crate::RecordKey::Incident { .. }
                | crate::RecordKey::Alert { .. }
                | crate::RecordKey::Delegation { .. }
                | crate::RecordKey::Subscription(..) => false,
            },
            Request::Document { .. }
            | Request::ReadAnswered { .. }
            | Request::Marked { .. }
            | Request::Closed { .. }
            | Request::Released { .. }
            | Request::Restarted { .. }
            | Request::Verdict { .. }
            | Request::Answer { .. }
            | Request::News { .. }
            | Request::Health { .. }
            | Request::WatchInterest { .. }
            | Request::Triage { .. }
            | Request::WatchDecision { .. }
            | Request::WakeProcedure { .. }
            | Request::AssociationRefused { .. }
            | Request::Described { .. }
            | Request::Refused { .. }
            | Request::Make { .. }
            | Request::Outcome { .. }
            | Request::Changed { .. }
            | Request::System(_) => false,
        };
        out.push(request);
    }
    if changed {
        let mut subjects = List::with_capacity(env.limits.reads);
        for (token, _) in &domain.client_subjects {
            subjects.push(*token).expect("bounded subjects");
        }
        if !subjects.is_empty() {
            out.push(Request::Marked { subjects });
        }
    }
}
