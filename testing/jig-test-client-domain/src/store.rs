//! Small-store records remain versioned by their owner (root.md, section 8).
use skein_lib::{RequestKey, RequestRecord};
pub const VERSION: u16 = 1;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Jig(jig_client::store::Record),
    First(crate::first::Record),
    Table { version: u16, record: RequestRecord<crate::Ask> },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Jig(jig_client::store::StoreKey),
    First(crate::first::Key),
    Table(RequestKey),
}
