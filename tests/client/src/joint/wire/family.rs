//! The application's actual scoped owner values (client/domain/root.md, 6–7).
//! This source uses the actual whole family consumer; it remains unregistered and unexecuted until the coherent wire increment.
use jig_test_connector::{ApplicationOutcome, ApplicationRequest, ClientDocument, ClientScope, ClientSubject};
use jig_test_domain_client::{
    connector_values::{ConnectorAnswer, ConnectorDocument, ConnectorLimits},
    family,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    Scope,
    CoreEffectReference,
    MissingValue,
    Identity,
    TooLarge,
}

/// # Errors
/// Refuses missing supplied mutation values, unsupported individual deployment scope, mismatched subjects or a whole owner value above bounds.
pub fn subject(value: family::Subject) -> Result<(ClientScope, ClientSubject), Invalid> {
    let project = match value.scope {
        family::Scope::Project(project) => project,
        family::Scope::Deployment => {
            return match value.kind {
                family::Kind::Deployment => Ok((ClientScope::Deployment, ClientSubject::Deployment)),
                family::Kind::Item(_)
                | family::Kind::Panel(_)
                | family::Kind::Result(_)
                | family::Kind::Effect(_)
                | family::Kind::OwnerEffect(_)
                | family::Kind::Outbox(_)
                | family::Kind::Hold(_)
                | family::Kind::Project(_) => Err(Invalid::Scope),
            };
        }
    };
    let now = match value.kind {
        family::Kind::Item(object) => ClientSubject::Object { project, object },
        family::Kind::Panel(task) => ClientSubject::Panel { project, task },
        family::Kind::Result(task) => ClientSubject::Result { project, task },
        family::Kind::Effect(_) => return Err(Invalid::CoreEffectReference),
        family::Kind::OwnerEffect(proposal) => ClientSubject::Effect { project, proposal },
        family::Kind::Outbox(entry) => ClientSubject::Outbox { project, entry },
        family::Kind::Hold(task) => ClientSubject::Hold { project, task },
        family::Kind::Project(named) => {
            if named != project {
                return Err(Invalid::Scope);
            }
            ClientSubject::Project { project }
        }
        family::Kind::Deployment => return Err(Invalid::Scope),
    };
    Ok((ClientScope::Project(project), now))
}

/// # Errors
/// Refuses missing supplied mutation values, unsupported individual deployment scope, mismatched subjects or a whole owner value above bounds.
pub fn request(value: &family::Ask) -> Result<(u32, ApplicationRequest), Invalid> {
    match value {
        family::Ask::Touch { project, item, revision, parameters } => {
            let value = parameters.ok_or(Invalid::MissingValue)?;
            Ok((*project, ApplicationRequest { object: *item, revision: revision.value, value, release: None }))
        }
        family::Ask::ChangeObject { project, object, revision, value } => Ok((
            *project,
            ApplicationRequest { object: *object, revision: revision.value, value: *value, release: None },
        )),
    }
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
    let (_, named) = subject(original)?;
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
    if !jig_test_client_kit::connector_values::document_fits(&result, limits) {
        return Err(Invalid::TooLarge);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_domain_client::contract::{ProposalKey, Proposer, Revision};
    use skein_lib::List;

    #[test]
    fn application_subject_translation_preserves_supplied_scope_and_refuses_an_inferred_project_or_maker() {
        let cases = [
            (family::Kind::Item(u64::MAX), ClientSubject::Object { project: u32::MAX, object: u64::MAX }),
            (family::Kind::Panel(u64::MAX), ClientSubject::Panel { project: u32::MAX, task: u64::MAX }),
            (family::Kind::Result(u64::MAX), ClientSubject::Result { project: u32::MAX, task: u64::MAX }),
            (family::Kind::OwnerEffect(u64::MAX), ClientSubject::Effect { project: u32::MAX, proposal: u64::MAX }),
            (family::Kind::Outbox(u64::MAX), ClientSubject::Outbox { project: u32::MAX, entry: u64::MAX }),
            (family::Kind::Hold(u64::MAX), ClientSubject::Hold { project: u32::MAX, task: u64::MAX }),
            (family::Kind::Project(u32::MAX), ClientSubject::Project { project: u32::MAX }),
        ];
        for (kind, expected) in cases {
            assert_eq!(
                subject(family::Subject { scope: family::Scope::Project(u32::MAX), kind }),
                Ok((ClientScope::Project(u32::MAX), expected))
            );
            assert_eq!(subject(family::Subject { scope: family::Scope::Deployment, kind }), Err(Invalid::Scope));
        }
        assert_eq!(
            subject(family::Subject { scope: family::Scope::Deployment, kind: family::Kind::Deployment }),
            Ok((ClientScope::Deployment, ClientSubject::Deployment))
        );
        assert_eq!(
            subject(family::Subject { scope: family::Scope::Project(4), kind: family::Kind::Project(5) }),
            Err(Invalid::Scope)
        );
        for proposer in [Proposer::Task(u64::MAX), Proposer::Party(u64::MAX)] {
            assert_eq!(
                subject(family::Subject {
                    scope: family::Scope::Project(4),
                    kind: family::Kind::Effect(ProposalKey { proposer, number: 7 })
                }),
                Err(Invalid::CoreEffectReference)
            );
        }
    }

    #[test]
    fn application_requests_keep_an_explicit_zero_and_owner_documents_reject_mismatched_or_partial_values() {
        assert_eq!(
            request(&family::Ask::Touch {
                project: u32::MAX,
                item: u64::MAX,
                revision: Revision { value: u64::MAX },
                parameters: Some(0)
            }),
            Ok((u32::MAX, ApplicationRequest { object: u64::MAX, revision: u64::MAX, value: 0, release: None }))
        );
        assert_eq!(
            request(&family::Ask::Touch { project: 4, item: 7, revision: Revision { value: 1 }, parameters: None }),
            Err(Invalid::MissingValue)
        );
        assert_eq!(
            request(&family::Ask::ChangeObject {
                project: u32::MAX,
                object: u64::MAX,
                revision: Revision { value: u64::MAX },
                value: 0
            }),
            Ok((u32::MAX, ApplicationRequest { object: u64::MAX, revision: u64::MAX, value: 0, release: None }))
        );
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
