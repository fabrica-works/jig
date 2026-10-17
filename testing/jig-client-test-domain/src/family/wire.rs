//! The application's actual scoped owner values (client/domain/root.md, 6–7).
use crate::{
    connector_values::{ConnectorAnswer, ConnectorDocument, ConnectorLimits},
    family,
};
use alloc::boxed::Box;
use jig_test_connector::{ApplicationOutcome, ApplicationRequest, ClientDocument, ClientScope, ClientSubject};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    EntranceRefusal,
    Identity,
    TooLarge,
}

/// Preserve the supplied scope and every scripted subject kind.
#[must_use]
pub fn subject(value: family::Subject) -> (ClientScope, jig_client_test_protocol_wire::v1::FamilySubject) {
    let scope = match value.scope {
        family::Scope::Project(project) => ClientScope::Project(project),
        family::Scope::Deployment => ClientScope::Deployment,
    };
    (scope, scripted::subject(value))
}
/// Recover the whole family subject, including jig proposal makers.
#[must_use]
pub fn subject_value(value: jig_client_test_protocol_wire::v1::FamilySubject) -> family::Subject {
    scripted::subject_value(value)
}
/// Carry absent parameters explicitly, separately from an explicit zero.
#[must_use]
pub fn request(value: &family::Ask) -> (u32, jig_client_test_protocol_wire::v1::ApplicationRequest) {
    use jig_client_test_protocol_wire::v1 as w;
    let (project, object, revision, value) = match value {
        family::Ask::Touch { project, item, revision, parameters } => (*project, *item, revision.value, *parameters),
        family::Ask::ChangeObject { project, object, revision, value } => {
            (*project, *object, revision.value, Some(*value))
        }
    };
    let value =
        w::ApplicationRequest::new(&w::CEILINGS, w::ApplicationRequestParts { object, revision, value, release: None })
            .expect("fixed family request");
    (project, value)
}

/// Admit a whole family subject to the connector's smaller vocabulary.
/// # Errors
/// Refuses a subject without a representation in that connector.
pub fn connector_subject(value: family::Subject) -> Result<(ClientScope, ClientSubject), Invalid> {
    let (scope, value) = subject(value);
    let Ok(value) = jig_client_test_protocol_wire::codec::family::connector_subject_value(value) else {
        return Err(Invalid::Identity);
    };
    Ok((scope, value))
}
/// Admit a whole family request to the connector's fixed-value vocabulary.
/// # Errors
/// Refuses absent values rather than constructing a connector parameter.
pub fn connector_request(value: &family::Ask) -> Result<(u32, ApplicationRequest), Invalid> {
    let (project, value) = request(value);
    let Ok(value) = jig_client_test_protocol_wire::codec::family::connector_request(value) else {
        return Err(Invalid::Identity);
    };
    Ok((project, value))
}

#[must_use]
pub fn answer(value: ApplicationOutcome) -> ConnectorAnswer {
    ConnectorAnswer { value }
}

