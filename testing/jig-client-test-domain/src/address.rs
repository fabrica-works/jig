//! Each address and place names the child whose page it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Address {
    Jig(jig_domain_client::address::Address),
    First(crate::first::Address),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Address(Address),
    Missing,
}
