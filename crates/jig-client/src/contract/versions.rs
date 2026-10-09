//! Jig versions, independent of application versions; client/domain/contract.md, 8.
//! Compatibility is decided on the first frame, before any page request leaves.
/// The inclusive range served by the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VersionRange {
    pub first: u16,
    pub last: u16,
}

/// This client's jig document and ask version.
pub const CLIENT_VERSION: u16 = 1;
impl VersionRange {
    /// Whether this version falls within a valid inclusive range.
    #[must_use]
    pub const fn contains(self, version: u16) -> bool {
        self.first <= self.last && self.first <= version && version <= self.last
    }
}
