//! The reference engine and worker service, composed on one loop (service.md, 7 and 10).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod accounts;
mod boundary;
mod tools;
mod values;
mod workers;
pub use accounts::AccountsRuntime;
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

#[expect(missing_debug_implementations, reason = "the service owns protocol account credentials")]
pub struct Service {
    io: Io,
    io_env: Env<io::Limits>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    raw_submissions: Queue<Submit>,
    kernel: accounts::Kernel,
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
            raw_submissions: Queue::with_capacity(queue),
            kernel: accounts::Kernel::new(
                io::operations(&limits).expect("io operations").checked_add(2).expect("file operations"),
            ),
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
        io::operations(&self.io_env.limits).expect("checked io operations").checked_add(2).expect("file operations")
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
            || !self.raw_submissions.is_empty()
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
            && self.raw_submissions.is_empty()
            && self.kernel.is_empty()
            && match &self.engine.accounts {
                Some(accounts) => accounts.is_empty(),
                None => true,
            }
    }
    pub fn shutdown(&mut self) {
        if self.shutdowns == 0 {
            self.shutdowns = 1;
            self.engine.events.push(jig_test_domain::Event::Stop);
        } else {
            self.engine.end = Some(End::Aborted);
        }
    }
    fn route_components(&mut self) {
        if let Some(workers) = &mut self.engine.workers {
            for _ in 0..workers.io.len() {
                if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions {
                    break;
                }
                let request = workers.io.pop().expect("worker io requests");
                io::down(&mut self.io, &self.io_env, request, &mut self.raw_submissions);
            }
        }
        if let Some(accounts) = &mut self.engine.accounts {
            for _ in 0..accounts.io.len() {
                if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions {
                    break;
                }
                let request = accounts::tag(accounts.io.pop().expect("account io requests"));
                io::down(&mut self.io, &self.io_env, request, &mut self.raw_submissions);
            }
        }
    }
    fn room(&self, max: io::MaxOut) -> bool {
        self.told.room() >= max.events && self.raw_submissions.room() >= max.submissions
    }
}

pub fn iterate(service: &mut Service, now: Time, wall: Wall) {
    service.io_env.now = now;
    service.io_env.wall = wall;
    if let Some(accounts) = &mut service.engine.accounts {
        accounts.clock(now, wall);
    }
    for _ in 0..service.io_env.limits.sockets.checked_add(service.io_env.limits.refusals).expect("io entries") {
        if !service.io.is_ready() || !service.room(io::MAX_OUT_RESUME) {
            break;
        }
        io::resume(&mut service.io, &service.io_env, &mut service.told, &mut service.raw_submissions);
    }
    if !service.io.is_ready() {
        for _ in 0..service.completions.len() {
            if !service.room(io::MAX_OUT_UP) {
                break;
            }
            let complete = service.completions.pop().expect("completion count");
            let file_pending = pending_files(service.engine.accounts.as_ref());
            let (files, complete) = service.kernel.complete(complete, &service.raw_submissions, file_pending);
            if files {
                service.engine.accounts.as_mut().expect("file operation has accounts").complete(complete);
            } else {
                io::up(&mut service.io, &service.io_env, complete, &mut service.told, &mut service.raw_submissions);
            }
        }
        if service.completions.is_empty() {
            for _ in 0..service.io_env.limits.sockets.checked_mul(2).expect("io timers") {
                if !service.io.is_due(now) || !service.room(io::MAX_OUT_FIRE) {
                    break;
                }
                io::fire(&mut service.io, &service.io_env, &mut service.told, &mut service.raw_submissions);
            }
        }
    }
    for _ in 0..service.told.len() {
        if !service.engine.io_room() {
            break;
        }
        let event = service.told.pop().expect("io event count");
        if let io::Event::Shutdown { .. } = event {
            service.shutdown();
            continue;
        }
        if workers::owner(&event) == Some(Token::new(1)) {
            if let io::Event::Closed { .. } = event {
                service.signals = None;
            }
            continue;
        }
        if match workers::owner(&event) {
            Some(owner) => accounts::owns(owner),
            None => false,
        } {
            if let Some(accounts) = &mut service.engine.accounts {
                accounts.up(accounts::untag(event));
            }
        } else if let Some(workers) = &mut service.engine.workers {
            if !workers.up(now, event, &mut service.engine.events) {
                service.engine.end = Some(End::Stopped(Stop::Listener));
            }
        } else {
            service.engine.end = Some(End::Stopped(Stop::Listener));
        }
    }
    engine::iterate(&mut service.engine, now, wall);
    if service.engine.end.is_some()
        && !service.signals_closing
        && service.raw_submissions.room() >= io::MAX_OUT_DOWN.submissions
        && let Some(entity) = service.signals
    {
        service.signals_closing = true;
        io::down(&mut service.io, &service.io_env, io::Request::Close { entity }, &mut service.raw_submissions);
    }
    service.engine.abort_if_ended();
    service.route_components();
    service.kernel.publish(false, &mut service.raw_submissions, &mut service.submissions);
    if let Some(accounts) = &mut service.engine.accounts {
        service.kernel.publish(true, &mut accounts.submitted, &mut service.submissions);
    }
    service.io.reclaim();
}

#[cfg(test)]
mod tests;

fn account_submissions(accounts: &accounts::Accounts) -> &Queue<Submit> {
    &accounts.submitted
}

fn pending_files(accounts: Option<&accounts::Accounts>) -> Option<&Queue<Submit>> {
    let accounts = accounts?;
    Some(account_submissions(accounts))
}
