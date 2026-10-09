//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
/// An application destination tagged with its owning child.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Address {
    Jig(jig_client::address::Address),
    Infrastructure(crate::infrastructure::Address),
    Observability(crate::observability::Address),
}
/// A decoded location or a missing address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Address(Address),
    Missing,
}
