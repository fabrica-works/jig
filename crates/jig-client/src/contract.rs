//! Independent client vocabulary for the complete domain contract.
//! See client/domain/contract.md, 2–8. List capacities and text lengths
//! are the decoding owner's limits; no engine or application type is shared.
pub mod admission;
pub mod admission_limits;
pub mod asks;
pub mod common;
pub mod documents;
pub mod entrance;
pub mod owner_wake;
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
pub use common::*;
pub use documents::*;
pub use entrance::*;
pub use owner_wake::*;
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
