use crate::v1 as w;
use skein_lib::List;

pub const PROJECT: u32 = 7;
pub const ENVIRONMENT: u64 = 17;
pub const REVISION: u64 = 3;
pub const UNTIL: u64 = 81;
const INCIDENT: u64 = 13;
const WATCH: u64 = 19;
const RAISED: u64 = 41;
const LEAD: u64 = 23;

pub fn request() -> w::InfrastructureAskRecord {
    let value = w::InfrastructureAskMoveEnd::new(
        &w::CEILINGS,
        w::InfrastructureAskMoveEndParts {
            project: PROJECT,
            environment: ENVIRONMENT,
            revision: REVISION,
            until: UNTIL,
        },
    )
    .expect("move end");
    w::InfrastructureAskRecord::new(
        &w::CEILINGS,
        w::InfrastructureAskRecordParts { value: w::InfrastructureAsk::MoveEnd(value) },
    )
    .expect("request record")
}

pub fn infrastructure_subject() -> w::InfrastructureSubject {
    w::InfrastructureSubject::new(
        &w::CEILINGS,
        w::InfrastructureSubjectParts {
            scope: w::InfrastructureScope::Deployment,
            kind: w::InfrastructureKind::Deployment,
        },
    )
    .expect("subject")
}

pub fn incident() -> w::ObservabilityIncident {
    w::ObservabilityIncident::new(
        &w::CEILINGS,
        w::ObservabilityIncidentParts {
            project: PROJECT,
            number: INCIDENT,
            revision: REVISION,
            watch: WATCH,
            raised: RAISED,
            service: w::ObservabilityService::new(
                &w::CEILINGS,
                w::ObservabilityServiceParts {
                    environment: b"staging".as_slice().into(),
                    name: b"api".as_slice().into(),
                },
            )
            .expect("service"),
            severity: 2,
            state: w::ObservabilityState::Alerting,
            timeline: List::with_capacity(0),
            cut: true,
            health: None,
            lead: Some(LEAD),
            proposals: List::with_capacity(0),
        },
    )
    .expect("incident")
}
