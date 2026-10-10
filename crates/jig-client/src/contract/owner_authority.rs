//! Owner authority facts without interpretation (client/domain/contract.md, 3.2 and 5).
//! Exact field vocabulary follows domain/authority.md, sections 3 and 4.
//! Document completeness belongs to the document wrapper; an admitted payload
//! carries only the authority. Literal pattern bytes are never rendered as paths.
use alloc::boxed::Box;
use skein_lib::{List, Wall};

/// Literal resource pattern, preserving segment boundaries and terminal bytes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerPattern {
    pub segments: List<Box<[u8]>>,
    pub last: OwnerPatternEnd,
}
/// The owner's exact terminal or open terminal prefix.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OwnerPatternEnd {
    Exact(Box<[u8]>),
    Open(Box<[u8]>),
}
/// A permission for one connector's numbered kind.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerGrant {
    pub connector: u16,
    pub kind: u16,
    pub pattern: OwnerPattern,
}
/// Authority executor codes, distinct from a task's concrete executor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerAuthorityExecutor {
    Charter(u32),
    Procedure(u32),
    Role(u32),
}
/// Delegation ceilings and the exact allowed executor codes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerDelegation {
    pub kinds: List<OwnerAuthorityExecutor>,
    pub tasks: u32,
    pub depth: u32,
}
/// A resource note permission, independent of note identity and project scope.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerResourceScope {
    pub connector: u16,
    pub pattern: OwnerPattern,
}
/// Whole admitted authority payload, retaining raw tools and note scope bits.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerAuthority {
    pub tools: u64,
    pub grants: List<OwnerGrant>,
    pub delegation: OwnerDelegation,
    pub spend: u64,
    pub deadline: Option<Wall>,
    pub notes: u8,
    pub note_resources: List<OwnerResourceScope>,
}
/// An owner document's authority projection and explicit cut metadata.
/// A cut projection does not establish a whole admitted replacement payload.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerAuthorityDocument {
    pub authority: OwnerAuthority,
    pub cut: bool,
}
impl OwnerAuthorityDocument {
    /// A whole replacement exists only when this owner projection is complete.
    #[must_use]
    pub fn whole(&self) -> Option<OwnerAuthority> {
        if self.cut { None } else { Some(self.authority.clone()) }
    }
}
