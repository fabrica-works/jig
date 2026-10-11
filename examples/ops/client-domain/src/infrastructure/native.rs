//! Whole Infrastructure facts supplied by the owner (examples.md, 8; root.md, 6).
//! Raw environment and pool names stay raw; no display identity or revision is invented.
use super::documents::Subject;
use alloc::boxed::Box;
use skein_lib::{List, Wall};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Service {
    /// Environment name.
    pub environment: Box<[u8]>,
    /// Service within it.
    pub name: Box<[u8]>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Environment {
    /// Pool whose slot it occupies.
    pub pool: Box<[u8]>,
    /// Environment name within the pool.
    pub name: Box<[u8]>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Pool(pub Box<[u8]>);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Resource {
    /// An exclusive service being remediated.
    Service(Service),
    /// One environment made under a pool.
    Environment(Environment),
    /// A pool's counted slots.
    Pool(Pool),
    /// A pool's quota fact.
    Quota(Pool),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Hold {
    /// One task at a time; a taken service waits.
    ExclusiveWait,
    /// One of the pool's counted slots; a full pool waits.
    PooledWait,
    /// Reads need no hold.
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceRole {
    /// The deployment owns it.
    Owned,
    /// It writes as a participant.
    Participant,
    /// It may read it.
    Context,
    /// It is known but unavailable.
    Unavailable,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Named {
    /// Resource named by the task.
    pub resource: Resource,
    /// Project-scoped role reported by this connector.
    pub role: ResourceRole,
    /// Hold the core takes.
    pub hold: Hold,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Purpose {
    /// One activation-qualified agent call.
    Call { attempt: u64, completion: u32, position: u32 },
    /// One stable procedure purpose.
    Procedure { purpose: u64 },
    /// One stable goal projection purpose.
    Projection { purpose: u64 },
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Key {
    /// Deployment that owns the operation.
    pub deployment: [u8; 16],
    /// Asking task.
    pub task: u64,
    /// The call, procedure or projection that asked.
    pub origin: Purpose,
    /// Stable purpose within that owner.
    pub purpose: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Backend {
    /// IDs make uncertain restarts recoverable by key.
    OperationIds,
    /// Without IDs, an uncertain restart is held for a person.
    NoOperationIds,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Effect {
    /// Restart once by operation ID where supported.
    Restart { service: Service, operation: u64 },
    /// Change replicas only from the count decided from.
    Scale { service: Service, from: u32, to: u32 },
    /// Change version only from the version decided from.
    Rollback { service: Service, from: Box<[u8]>, to: Box<[u8]> },
    /// Create once by key, priced at its maximum.
    CreateEnvironment { environment: Environment, until: u64, price: u64 },
    /// Delete only the environment created under this key.
    TearDown { environment: Environment, created_by: Key },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Form {
    /// Make a named object.
    Creation,
    /// Change from one state to another.
    Transition,
    /// Set one final value.
    Set,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Recovery {
    /// Find the operation's stable key.
    Keyed,
    /// Retry only under the original condition.
    Conditional,
    /// No safe automatic retry.
    Unrecoverable,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Description {
    /// Connector-local effect kind number.
    pub kind: u16,
    /// Resources checked against authority and holds.
    pub resources: Box<[Resource]>,
    /// Whether the system checks the starting state when applying.
    pub condition: bool,
    /// Creation, transition or set.
    pub form: Form,
    /// Recovery offered by this backend.
    pub recovery: Recovery,
    /// Maximum cost, if this is priced.
    pub price: Option<u64>,
    /// State the requirement judges are asked about.
    pub state: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Phase {
    /// Saved, not yet sent.
    Kept,
    /// An attempt was committed for release.
    Sent,
    /// The answer was lost; lookup or deadline remains.
    Uncertain,
    /// A person must settle the effect.
    Held,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    /// Admitted owner scope, retained when known.
    pub project: Option<u32>,
    /// Core-owned outbox number, separate from the stable effect key.
    pub number: u64,
    /// Deployment-scoped key.
    pub key: Key,
    /// Connector-owned effect.
    pub effect: Effect,
    /// Attempt number.
    pub attempt: u32,
    /// Absolute system time when an uncertain attempt may be retried.
    pub deadline: u64,
    /// Current phase.
    pub phase: Phase,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ServiceFact {
    /// Current version.
    pub version: Box<[u8]>,
    /// Current replicas.
    pub replicas: u32,
    /// Whether the service is healthy, as observed by infrastructure.
    pub healthy: bool,
    /// Revision changed by either hand.
    pub revision: u64,
    /// System time of the read.
    pub observed: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EnvironmentFact {
    /// Creation key, if it exists.
    pub created_by: Key,
    /// System time when provisioning completes.
    pub ready_at: u64,
    /// Lease end.
    pub until: u64,
    /// Maximum price charged.
    pub price: u64,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Procedure {
    /// Remediate a service and wait for health.
    Remediate { service: Service, operation: u64, deadline: u64 },
    /// Set replicas after reading their current count afresh.
    Scale { service: Service, replicas: u32, deadline: u64 },
    /// Create an environment and wait until it is ready.
    Provision { environment: Environment, until: u64, price: u64, deadline: u64 },
    /// Tear down an environment on release.
    TearDown { environment: Environment, created_by: Key, deadline: u64 },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProcedurePhase {
    /// Evaluate current facts and decide once.
    Active,
    /// A scale procedure has requested fresh replica facts.
    ReadingFacts,
    /// Fresh replica facts are ready for the scale decision.
    FactsReady,
    /// An effect was asked for and is not settled.
    EffectOutstanding,
    /// Wait for a fact to reach the requested condition.
    WaitingCondition,
    /// Provisioned environment remains owned until task release.
    Holding,
    /// The task ended.
    Done,
    /// A bound or drift needs a person.
    Held,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProcedureState {
    /// Task executing it.
    pub task: u64,
    /// Its parameters.
    pub procedure: Procedure,
    /// Its progress.
    pub phase: ProcedurePhase,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DriftKind {
    Changed,
    Deleted,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DriftAccount {
    pub resource: Resource,
    pub kind: DriftKind,
    /// Owner observation time unless an attributed system observation supplies it.
    pub at: Wall,
    pub by: Option<Box<[u8]>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EnvironmentEnd {
    pub revision: u64,
    pub until: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ClientPool {
    pub quota: u32,
    pub used: u32,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ResourceFact {
    Service(Option<ServiceFact>),
    Environment { fact: Option<EnvironmentFact>, end: Option<EnvironmentEnd> },
    Pool(Option<ClientPool>),
    Quota(Option<ClientPool>),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClientResource {
    pub named: Named,
    pub fact: ResourceFact,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClientEnvironment {
    pub environment: Environment,
    pub fact: EnvironmentFact,
    pub end: Option<EnvironmentEnd>,
    pub pool: Option<ClientPool>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Value {
    Environment {
        environment: Environment,
        end_revision: u64,
        fact: EnvironmentFact,
        end: Option<EnvironmentEnd>,
        pool: Option<ClientPool>,
    },
    Panel {
        task: u64,
        state: ProcedureState,
        resources: List<ClientResource>,
        effects: List<Entry>,
    },
    Result {
        task: u64,
        state: ProcedureState,
        resources: List<ClientResource>,
    },
    Effect {
        proposal: u64,
        task: u64,
        effect: Effect,
        description: Description,
    },
    Outbox {
        entry: Entry,
        description: Description,
    },
    Hold {
        task: u64,
        resources: List<ClientResource>,
        drift: List<DriftAccount>,
        effects: List<Entry>,
    },
    Project {
        project: u32,
        resources: List<ClientResource>,
        environments: List<ClientEnvironment>,
    },
    Deployment {
        backend: Backend,
        effects: List<Entry>,
        made: u32,
        proposal_limit: u32,
        staged_limit: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub subject: Subject,
    pub value: Value,
}
fn text(value: &[u8], limits: &super::Limits) -> Option<u64> {
    if value.len() > usize::try_from(limits.text).ok()? {
        return None;
    }
    u64::try_from(value.len()).ok()
}
fn resource(value: &Resource, limits: &super::Limits) -> Option<u64> {
    match value {
        Resource::Service(service) => text(&service.environment, limits)?.checked_add(text(&service.name, limits)?),
        Resource::Environment(environment) => {
            text(&environment.pool, limits)?.checked_add(text(&environment.name, limits)?)
        }
        Resource::Pool(pool) | Resource::Quota(pool) => text(&pool.0, limits),
    }
}
fn effect(value: &Effect, limits: &super::Limits) -> Option<u64> {
    match value {
        Effect::Restart { service, .. } | Effect::Scale { service, .. } => service_price(service, limits),
        Effect::Rollback { service, from, to } => {
            service_price(service, limits)?.checked_add(text(from, limits)?)?.checked_add(text(to, limits)?)
        }
        Effect::CreateEnvironment { environment, .. } | Effect::TearDown { environment, .. } => {
            environment_price(environment, limits)
        }
    }
}
fn description(value: &Description, limits: &super::Limits) -> Option<u64> {
    if value.resources.len() > usize::try_from(limits.rows).ok()? {
        return None;
    }
    let mut bytes =
        u64::try_from(value.resources.len()).ok()?.checked_mul(u64::try_from(size_of::<Resource>()).ok()?)?;
    for value in &value.resources {
        bytes = bytes.checked_add(resource(value, limits)?)?;
    }
    Some(bytes)
}
fn state(value: &ProcedureState, limits: &super::Limits) -> Option<u64> {
    match &value.procedure {
        Procedure::Remediate { service, .. } | Procedure::Scale { service, .. } => service_price(service, limits),
        Procedure::Provision { environment, .. } | Procedure::TearDown { environment, .. } => {
            environment_price(environment, limits)
        }
    }
}
fn resources(values: &List<ClientResource>, limits: &super::Limits) -> Option<u64> {
    if values.capacity() > limits.rows {
        return None;
    }
    let mut bytes = List::<ClientResource>::worst_case(values.capacity())?;
    for value in values {
        bytes = bytes.checked_add(resource(&value.named.resource, limits)?)?;
        match &value.fact {
            ResourceFact::Service(value) => {
                if let Some(value) = value {
                    bytes = bytes.checked_add(text(&value.version, limits)?)?;
                }
            }
            ResourceFact::Environment { .. } | ResourceFact::Pool(_) | ResourceFact::Quota(_) => {}
        }
    }
    Some(bytes)
}
fn effects(values: &List<Entry>, limits: &super::Limits) -> Option<u64> {
    if values.capacity() > limits.rows {
        return None;
    }
    let mut bytes = List::<Entry>::worst_case(values.capacity())?;
    for value in values {
        bytes = bytes.checked_add(effect(&value.effect, limits)?)?;
    }
    Some(bytes)
}
#[must_use]
pub fn price(document: &Document, limits: &super::Limits) -> Option<u64> {
    if !identity(document) {
        return None;
    }
    let bytes = match &document.value {
        Value::Environment { environment, .. } => environment_price(environment, limits)?,
        Value::Panel { state: value, resources: values, effects: entries, .. } => {
            state(value, limits)?.checked_add(resources(values, limits)?)?.checked_add(effects(entries, limits)?)?
        }
        Value::Result { state: value, resources: values, .. } => {
            state(value, limits)?.checked_add(resources(values, limits)?)?
        }
        Value::Effect { effect: value, description: words, .. } => {
            effect(value, limits)?.checked_add(description(words, limits)?)?
        }
        Value::Outbox { entry, description: words } => {
            effect(&entry.effect, limits)?.checked_add(description(words, limits)?)?
        }
        Value::Hold { resources: values, drift, effects: entries, .. } => {
            if drift.capacity() > limits.rows {
                return None;
            }
            let mut bytes = resources(values, limits)?
                .checked_add(effects(entries, limits)?)?
                .checked_add(List::<DriftAccount>::worst_case(drift.capacity())?)?;
            for account in drift {
                bytes = bytes.checked_add(resource(&account.resource, limits)?)?;
                if let Some(by) = &account.by {
                    bytes = bytes.checked_add(text(by, limits)?)?;
                }
            }
            bytes
        }
        Value::Project { resources: values, environments, .. } => {
            if environments.capacity() > limits.rows {
                return None;
            }
            let mut bytes = resources(values, limits)?
                .checked_add(List::<ClientEnvironment>::worst_case(environments.capacity())?)?;
            for value in environments {
                bytes = bytes.checked_add(environment_price(&value.environment, limits)?)?;
            }
            bytes
        }
        Value::Deployment { effects: entries, .. } => effects(entries, limits)?,
    };
    bytes.checked_add(u64::try_from(size_of::<Document>()).ok()?)
}

fn identity(document: &Document) -> bool {
    use super::documents::{Kind, Scope};
    match &document.value {
        Value::Environment { environment, .. } => {
            document.subject.kind == Kind::NativeEnvironment(environment.clone())
                && project_scope(document.subject.scope)
        }
        Value::Panel { task, state, .. } => {
            document.subject.kind == Kind::Panel(*task) && state.task == *task && project_scope(document.subject.scope)
        }
        Value::Result { task, state, .. } => {
            document.subject.kind == Kind::Result(*task) && state.task == *task && project_scope(document.subject.scope)
        }
        Value::Effect { proposal, .. } => {
            document.subject.kind == Kind::OwnerEffect(*proposal) && project_scope(document.subject.scope)
        }
        Value::Outbox { entry, .. } => {
            document.subject.kind == Kind::Outbox(entry.number)
                && match document.subject.scope {
                    Scope::Project(project) => entry.project.is_none() || entry.project == Some(project),
                    Scope::Deployment => false,
                }
        }
        Value::Hold { task, .. } => document.subject.kind == Kind::Hold(*task) && project_scope(document.subject.scope),
        Value::Project { project, .. } => {
            document.subject.scope == Scope::Project(*project)
                && match &document.subject.kind {
                    Kind::Project(number) => *number == *project,
                    Kind::NativeEnvironments { .. } => true,
                    Kind::Environments { .. }
                    | Kind::Environment(_)
                    | Kind::NativeEnvironment(_)
                    | Kind::Panel(_)
                    | Kind::Result(_)
                    | Kind::Effect(_)
                    | Kind::OwnerEffect(_)
                    | Kind::Outbox(_)
                    | Kind::Hold(_)
                    | Kind::Deployment => false,
                }
        }
        Value::Deployment { .. } => {
            document.subject.scope == Scope::Deployment && document.subject.kind == Kind::Deployment
        }
    }
}

fn project_scope(scope: super::documents::Scope) -> bool {
    match scope {
        super::documents::Scope::Project(_) => true,
        super::documents::Scope::Deployment => false,
    }
}

fn service_price(value: &Service, limits: &super::Limits) -> Option<u64> {
    text(&value.environment, limits)?.checked_add(text(&value.name, limits)?)
}
fn environment_price(value: &Environment, limits: &super::Limits) -> Option<u64> {
    text(&value.pool, limits)?.checked_add(text(&value.name, limits)?)
}
