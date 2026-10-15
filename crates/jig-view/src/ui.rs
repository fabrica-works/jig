//! Portable UI trees, building and diffs (skein's ui.md, sections 3, 5 and 7).
pub(crate) mod builder;
pub(crate) mod diff;
pub mod markdown;
pub(crate) mod tree;
mod view;

pub use builder::{Builder, Slot, SlotId};
pub use diff::{Attribute, Made, Patch, diff};
pub use tree::{
    Drawing, DrawingKind, Key, Modifier, Node, NodeId, Owner, Picture, Region, Role, Run, Shortcut, SlotRef, State,
    Style, TextRun, Tree, Value,
};
pub use view::View;

#[cfg(test)]
pub(crate) mod tests;
