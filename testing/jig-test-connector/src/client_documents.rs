//! Connector-owned subjects and parts (`domain/connectors.md`, 5.5;
//! `client/domain/root.md`, section 6). Reads do not mutate their source state.
use crate::{ApplicationObject, Domain, Limits, Request};
use skein_lib::{List, Queue, Token};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ClientScope {
    Project(u32),
    Deployment,
}

/// The root gives this owner's typed subject a fresh opaque name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ClientSubject {
    Object { project: u32, object: u64 },
    Panel { project: u32, task: u64 },
    Result { project: u32, task: u64 },
    Effect { project: u32, proposal: u64 },
    Hold { project: u32, task: u64 },
    Project { project: u32 },
    Deployment,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DocumentMissing {
    Unknown,
    NotAllowed,
}

/// What this owner knows of a named resource, including an absent/stale fact.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClientResource {
    pub path: crate::Path,
    pub role: crate::ResourceRole,
    pub hold: crate::Hold,
    pub fact: Option<crate::Fact>,
}

/// Whole current values; paths and lists keep this connector's bounds.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ClientDocument {
    Object {
        object: u64,
        now: ApplicationObject,
    },
    Panel {
        task: u64,
        procedure: Option<crate::ProcedureState>,
        result: Option<u16>,
        resources: List<ClientResource>,
        effects: List<crate::OutboxEntry>,
    },
    Result {
        task: u64,
        code: u16,
    },
    Effect {
        proposal: u64,
        task: u64,
        effect: crate::Effect,
        description: crate::Description,
    },
    Hold {
        task: u64,
        resources: List<ClientResource>,
        effects: List<crate::OutboxEntry>,
    },
    Project {
        project: u32,
        resources: List<ClientResource>,
    },
    Deployment {
        outbox: List<crate::OutboxEntry>,
        subject_limit: u32,
        staged_limit: u32,
    },
}

pub(crate) fn prepare(domain: &mut Domain, token: Token, subject: ClientSubject) {
    // A full bounded table leaves this name unknown; construction gives one terminal.
    let _prepared = domain.client_subjects.insert(token, subject);
}

fn scoped(scope: ClientScope, project: u32) -> Result<(), DocumentMissing> {
    if scope == ClientScope::Project(project) { Ok(()) } else { Err(DocumentMissing::Unknown) }
}

fn task(domain: &Domain, project: u32, number: u64) -> Result<&[crate::Path], DocumentMissing> {
    match domain.tasks.get(&number) {
        Some(crate::Record::Task { project: owner, resources, .. }) if *owner == project => Ok(resources),
        Some(
            crate::Record::Task { .. }
            | crate::Record::ApplicationObject { .. }
            | crate::Record::ApplicationAnswer { .. }
            | crate::Record::Proposal { .. }
            | crate::Record::Procedure(_)
            | crate::Record::Result { .. }
            | crate::Record::Adoption { .. }
            | crate::Record::Subscription { .. }
            | crate::Record::Pool { .. }
            | crate::Record::Outbox(_)
            | crate::Record::Made { .. },
        )
        | None => Err(DocumentMissing::Unknown),
    }
}

fn resource(domain: &Domain, project: u32, path: &crate::Path) -> Result<ClientResource, DocumentMissing> {
    let role = domain.resource_role(project, path).ok_or(DocumentMissing::Unknown)?;
    let specification = crate::domain::resource_spec(&domain.config, path).ok_or(DocumentMissing::Unknown)?;
    Ok(ClientResource { path: path.clone(), role, hold: specification.hold, fact: domain.facts.get(path).copied() })
}

fn resources(
    domain: &Domain,
    limits: &Limits,
    project: u32,
    paths: &[crate::Path],
) -> Result<List<ClientResource>, DocumentMissing> {
    let mut values = List::with_capacity(limits.resources_per_task);
    for path in paths {
        values.push(resource(domain, project, path)?).expect("admitted task resource count");
    }
    Ok(values)
}

