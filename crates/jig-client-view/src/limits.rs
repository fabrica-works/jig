//! Configured tree storage (client/domain/view.md, section 10).
//! The caller derives these capacities from the domain and its own fragments.
//! `worst_case` counts owned heap, excluding allocator overhead, for two trees
//! and builders with their splice scratch. Patch storage is added by the diff's owner.

use crate::builder::PlacedSlot;
use crate::{Node, Slot, State, Style, TextRun};
use skein_lib::{List, Stack};

/// Capacities checked before each builder allocation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub nodes: u32,
    pub depth: u32,
    /// Name, runs and field value together on a node.
    pub text_per_node: u32,
    /// Name, runs and field value together across the tree.
    pub text_bytes: u32,
    pub drawing_bytes: u32,
    pub styles_per_node: u32,
    pub patches: u32,
    pub bindings: u32,
    pub slots: u32,
    pub markdown_depth: u32,
}

/// Heap for a visible tree, two builders, splice scratch and slot snapshots.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let nodes = List::<Node>::worst_case(limits.nodes)?;
    let states = List::<State>::worst_case(9)?.checked_mul(u64::from(limits.nodes))?;
    let styles = List::<Style>::worst_case(limits.styles_per_node)?.checked_mul(u64::from(limits.nodes))?;
    let runs = List::<TextRun>::worst_case(limits.text_per_node)?.checked_mul(u64::from(limits.nodes))?;
    let drawings = u64::from(limits.drawing_bytes).checked_mul(u64::from(limits.nodes))?;
    let payloads = u64::from(limits.text_bytes).checked_add(drawings)?;
    let tree = nodes.checked_add(states)?.checked_add(styles)?.checked_add(runs)?.checked_add(payloads)?;
    let trees = tree.checked_mul(3)?.checked_add(nodes.checked_mul(2)?)?;
    let stacks = Stack::<u32>::worst_case(limits.depth)?.checked_mul(3)?;
    let slots = List::<PlacedSlot>::worst_case(limits.slots)?
        .checked_mul(3)?
        .checked_add(List::<Slot>::worst_case(limits.slots)?.checked_mul(2)?)?;
    trees.checked_add(stacks)?.checked_add(slots)
}
