//! Whole specification values and explicit document loss (client/domain/contract.md, 5.2–5.3).
use super::Text;
use skein_lib::List;

/// A complete replacement, with numbered parameters and historical input identities.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Specification {
    pub words: Text,
    pub parameters: List<Parameter>,
    pub inputs: List<u64>,
}

/// A supplied specification and its owner's explicit loss marker.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificationDocument {
    pub specification: Specification,
    pub cut: bool,
}

/// An opaque named specification value; resources retain their connector identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Parameter {
    Number { name: u32, value: u64 },
    Bytes { name: u32, value: Text },
    Resource { name: u32, connector: u16, resource: u64 },
}
