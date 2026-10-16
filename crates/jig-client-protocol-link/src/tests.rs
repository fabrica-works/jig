use crate::*;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_domain::{Answered, Ask, Family, Readable, Readout, Watchable};
use jig_client_protocol_wire::v7 as w;
use skein_lib::{Queue, Reader, RequestEnvelope, Time, Token};
mod attempts;
mod documents;
mod fixtures;
mod pages;

mod reads;

mod stream;

#[test]
fn the_link_serves_the_wire_s_version_alone() {
    assert!(serves(c::VersionRange { first: 7, last: 7 }));
    assert!(!serves(c::VersionRange { first: 6, last: 6 }));
}

mod store;

mod sign_in;
