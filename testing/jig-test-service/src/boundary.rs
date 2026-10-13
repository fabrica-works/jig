//! The caller's startup values and the engine's temporary ports (service.md, 7).
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_protocol_config::Problems;
use jig_test_domain as root;
use skein_lib::{List, Queue, Secret};

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
pub enum StoreRequest {
    Commit { number: u64, writes: Queue<root::Write> },
    Restart(core::RestartStep),
    Load(core::Now),
    Read(core::Held),
}
#[derive(Debug)]
pub enum StoreAnswer {
    Committed { number: u64, fill: u64 },
    Failed { number: u64 },
    Restored { step: core::RestartStep, rows: Box<[root::Record]> },
    Core(Box<core::Event>),
    Transcript { task: u64, rows: Box<[core::TurnRecord]>, done: bool },
}
#[derive(Debug)]
pub enum PartyOutput {
    Held(root::Delivery),
    Now(core::Now),
    Busy(Box<root::Event>),
}
