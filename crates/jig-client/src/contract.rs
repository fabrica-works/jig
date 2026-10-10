//! Independent client vocabulary for the complete domain contract.
//! See client/domain/contract.md, 2–8. List capacities and text lengths
//! are the decoding owner's limits; no engine or application type is shared.
pub mod admission;
pub mod admission_limits;
pub mod asks;
pub mod chip_phase;
pub mod common;
pub mod documents;
pub mod entrance;
pub mod live_call;
pub mod owner_part;
pub mod owner_wake;
pub mod policy_edit;
pub mod proposed_task;
pub mod reads;
pub mod recurring;
pub mod refusals;
pub mod run_facts;
pub mod specification;
pub mod turn;
pub mod versions;
pub mod watches;
pub use admission_limits::*;
pub use asks::*;
pub use chip_phase::*;
pub use common::*;
pub use documents::*;
pub use entrance::*;
pub use live_call::*;
pub use owner_part::*;
pub use owner_wake::*;
pub use policy_edit::*;
pub use proposed_task::*;
pub use reads::*;
pub use recurring::*;
pub use refusals::*;
pub use run_facts::*;
pub use specification::*;
mod document_text;
pub use document_text::*;
pub use turn::*;
pub use versions::*;
pub use watches::*;

pub mod owner_authority;
pub use owner_authority::*;

pub mod result_details;
pub use result_details::{OwnerRequester, OwnerTaskLink, OwnerTaskStatus, ResultDetails};
pub mod owner_budget;
pub use owner_budget::*;

pub mod owner_outcome;
pub use owner_outcome::*;

pub mod owner_chip;
pub use owner_chip::*;
