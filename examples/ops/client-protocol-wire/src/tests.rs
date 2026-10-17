use crate::{
    codec::{self, infrastructure as infra, observability as obs},
    v1 as w,
};
use skein_lib::List;

mod fixtures;
use fixtures::{ENVIRONMENT, PROJECT, REVISION, UNTIL, request};

#[test]
fn an_environment_request_round_trips_through_its_bytes() {
    let value = request();
    assert_eq!(infra::ask_value(&infra::ask(&value)), Ok(value));
}

#[test]
fn an_environment_answer_round_trips_through_its_bytes() {
    let value = w::InfrastructureAnswerMovedEnd::new(
        &w::CEILINGS,
        w::InfrastructureAnswerMovedEndParts { environment: ENVIRONMENT, revision: REVISION, until: UNTIL },
    )
    .expect("answer");
    let value = w::InfrastructureAnswerRecord::new(
        &w::CEILINGS,
        w::InfrastructureAnswerRecordParts { value: w::InfrastructureAnswer::MovedEnd(value) },
    )
    .expect("answer record");
    assert_eq!(infra::answer(&infra::answer_record(&value)), Ok(value));
}

#[test]
fn an_infrastructure_subject_round_trips_through_its_bytes() {
    let subject = fixtures::infrastructure_subject();
    let value =
        w::InfrastructureSubjectRecord::new(&w::CEILINGS, w::InfrastructureSubjectRecordParts { value: subject })
            .expect("subject record");
    assert_eq!(infra::subject_value(&infra::subject(&value)), Ok(value));
}

#[test]
fn an_observability_subject_round_trips_through_its_bytes() {
    let value = w::ObservabilitySubjectRecord::new(
        &w::CEILINGS,
        w::ObservabilitySubjectRecordParts {
            value: w::ObservabilitySubject::new(
                &w::CEILINGS,
                w::ObservabilitySubjectParts {
                    scope: w::ObservabilityScope::Deployment,
                    kind: w::ObservabilityKind::Deployment,
                },
            )
            .expect("subject"),
        },
    )
    .expect("subject record");
    assert_eq!(obs::subject_value(&obs::subject(&value)), Ok(value));
}

#[test]
fn an_environment_document_round_trips_through_its_bytes() {
    let value = w::InfrastructureDocumentEnvironments::new(
        &w::CEILINGS,
        w::InfrastructureDocumentEnvironmentsParts {
            subject: fixtures::infrastructure_subject(),
            revision: REVISION,
            pools: List::with_capacity(0),
            environments: List::with_capacity(0),
        },
    )
    .expect("environment list");
    let value = w::InfrastructureDocumentRecord::new(
        &w::CEILINGS,
        w::InfrastructureDocumentRecordParts { value: w::InfrastructureDocument::Environments(value) },
    )
    .expect("document record");
    assert_eq!(infra::document(&infra::document_record(&value)), Ok(value));
}

#[test]
fn an_incident_document_round_trips_through_its_bytes() {
    let incident = fixtures::incident();
    assert_eq!(incident.project(), PROJECT);
    let value =
        w::ObservabilityDocumentIncident::new(&w::CEILINGS, w::ObservabilityDocumentIncidentParts { value: incident })
            .expect("incident document");
    let value = w::ObservabilityDocumentRecord::new(
        &w::CEILINGS,
        w::ObservabilityDocumentRecordParts { value: w::ObservabilityDocument::Incident(value) },
    )
    .expect("document record");
    assert_eq!(obs::document(&obs::document_record(&value)), Ok(value));
}

#[test]
fn a_record_of_another_version_is_undecodable() {
    let mut bytes = infra::ask(&request());
    bytes[1] = 2;
    assert_eq!(infra::ask_value(&bytes), Err(codec::Undecodable));
}

#[test]
fn trailing_bytes_are_undecodable() {
    let bytes = infra::ask(&request());
    let mut longer = skein_lib::Writer::new(bytes.len() + 1);
    longer.put(&bytes).expect("request");
    longer.put(&[0]).expect("trailing byte");
    assert_eq!(infra::ask_value(&longer.finish()), Err(codec::Undecodable));
}

#[test]
fn a_truncated_record_is_undecodable() {
    let bytes = infra::ask(&request());
    assert_eq!(infra::ask_value(&bytes[..bytes.len() - 1]), Err(codec::Undecodable));
}
