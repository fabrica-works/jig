//! Agent bounds given in a hello (domain/engine.md, section 8).
use alloc::boxed::Box;
use skein_lib::Duration;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ModelBound {
    pub name: Box<[u8]>,
    pub window: u32,
    pub output: u32,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Bounds {
    pub program_endpoints: Box<[u32]>,
    pub models: Box<[ModelBound]>,
    pub turns: u32,
    pub spend: u64,
    pub time: Duration,
    pub waiting: Duration,
    pub checks: Duration,
    pub message_bytes: u32,
    pub inbox: u32,
    pub unacknowledged: u32,
}
