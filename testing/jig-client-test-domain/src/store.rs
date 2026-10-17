//! Each child's own records, wrapped by the root (root.md, 8).
use alloc::boxed::Box;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Jig(jig_client_view::store::Record),
    First(crate::first::Record),
    Client(jig_client::Record),
    Unlinked(skein_lib::RequestRecord<jig_client::Held>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Key {
    Jig(jig_client_view::store::StoreKey),
    First(crate::first::Key),
    Client(Box<[u8]>),
    Unlinked(skein_lib::RequestKey),
}
