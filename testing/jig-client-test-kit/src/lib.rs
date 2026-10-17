//! The test application's whole value admission and allocation pricing kit.
//! Ordinary Rust, with optional delivery and presentation checks for worlds.
#![forbid(unsafe_code)]
pub mod check;
pub mod connector_values;
pub use jig_client_test_domain::connector_values::ConnectorLimits as Bounds;
