//! Content free observations for operators and worlds.
//! See client/domain/client.md, 4.2; fact emission must never change decisions.
use jig_client_domain::contract::Limit;
/// One diagnostic observation without person words.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Fact {
    /// The client started.
    Started,
    /// The shared link changed.
    LinkChanged,
    /// A configured client limit refused work.
    Refused { which: Limit },
    /// A page abandoned a read.
    ReadAbandoned,
    /// An old terminal was dropped.
    StaleTerminal,
    /// A sign in ended.
    SignedOut,
    /// The frame refused this version.
    OutOfDate,
}
