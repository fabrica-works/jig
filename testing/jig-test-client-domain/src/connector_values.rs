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
