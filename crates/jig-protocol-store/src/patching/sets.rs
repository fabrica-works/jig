//! Static owner data handed in by the shell (protocol/store.md 5.3 and 10.3).

/// Whose patch levels the format row keeps, upstream first.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Owner {
    Skein,
    Smith,
    Jig,
    Application,
}

#[derive(Clone, Copy, Debug)]
pub struct PatchSet {
    pub owner: Owner,
    pub name: &'static str,
    pub families: &'static [KeptFamily],
    /// Patch texts in number order.
    pub patches: &'static [&'static str],
    /// The owner's mark across which windows start fresh.
    pub fresh: Option<&'static str>,
}
/// Which of the deployment's two maps a kept family's records live in (store.md 9, 10.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeptMap {
    Store,
    Secrets,
}

#[derive(Clone, Copy, Debug)]
pub struct KeptFamily {
    pub name: &'static str,
    /// Kept texts in version order.
    pub texts: &'static [&'static str],
    pub map: KeptMap,
    pub keys: &'static [Keys],
    /// The declaration used as a transcript frame root.
    pub frames: Option<&'static str>,
}
#[derive(Clone, Copy, Debug)]
pub struct Keys {
    /// Inclusive; application's own bytes, whole map keys for other owners.
    pub from: &'static [u8],
    /// Exclusive, in the same key space as `from`.
    pub to: &'static [u8],
    /// Declaration of the records in this range.
    pub root: &'static str,
}
#[derive(Clone, Copy, Debug)]
pub struct PatchSets {
    /// At most one set per owner.
    pub sets: &'static [PatchSet],
}
impl PatchSets {
    pub const NONE: PatchSets = PatchSets { sets: &[] };
}
