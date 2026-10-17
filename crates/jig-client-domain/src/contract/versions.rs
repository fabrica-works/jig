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

/// The owner of versioned records (contract.md, 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Jig,
    Family(u16),
}
/// One inclusive range of an owner's versions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Served {
    pub owner: Owner,
    pub first: u32,
    pub last: u32,
}
/// Nothing was read, made or kept in this owner's version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotServed {
    pub owner: Owner,
    pub served: skein_lib::List<Served>,
}
/// The admission scope of an application's request or subject.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Project(u32),
    Deployment,
    Own,
}
/// The bounded page carried by a family read or watch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Window {
    pub rows: u32,
    pub after: Option<super::Cursor>,
}
