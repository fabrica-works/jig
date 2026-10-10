//! Exact configured connector-kind inclusion rows (domain/authority.md, 4–5;
//! client/domain/contract.md, 3.2). Equality is implicit. The receiving owner
//! admits the complete list, preserving its order and duplicates, or refuses it.

/// A grant of `kind` includes `implies` on this connector only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OwnerKindImplication {
    pub connector: u16,
    pub kind: u16,
    pub implies: u16,
}