fn effects(domain: &Domain, limits: &Limits, task: Option<u64>) -> List<crate::OutboxEntry> {
    let mut effects = List::with_capacity(limits.entries);
    for (_, entry) in &domain.outbox.entries {
        if task.is_none() || task == Some(entry.task) {
            effects.push(entry.clone()).expect("bounded outbox entries");
        }
    }
    effects
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
        ClientSubject::Object { project, object } => {
            scoped(scope, *project)?;
            let specification = domain.specification_resource(*object).ok_or(DocumentMissing::Unknown)?;
            match domain.resource_role(*project, &specification.path) {
                Some(
                    crate::ResourceRole::Owned | crate::ResourceRole::Participating | crate::ResourceRole::Context,
                ) => {}
                Some(crate::ResourceRole::Unavailable) | None => return Err(DocumentMissing::Unknown),
            }
            if let Some(fact) = domain.facts.get(&specification.path)
                && fact.state.is_none()
                && !fact.pending
            {
                return Err(DocumentMissing::Unknown);
            }
            Ok(ClientDocument::Object { object: *object, now: domain.application.object(*object) })
        }
        ClientSubject::Panel { project, task: number } => {
            scoped(scope, *project)?;
            let paths = task(domain, *project, *number)?;
            Ok(ClientDocument::Panel {
                task: *number,
                procedure: domain.procedures.get(number).cloned(),
                result: domain.results.get(number).copied(),
                resources: resources(domain, limits, *project, paths)?,
                effects: effects(domain, limits, Some(*number)),
            })
        }
        ClientSubject::Result { project, task: number } => {
            scoped(scope, *project)?;
            task(domain, *project, *number)?;
            let code = domain.results.get(number).copied().ok_or(DocumentMissing::Unknown)?;
            Ok(ClientDocument::Result { task: *number, code })
        }
        ClientSubject::Effect { project, proposal } => {
            scoped(scope, *project)?;
            let (number, effect) = domain.outbox.proposals.get(proposal).ok_or(DocumentMissing::Unknown)?;
            task(domain, *project, *number)?;
            let mut kind = None;
            for configured in &domain.config.kinds {
                if configured.kind == effect.kind {
                    kind = Some(configured);
                    break;
                }
            }
            let kind = kind.ok_or(DocumentMissing::Unknown)?;
            let description = crate::Description {
                kind: effect.kind,
                resources: effect.resources.clone(),
                purpose: effect.purpose,
                condition: effect.condition.is_some(),
                form: kind.form,
                recovery: kind.recovery,
                price: kind.price,
                state: effect.state,
            };
            Ok(ClientDocument::Effect { proposal: *proposal, task: *number, effect: effect.clone(), description })
        }
        ClientSubject::Hold { project, task: number } => {
            scoped(scope, *project)?;
            Ok(ClientDocument::Hold {
                task: *number,
                resources: resources(domain, limits, *project, task(domain, *project, *number)?)?,
                effects: effects(domain, limits, Some(*number)),
            })
        }
        ClientSubject::Project { project } => {
            scoped(scope, *project)?;
            let mut resources = List::with_capacity(limits.adoptions);
            for ((owner, path), _) in &domain.adoptions {
                if owner == project {
                    resources.push(resource(domain, *project, path)?).expect("bounded adoptions");
                }
            }
            Ok(ClientDocument::Project { project: *project, resources })
        }
        ClientSubject::Deployment => {
            if scope != ClientScope::Deployment {
                return Err(DocumentMissing::Unknown);
            }
            Ok(ClientDocument::Deployment {
                outbox: effects(domain, limits, None),
                subject_limit: limits.values,
                staged_limit: limits.staged,
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

/// Owned allocation moved to the root's journal, including nested paths.
#[must_use]
pub fn document_worst_case(limits: &Limits) -> Option<u64> {
    let path = u64::from(limits.path_segments).checked_mul(u64::from(limits.segment_bytes).checked_add(16)?)?;
    let paths = u64::from(limits.resources_per_effect)
        .checked_mul(path.checked_add(u64::try_from(size_of::<crate::Path>()).ok()?)?)?;
    let effects = List::<crate::OutboxEntry>::worst_case(limits.entries)?
        .checked_add(u64::from(limits.entries).checked_mul(paths)?)?;
    let resources = List::<ClientResource>::worst_case(limits.resources_per_task)?
        .checked_add(u64::from(limits.resources_per_task).checked_mul(path)?)?;
    let panel = resources.checked_add(effects)?.checked_add(path)?;
    let project = List::<ClientResource>::worst_case(limits.adoptions)?
        .checked_add(u64::from(limits.adoptions).checked_mul(path)?)?;
    u64::try_from(size_of::<ClientDocument>()).ok()?.checked_add(panel.max(project).max(paths.checked_mul(2)?))
}
