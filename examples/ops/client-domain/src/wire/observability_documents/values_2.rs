//! Whole-value mappings for observability/documents.
use crate::observability::documents as d;
use crate::wire::{Invalid, w};
use alloc::boxed::Box;
use skein_lib::List;
pub(crate) fn document_encode(value: &d::Document) -> Result<w::ObservabilityDocument, Invalid> {
    Ok(match value {
        d::Document::Incidents { subject, revision, incidents } => {
            let parts = w::ObservabilityDocumentIncidentsParts {
                subject: crate::wire::observability_documents::subject_encode(subject)?,
                revision: revision.value,
                incidents: {
                    let source = incidents;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source {
                        let value = crate::wire::observability_documents::incident_encode(item)?;
                        if values.push(value).is_err() {
                            return Err(Invalid);
                        }
                    }
                    values
                },
            };
            let Ok(value) = w::ObservabilityDocumentIncidents::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityDocument::Incidents(value)
        }
        d::Document::Incident(value) => {
            let parts = w::ObservabilityDocumentIncidentParts {
                value: crate::wire::observability_documents::incident_encode(value)?,
            };
            let Ok(value) = w::ObservabilityDocumentIncident::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityDocument::Incident(value)
        }
        d::Document::Part(value) => {
            let parts = w::ObservabilityDocumentPartParts {
                value: crate::wire::observability_documents::part_document_encode(value)?,
            };
            let Ok(value) = w::ObservabilityDocumentPart::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityDocument::Part(value)
        }
        d::Document::Native(value) => {
            let parts = w::ObservabilityDocumentNativeParts {
                value: crate::wire::observability_native::document_encode(value)?,
            };
            let Ok(value) = w::ObservabilityDocumentNative::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityDocument::Native(value)
        }
    })
}
pub(crate) fn document_decode(value: w::ObservabilityDocument) -> d::Document {
    match value {
        w::ObservabilityDocument::Incidents(value) => {
            let value = value.into_parts();
            d::Document::Incidents {
                subject: crate::wire::observability_documents::subject_decode(value.subject),
                revision: jig_client::contract::Revision { value: value.revision },
                incidents: {
                    let source = value.incidents;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source.into_boxed() {
                        let value = crate::wire::observability_documents::incident_decode(item);
                        assert!(values.push(value).is_ok(), "same list capacity");
                    }
                    values
                },
            }
        }
        w::ObservabilityDocument::Incident(value) => {
            let value = value.into_parts();
            d::Document::Incident(crate::wire::observability_documents::incident_decode(value.value))
        }
        w::ObservabilityDocument::Part(value) => {
            let value = value.into_parts();
            d::Document::Part(Box::new(crate::wire::observability_documents::part_document_decode(value.value)))
        }
        w::ObservabilityDocument::Native(value) => {
            let value = value.into_parts();
            d::Document::Native(Box::new(crate::wire::observability_native::document_decode(value.value)))
        }
    }
}
