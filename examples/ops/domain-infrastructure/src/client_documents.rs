//! Infrastructure-owned subjects and parts (`domain/connectors.md`, 5.5;
//! `client/domain/root.md`, section 6; `examples.md`, section 8).
use crate::{Domain, Environment, EnvironmentFact, Limits, Request, Resource};
use skein_lib::{List, Queue, Token};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ClientScope {
    Project(u32),
    Deployment,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ClientSubject {
    Environment { project: u32, environment: Environment },
    Panel { project: u32, task: u64 },
    Result { project: u32, task: u64 },
    Effect { project: u32, proposal: u64 },
    Outbox { project: u32, entry: u64 },
    Hold { project: u32, task: u64 },
    Project { project: u32 },
    Deployment,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DocumentMissing {
    Unknown,
    NotAllowed,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ClientPool {
    pub quota: u32,
    pub used: u32,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ResourceFact {
    Service(Option<crate::ServiceFact>),
    Environment { fact: Option<EnvironmentFact>, end: Option<crate::EnvironmentEnd> },
    Pool(Option<ClientPool>),
    Quota(Option<ClientPool>),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClientResource {
    pub named: crate::Named,
    pub fact: ResourceFact,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClientEnvironment {
    pub environment: Environment,
    pub fact: EnvironmentFact,
    pub end: Option<crate::EnvironmentEnd>,
    pub pool: Option<ClientPool>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ClientDocument {
    Environment {
        environment: Environment,
        fact: EnvironmentFact,
        end: Option<crate::EnvironmentEnd>,
        pool: Option<ClientPool>,
    },
    Panel {
        task: u64,
        state: crate::ProcedureState,
        resources: List<ClientResource>,
        effects: List<crate::Entry>,
    },
    Result {
        task: u64,
        state: crate::ProcedureState,
        resources: List<ClientResource>,
    },
    Effect {
        proposal: u64,
        task: u64,
        effect: crate::Effect,
        description: crate::Description,
    },
    Outbox {
        entry: crate::Entry,
        description: crate::Description,
    },
    Hold {
        task: u64,
        resources: List<ClientResource>,
        drift: List<crate::DriftAccount>,
        effects: List<crate::Entry>,
    },
    Project {
        project: u32,
        resources: List<ClientResource>,
        environments: List<ClientEnvironment>,
    },
    Deployment {
        backend: crate::Backend,
        effects: List<crate::Entry>,
        made: u32,
        request_limit: u32,
        staged_limit: u32,
    },
}

pub(crate) fn prepare(domain: &mut Domain, limits: &Limits, token: Token, subject: ClientSubject) {
    let valid = match &subject {
        ClientSubject::Environment { environment, .. } => {
            crate::domain::resource_valid(&Resource::Environment(environment.clone()), limits)
        }
        ClientSubject::Panel { .. }
        | ClientSubject::Result { .. }
        | ClientSubject::Effect { .. }
        | ClientSubject::Outbox { .. }
        | ClientSubject::Hold { .. }
        | ClientSubject::Project { .. }
        | ClientSubject::Deployment => true,
    };
    if valid {
        let _prepared = domain.client_subjects.insert(token, subject);
    }
}

fn scoped(scope: ClientScope, project: u32) -> Result<(), DocumentMissing> {
    if scope == ClientScope::Project(project) { Ok(()) } else { Err(DocumentMissing::Unknown) }
}

#[expect(clippy::manual_map, reason = "step code uses no closure-taking methods")]
pub(crate) fn environment_owner(domain: &Domain, environment: &Environment, fact: &EnvironmentFact) -> Option<u32> {
    if let Some((key, project)) = domain.made.get(environment) {
        if *key != fact.created_by {
            return None;
        }
        if project.is_some() {
            return *project;
        }
    }
    match domain.reliance.get(&fact.created_by.task) {
        Some((project, _)) => Some(*project),
        None => None,
    }
}

fn task(domain: &Domain, project: u32, task: u64) -> Result<&[crate::Named], DocumentMissing> {
    match domain.reliance.get(&task) {
        Some((owner, resources)) => {
            if *owner == project {
                Ok(resources)
            } else {
                Err(DocumentMissing::Unknown)
            }
        }
        None => Err(DocumentMissing::Unknown),
    }
}

#[expect(clippy::manual_map, reason = "step code uses no closure-taking methods")]
fn pool(domain: &Domain, name: &crate::Pool) -> Option<ClientPool> {
    match domain.pools.get(name) {
        Some((quota, used)) => Some(ClientPool { quota: *quota, used: *used }),
        None => None,
    }
}

#[expect(clippy::manual_map, reason = "step code uses no closure-taking methods")]
fn resource(domain: &Domain, named: &crate::Named) -> ClientResource {
    let fact = match &named.resource {
        Resource::Service(service) => ResourceFact::Service(match domain.services.get(service) {
            Some(fact) => Some(fact.clone()),
            None => None,
        }),
        Resource::Environment(environment) => ResourceFact::Environment {
            fact: match domain.environments.get(environment) {
                Some(fact) => *fact,
                None => None,
            },
            end: domain.application.end(environment),
        },
        Resource::Pool(name) => ResourceFact::Pool(pool(domain, name)),
        Resource::Quota(name) => ResourceFact::Quota(pool(domain, name)),
    };
    ClientResource { named: named.clone(), fact }
}

fn resources(domain: &Domain, limits: &Limits, named: &[crate::Named]) -> List<ClientResource> {
    let mut resources = List::with_capacity(limits.resources_per_task);
    for named in named {
        resources.push(resource(domain, named)).expect("bounded task names");
    }
    resources
}

fn effects(domain: &Domain, limits: &Limits, task: Option<u64>) -> List<crate::Entry> {
    let mut values = List::with_capacity(limits.effects);
    for (_, entry) in &domain.effects {
        if task.is_none() || task == Some(entry.key.task) {
            values.push(entry.clone()).expect("bounded outbox");
        }
    }
    values
}

#[expect(clippy::too_many_lines, reason = "each part constructs its owner's bounded current vocabulary")]
fn make(
    domain: &Domain,
    limits: &Limits,
    scope: ClientScope,
    token: Token,
    _party: u64,
) -> Result<ClientDocument, DocumentMissing> {
    let subject = domain.client_subjects.get(&token).ok_or(DocumentMissing::Unknown)?;
    match subject {
        ClientSubject::Environment { project, environment } => {
            scoped(scope, *project)?;
            let fact = match domain.environments.get(environment) {
                Some(Some(fact)) => *fact,
                Some(None) | None => return Err(DocumentMissing::Unknown),
            };
            if environment_owner(domain, environment, &fact) != Some(*project) {
                return Err(DocumentMissing::Unknown);
            }
            Ok(ClientDocument::Environment {
                environment: environment.clone(),
                fact,
                end: domain.application.end(environment),
                pool: pool(domain, &crate::Pool(environment.pool.clone())),
            })
        }
        ClientSubject::Panel { project, task: number } => {
            scoped(scope, *project)?;
            let named = task(domain, *project, *number)?;
            let state = domain.procedures.get(number).ok_or(DocumentMissing::Unknown)?.clone();
            Ok(ClientDocument::Panel {
                task: *number,
                state,
                resources: resources(domain, limits, named),
                effects: effects(domain, limits, Some(*number)),
            })
        }
        ClientSubject::Result { project, task: number } => {
            scoped(scope, *project)?;
            let named = task(domain, *project, *number)?;
            let state = domain.procedures.get(number).ok_or(DocumentMissing::Unknown)?.clone();
            match state.phase {
                crate::ProcedurePhase::Holding | crate::ProcedurePhase::Done => {}
                crate::ProcedurePhase::Active
                | crate::ProcedurePhase::ReadingFacts
                | crate::ProcedurePhase::FactsReady
                | crate::ProcedurePhase::EffectOutstanding
                | crate::ProcedurePhase::WaitingCondition
                | crate::ProcedurePhase::Held => return Err(DocumentMissing::Unknown),
            }
            Ok(ClientDocument::Result { task: *number, state, resources: resources(domain, limits, named) })
        }
        ClientSubject::Effect { project, proposal } => {
            scoped(scope, *project)?;
            let (number, effect) = domain.proposals.get(proposal).ok_or(DocumentMissing::Unknown)?;
            task(domain, *project, *number)?;
            Ok(ClientDocument::Effect {
                proposal: *proposal,
                task: *number,
                effect: effect.clone(),
                description: crate::domain::describe(effect, domain.backend),
            })
        }
        ClientSubject::Outbox { project, entry: number } => {
            scoped(scope, *project)?;
            for (_, entry) in &domain.effects {
                if entry.number == *number {
                    let owner = match entry.project {
                        Some(owner) => owner,
                        None => {
                            task(domain, *project, entry.key.task)?;
                            *project
                        }
                    };
                    if owner != *project {
                        return Err(DocumentMissing::Unknown);
                    }
                    return Ok(ClientDocument::Outbox {
                        entry: entry.clone(),
                        description: crate::domain::describe(&entry.effect, domain.backend),
                    });
                }
            }
            Err(DocumentMissing::Unknown)
        }
        ClientSubject::Hold { project, task: number } => {
            scoped(scope, *project)?;
            let named = task(domain, *project, *number)?;
            let mut drift = List::with_capacity(limits.resources_per_task);
            for named in named {
                if let Some(account) = domain.drifts.get(&named.resource) {
                    drift.push(account.clone()).expect("bounded named drift");
                }
            }
            Ok(ClientDocument::Hold {
                task: *number,
                resources: resources(domain, limits, named),
                drift,
                effects: effects(domain, limits, Some(*number)),
            })
        }
        ClientSubject::Project { project } => {
            scoped(scope, *project)?;
            let mut resources: List<ClientResource> =
                List::with_capacity(limits.tasks.checked_mul(limits.resources_per_task).expect("project name bound"));
            for (_, (owner, names)) in &domain.reliance {
                if *owner == *project {
                    for named in names {
                        let mut present = false;
                        for current in &resources {
                            if current.named.resource == named.resource {
                                present = true;
                            }
                        }
                        if !present {
                            resources.push(resource(domain, named)).expect("bounded project names");
                        }
                    }
                }
            }
            let mut environments = List::with_capacity(limits.environments);
            for (environment, fact) in &domain.environments {
                if let Some(fact) = fact
                    && environment_owner(domain, environment, fact) == Some(*project)
                {
                    environments
                        .push(ClientEnvironment {
                            environment: environment.clone(),
                            fact: *fact,
                            end: domain.application.end(environment),
                            pool: pool(domain, &crate::Pool(environment.pool.clone())),
                        })
                        .expect("bounded environments");
                }
            }
            Ok(ClientDocument::Project { project: *project, resources, environments })
        }
        ClientSubject::Deployment => {
            if scope != ClientScope::Deployment {
                return Err(DocumentMissing::Unknown);
            }
            Ok(ClientDocument::Deployment {
                backend: domain.backend,
                effects: effects(domain, limits, None),
                made: domain.made.len(),
                request_limit: limits.proposals,
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

/// Includes nested names, versions, description resources, retained facts and outboxes.
#[must_use]
pub fn document_worst_case(limits: &Limits) -> Option<u64> {
    let name = u64::from(limits.name_bytes);
    let task = List::<ClientResource>::worst_case(limits.resources_per_task)?
        .checked_add(u64::from(limits.resources_per_task).checked_mul(name.checked_mul(3)?)?)?;
    let effects = List::<crate::Entry>::worst_case(limits.effects)?
        .checked_add(u64::from(limits.effects).checked_mul(name.checked_mul(4)?)?)?;
    let drift = List::<crate::DriftAccount>::worst_case(limits.resources_per_task)?
        .checked_add(u64::from(limits.resources_per_task).checked_mul(name.checked_mul(3)?)?)?;
    let panel = task.checked_add(effects)?.checked_add(name.checked_mul(2)?)?;
    let hold = task.checked_add(effects)?.checked_add(drift)?;
    let project_count = limits.tasks.checked_mul(limits.resources_per_task)?;
    let project = List::<ClientResource>::worst_case(project_count)?
        .checked_add(u64::from(project_count).checked_mul(name.checked_mul(3)?)?)?
        .checked_add(List::<ClientEnvironment>::worst_case(limits.environments)?)?
        .checked_add(u64::from(limits.environments).checked_mul(name.checked_mul(3)?)?)?;
    let effect = name.checked_mul(7)?.checked_add(u64::try_from(size_of::<Resource>()).ok()?.checked_mul(2)?)?;
    u64::try_from(size_of::<ClientDocument>()).ok()?.checked_add(panel.max(hold).max(project).max(effect))
}
