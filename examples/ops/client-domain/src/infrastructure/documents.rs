//! Keep: scoped whole documents, bounded facts and revision-naming requests.
//! Replace: infrastructure resources, procedures and the application's own forms.
//! The owner supplies prices, end facts and drift (examples.md, 3 and 8).
use alloc::boxed::Box;
use jig_client::contract::{ObjectKey, Part, ProposalKey, Revision};
use skein_lib::{List, Wall};
/// Scope supplied with every read and watch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    Project(u32),
    Deployment,
}
/// Whole-document identity; proposed effects and retained entries are distinct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Environments { pool: Option<u32> },
    Environment(u64),
    Panel(u64),
    Result(u64),
    Effect(ProposalKey),
    Outbox(u64),
    Hold(u64),
    Project(u32),
    Deployment,
}
/// A subject and its actual scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subject {
    pub scope: Scope,
    pub kind: Kind,
}
/// A pool's named, owner-supplied capacity facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pool {
    pub number: u32,
    pub name: Box<[u8]>,
    pub quota: Option<u32>,
    pub used: Option<u32>,
}
/// An environment's supplied identity, revision, price and end.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Environment {
    pub project: u32,
    pub number: u64,
    pub pool: u32,
    pub name: Box<[u8]>,
    pub revision: Revision,
    pub created_by: Creation,
    pub ready_at: Option<Wall>,
    pub ready: Option<bool>,
    pub cost: Option<u64>,
    /// Actual backend lease; the desired end may still be reconciling.
    pub observed_until: Option<Wall>,
    /// The owner end record is absent until it actually exists.
    pub end: Option<End>,
}
/// A committed desired end; it is distinct from the backend's observed lease.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct End {
    pub revision: Revision,
    pub until: Wall,
}
/// A fully qualified infrastructure service.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Service {
    pub environment: Box<[u8]>,
    pub name: Box<[u8]>,
}
/// Whether restart effects have operation IDs, as supplied by the backend owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    OperationIds,
    NoOperationIds,
}
/// A creation's durable origin, independent of the environment's display number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Creation {
    pub deployment: [u8; 16],
    pub task: u64,
    pub origin: Purpose,
    pub purpose: u64,
}
/// The owner's stable call, procedure or projection identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    /// A call inside a particular attempt and completion.
    Call { attempt: u64, completion: u64, position: u32 },
    /// A procedure's stable effect purpose.
    Procedure { purpose: u64 },
    /// A goal projection's stable effect purpose.
    Projection { purpose: u64 },
}
/// The owner's phase of a procedure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Active,
    ReadingFacts,
    FactsReady,
    EffectOutstanding,
    WaitingCondition,
    Holding,
    Done,
    Held,
}
/// A procedure and the facts it asks the person to see.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Procedure {
    Remediate { service: Service, operation: u64, deadline: Wall },
    Scale { service: Service, replicas: u32, deadline: Wall },
    Provision { environment: u64, until: Wall, price: u64, deadline: Wall },
    TearDown { environment: u64, created_by: Creation, deadline: Wall },
}
/// A task's current procedure fact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Panel {
    pub task: u64,
    pub procedure: Procedure,
    pub phase: Phase,
}
/// One named resource; no unknown system fact is replaced by zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resource {
    pub name: Box<[u8]>,
    pub role: ResourceRole,
    pub fact: ResourceFact,
}
/// Why the task names the resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceRole {
    Owned,
    Participating,
    Context,
}
/// Current resource facts supplied by infrastructure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceFact {
    Service {
        version: Option<Box<[u8]>>,
        replicas: Option<u32>,
        healthy: Option<bool>,
        revision: Option<u64>,
        observed: Option<Wall>,
    },
    Environment(Option<Environment>),
    Pool(Option<Pool>),
    Quota(Option<Pool>),
}
/// Recovery promise for a proposed or outstanding effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recovery {
    Keyed,
    Conditional,
    Unrecoverable,
}
/// The supplied effect description, including its observed facts and maximum price.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Description {
    pub words: Box<[u8]>,
    pub resource: Box<[u8]>,
    pub kind: u16,
    pub recovery: Recovery,
    pub price: Option<u64>,
    pub condition: bool,
    pub form: Form,
    pub state: u64,
}
/// The owner's effect form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Creation,
    Transition,
    Set,
}
/// The proposed or retained effect, with the original parameters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectAction {
    Restart { service: Service, operation: u64 },
    Scale { service: Service, from: u32, to: u32 },
    Rollback { service: Service, from: Box<[u8]>, to: Box<[u8]> },
    CreateEnvironment { environment: u64, until: Wall, price: u64 },
    TearDown { environment: u64, created_by: Creation },
}
/// Current outbox state, copied from its owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectState {
    Kept,
    Sent,
    Uncertain,
    Held,
}
/// A retained entry and its owned description.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effect {
    pub entry: u64,
    pub task: u64,
    pub project: Option<u32>,
    pub origin: Creation,
    pub description: Option<Description>,
    pub state: EffectState,
    pub attempt: u32,
    pub deadline: Wall,
    pub action: EffectAction,
}
/// Drift observed by infrastructure, with only the attribution it knows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drift {
    pub resource: Box<[u8]>,
    pub kind: DriftKind,
    pub actor: Option<Box<[u8]>>,
    pub at: Option<Wall>,
    pub observed: Wall,
}
/// The owner's account of a hand change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriftKind {
    Deleted,
    Changed,
    Replaced,
    Ambiguous,
}
/// A connector document in a jig slot; fields absent from its source stay absent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartDocument {
    pub subject: Subject,
    pub revision: Revision,
    pub panel: Option<Panel>,
    pub description: Option<Description>,
    pub effect: Option<EffectAction>,
    pub resources: List<Resource>,
    pub effects: List<Effect>,
    pub drift: List<Drift>,
    pub environments: List<Environment>,
    pub pools: List<Pool>,
    pub backend: Option<Backend>,
    pub made: Option<u32>,
    pub proposal_limit: Option<u32>,
    pub staged_limit: Option<u32>,
}
/// One whole owner document, admitted without partial truncation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Document {
    Environments { subject: Subject, revision: Revision, pools: List<Pool>, environments: List<Environment> },
    Environment(Environment),
    Part(Box<PartDocument>),
}
impl Document {
    /// The identity echoed by this whole document.
    #[must_use]
    pub fn subject(&self) -> Subject {
        match self {
            Self::Environments { subject, .. } => *subject,
            Self::Environment(environment) => {
                Subject { scope: Scope::Project(environment.project), kind: Kind::Environment(environment.number) }
            }
            Self::Part(part) => part.subject,
        }
    }
    /// The owner's current document revision.
    #[must_use]
    pub fn revision(&self) -> Revision {
        match self {
            Self::Environments { revision, .. } => *revision,
            Self::Environment(environment) => environment.revision,
            Self::Part(part) => part.revision,
        }
    }
    /// Whole-value admission against configured bounds.
    #[must_use]
    pub fn fits(&self, limits: &super::Limits) -> bool {
        match price(self, limits) {
            Some(bytes) => match bytes.checked_add(u64::try_from(size_of::<Document>()).expect("document layout")) {
                Some(bytes) => bytes <= limits.document_bytes,
                None => false,
            },
            None => false,
        }
    }
    /// These family pages request no jig objects independently.
    #[must_use]
    pub fn objects(&self, rows: u32) -> List<ObjectKey> {
        List::with_capacity(rows.checked_add(1).expect("object bound"))
    }
}
fn text_price(text: &[u8], limits: &super::Limits) -> Option<u64> {
    if text.len() > usize::try_from(limits.text).ok()? {
        return None;
    }
    u64::try_from(text.len()).ok()
}
fn environment_price(environment: &Environment, limits: &super::Limits) -> Option<u64> {
    text_price(&environment.name, limits)
}
fn pool_price(pool: &Pool, limits: &super::Limits) -> Option<u64> {
    text_price(&pool.name, limits)
}
fn description_price(description: &Description, limits: &super::Limits) -> Option<u64> {
    text_price(&description.words, limits)?.checked_add(text_price(&description.resource, limits)?)
}
fn service_price(service: &Service, limits: &super::Limits) -> Option<u64> {
    text_price(&service.environment, limits)?.checked_add(text_price(&service.name, limits)?)
}
fn effect_price(effect: &EffectAction, limits: &super::Limits) -> Option<u64> {
    match effect {
        EffectAction::Restart { service, .. } | EffectAction::Scale { service, .. } => service_price(service, limits),
        EffectAction::Rollback { service, from, to } => {
            service_price(service, limits)?.checked_add(text_price(from, limits)?)?.checked_add(text_price(to, limits)?)
        }
        EffectAction::CreateEnvironment { .. } | EffectAction::TearDown { .. } => Some(0),
    }
}
fn resource_price(resource: &Resource, limits: &super::Limits) -> Option<u64> {
    let mut bytes = text_price(&resource.name, limits)?;
    match &resource.fact {
        ResourceFact::Service { version, .. } => {
            if let Some(version) = version {
                bytes = bytes.checked_add(text_price(version, limits)?)?;
            }
        }
        ResourceFact::Environment(environment) => {
            if let Some(environment) = environment {
                bytes = bytes.checked_add(environment_price(environment, limits)?)?;
            }
        }
        ResourceFact::Pool(pool) | ResourceFact::Quota(pool) => {
            if let Some(pool) = pool {
                bytes = bytes.checked_add(pool_price(pool, limits)?)?;
            }
        }
    }
    Some(bytes)
}
fn price(document: &Document, limits: &super::Limits) -> Option<u64> {
    match document {
        Document::Environment(environment) => environment_price(environment, limits),
        Document::Environments { pools, environments, .. } => {
            if pools.capacity() > limits.rows || environments.capacity() > limits.rows {
                return None;
            }
            let mut bytes = List::<Pool>::worst_case(pools.capacity())?
                .checked_add(List::<Environment>::worst_case(environments.capacity())?)?;
            for pool in pools {
                bytes = bytes.checked_add(pool_price(pool, limits)?)?;
            }
            for environment in environments {
                bytes = bytes.checked_add(environment_price(environment, limits)?)?;
            }
            Some(bytes)
        }
        Document::Part(part) => {
            if part.resources.capacity() > limits.rows
                || part.effects.capacity() > limits.rows
                || part.drift.capacity() > limits.rows
                || part.environments.capacity() > limits.rows
                || part.pools.capacity() > limits.rows
            {
                return None;
            }
            let mut bytes = u64::try_from(size_of::<PartDocument>())
                .expect("part layout")
                .checked_add(List::<Resource>::worst_case(part.resources.capacity())?)?
                .checked_add(List::<Effect>::worst_case(part.effects.capacity())?)?
                .checked_add(List::<Drift>::worst_case(part.drift.capacity())?)?
                .checked_add(List::<Environment>::worst_case(part.environments.capacity())?)?
                .checked_add(List::<Pool>::worst_case(part.pools.capacity())?)?;
            if let Some(panel) = &part.panel {
                match &panel.procedure {
                    Procedure::Remediate { service, .. } | Procedure::Scale { service, .. } => {
                        bytes = bytes.checked_add(service_price(service, limits)?)?;
                    }
                    Procedure::Provision { .. } | Procedure::TearDown { .. } => {}
                }
            }
            if let Some(effect) = &part.effect {
                bytes = bytes.checked_add(effect_price(effect, limits)?)?;
            }
            if let Some(description) = &part.description {
                bytes = bytes.checked_add(description_price(description, limits)?)?;
            }
            for resource in &part.resources {
                bytes = bytes.checked_add(resource_price(resource, limits)?)?;
            }
            for effect in &part.effects {
                bytes = bytes
                    .checked_add(match &effect.description {
                        Some(description) => description_price(description, limits)?,
                        None => 0,
                    })?
                    .checked_add(effect_price(&effect.action, limits)?)?;
            }
            for drift in &part.drift {
                bytes = bytes.checked_add(text_price(&drift.resource, limits)?)?;
                if let Some(actor) = &drift.actor {
                    bytes = bytes.checked_add(text_price(actor, limits)?)?;
                }
            }
            for environment in &part.environments {
                bytes = bytes.checked_add(environment_price(environment, limits)?)?;
            }
            for pool in &part.pools {
                bytes = bytes.checked_add(pool_price(pool, limits)?)?;
            }
            Some(bytes)
        }
    }
}
/// A keyed request naming exactly the revision the person saw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    MoveEnd { project: u32, environment: u64, revision: Revision, until: Wall },
}
/// A faithful bounded refusal; admission never becomes a false Gone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Role { held: u32 },
    Past,
    Exhausted,
    SignedOut,
    NotAllowed,
    Conflict,
    Invalid,
}
/// A durable environment-request terminal, or current facts showing its revision moved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    MovedEnd { environment: u64, revision: Revision, until: Wall },
    Moved { environment: u64, revision: Revision, until: Option<Wall> },
    Gone,
    Refused { why: Refusal },
    EntranceRefused(jig_client::contract::EntranceRefusal),
}
/// A scoped whole-read terminal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadResult {
    Value(Box<Document>),
    Unknown,
    Busy,
    SignedOut,
}
/// A level-triggered watch callback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WatchEvent {
    Opened,
    Alive,
    Delivered(Box<Document>),
}
/// One terminal for every opened watch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchEnd {
    Closed,
    Dropped,
    Unknown,
    SignedOut,
}
/// The subject for a part; resources and kinds need words only.
#[must_use]
pub fn subject(part: &Part, scope: Scope) -> Option<Subject> {
    let kind = match part {
        Part::Panel { task, .. } => Kind::Panel(*task),
        Part::Result { task, .. } => Kind::Result(*task),
        Part::Effect { proposal, .. } => Kind::Effect(*proposal),
        Part::Outbox { scope, entry, .. } => {
            let scope = match scope {
                jig_client::contract::PartScope::Project(project) => Scope::Project(*project),
                jig_client::contract::PartScope::Deployment => Scope::Deployment,
            };
            return Some(Subject { scope, kind: Kind::Outbox(*entry) });
        }
        Part::Hold { task, .. } => Kind::Hold(*task),
        Part::Project { project, .. } => {
            return Some(Subject { scope: Scope::Project(*project), kind: Kind::Project(*project) });
        }
        Part::Deployment { .. } => return Some(Subject { scope: Scope::Deployment, kind: Kind::Deployment }),
        // Native connector proposal numbers do not name this side's typed maker effects.
        Part::OwnerEffect { .. } | Part::Connector { .. } | Part::Resource { .. } | Part::Kind { .. } => return None,
    };
    Some(Subject { scope, kind })
}
