//! Whole subtree facts and supplied folds (client/domain/contract.md, 4 and 7;
//! pages.md, 5.2). Parentage, counts and continuation belong to the source.
use super::{Chip, Cursor, OwnerPhaseCounts, OwnerTaskLink, ProposalKey};
use skein_lib::List;
/// One supplied descendant, with independent dependency loss.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerTreeRow {
    pub chip: Chip,
    pub parent: u64,
    pub depth: u32,
    pub proposals: List<ProposalKey>,
    pub spent: u64,
    pub dependencies: List<OwnerTaskLink>,
    pub cut: bool,
}
/// Counts supplied for one parent independently of visible descendants.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerTreeFold {
    pub parent: u64,
    pub phases: OwnerPhaseCounts,
    pub held_below: u32,
}
/// A whole root, descendant window, per-parent folds and opaque continuation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerTree {
    pub root: Chip,
    pub rows: List<OwnerTreeRow>,
    pub folded: List<OwnerTreeFold>,
    pub held_below: u32,
    pub next: Option<Cursor>,
}
