//! The test application's whole value admission and allocation pricing kit.
//! Ordinary Rust, with no world, lifecycle, permission or delivery policy.
#![forbid(unsafe_code)]
pub mod connector_values;
pub use jig_test_domain_client::connector_values::ConnectorLimits as Bounds;
