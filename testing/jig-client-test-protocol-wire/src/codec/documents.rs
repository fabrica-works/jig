//! Complete connector documents and their nested collections.
use super::parts::{
    decode_application_object, decode_clientdocumentdeployment_outbox, decode_clientdocumenthold_effects,
    decode_clientdocumenthold_resources, decode_clientdocumentpanel_effects, decode_clientdocumentpanel_resources,
    decode_clientdocumentproject_resources, decode_description, decode_diagnostic_availability, decode_effect,
    decode_outbox_entry, decode_procedure_state, encode_application_object, encode_clientdocumentdeployment_outbox,
    encode_clientdocumenthold_effects, encode_clientdocumenthold_resources, encode_clientdocumentpanel_effects,
    encode_clientdocumentpanel_resources, encode_clientdocumentproject_resources, encode_description,
    encode_diagnostic_availability, encode_effect, encode_outbox_entry, encode_procedure_state,
};
use super::{Undecodable, d, w};
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn encode_client_document(value: &d::ClientDocument) -> w::ClientDocument {
    match value {
        d::ClientDocument::Object { object, now } => w::ClientDocument::Object(
            w::ClientDocumentObject::new(
                &w::CEILINGS,
                w::ClientDocumentObjectParts { object: *object, now: encode_application_object(now) },
            )
            .expect("admitted fields fit schema"),
        ),
        d::ClientDocument::Panel { task, procedure, result, resources, effects } => w::ClientDocument::Panel(
            w::ClientDocumentPanel::new(
                &w::CEILINGS,
                w::ClientDocumentPanelParts {
                    task: *task,
                    procedure: match procedure {
                        Some(value) => Some(encode_procedure_state(value)),
                        None => None,
                    },
                    result: match result {
                        Some(value) => Some(*value),
                        None => None,
                    },
                    resources: encode_clientdocumentpanel_resources(resources),
                    effects: encode_clientdocumentpanel_effects(effects),
                },
            )
            .expect("admitted fields fit schema"),
        ),
        d::ClientDocument::Result { task, code } => w::ClientDocument::Result(
            w::ClientDocumentResult::new(&w::CEILINGS, w::ClientDocumentResultParts { task: *task, code: *code })
                .expect("admitted fields fit schema"),
        ),
        d::ClientDocument::Effect { proposal, task, effect, description } => w::ClientDocument::Effect(
            w::ClientDocumentEffect::new(
                &w::CEILINGS,
                w::ClientDocumentEffectParts {
                    proposal: *proposal,
                    task: *task,
                    effect: encode_effect(effect),
                    description: encode_description(description),
                },
            )
            .expect("admitted fields fit schema"),
        ),
        d::ClientDocument::Outbox { entry, description } => w::ClientDocument::Outbox(
            w::ClientDocumentOutbox::new(
                &w::CEILINGS,
                w::ClientDocumentOutboxParts {
                    entry: encode_outbox_entry(entry),
                    description: encode_description(description),
                },
            )
            .expect("admitted fields fit schema"),
        ),
        d::ClientDocument::Hold { task, resources, effects } => w::ClientDocument::Hold(
            w::ClientDocumentHold::new(
                &w::CEILINGS,
                w::ClientDocumentHoldParts {
                    task: *task,
                    resources: encode_clientdocumenthold_resources(resources),
                    effects: encode_clientdocumenthold_effects(effects),
                },
            )
            .expect("admitted fields fit schema"),
        ),
        d::ClientDocument::Project { project, resources } => w::ClientDocument::Project(
            w::ClientDocumentProject::new(
                &w::CEILINGS,
                w::ClientDocumentProjectParts {
                    project: *project,
                    resources: encode_clientdocumentproject_resources(resources),
                },
            )
            .expect("admitted fields fit schema"),
        ),
        d::ClientDocument::Deployment { outbox, subject_limit, staged_limit, request_budget, system_last_response } => {
            w::ClientDocument::Deployment(
                w::ClientDocumentDeployment::new(
                    &w::CEILINGS,
                    w::ClientDocumentDeploymentParts {
                        outbox: encode_clientdocumentdeployment_outbox(outbox),
                        subject_limit: *subject_limit,
                        staged_limit: *staged_limit,
                        request_budget: encode_diagnostic_availability(*request_budget),
                        system_last_response: encode_diagnostic_availability(*system_last_response),
                    },
                )
                .expect("admitted fields fit schema"),
            )
        }
    }
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn decode_client_document(
    value: w::ClientDocument,
    limits: &d::Limits,
) -> Result<d::ClientDocument, Undecodable> {
    Ok(match value {
        w::ClientDocument::Object(value) => {
            let w::ClientDocumentObjectParts { object, now } = value.into_parts();
            d::ClientDocument::Object { object, now: decode_application_object(now, limits) }
        }
        w::ClientDocument::Panel(value) => {
            let w::ClientDocumentPanelParts { task, procedure, result, resources, effects } = value.into_parts();
            d::ClientDocument::Panel {
                task,
                procedure: match procedure {
                    Some(value) => Some(decode_procedure_state(value, limits)?),
                    None => None,
                },
                result: match result {
                    Some(value) => Some(value),
                    None => None,
                },
                resources: decode_clientdocumentpanel_resources(resources, limits)?,
                effects: decode_clientdocumentpanel_effects(effects, limits)?,
            }
        }
        w::ClientDocument::Result(value) => {
            let w::ClientDocumentResultParts { task, code } = value.into_parts();
            d::ClientDocument::Result { task, code }
        }
        w::ClientDocument::Effect(value) => {
            let w::ClientDocumentEffectParts { proposal, task, effect, description } = value.into_parts();
            d::ClientDocument::Effect {
                proposal,
                task,
                effect: decode_effect(effect, limits)?,
                description: decode_description(description, limits)?,
            }
        }
        w::ClientDocument::Outbox(value) => {
            let w::ClientDocumentOutboxParts { entry, description } = value.into_parts();
            d::ClientDocument::Outbox {
                entry: decode_outbox_entry(entry, limits)?,
                description: decode_description(description, limits)?,
            }
        }
        w::ClientDocument::Hold(value) => {
            let w::ClientDocumentHoldParts { task, resources, effects } = value.into_parts();
            d::ClientDocument::Hold {
                task,
                resources: decode_clientdocumenthold_resources(resources, limits)?,
                effects: decode_clientdocumenthold_effects(effects, limits)?,
            }
        }
        w::ClientDocument::Project(value) => {
            let w::ClientDocumentProjectParts { project, resources } = value.into_parts();
            d::ClientDocument::Project {
                project,
                resources: decode_clientdocumentproject_resources(resources, limits)?,
            }
        }
        w::ClientDocument::Deployment(value) => {
            let w::ClientDocumentDeploymentParts {
                outbox,
                subject_limit,
                staged_limit,
                request_budget,
                system_last_response,
            } = value.into_parts();
            d::ClientDocument::Deployment {
                outbox: decode_clientdocumentdeployment_outbox(outbox, limits)?,
                subject_limit,
                staged_limit,
                request_budget: decode_diagnostic_availability(request_budget, limits),
                system_last_response: decode_diagnostic_availability(system_last_response, limits),
            }
        }
    })
}
