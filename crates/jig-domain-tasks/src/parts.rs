//! Opaque connector references carried by person approval contracts
//! (domain/people.md, section 8; client/domain/objects.md, section 7).
use alloc::boxed::Box;
use skein_lib::List;

/// Connector-owned value's kind, carried only as a reference.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PartKind {
    /// A named resource or its state.
    Resource,
    /// A counted resource pool.
    Pool,
    /// A task's connector result.
    Result,
    /// A procedure's panel or kind.
    Procedure,
    /// A proposed effect's description.
    Effect,
    /// An existing connector-owned outbox entry.
    Outbox,
    /// The connector's account of drift or a failed effect for the numbered held task.
    Hold,
}

/// The core's key for a connector-owned part; keys are never shortened.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PartKey {
    /// Literal resource path segments.
    Path(List<Box<[u8]>>),
    /// A numbered kind, resource, task or proposal.
    Number(u64),
}

/// A reference to application data; no application value is copied.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Part {
    pub connector: u16,
    pub kind: PartKind,
    pub key: PartKey,
}
