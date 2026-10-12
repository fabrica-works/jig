//! Ops's connector identities, shared by both application roots.
//! This module keeps no state and interprets no connector values.
//! `number` and `connector` translate the fixed routing vocabulary
//! (client/domain/root.md, section 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

/// A connector whose owner values are routed by either root.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Connector {
    /// Observed services, alerts and incidents.
    Observability,
    /// Production resources, procedures and environments.
    Infrastructure,
}

/// The connector's fixed number in jig's vocabulary.
#[must_use]
pub const fn number(connector: Connector) -> u16 {
    match connector {
        Connector::Observability => 1,
        Connector::Infrastructure => 2,
    }
}

/// The owner of a known number; unknown numbers have no connector.
#[must_use]
pub const fn connector(number: u16) -> Option<Connector> {
    match number {
        1 => Some(Connector::Observability),
        2 => Some(Connector::Infrastructure),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
