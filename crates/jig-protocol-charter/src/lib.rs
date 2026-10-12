//! Translation of a core-owned charter into Smith's run vocabulary
//! (domain/hosts.md, section 8). The application's root supplies rendered
//! sections and connector tools; this crate owns no application policy.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

mod boundary;
mod call;
mod charter;
mod encode;
mod result;
#[cfg(test)]
mod tests;

pub use boundary::{
    Budget, ChangeRule, Charter, Contract, Conventions, FieldRule, ItemRule, Items, Model, Prices, Reserve, Section,
    TextRule, Tool, ToolEffect, VerdictRule, WorkspaceTools,
};
pub use call::{CallKey, call_key};
pub use charter::charter;
pub use encode::{EndpointName, encode};
pub use result::{Field, RunResult, Unreadable, result};
