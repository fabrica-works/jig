//! Jig versions, independent of application versions; client/domain/contract.md, 8.
//! Compatibility is supplied by the protocol stream hello before page requests leave.
/// The inclusive range served by the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VersionRange {
    pub first: u32,
    pub last: u32,
}

impl VersionRange {
    /// Whether this version falls within a valid inclusive range.
    #[must_use]
    pub const fn contains(self, version: u32) -> bool {
        self.first <= self.last && self.first <= version && version <= self.last
    }
}
