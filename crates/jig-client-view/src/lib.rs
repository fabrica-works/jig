//! Platform-neutral client trees (client/domain/view.md, sections 3 and 4).
//!
//! The builder owns preorder nodes and an open-parent stack. It knows no
//! domain objects, platform elements or layout. Its calls assert the limits
//! declared by the caller; `finish` requires every opened node to be closed.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod binding;
mod builder;
mod diff;
mod limits;
mod render;
mod tree;

pub use binding::{Act, Binding, BoundAct, number};
pub use builder::{Builder, Slot, SlotId};
pub use diff::{Attribute, Made, Patch, diff};
pub use limits::{Limits, worst_case};
pub use render::View;
pub use tree::{
    Drawing, DrawingKind, Key, Node, NodeId, Owner, Role, Run, SlotRef, State, Style, TextRun, Tree, Value,
};

#[cfg(test)]
mod tests;
