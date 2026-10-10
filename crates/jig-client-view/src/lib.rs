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
pub mod cards;
pub mod confirmation;
pub mod document_text;
mod instructions;
pub mod literal_segment;
pub mod live_call_words;
pub mod notices;
pub mod owner_wake;
pub mod owner_wake_words;
pub mod owner_words;
pub mod pages;
pub mod proposed_task;
pub mod proposed_task_words;
pub mod specification;
pub mod styles;
mod task_result;
pub mod turns;
pub use styles::JigStyle;
mod chip_phase;
mod diff;
pub mod frame;
mod frame_facts;
mod limits;
pub mod markdown;
pub mod phrases;
mod render;
mod runtime;
mod sizes;
mod tree;
pub mod words;

pub use binding::{Act, Binding, BoundAct, JigBindings, JigMeaning, decode, jig_bindings_worst_case, number};
pub use builder::{Builder, Slot, SlotId};
pub use diff::{Attribute, Made, Patch, diff};
pub use limits::{Limits, worst_case};
pub use pages::{after_page, jig};
pub use render::View;
pub use sizes::{
    card_nodes, chip_nodes, confirmation_nodes, conversation_nodes, conversations_nodes, deployment_nodes,
    escalation_nodes, frame_nodes, inbox_nodes, not_the_work_nodes, notices_nodes, page_nodes, person_task_nodes,
    project_nodes, proposal_nodes, question_nodes, refusal_nodes, reply_nodes, result_nodes, task_nodes,
};
pub use tree::{
    Drawing, DrawingKind, Key, Node, NodeId, Owner, Role, Run, SlotRef, State, Style, TextRun, Tree, Value,
};
pub use turns::conversation;

#[cfg(test)]
mod tests;

pub mod owner_authority;
pub mod owner_authority_words;
pub mod owner_budget;
pub mod owner_budget_words;
pub mod owner_result;
pub mod owner_result_words;
pub mod policy_edit;

pub mod owner_outcome;
pub mod owner_outcome_words;

pub mod owner_chip;
pub mod owner_chip_words;

pub mod owner_amendment;
pub mod owner_amendment_words;

pub mod owner_activity;
pub mod owner_activity_words;

pub mod owner_history;
pub mod owner_history_words;

pub mod owner_milestone;
pub mod owner_milestone_words;

pub mod owner_inbox_words;

pub mod owner_tree;
pub mod owner_tree_words;

pub mod owner_proposal;
pub mod owner_proposal_words;
