//! The application's own wire, at the engine's protocol boundary (client.md, 11).
use alloc::boxed::Box;
use jig_test_connector as d;
use jig_test_protocol_client_wire::v1 as w;
use skein_lib::{Reader, Writer};
mod documents;
mod facts;
mod parts;
use parts::{decode_application_request, encode_application_request};

/// A record outside this family's version, shape or configured bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Undecodable;
/// Scope carried by the record, beside the complete connector value.
#[derive(Debug, PartialEq, Eq)]
pub struct DecodedRequest {
    pub project: u32,
    pub value: d::ApplicationRequest,
}
/// The family checks each field and count; the connector owns admission sums.
#[must_use]
pub fn limits(limits: &d::Limits) -> Option<w::Limits> {
    let mut bounds = w::CEILINGS;
    bounds.segment_bytes = limits.segment_bytes;
    bounds.resource_path_segments = limits.path_segments;
    bounds.effect_resources = limits.resources_per_effect;
    bounds.description_resources = limits.resources_per_effect;
    bounds.client_document_panel_resources = limits.resources_per_task;
    bounds.client_document_hold_resources = limits.resources_per_task;
    bounds.client_document_project_resources = limits.resources;
    bounds.client_document_panel_effects = limits.entries;
    bounds.client_document_hold_effects = limits.entries;
    bounds.client_document_deployment_outbox = limits.entries;
    w::check_limits(&bounds).ok()?;
    Some(bounds)
}
/// Encode only a connector request; no digest is reconstructed from its fields.
#[must_use]
pub fn request_record(project: u32, value: &d::ApplicationRequest) -> Box<[u8]> {
    let record = w::Request::new(&w::CEILINGS, w::RequestParts { project, value: encode_application_request(value) })
        .expect("admitted request fits schema");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("bounded record"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode before constructing the root's input; its supplied digest stays untouched.
pub fn request(bytes: &[u8], bound: &d::Limits) -> Result<DecodedRequest, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Request::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    let w::RequestParts { project, value } = record.into_parts();
    Ok(DecodedRequest { project, value: decode_application_request(value, bound) })
}
#[must_use]
pub fn outcome(value: &d::ApplicationOutcome) -> Box<[u8]> {
    let record = w::Outcome::new(&w::CEILINGS, w::OutcomeParts { value: parts::encode_application_outcome(value) })
        .expect("admitted outcome fits schema");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("bounded record"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
pub fn outcome_value(bytes: &[u8], bound: &d::Limits) -> Result<d::ApplicationOutcome, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Outcome::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    Ok(parts::decode_application_outcome(record.into_parts().value, bound))
}
#[must_use]
pub fn subject(value: &d::ClientSubject) -> Box<[u8]> {
    let record = w::Subject::new(&w::CEILINGS, w::SubjectParts { value: parts::encode_client_subject(value) })
        .expect("admitted subject fits schema");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("bounded record"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
pub fn subject_value(bytes: &[u8], bound: &d::Limits) -> Result<d::ClientSubject, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Subject::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    Ok(parts::decode_client_subject(record.into_parts().value, bound))
}
#[must_use]
pub fn read(value: &d::ClientSubject) -> Box<[u8]> {
    let record = w::Read::new(&w::CEILINGS, w::ReadParts { value: parts::encode_client_subject(value) })
        .expect("admitted read fits schema");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("bounded record"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
pub fn read_value(bytes: &[u8], bound: &d::Limits) -> Result<d::ClientSubject, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Read::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    Ok(parts::decode_client_subject(record.into_parts().value, bound))
}
#[must_use]
pub fn document(value: &d::ClientDocument) -> Box<[u8]> {
    let record = w::Document::new(&w::CEILINGS, w::DocumentParts { value: documents::encode_client_document(value) })
        .expect("admitted document fits schema");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("bounded record"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
pub fn document_value(bytes: &[u8], bound: &d::Limits) -> Result<d::ClientDocument, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::Document::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    documents::decode_client_document(record.into_parts().value, bound)
}

/// Decode the owner body before handing its unchanged admission envelope to the root.
pub fn to_root(
    request: jig_protocol_engine_clients::ToApplication,
    bound: &d::Limits,
    rows: u32,
) -> Result<jig_test_domain::Event, Box<jig_protocol_engine_clients::Event>> {
    use jig_protocol_engine_clients::{Event, FromApplication, ToApplication};
    match request {
        ToApplication::Request { reply_to, sign_in, key, family, version, record, digest } => {
            let decoded = if family == 1 && version == jig_test_protocol_client_wire::VERSION {
                self::request(&record, bound)
            } else {
                Err(Undecodable)
            };
            let Ok(decoded) = decoded else {
                return Err(Box::new(Event::Application(FromApplication::Undecodable { to: reply_to })));
            };
            Ok(jig_test_domain::Event::Application {
                reply_to,
                sign_in,
                key,
                project: decoded.project,
                number: family,
                digest,
                value: decoded.value,
            })
        }
        ToApplication::Read { reply, sign_in, family, version, record } => {
            let decoded = if family == 1 && version == jig_test_protocol_client_wire::VERSION {
                read_value(&record, bound)
            } else {
                Err(Undecodable)
            };
            let Ok(subject) = decoded else {
                return Err(Box::new(Event::Application(FromApplication::Read { reply, family, record: None })));
            };
            Ok(jig_test_domain::Event::ReadApplication {
                reply,
                sign_in,
                scope: scope(subject),
                number: family,
                subject,
            })
        }
        ToApplication::Watch { watcher, sign_in, family, version, record } => {
            let decoded = if family == 1 && version == jig_test_protocol_client_wire::VERSION {
                subject_value(&record, bound)
            } else {
                Err(Undecodable)
            };
            let Ok(subject) = decoded else {
                return Err(Box::new(Event::WatchRefused { watcher, why: jig_domain_core::WatchRefusal::Unknown }));
            };
            Ok(jig_test_domain::Event::WatchApplication {
                watcher,
                sign_in,
                scope: scope(subject),
                number: family,
                subject,
                window: jig_domain_core::documents::Window { rows, after: None },
            })
        }
    }
}
fn scope(subject: d::ClientSubject) -> jig_domain_core::documents::ApplicationScope {
    use jig_domain_core::documents::ApplicationScope;
    match subject {
        d::ClientSubject::Object { project, .. }
        | d::ClientSubject::Panel { project, .. }
        | d::ClientSubject::Result { project, .. }
        | d::ClientSubject::Effect { project, .. }
        | d::ClientSubject::Outbox { project, .. }
        | d::ClientSubject::Hold { project, .. }
        | d::ClientSubject::Project { project } => ApplicationScope::Project(project),
        d::ClientSubject::Deployment => ApplicationScope::Deployment,
    }
}
/// A family's read terminal retains its exact absence reason in its own record.
#[must_use]
pub fn read_answer(answer: &Result<d::ClientDocument, d::DocumentMissing>) -> Box<[u8]> {
    let value = match answer {
        Ok(value) => w::ReadResult::Value(Box::new(
            w::ReadValue::new(&w::CEILINGS, w::ReadValueParts { value: documents::encode_client_document(value) })
                .expect("admitted document fits schema"),
        )),
        Err(d::DocumentMissing::Unknown) => w::ReadResult::Unknown,
        Err(d::DocumentMissing::NotAllowed) => w::ReadResult::NotAllowed,
    };
    let record = w::ReadAnswer::new(&w::CEILINGS, w::ReadAnswerParts { value }).expect("bounded read answer");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("bounded record"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
pub fn read_answer_value(
    bytes: &[u8],
    bound: &d::Limits,
) -> Result<Result<d::ClientDocument, d::DocumentMissing>, Undecodable> {
    let bounds = limits(bound).ok_or(Undecodable)?;
    let mut reader = Reader::new(bytes);
    let Ok(record) = w::ReadAnswer::decode(&bounds, &mut reader) else { return Err(Undecodable) };
    if reader.remaining() != 0 {
        return Err(Undecodable);
    }
    Ok(match record.into_parts().value {
        w::ReadResult::Value(value) => Ok(documents::decode_client_document(value.into_parts().value, bound)?),
        w::ReadResult::Unknown => Err(d::DocumentMissing::Unknown),
        w::ReadResult::NotAllowed => Err(d::DocumentMissing::NotAllowed),
    })
}
