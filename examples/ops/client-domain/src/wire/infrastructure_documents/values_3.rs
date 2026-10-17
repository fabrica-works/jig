//! Whole-value mappings for infrastructure/documents.
use crate::infrastructure::documents as d;
use crate::wire::{Invalid, w};
use alloc::boxed::Box;
use skein_lib::List;
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn drift_encode(value: &d::Drift) -> Result<w::InfrastructureDrift, Invalid> {
    let parts = w::InfrastructureDriftParts {
        resource: (value.resource).clone(),
        kind: drift_kind_encode(value.kind),
        actor: match &(value.actor) {
            Some(item) => Some((*item).clone()),
            None => None,
        },
        at: match &(value.at) {
            Some(item) => Some((*item).as_nanos()),
            None => None,
        },
        observed: (value.observed).as_nanos(),
    };
    let Ok(value) = w::InfrastructureDrift::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn drift_decode(value: w::InfrastructureDrift) -> d::Drift {
    let value = value.into_parts();
    d::Drift {
        resource: value.resource,
        kind: drift_kind_decode(value.kind),
        actor: match value.actor {
            Some(item) => Some(item),
            None => None,
        },
        at: match value.at {
            Some(item) => Some(skein_lib::Wall::from_nanos(item)),
            None => None,
        },
        observed: skein_lib::Wall::from_nanos(value.observed),
    }
}

pub(crate) fn drift_kind_encode(value: d::DriftKind) -> w::InfrastructureDriftKind {
    match value {
        d::DriftKind::Deleted => w::InfrastructureDriftKind::Deleted,
        d::DriftKind::Changed => w::InfrastructureDriftKind::Changed,
        d::DriftKind::Replaced => w::InfrastructureDriftKind::Replaced,
        d::DriftKind::Ambiguous => w::InfrastructureDriftKind::Ambiguous,
    }
}
pub(crate) fn drift_kind_decode(value: w::InfrastructureDriftKind) -> d::DriftKind {
    match value {
        w::InfrastructureDriftKind::Deleted => d::DriftKind::Deleted,
        w::InfrastructureDriftKind::Changed => d::DriftKind::Changed,
        w::InfrastructureDriftKind::Replaced => d::DriftKind::Replaced,
        w::InfrastructureDriftKind::Ambiguous => d::DriftKind::Ambiguous,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn part_document_encode(value: &d::PartDocument) -> Result<w::InfrastructurePartDocument, Invalid> {
    let parts = w::InfrastructurePartDocumentParts {
        subject: crate::wire::infrastructure_documents::subject_encode(&(value.subject))?,
        revision: (value.revision).value,
        panel: match &(value.panel) {
            Some(item) => Some(crate::wire::infrastructure_documents::panel_encode(item)?),
            None => None,
        },
        description: match &(value.description) {
            Some(item) => Some(crate::wire::infrastructure_documents::description_encode(item)?),
            None => None,
        },
        effect: match &(value.effect) {
            Some(item) => Some(crate::wire::infrastructure_documents::effect_action_encode(item)?),
            None => None,
        },
        resources: {
            let source = &(value.resources);
            let mut values = List::with_capacity(source.capacity());
            for item in source {
                let value = crate::wire::infrastructure_documents::resource_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        effects: {
            let source = &(value.effects);
            let mut values = List::with_capacity(source.capacity());
            for item in source {
                let value = crate::wire::infrastructure_documents::effect_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        drift: {
            let source = &(value.drift);
            let mut values = List::with_capacity(source.capacity());
            for item in source {
                let value = drift_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        environments: {
            let source = &(value.environments);
            let mut values = List::with_capacity(source.capacity());
            for item in source {
                let value = crate::wire::infrastructure_documents::environment_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        pools: {
            let source = &(value.pools);
            let mut values = List::with_capacity(source.capacity());
            for item in source {
                let value = crate::wire::infrastructure_documents::pool_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        backend: match &(value.backend) {
            Some(item) => Some(crate::wire::infrastructure_documents::backend_encode(*item)),
            None => None,
        },
        made: match &(value.made) {
            Some(item) => Some(*item),
            None => None,
        },
        proposal_limit: match &(value.proposal_limit) {
            Some(item) => Some(*item),
            None => None,
        },
        staged_limit: match &(value.staged_limit) {
            Some(item) => Some(*item),
            None => None,
        },
    };
    let Ok(value) = w::InfrastructurePartDocument::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn part_document_decode(value: w::InfrastructurePartDocument) -> d::PartDocument {
    let value = value.into_parts();
    d::PartDocument {
        subject: crate::wire::infrastructure_documents::subject_decode(value.subject),
        revision: jig_client::contract::Revision { value: value.revision },
        panel: match value.panel {
            Some(item) => Some(crate::wire::infrastructure_documents::panel_decode(item)),
            None => None,
        },
        description: match value.description {
            Some(item) => Some(crate::wire::infrastructure_documents::description_decode(item)),
            None => None,
        },
        effect: match value.effect {
            Some(item) => Some(crate::wire::infrastructure_documents::effect_action_decode(item)),
            None => None,
        },
        resources: {
            let source = value.resources;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = crate::wire::infrastructure_documents::resource_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values
        },
        effects: {
            let source = value.effects;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = crate::wire::infrastructure_documents::effect_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values
        },
        drift: {
            let source = value.drift;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = drift_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values
        },
        environments: {
            let source = value.environments;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = crate::wire::infrastructure_documents::environment_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values
        },
        pools: {
            let source = value.pools;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = crate::wire::infrastructure_documents::pool_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values
        },
        backend: match value.backend {
            Some(item) => Some(crate::wire::infrastructure_documents::backend_decode(item)),
            None => None,
        },
        made: match value.made {
            Some(item) => Some(item),
            None => None,
        },
        proposal_limit: match value.proposal_limit {
            Some(item) => Some(item),
            None => None,
        },
        staged_limit: match value.staged_limit {
            Some(item) => Some(item),
            None => None,
        },
    }
}

pub(crate) fn document_encode(value: &d::Document) -> Result<w::InfrastructureDocument, Invalid> {
    Ok(match value {
        d::Document::Native(value) => {
            let parts = w::InfrastructureDocumentNativeParts {
                value: crate::wire::infrastructure_native::document_encode(value)?,
            };
            let Ok(value) = w::InfrastructureDocumentNative::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureDocument::Native(value)
        }
        d::Document::Environments { subject, revision, pools, environments } => {
            let parts = w::InfrastructureDocumentEnvironmentsParts {
                subject: crate::wire::infrastructure_documents::subject_encode(subject)?,
                revision: revision.value,
                pools: {
                    let source = pools;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source {
                        let value = crate::wire::infrastructure_documents::pool_encode(item)?;
                        if values.push(value).is_err() {
                            return Err(Invalid);
                        }
                    }
                    values
                },
                environments: {
                    let source = environments;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source {
                        let value = crate::wire::infrastructure_documents::environment_encode(item)?;
                        if values.push(value).is_err() {
                            return Err(Invalid);
                        }
                    }
                    values
                },
            };
            let Ok(value) = w::InfrastructureDocumentEnvironments::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureDocument::Environments(value)
        }
        d::Document::Environment(value) => {
            let parts = w::InfrastructureDocumentEnvironmentParts {
                value: crate::wire::infrastructure_documents::environment_encode(value)?,
            };
            let Ok(value) = w::InfrastructureDocumentEnvironment::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureDocument::Environment(value)
        }
        d::Document::Part(value) => {
            let parts = w::InfrastructureDocumentPartParts { value: part_document_encode(value)? };
            let Ok(value) = w::InfrastructureDocumentPart::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureDocument::Part(value)
        }
    })
}
pub(crate) fn document_decode(value: w::InfrastructureDocument) -> d::Document {
    match value {
        w::InfrastructureDocument::Native(value) => {
            let value = value.into_parts();
            d::Document::Native(Box::new(crate::wire::infrastructure_native::document_decode(value.value)))
        }
        w::InfrastructureDocument::Environments(value) => {
            let value = value.into_parts();
            d::Document::Environments {
                subject: crate::wire::infrastructure_documents::subject_decode(value.subject),
                revision: jig_client::contract::Revision { value: value.revision },
                pools: {
                    let source = value.pools;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source.into_boxed() {
                        let value = crate::wire::infrastructure_documents::pool_decode(item);
                        assert!(values.push(value).is_ok(), "same list capacity");
                    }
                    values
                },
                environments: {
                    let source = value.environments;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source.into_boxed() {
                        let value = crate::wire::infrastructure_documents::environment_decode(item);
                        assert!(values.push(value).is_ok(), "same list capacity");
                    }
                    values
                },
            }
        }
        w::InfrastructureDocument::Environment(value) => {
            let value = value.into_parts();
            d::Document::Environment(crate::wire::infrastructure_documents::environment_decode(value.value))
        }
        w::InfrastructureDocument::Part(value) => {
            let value = value.into_parts();
            d::Document::Part(Box::new(part_document_decode(value.value)))
        }
    }
}

pub(crate) fn ask_encode(value: &d::Ask) -> Result<w::InfrastructureAsk, Invalid> {
    Ok(match value {
        d::Ask::MoveEnd { project, environment, revision, until } => {
            let parts = w::InfrastructureAskMoveEndParts {
                project: (*project),
                environment: (*environment),
                revision: revision.value,
                until: (*until).as_nanos(),
            };
            let Ok(value) = w::InfrastructureAskMoveEnd::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureAsk::MoveEnd(value)
        }
        d::Ask::NativeMoveEnd { object, project, environment, revision, until } => {
            let parts = w::InfrastructureAskNativeMoveEndParts {
                object: (*object).raw(),
                project: (*project),
                environment: crate::wire::infrastructure_native::environment_encode(environment)?,
                revision: revision.value,
                until: (*until),
            };
            let Ok(value) = w::InfrastructureAskNativeMoveEnd::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureAsk::NativeMoveEnd(value)
        }
    })
}
pub(crate) fn ask_decode(value: w::InfrastructureAsk) -> d::Ask {
    match value {
        w::InfrastructureAsk::MoveEnd(value) => {
            let value = value.into_parts();
            d::Ask::MoveEnd {
                project: value.project,
                environment: value.environment,
                revision: jig_client::contract::Revision { value: value.revision },
                until: skein_lib::Wall::from_nanos(value.until),
            }
        }
        w::InfrastructureAsk::NativeMoveEnd(value) => {
            let value = value.into_parts();
            d::Ask::NativeMoveEnd {
                object: skein_lib::Token::new(value.object),
                project: value.project,
                environment: crate::wire::infrastructure_native::environment_decode(value.environment),
                revision: jig_client::contract::Revision { value: value.revision },
                until: value.until,
            }
        }
    }
}

pub(crate) fn refusal_encode(value: d::Refusal) -> Result<w::InfrastructureRefusal, Invalid> {
    Ok(match value {
        d::Refusal::Role { held } => {
            let parts = w::InfrastructureRefusalRoleParts { held };
            let Ok(value) = w::InfrastructureRefusalRole::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureRefusal::Role(value)
        }
        d::Refusal::Past => w::InfrastructureRefusal::Past,
        d::Refusal::Exhausted => w::InfrastructureRefusal::Exhausted,
        d::Refusal::SignedOut => w::InfrastructureRefusal::SignedOut,
        d::Refusal::NotAllowed => w::InfrastructureRefusal::NotAllowed,
        d::Refusal::Conflict => w::InfrastructureRefusal::Conflict,
        d::Refusal::Invalid => w::InfrastructureRefusal::Invalid,
    })
}
pub(crate) fn refusal_decode(value: w::InfrastructureRefusal) -> d::Refusal {
    match value {
        w::InfrastructureRefusal::Role(value) => {
            let value = value.into_parts();
            d::Refusal::Role { held: value.held }
        }
        w::InfrastructureRefusal::Past => d::Refusal::Past,
        w::InfrastructureRefusal::Exhausted => d::Refusal::Exhausted,
        w::InfrastructureRefusal::SignedOut => d::Refusal::SignedOut,
        w::InfrastructureRefusal::NotAllowed => d::Refusal::NotAllowed,
        w::InfrastructureRefusal::Conflict => d::Refusal::Conflict,
        w::InfrastructureRefusal::Invalid => d::Refusal::Invalid,
    }
}