/// # Errors
/// Refuses missing supplied mutation values, unsupported individual deployment scope, mismatched subjects or a whole owner value above bounds.
pub fn document(
    original: family::Subject,
    value: ClientDocument,
    limits: &ConnectorLimits,
) -> Result<ConnectorDocument, Invalid> {
    let (_, named) = connector_subject(original)?;
    let same = match named {
        ClientSubject::Object { object, .. } => match &value {
            ClientDocument::Object { object: actual, .. } => *actual == object,
            ClientDocument::Panel { .. }
            | ClientDocument::Result { .. }
            | ClientDocument::Effect { .. }
            | ClientDocument::Outbox { .. }
            | ClientDocument::Hold { .. }
            | ClientDocument::Project { .. }
            | ClientDocument::Deployment { .. } => false,
        },
        ClientSubject::Panel { task, .. } => match &value {
            ClientDocument::Panel { task: actual, .. } => *actual == task,
            ClientDocument::Object { .. }
            | ClientDocument::Result { .. }
            | ClientDocument::Effect { .. }
            | ClientDocument::Outbox { .. }
            | ClientDocument::Hold { .. }
            | ClientDocument::Project { .. }
            | ClientDocument::Deployment { .. } => false,
        },
        ClientSubject::Result { task, .. } => match &value {
            ClientDocument::Result { task: actual, .. } => *actual == task,
            ClientDocument::Object { .. }
            | ClientDocument::Panel { .. }
            | ClientDocument::Effect { .. }
            | ClientDocument::Outbox { .. }
            | ClientDocument::Hold { .. }
            | ClientDocument::Project { .. }
            | ClientDocument::Deployment { .. } => false,
        },
        ClientSubject::Effect { proposal, .. } => match &value {
            ClientDocument::Effect { proposal: actual, .. } => *actual == proposal,
            ClientDocument::Object { .. }
            | ClientDocument::Panel { .. }
            | ClientDocument::Result { .. }
            | ClientDocument::Outbox { .. }
            | ClientDocument::Hold { .. }
            | ClientDocument::Project { .. }
            | ClientDocument::Deployment { .. } => false,
        },
        ClientSubject::Outbox { entry, .. } => match &value {
            ClientDocument::Outbox { entry: actual, .. } => actual.number == entry,
            ClientDocument::Object { .. }
            | ClientDocument::Panel { .. }
            | ClientDocument::Result { .. }
            | ClientDocument::Effect { .. }
            | ClientDocument::Hold { .. }
            | ClientDocument::Project { .. }
            | ClientDocument::Deployment { .. } => false,
        },
        ClientSubject::Hold { task, .. } => match &value {
            ClientDocument::Hold { task: actual, .. } => *actual == task,
            ClientDocument::Object { .. }
            | ClientDocument::Panel { .. }
            | ClientDocument::Result { .. }
            | ClientDocument::Effect { .. }
            | ClientDocument::Outbox { .. }
            | ClientDocument::Project { .. }
            | ClientDocument::Deployment { .. } => false,
        },
        ClientSubject::Project { project } => match &value {
            ClientDocument::Project { project: actual, .. } => *actual == project,
            ClientDocument::Object { .. }
            | ClientDocument::Panel { .. }
            | ClientDocument::Result { .. }
            | ClientDocument::Effect { .. }
            | ClientDocument::Outbox { .. }
            | ClientDocument::Hold { .. }
            | ClientDocument::Deployment { .. } => false,
        },
        ClientSubject::Deployment => match &value {
            ClientDocument::Deployment { .. } => true,
            ClientDocument::Object { .. }
            | ClientDocument::Panel { .. }
            | ClientDocument::Result { .. }
            | ClientDocument::Effect { .. }
            | ClientDocument::Outbox { .. }
            | ClientDocument::Hold { .. }
            | ClientDocument::Project { .. } => false,
        },
    };
    if !same {
        return Err(Invalid::Identity);
    }
    let result = ConnectorDocument { subject: original, value: Box::new(value) };
    if !crate::connector_bounds::document_fits(&result, limits) {
        return Err(Invalid::TooLarge);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_client::contract::Revision;
    use skein_lib::List;

    #[test]
    fn application_requests_keep_an_explicit_zero_and_owner_documents_reject_mismatched_or_partial_values() {
        let (_, encoded) = request(&family::Ask::ChangeObject {
            project: u32::MAX,
            object: u64::MAX,
            revision: Revision { value: u64::MAX },
            value: 0,
        });
        assert_eq!(*encoded.value(), Some(0));
        let original = family::Subject { scope: family::Scope::Project(4), kind: family::Kind::Panel(77) };
        let limits = ConnectorLimits { rows: 3, paths: 3, segments: 3, segment_bytes: 64, bytes: 100_000 };
        let panel = |task, capacity| ClientDocument::Panel {
            task,
            procedure: None,
            result: None,
            resources: List::with_capacity(capacity),
            effects: List::with_capacity(0),
        };
        assert_eq!(document(original, panel(78, 0), &limits), Err(Invalid::Identity));
        assert_eq!(document(original, panel(77, 4), &limits), Err(Invalid::TooLarge));
        let received = document(original, panel(77, 3), &limits).expect("the complete supplied fixture is admitted");
        assert_eq!(received.subject, original);
        assert_eq!(received.revision(), None);
        assert_eq!(received.value.as_ref(), &panel(77, 3));
        for value in
            [ApplicationOutcome::Role { held: u32::MAX }, ApplicationOutcome::Unknown, ApplicationOutcome::Exhausted]
        {
            assert_eq!(answer(value).value, value);
        }
    }
}

mod scripted {
    //! Exact scripted-family fields, separate from the connector's vocabulary.
    use crate::family as d;
    use jig_client::contract::{ProposalKey, Proposer, Revision};
    use jig_client_test_protocol_wire::v1 as w;
    use skein_lib::List;

    pub(super) fn key(value: ProposalKey) -> w::FamilyProposalKey {
        let proposer = match value.proposer {
            Proposer::Task(value) => w::FamilyProposer::Task(number(value)),
            Proposer::Party(value) => w::FamilyProposer::Party(number(value)),
        };
        w::FamilyProposalKey::new(&w::CEILINGS, w::FamilyProposalKeyParts { proposer, number: value.number })
            .expect("fixed proposal key")
    }
    pub(super) fn key_value(value: w::FamilyProposalKey) -> ProposalKey {
        let value = value.into_parts();
        let proposer = match value.proposer {
            w::FamilyProposer::Task(value) => Proposer::Task(value.value()),
            w::FamilyProposer::Party(value) => Proposer::Party(value.value()),
        };
        ProposalKey { proposer, number: value.number }
    }
    fn number(value: u64) -> w::FamilyNumber {
        w::FamilyNumber::new(&w::CEILINGS, w::FamilyNumberParts { value }).expect("fixed number")
    }
    fn project(value: u32) -> w::FamilyProject {
        w::FamilyProject::new(&w::CEILINGS, w::FamilyProjectParts { value }).expect("fixed project")
    }
    pub(super) fn subject(value: d::Subject) -> w::FamilySubject {
        let scope = match value.scope {
            d::Scope::Project(value) => w::FamilyScope::Project(project(value)),
            d::Scope::Deployment => w::FamilyScope::Deployment,
        };
        let kind = match value.kind {
            d::Kind::Item(value) => w::FamilyKind::Item(number(value)),
            d::Kind::Panel(value) => w::FamilyKind::Panel(number(value)),
            d::Kind::Result(value) => w::FamilyKind::Result(number(value)),
            d::Kind::Effect(value) => w::FamilyKind::Effect(key(value)),
            d::Kind::OwnerEffect(value) => w::FamilyKind::OwnerEffect(number(value)),
            d::Kind::Outbox(value) => w::FamilyKind::Outbox(number(value)),
            d::Kind::Hold(value) => w::FamilyKind::Hold(number(value)),
            d::Kind::Project(value) => w::FamilyKind::Project(project(value)),
            d::Kind::Deployment => w::FamilyKind::Deployment,
        };
        w::FamilySubject::new(&w::CEILINGS, w::FamilySubjectParts { scope, kind }).expect("fixed subject")
    }
    pub(super) fn subject_value(value: w::FamilySubject) -> d::Subject {
        let value = value.into_parts();
        let scope = match value.scope {
            w::FamilyScope::Project(value) => d::Scope::Project(value.value()),
            w::FamilyScope::Deployment => d::Scope::Deployment,
        };
        let kind = match value.kind {
            w::FamilyKind::Item(value) => d::Kind::Item(value.value()),
            w::FamilyKind::Panel(value) => d::Kind::Panel(value.value()),
            w::FamilyKind::Result(value) => d::Kind::Result(value.value()),
            w::FamilyKind::Effect(value) => d::Kind::Effect(key_value(value)),
            w::FamilyKind::OwnerEffect(value) => d::Kind::OwnerEffect(value.value()),
            w::FamilyKind::Outbox(value) => d::Kind::Outbox(value.value()),
            w::FamilyKind::Hold(value) => d::Kind::Hold(value.value()),
            w::FamilyKind::Project(value) => d::Kind::Project(value.value()),
            w::FamilyKind::Deployment => d::Kind::Deployment,
        };
        d::Subject { scope, kind }
    }
    pub(super) fn item(value: &d::Item) -> w::Item {
        let state = match value.state {
            d::ItemState::Ready => w::ItemState::Ready,
            d::ItemState::Held => w::ItemState::Held,
            d::ItemState::Ended => w::ItemState::Ended,
        };
        let mut proposals = List::with_capacity(value.proposals.len());
        for proposal in &value.proposals {
            proposals.push(key(*proposal)).expect("reserved proposal keys");
        }
        w::Item::new(
            &w::CEILINGS,
            w::ItemParts {
                project: value.project,
                key: value.key,
                revision: value.revision.value,
                state,
                value: value.value,
                task: value.task,
                second: value.second,
                proposals,
                touchable: value.touchable,
            },
        )
        .expect("admitted item")
    }
    pub(super) fn item_value(value: w::Item) -> d::Item {
        let value = value.into_parts();
        let state = match value.state {
            w::ItemState::Ready => d::ItemState::Ready,
            w::ItemState::Held => d::ItemState::Held,
            w::ItemState::Ended => d::ItemState::Ended,
        };
        let mut proposals = List::with_capacity(value.proposals.len());
        for proposal in &value.proposals {
            proposals.push(key_value(proposal.clone())).expect("reserved proposal keys");
        }
        d::Item {
            project: value.project,
            key: value.key,
            revision: Revision { value: value.revision },
            state,
            value: value.value,
            task: value.task,
            second: value.second,
            proposals,
            touchable: value.touchable,
        }
    }
    #[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
    pub(super) fn part(value: &d::PartDocument) -> w::PartDocument {
        let parameters = match value.parameters {
            Some(value) => Some(
                w::Parameters::new(
                    &w::CEILINGS,
                    w::ParametersParts {
                        item: value.item,
                        revision: value.revision.value,
                        value: value.value,
                        amendable: value.amendable,
                    },
                )
                .expect("fixed parameters"),
            ),
            None => None,
        };
        w::PartDocument::new(
            &w::CEILINGS,
            w::PartDocumentParts {
                subject: subject(value.subject),
                revision: value.revision.value,
                value: value.value,
                parameters,
            },
        )
        .expect("fixed part")
    }
    pub(super) fn part_value(value: w::PartDocument) -> d::PartDocument {
        let value = value.into_parts();
        let parameters = match value.parameters {
            Some(value) => {
                let value = value.into_parts();
                Some(d::Parameters {
                    item: value.item,
                    revision: Revision { value: value.revision },
                    value: value.value,
                    amendable: value.amendable,
                })
            }
            None => None,
        };
        d::PartDocument {
            subject: subject_value(value.subject),
            revision: Revision { value: value.revision },
            value: value.value,
            parameters,
        }
    }
}

/// Encode every scripted field and the complete connector document, with its supplied subject.
#[must_use]
pub fn encode_document(value: &family::Document) -> Box<[u8]> {
    use jig_client_test_protocol_wire::{codec, v1 as w};
    let value = match value {
        family::Document::Item(value) => w::FamilyDocument::Item(scripted::item(value)),
        family::Document::Part(value) => w::FamilyDocument::Part(scripted::part(value)),
        family::Document::Connector(value) => w::FamilyDocument::Connector(
            w::FamilyConnectorDocument::new(
                &w::CEILINGS,
                w::FamilyConnectorDocumentParts {
                    subject: scripted::subject(value.subject),
                    value: codec::family::connector_document(&value.value),
                },
            )
            .expect("admitted connector document"),
        ),
    };
    codec::family::document(value)
}
/// # Errors
/// Refuses malformed records or values outside the family's configured bounds.
pub fn decode_document(
    bytes: &[u8],
    bounds: &jig_test_connector::Limits,
    limits: &ConnectorLimits,
) -> Result<family::Document, Invalid> {
    use jig_client_test_protocol_wire::{codec, v1 as w};
    let Ok(value) = codec::family::document_value(bytes, bounds, limits.rows) else { return Err(Invalid::TooLarge) };
    let value = match value {
        w::FamilyDocument::Item(value) => family::Document::Item(scripted::item_value(value)),
        w::FamilyDocument::Part(value) => family::Document::Part(scripted::part_value(value)),
        w::FamilyDocument::Connector(value) => {
            let value = value.into_parts();
            let subject = scripted::subject_value(value.subject);
            let Ok(value) = codec::family::connector_document_value(value.value, bounds) else {
                return Err(Invalid::TooLarge);
            };
            return match document(subject, value, limits) {
                Ok(value) => Ok(family::Document::Connector(value)),
                Err(why) => Err(why),
            };
        }
    };
    if !value.fits(limits.rows) {
        return Err(Invalid::TooLarge);
    }
    Ok(value)
}

/// # Errors
/// Entrance refusals belong to jig's wire and have no application record.
pub fn encode_answer(value: &family::Answer) -> Result<Box<[u8]>, Invalid> {
    use jig_client_test_protocol_wire::{codec, v1 as w};
    let value = match value {
        family::Answer::Touched { item, revision } => w::FamilyAnswer::Touched(
            w::FamilyRevision::new(&w::CEILINGS, w::FamilyRevisionParts { item: *item, revision: revision.value })
                .expect("fixed family revision"),
        ),
        family::Answer::Moved { item, revision } => w::FamilyAnswer::Moved(
            w::FamilyRevision::new(&w::CEILINGS, w::FamilyRevisionParts { item: *item, revision: revision.value })
                .expect("fixed family revision"),
        ),
        family::Answer::Gone => w::FamilyAnswer::Gone,
        family::Answer::Refused(_) => return Err(Invalid::EntranceRefusal),
        family::Answer::Connector(value) => w::FamilyAnswer::Connector(codec::family::connector_answer(&value.value)),
    };
    Ok(codec::family::answer(value))
}
/// # Errors
/// Refuses malformed records or values outside the family's configured bounds.
pub fn decode_answer(bytes: &[u8], bounds: &jig_test_connector::Limits) -> Result<family::Answer, Invalid> {
    use jig_client::contract::Revision;
    use jig_client_test_protocol_wire::{codec, v1 as w};
    let Ok(value) = codec::family::answer_value(bytes, bounds) else { return Err(Invalid::TooLarge) };
    let value = match value {
        w::FamilyAnswer::Touched(value) => {
            let value = value.into_parts();
            family::Answer::Touched { item: value.item, revision: Revision { value: value.revision } }
        }
        w::FamilyAnswer::Moved(value) => {
            let value = value.into_parts();
            family::Answer::Moved { item: value.item, revision: Revision { value: value.revision } }
        }
        w::FamilyAnswer::Gone => family::Answer::Gone,
        w::FamilyAnswer::Connector(value) => {
            family::Answer::Connector(answer(codec::family::connector_answer_value(value, bounds)))
        }
    };
    Ok(value)
}
