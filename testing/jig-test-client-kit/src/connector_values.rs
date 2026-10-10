//! Whole allocation admission, owned by the receiving client domain
//! (programming-model.md, 6.3; root.md, 6–7).
pub use jig_test_client_domain::connector_bounds::{
    answer_bytes, answer_fits, document_bytes, document_fits, document_heap, document_worst_case,
};
