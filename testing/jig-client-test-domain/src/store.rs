//! Small-store records remain versioned by their owner (root.md, section 8).
use skein_lib::{RequestKey, RequestRecord};
pub const VERSION: u16 = 2;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Jig(jig_domain_client::store::Record),
    First(crate::first::Record),
    Table { version: u16, record: RequestRecord<Held> },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Jig(jig_domain_client::store::StoreKey),
    First(crate::first::Key),
    Table(RequestKey),
}

/// Root vocabulary keeps its application request beside the client key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    pub key: jig_client_domain::Key,
    pub request: crate::Ask,
}
