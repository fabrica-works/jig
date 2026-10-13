//! The reference engine and worker service, composed on one loop (service.md, 7 and 10).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod boundary;
mod engine;
pub mod limits;
mod route;
mod startup;
mod stop;
pub mod system;
pub use boundary::{End, Files, PartyOutput, Seeds, Stop, StoreAnswer, StoreRequest};
pub use engine::Engine;
pub use limits::Limits;
use skein_io::{
    self as io, Io,
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{Env, Queue, Time, Token, Wall};
pub use startup::{named, startup};

/// The worker's parsed agent values are supplied by its caller.
#[derive(Debug)]
pub struct Agent {
    pub endpoints: smith_protocol_channel::Endpoints,
    pub limits: smith_host_protocol::Limits,
    pub bounds: jig_domain_host::Bounds,
}

#[derive(Debug)]
pub struct Service {
    io: Io,
    io_env: Env<io::Limits>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    told: Queue<io::Event>,
    engine: Engine,
    signals: Option<Token>,
    signals_closing: bool,
    shutdowns: u8,
    bytes: u64,
}
impl Service {
    fn new(engine: Engine, limits: io::Limits, bytes: u64) -> Self {
        let queue = engine.max_out.checked_mul(2).expect("service queues fit");
        Self {
            io: Io::new(&limits),
            io_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            completions: Queue::with_capacity(queue),
            submissions: Queue::with_capacity(queue),
            told: Queue::with_capacity(queue),
            engine,
            signals: None,
            signals_closing: false,
            shutdowns: 0,
            bytes,
        }
    }
    pub fn adopt_signals(&mut self, descriptor: Fd) -> Result<(), Fd> {
        if self.signals.is_some() {
            return Err(descriptor);
        }
        match self.io.adopt_signals_for(descriptor, Token::new(1)) {
            Ok(source) => {
                self.signals = Some(source);
                Ok(())
            }
            Err(descriptor) => Err(descriptor),
        }
    }
    pub const fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    pub const fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    pub const fn engine(&mut self) -> Option<&mut Engine> {
        Some(&mut self.engine)
    }
    #[must_use]
    pub const fn end(&self) -> Option<&End> {
        self.engine.end.as_ref()
    }
    #[must_use]
    pub const fn worst_case(&self) -> u64 {
        self.bytes
    }
    #[must_use]
    pub fn operations(&self) -> u32 {
        io::operations(&self.io_env.limits).expect("checked io operations")
    }
    #[must_use]
    pub fn next_policy_deadline(&self) -> Option<Time> {
        self.engine.next_deadline()
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        match self.io.next_deadline() {
            Some(deadline) => Some(match self.engine.next_deadline() {
                Some(engine) => deadline.min(engine),
                None => deadline,
            }),
            None => self.engine.next_deadline(),
        }
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.io.is_ready()
            || self.io.is_due(now)
            || !self.completions.is_empty()
            || !self.told.is_empty()
            || self.engine.work_pending(now)
            || (self.engine.end.is_some() && self.signals.is_some() && !self.signals_closing)
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.engine.end.is_some()
            && self.io.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
            && self.told.is_empty()
    }
    pub fn shutdown(&mut self) {
        if self.shutdowns == 0 {
            self.shutdowns = 1;
            self.engine.events.push(jig_test_domain::Event::Stop);
        } else {
            self.engine.end = Some(End::Aborted);
        }
    }
    fn room(&self, max: io::MaxOut) -> bool {
        self.told.room() >= max.events && self.submissions.room() >= max.submissions
    }
}

pub fn iterate(service: &mut Service, now: Time, wall: Wall) {
    service.io_env.now = now;
    service.io_env.wall = wall;
    for _ in 0..service.io_env.limits.sockets.checked_add(service.io_env.limits.refusals).expect("io entries") {
        if !service.io.is_ready() || !service.room(io::MAX_OUT_RESUME) {
            break;
        }
        io::resume(&mut service.io, &service.io_env, &mut service.told, &mut service.submissions);
    }
    if !service.io.is_ready() {
        for _ in 0..service.completions.len() {
            if !service.room(io::MAX_OUT_UP) {
                break;
            }
            let complete = service.completions.pop().expect("completion count");
            io::up(&mut service.io, &service.io_env, complete, &mut service.told, &mut service.submissions);
        }
        if service.completions.is_empty() {
            for _ in 0..service.io_env.limits.sockets.checked_mul(2).expect("io timers") {
                if !service.io.is_due(now) || !service.room(io::MAX_OUT_FIRE) {
                    break;
                }
                io::fire(&mut service.io, &service.io_env, &mut service.told, &mut service.submissions);
            }
        }
    }
    for _ in 0..service.told.len() {
        if service.engine.events.room() == 0 {
            break;
        }
        match service.told.pop().expect("io event count") {
            io::Event::Shutdown { .. } => service.shutdown(),
            io::Event::Closed { .. } => service.signals = None,
            io::Event::Failed { .. }
            | io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Stream { .. }
            | io::Event::Output { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. } => service.engine.end = Some(End::Stopped(Stop::Listener)),
        }
    }
    engine::iterate(&mut service.engine, now, wall);
    if service.engine.end.is_some()
        && !service.signals_closing
        && service.submissions.room() >= io::MAX_OUT_DOWN.submissions
        && let Some(entity) = service.signals
    {
        service.signals_closing = true;
        io::down(&mut service.io, &service.io_env, io::Request::Close { entity }, &mut service.submissions);
    }
    service.io.reclaim();
}

#[cfg(test)]
mod tests;
