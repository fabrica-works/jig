//! The native client's HTTP transport (protocol/client.md, sections 8 and 12).
#![forbid(unsafe_code)]
mod connections;
mod sign_in;
pub use connections::{Conn, Connections, Limits};
pub use sign_in::{Signing, Unlinked};
#[cfg(test)]
mod tests;
