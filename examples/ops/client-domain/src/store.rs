//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
use skein_lib::{RequestKey, RequestRecord};
pub const VERSION: u16 = 1;
/// A versioned child or request-table record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Jig(jig_client::store::Record),
    Infrastructure(crate::infrastructure::Record),
    Observability(crate::observability::Record),
    Table { version: u16, record: RequestRecord<crate::Ask> },
}
/// The store key tagged with its owning child or table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Jig(jig_client::store::StoreKey),
    Infrastructure(crate::infrastructure::Key),
    Observability(crate::observability::Key),
    Table(RequestKey),
}
