//! Incident history belongs to observability (`examples.md`, sections 3 and 8).
use crate::{Alert, Domain, Fact, Limits, Record, Request, Service};
use skein_lib::{Env, List, Queue, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum IncidentState {
    Alerting,
    Resolved,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct IncidentHealth {
    pub observed: u64,
    pub healthy_replicas: u32,
    pub errors: u32,
    pub error_rate_percent: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct IncidentPoint {
    /// When this owner observed the change, rather than a claimed system actor's time.
    pub at: Wall,
    /// None is the alert's arrival; Some is a later service observation.
    pub health: Option<IncidentHealth>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Incident {
    pub project: u32,
    pub number: u64,
    pub watch: u64,
    pub raised: Wall,
    pub service: Service,
    pub severity: u8,
    pub state: IncidentState,
    pub timeline: List<IncidentPoint>,
    pub cut: bool,
}

pub(crate) fn capacity(limits: &Limits) -> Option<u32> {
    limits.watches.checked_mul(limits.alerts_per_batch)
}

fn append(incident: &mut Incident, point: IncidentPoint) {
    if incident.timeline.room() == 0 {
        // Keep the newest bounded suffix and state explicitly that its prefix was cut.
        for index in 1..incident.timeline.len() {
            let next = *incident.timeline.get(index).expect("bounded timeline index");
            *incident
                .timeline
                .get_mut(index.checked_sub(1).expect("index starts at one"))
                .expect("preceding timeline index") = next;
        }
        let last = incident.timeline.len().checked_sub(1).expect("nonzero timeline capacity");
        *incident.timeline.get_mut(last).expect("last timeline entry") = point;
        incident.cut = true;
    } else {
        incident.timeline.push(point).expect("timeline room checked");
    }
}

pub(crate) fn raised(domain: &mut Domain, env: &Env<Limits>, task: u64, alert: &Alert, out: &mut Queue<Request>) {
    let Some(watch) = domain.watches.get(&task) else { return };
    let key = (watch.project, alert.number);
    if domain.incidents.contains_key(&key) || domain.incidents.len() >= capacity(&env.limits).expect("incident bound") {
        return;
    }
    let mut incident = Incident {
        project: watch.project,
        number: alert.number,
        watch: task,
        raised: env.wall,
        service: alert.service.clone(),
        severity: alert.severity,
        state: IncidentState::Alerting,
        timeline: List::with_capacity(env.limits.samples_per_service.max(1)),
        cut: false,
    };
    append(&mut incident, IncidentPoint { at: env.wall, health: None });
    domain.incidents.insert(key, incident.clone()).expect("incident capacity checked");
    out.push(Request::Save { record: Record::Incident(incident) });
}

pub(crate) fn observed(
    domain: &mut Domain,
    env: &Env<Limits>,
    service: &Service,
    fact: &Fact,
    out: &mut Queue<Request>,
) {
    let health = IncidentHealth {
        observed: fact.observed,
        healthy_replicas: fact.healthy_replicas,
        errors: fact.errors,
        error_rate_percent: fact.error_rate_percent,
    };
    let mut keys = List::with_capacity(capacity(&env.limits).expect("incident bound"));
    for (key, incident) in &domain.incidents {
        if incident.service == *service && incident.state == IncidentState::Alerting {
            keys.push(*key).expect("bounded incidents");
        }
    }
    for key in &keys {
        let incident = domain.incidents.get_mut(key).expect("selected incident");
        let same = match incident.timeline.last() {
            Some(point) => point.health == Some(health),
            None => false,
        };
        if !same {
            append(incident, IncidentPoint { at: env.wall, health: Some(health) });
            if health.healthy_replicas > 0 && health.errors == 0 && health.error_rate_percent == 0 {
                incident.state = IncidentState::Resolved;
            }
            out.push(Request::Save { record: Record::Incident(incident.clone()) });
        }
    }
}

pub(crate) fn worst_case(limits: &Limits) -> Option<u64> {
    let count = capacity(limits)?;
    skein_lib::Map::<(u32, u64), Incident>::worst_case(count)?
        .checked_add(List::<(u32, u64)>::worst_case(count)?)?
        .checked_add(
            u64::from(count).checked_mul(u64::from(limits.name_bytes).checked_mul(2)?.checked_add(List::<
                IncidentPoint,
            >::worst_case(
                limits.samples_per_service.max(1),
            )?)?)?,
        )
}
