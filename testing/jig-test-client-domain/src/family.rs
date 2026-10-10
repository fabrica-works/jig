//! The connector's documents and one revision-naming keyed request. Application
//! values and forms stay here, never in jig's requests (root.md, sections 6–7).
use jig_client::contract::{EntranceRefusal, ObjectKey, Part, ProposalKey, Revision};
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
}
impl Document {
    #[must_use]
    pub fn subject(&self) -> Subject {
        match self {
            Self::Item(item) => Subject { scope: Scope::Project(item.project), kind: Kind::Item(item.key) },
            Self::Part(part) => part.subject,
        }
    }
    #[must_use]
    pub fn revision(&self) -> Revision {
        match self {
            Self::Item(item) => item.revision,
            Self::Part(part) => part.revision,
        }
    }
    #[must_use]
    pub fn fits(&self, rows: u32) -> bool {
        match self {
            Self::Item(item) => item.proposals.capacity() <= rows,
            Self::Part(_) => true,
        }
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
            Self::Part(_) => {}
        }
        objects
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Touch { project: u32, item: u64, revision: Revision, parameters: Option<u64> },
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
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadResult {
    Value(Document),
    Unknown,
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
    SignedOut,
}
#[must_use]
pub fn subject(part: &Part, scope: Scope) -> Option<Subject> {
    let kind = match part {
        Part::Panel { task, .. } => Kind::Panel(*task),
        Part::Result { task, .. } => Kind::Result(*task),
        Part::Effect { proposal, .. } => Kind::Effect(*proposal),
        Part::Hold { task, .. } => Kind::Hold(*task),
        Part::Project { project, .. } => {
            return Some(Subject { scope: Scope::Project(*project), kind: Kind::Project(*project) });
        }
        Part::Deployment { .. } => return Some(Subject { scope: Scope::Deployment, kind: Kind::Deployment }),
        Part::Connector { .. } | Part::Resource { .. } | Part::Kind { .. } => return None,
    };
    Some(Subject { scope, kind })
}
