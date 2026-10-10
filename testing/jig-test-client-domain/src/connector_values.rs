//! Whole owner values for parts and family objects (client/domain/root.md, 6–7).
//! The subject is supplied by the original scoped read/watch; these values do
//! not reconstruct it from a page, a numeric effect reference or display bytes.
use crate::family::Subject;
use alloc::boxed::Box;
pub use jig_test_connector::{
    ApplicationObject, ApplicationOutcome, Attempt, ClientDocument, ClientResource, Description, Effect, EffectPhase,
    Fact, Form, Hold, HoldMode, Key, OutboxEntry, Path, ProcedureState, Recovery, ResourceRole, application_core,
};

/// The owner's exact document and independently retained request subject.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectorDocument {
    pub subject: Subject,
    pub value: Box<ClientDocument>,
}
impl ConnectorDocument {
    /// The original request supplies scope; the value supplies its own inner identity.
    /// Individual documents have project scope, never a reconstructed deployment project.
    #[must_use]
    pub fn matches_subject(&self) -> bool {
        use crate::family::{Kind, Scope};
        match self.value.as_ref() {
            ClientDocument::Deployment { .. } => {
                self.subject.scope == Scope::Deployment && self.subject.kind == Kind::Deployment
            }
            ClientDocument::Project { project, .. } => {
                self.subject.scope == Scope::Project(*project) && self.subject.kind == Kind::Project(*project)
            }
            ClientDocument::Object { object, .. } => {
                project_scope(self.subject.scope) && self.subject.kind == Kind::Item(*object)
            }
            ClientDocument::Panel { task, .. } => {
                project_scope(self.subject.scope) && self.subject.kind == Kind::Panel(*task)
            }
            ClientDocument::Result { task, .. } => {
                project_scope(self.subject.scope) && self.subject.kind == Kind::Result(*task)
            }
            ClientDocument::Effect { proposal, .. } => {
                project_scope(self.subject.scope) && self.subject.kind == Kind::OwnerEffect(*proposal)
            }
            ClientDocument::Outbox { entry, .. } => {
                project_scope(self.subject.scope) && self.subject.kind == Kind::Outbox(entry.number)
            }
            ClientDocument::Hold { task, .. } => {
                project_scope(self.subject.scope) && self.subject.kind == Kind::Hold(*task)
            }
        }
    }
    /// Only an Object supplies a revision. Parts are unversioned snapshots.
    #[must_use]
    pub fn revision(&self) -> Option<u64> {
        match self.value.as_ref() {
            ClientDocument::Object { now, .. } => Some(now.revision),
            ClientDocument::Panel { .. }
            | ClientDocument::Result { .. }
            | ClientDocument::Effect { .. }
            | ClientDocument::Outbox { .. }
            | ClientDocument::Hold { .. }
            | ClientDocument::Project { .. }
            | ClientDocument::Deployment { .. } => None,
        }
    }
}

/// A complete closed owner terminal, including changed/moved values and refusals.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConnectorAnswer {
    pub value: ApplicationOutcome,
}

/// Structural envelope for every owner document, independently of its subject.
/// List bounds apply to allocated capacity, path bounds to literal segments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConnectorLimits {
    pub rows: u32,
    pub paths: u32,
    pub segments: u32,
    pub segment_bytes: u32,
    /// Inline carrier plus every owned allocation, excluding allocator overhead.
    pub bytes: u64,
}

fn project_scope(scope: crate::family::Scope) -> bool {
    match scope {
        crate::family::Scope::Project(_) => true,
        crate::family::Scope::Deployment => false,
    }
}
