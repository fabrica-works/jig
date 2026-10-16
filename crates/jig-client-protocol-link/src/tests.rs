use crate::*;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_domain::{Answered, Ask, Family, Readable, Readout, Watchable};
use jig_client_protocol_wire::v8 as w;
use skein_lib::{Queue, Reader, RequestEnvelope, Time, Token};
mod attempts;
mod documents;
mod fixtures;
mod pages;

mod reads;

mod stream;

#[test]
fn the_link_serves_the_wire_s_version_alone() {
    assert!(serves(c::VersionRange { first: 8, last: 8 }));
    assert!(!serves(c::VersionRange { first: 7, last: 7 }));
}

const TOO_MANY_OWNERS: [jig_client_domain::Spoken; 16] = {
    let mut owners = [jig_client_domain::Spoken { family: 0, version: 1 }; 16];
    let mut index = 0;
    let mut family = 0;
    while index < 16 {
        owners[index].family = family;
        index += 1;
        family += 1;
    }
    owners
};
const DUPLICATE_OWNERS: [jig_client_domain::Spoken; 2] = [jig_client_domain::Spoken { family: 19, version: 1 }; 2];
const INVALID_OWNERS: [&[jig_client_domain::Spoken]; 2] = [&TOO_MANY_OWNERS, &DUPLICATE_OWNERS];
#[test]
fn owner_declarations_fit_one_hello_without_duplicates() {
    let bounds = fixtures::limits();
    for owners in INVALID_OWNERS {
        let client = jig_client_domain::Limits { owners, ..bounds.client };
        assert_eq!(Limits::new(client, bounds.exchanges), Err(Unusable::Limits), "{owners:?}");
    }
    let client = jig_client_domain::Limits { owners: &TOO_MANY_OWNERS[..15], ..bounds.client };
    assert_eq!(Limits::new(client, bounds.exchanges).expect("fifteen distinct owners").client.owners.len(), 15);
}

mod store;

mod sign_in;
