use crate::*;
use alloc::boxed::Box;
use jig_client_protocol_wire::v7 as w;
use jig_domain_client::contract as c;
use skein_lib::{Queue, Reader, RequestEnvelope, Time, Token};
mod attempts;
mod documents;
mod fixtures;
mod pages;

mod reads;

mod stream;

#[test]
fn the_client_version_matches_its_wire_version() {
    assert_eq!(c::CLIENT_VERSION, jig_client_protocol_wire::VERSION);
}

mod store;
