//! The native client's HTTP transport (protocol/client.md, sections 8 and 12).
#![forbid(unsafe_code)]
mod connections;
mod files;
mod shell;
pub use connections::{Conn, Connections, Limits};
pub use shell::{Settings, Shell, Transport};
#[cfg(test)]
mod tests;
