//! The connector's documents and one revision-naming keyed request. Application
//! values and forms stay here, never in jig's requests (root.md, sections 6–7).
use jig_client_domain::contract::{
    EntranceRefusal, ObjectKey, OwnerPart, OwnerPartKey, OwnerPartKind, Part, ProposalKey, Revision,
};
use skein_lib::List;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    Project(u32),
    Deployment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Item(u64),
    Panel(u64),
    Result(u64),
    Effect(ProposalKey),
    /// A raw proposal reference owned by this connector, separate from jig makers.
    OwnerEffect(u64),
    Outbox(u64),
    Hold(u64),
    Project(u32),
    Deployment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subject {
    pub scope: Scope,
    pub kind: Kind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemState {
    Ready,
    Held,
    Ended,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub project: u32,
    pub key: u64,
    pub revision: Revision,
    pub state: ItemState,
    pub value: u64,
    pub task: u64,
    pub second: Option<u64>,
    pub proposals: List<ProposalKey>,
    pub touchable: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parameters {
    pub item: u64,
    pub revision: Revision,
    pub value: u64,
    pub amendable: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartDocument {
    pub subject: Subject,
    pub revision: Revision,
    pub value: u64,
    pub parameters: Option<Parameters>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Document {
    Item(Item),
    Part(PartDocument),
    /// Complete actual owner value; seven part variants have no revision.
    Connector(crate::connector_values::ConnectorDocument),
}
impl Document {
    #[must_use]
    pub fn subject(&self) -> Subject {
        match self {
            Self::Item(item) => Subject { scope: Scope::Project(item.project), kind: Kind::Item(item.key) },
            Self::Part(part) => part.subject,
            Self::Connector(value) => value.subject,
        }
    }
    #[must_use]
    #[expect(clippy::manual_map, reason = "step code matches optional values explicitly")]
    pub fn revision(&self) -> Option<Revision> {
        match self {
            Self::Item(item) => Some(item.revision),
            Self::Part(part) => Some(part.revision),
            Self::Connector(value) => match value.revision() {
                Some(value) => Some(Revision { value }),
                None => None,
            },
        }
    }
    #[must_use]
    pub fn fits(&self, rows: u32) -> bool {
        match self {
            Self::Item(item) => item.proposals.capacity() <= rows,
            Self::Part(_) => true,
            // Rows alone cannot admit the owner's nested paths and allocation.
            Self::Connector(_) => false,
        }
    }
    #[must_use]
    pub fn sections(&self) -> List<jig_client_view::page::Section> {
        let mut sections = List::with_capacity(2);
        match self {
            Self::Item(item) => {
                for task in [Some(item.task), item.second].into_iter().flatten() {
                    sections
                        .push(jig_client_view::page::Section::Task {
                            task,
                            section: jig_client_view::address::TaskSection::Plan,
                        })
                        .expect("two section names fit");
                }
            }
            Self::Part(_) | Self::Connector(_) => {}
        }
        sections
    }
    #[must_use]
    pub fn objects(&self, capacity: u32) -> List<ObjectKey> {
        let mut objects = List::with_capacity(capacity.checked_add(1).expect("object names fit"));
        match self {
            Self::Item(item) => {
                if objects.push(ObjectKey::Task(item.task)).is_ok() {
                    for proposal in &item.proposals {
                        if objects.push(ObjectKey::Proposal(*proposal)).is_err() {
                            break;
                        }
                    }
                }
            }
            Self::Part(_) | Self::Connector(_) => {}
        }
        objects
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Touch {
        project: u32,
        item: u64,
        revision: Revision,
        parameters: Option<u64>,
    },
    /// The application's complete value-bearing request from its own Object form.
    ChangeObject {
        project: u32,
        object: u64,
        revision: Revision,
        value: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    Touched {
        item: u64,
        revision: Revision,
    },
    Moved {
        item: u64,
        revision: Revision,
    },
    Gone,
    /// A refusal at the shared application entrance, retaining its actual facts.
    Refused(EntranceRefusal),
    /// Complete closed application terminal; no maker or role is inferred.
    Connector(crate::connector_values::ConnectorAnswer),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadResult {
    Value(Document),
    Unknown,
    /// The connector denies this read.
    NotAllowed,
    /// The supplied subject has no representation in this owner's scoped API.
    Unsupported,
    Busy,
    SignedOut,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WatchEvent {
    Opened,
    Alive,
    Delivered(Document),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchEnd {
    Closed,
    Dropped,
    Unknown,
    Unsupported,
    SignedOut,
    /// The party may not see this subject.
    NotAllowed,
    /// The subject ended and left the engine.
    SubjectEnded,
    /// The engine is stopping.
    Stopping,
    /// Watch admission is busy.
    Busy,
}
#[must_use]
pub fn subject(part: &Part, scope: Scope) -> Option<Subject> {
    let kind = match part {
        Part::Panel { task, .. } => Kind::Panel(*task),
        Part::Result { task, .. } => Kind::Result(*task),
        Part::Effect { proposal, .. } => Kind::Effect(*proposal),
        Part::OwnerEffect { proposal, .. } => Kind::OwnerEffect(*proposal),
        Part::Outbox { scope, entry, .. } => {
            let scope = match scope {
                jig_client_domain::contract::PartScope::Project(project) => Scope::Project(*project),
                jig_client_domain::contract::PartScope::Deployment => Scope::Deployment,
            };
            return Some(Subject { scope, kind: Kind::Outbox(*entry) });
        }
        Part::Hold { task, .. } => Kind::Hold(*task),
        Part::Project { project, .. } => {
            return Some(Subject { scope: Scope::Project(*project), kind: Kind::Project(*project) });
        }
        Part::Deployment { .. } => return Some(Subject { scope: Scope::Deployment, kind: Kind::Deployment }),
        Part::Connector { .. } | Part::Resource { .. } | Part::Kind { .. } => return None,
    };
    Some(Subject { scope, kind })
}

/// The explicit held-task account only (root.md, section 6; QUESTIONS.md, 47).
/// A resource origin is a distinct reference and cannot supply this identity.
#[must_use]
pub fn owner_hold(part: &OwnerPart) -> Option<Part> {
    match part.kind {
        OwnerPartKind::Hold => match part.key {
            OwnerPartKey::Number(task) => Some(Part::Hold { connector: part.connector, task }),
            OwnerPartKey::Path(_) => None,
        },
        OwnerPartKind::Resource
        | OwnerPartKind::Pool
        | OwnerPartKind::Result
        | OwnerPartKind::Procedure
        | OwnerPartKind::Effect
        | OwnerPartKind::Outbox => None,
    }
}

/// A connector-owned effect reference keeps its numbered namespace (root.md, 6).
#[must_use]
pub fn owner_effect(part: &OwnerPart) -> Option<Part> {
    match part.kind {
        OwnerPartKind::Effect => match part.key {
            OwnerPartKey::Number(proposal) => Some(Part::OwnerEffect { connector: part.connector, proposal }),
            OwnerPartKey::Path(_) => None,
        },
        OwnerPartKind::Resource
        | OwnerPartKind::Pool
        | OwnerPartKind::Result
        | OwnerPartKind::Procedure
        | OwnerPartKind::Hold
        | OwnerPartKind::Outbox => None,
    }
}
