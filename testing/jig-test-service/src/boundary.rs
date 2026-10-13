//! The caller's startup values and the engine's temporary ports (service.md, 7).
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_protocol_config::Problems;
use jig_test_domain as root;
use skein_lib::{List, Secret};

#[derive(Debug)]
pub struct Files {
    pub entries: List<(Box<[u8]>, Secret)>,
}
#[derive(Debug)]
pub struct Seeds {
    pub deployment: [u8; 16],
    pub seed: u64,
    pub crypto: Secret,
}
#[derive(PartialEq, Eq, Debug)]
pub enum End {
    Drained,
    Aborted,
    Stopped(Stop),
}
#[derive(PartialEq, Eq, Debug)]
pub enum Stop {
    CommitFailed { number: u64 },
    Start(Problems),
    Listener,
}
#[derive(Debug)]
pub enum PartyOutput {
    Held(root::Delivery),
    Now(core::Now),
    Busy(Box<root::Event>),
}
