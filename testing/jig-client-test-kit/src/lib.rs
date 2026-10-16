//! The test application's whole value admission and allocation pricing kit.
//! Ordinary Rust, with no world, lifecycle, permission or delivery policy.
#![forbid(unsafe_code)]
pub mod connector_values;
pub use jig_client_test_domain::connector_values::ConnectorLimits as Bounds;
