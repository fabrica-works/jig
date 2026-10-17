//! Keep: the child routing shape and exhaustive connector variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
use alloc::boxed::Box;
/// A versioned child or client record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Jig(jig_client_view::store::Record),
    Infrastructure(crate::infrastructure::Record),
    Observability(crate::observability::Record),
    Client(jig_client::Record),
    Unlinked(skein_lib::RequestRecord<jig_client::Held>),
}
/// The store key tagged with its owning child or client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Key {
    Jig(jig_client_view::store::StoreKey),
    Infrastructure(crate::infrastructure::Key),
    Observability(crate::observability::Key),
    Client(Box<[u8]>),
    Unlinked(skein_lib::RequestKey),
}
