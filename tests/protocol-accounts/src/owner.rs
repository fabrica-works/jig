//! One process composing the real child, translation, table and driver.
use crate::fixtures::{self, ACCOUNT_CHAT, ROOM};
use crate::world::Story;
use jig_domain_accounts as child;
use jig_protocol_accounts::{
    table::{Table, Value},
    translate::{self, Up},
};
use skein_io::{self as io, kernel};
use skein_lib::{Env, List, Queue, Secret, Time, Wall};
use skein_oauth_accounts as driver;
use skein_world::{Host, files};
use std::net::SocketAddr;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Heard {
    Granted { account: u32, generation: u64 },
    Expiring { account: u32, generation: u64 },
    Failed { account: u32, failure: child::Failure },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trace {
    Requested(child::ToComponent),
    Heard(Heard),
    Closed,
}
pub struct Owner {
    io: io::Io,
    component: Option<driver::Component>,
    limits: driver::Limits,
    child: child::Domain,
    child_limits: child::Limits,
    child_out: Queue<child::Request>,
    table: Table,
    events: Queue<io::Event>,
    requests: Queue<io::Request>,
    files: Queue<driver::FileLower>,
    file_driver: files::Driver,
    above: Queue<driver::Event>,
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
    pub(crate) address: Option<SocketAddr>,
    heard: Vec<Heard>,
    facts: Vec<child::Fact>,
    trace: Vec<Trace>,
    close_requested: Option<()>,
    reject_requested: Option<()>,
    closing: bool,
    closed: bool,
    usable: bool,
    now: Time,
    story: Story,
    account: u32,
}
impl std::fmt::Debug for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Owner")
            .field("heard", &self.heard)
            .field("trace", &self.trace)
            .field("closed", &self.closed)
            .finish_non_exhaustive()
    }
}
impl Owner {
    pub(crate) fn new(root: kernel::Fd, story: Story) -> Self {
        let limits = fixtures::limits();
        let child_limits = fixtures::child_limits();
        let account = if matches!(story, Story::HandedIn { .. } | Story::HandedInRejected) {
            fixtures::ACCOUNT_HANDED
        } else {
            ACCOUNT_CHAT
        };
        Self {
            io: io::Io::new(&limits.io),
            component: None,
            child: child::Domain::new(&child_limits, Box::new([account])),
            child_out: Queue::with_capacity(ROOM),
            child_limits,
            table: Table::new(limits.accounts, limits.client.document.token_bytes),
            events: Queue::with_capacity(ROOM),
            requests: Queue::with_capacity(ROOM),
            files: Queue::with_capacity(ROOM),
            above: Queue::with_capacity(ROOM),
            completions: Queue::with_capacity(ROOM),
            submissions: Queue::with_capacity(ROOM),
            file_driver: files::Driver::new(root, 4, limits.client.document.record_bytes, limits.file_stall, 1234),
            limits,
            address: None,
            heard: Vec::with_capacity(8),
            facts: Vec::with_capacity(32),
            trace: Vec::with_capacity(16),
            close_requested: None,
            reject_requested: None,
            closing: false,
            closed: false,
            usable: false,
            now: Time::ZERO,
            story,
            account,
        }
    }
    #[must_use]
    pub fn child_heard(&self) -> &[Heard] {
        &self.heard
    }
    #[must_use]
    pub fn facts(&self) -> &[child::Fact] {
        &self.facts
    }
    #[must_use]
    pub fn trace(&self) -> &[Trace] {
        &self.trace
    }
    #[must_use]
    pub fn table_value(&self, account: u32, generation: u64) -> Option<&[u8]> {
        Some(self.table.value(account, generation, self.now)?.token.expose())
    }
    #[must_use]
    pub fn became_usable(&self) -> bool {
        self.usable
    }
    #[must_use]
    pub fn closed(&self) -> bool {
        self.closed
    }
    pub(crate) fn close(&mut self) {
        self.close_requested = Some(());
    }
    pub(crate) fn reject(&mut self) {
        self.reject_requested = Some(());
    }
    pub(crate) fn unstarted(&self) -> bool {
        self.component.is_none()
    }
    fn child_event(&mut self, now: Time, wall: Wall, event: child::Event) {
        child::step(&mut self.child, &Env { now, wall, limits: self.child_limits }, event, &mut self.child_out);
        self.usable |= self.child.usable(self.account);
        while let Some(fact) = self.child.pop_fact() {
            self.facts.push(fact);
        }
    }
    fn start(&mut self, env: &Env<driver::Limits>) {
        let mut configured = List::with_capacity(1);
        assert!(
            configured
                .push(if matches!(self.story, Story::HandedIn { .. } | Story::HandedInRejected) {
                    driver::Account::HandedIn
                } else {
                    driver::Account::SignIn {
                        registration: fixtures::registration(),
                        endpoint: driver::Endpoint {
                            address: self.address.expect("issuer address"),
                            transport: driver::Transport::Plaintext,
                        },
                        keeper: driver::Keeper::Private {
                            root: self.file_driver.root(),
                            directory: skein_lib::bytes::copy_of(b"secret"),
                            file: skein_lib::bytes::copy_of(b"record"),
                        },
                    }
                })
                .is_ok()
        );
        let crypto = skein_lib::CryptoRng::new(&skein_world::crypto_seed(17), 0).expect("world crypto seed");
        self.component = Some(driver::Component::new(configured, &self.limits, crypto).expect("configured driver"));
        if let Some(lapses_in) = fixtures::handed_lapse(self.story) {
            let request = jig_protocol_accounts::handed_in::hand_in(
                self.account,
                Secret::new(skein_lib::bytes::copy_of(fixtures::TOKEN_FIRST)),
                Wall::from_nanos(env.wall.as_nanos().saturating_add(lapses_in.as_nanos())),
                None,
                &self.limits.client.document,
            )
            .expect("handed-in fixture");
            self.component.as_mut().expect("started").down(
                env,
                request,
                &mut self.above,
                &mut self.requests,
                &mut self.files,
            );
        }
        self.child_event(env.now, env.wall, child::Event::Want);
    }
    fn child_requests(&mut self, env: &Env<driver::Limits>) {
        while let Some(request) = self.child_out.pop() {
            match request {
                child::Request::Component(request) => {
                    self.trace.push(Trace::Requested(request));
                    self.component.as_mut().expect("started").down(
                        env,
                        translate::down(request),
                        &mut self.above,
                        &mut self.requests,
                        &mut self.files,
                    );
                }
                child::Request::Granted { grant: _ } => (),
                child::Request::Closed => {
                    self.closed = true;
                    self.trace.push(Trace::Closed);
                    self.file_driver.close_root(env.now);
                }
            }
        }
    }
    fn observe(&mut self, env: &Env<driver::Limits>) {
        while let Some(event) = self.above.pop() {
            let event = match translate::up(event, env.now) {
                Up::Granted { event, account, generation, value, expires } => {
                    self.table
                        .granted(
                            account,
                            generation,
                            Value { token: value, account_id: Secret::new(skein_lib::bytes::copy_of(b"")) },
                            expires,
                        )
                        .expect("configured value fits");
                    event
                }
                Up::Child(event) => event,
            };
            let heard = match event {
                child::FromComponent::Granted { account, generation, valid: _ } => {
                    Some(Heard::Granted { account, generation })
                }
                child::FromComponent::Expiring { account, generation } => Some(Heard::Expiring { account, generation }),
                child::FromComponent::Failed { account, failure } => Some(Heard::Failed { account, failure }),
                child::FromComponent::Closed => None,
            };
            if let Some(heard) = heard {
                self.heard.push(heard);
                self.trace.push(Trace::Heard(heard));
            }
            self.child_event(env.now, env.wall, child::Event::Component(event));
        }
    }
    fn private_io(&mut self, env: &Env<driver::Limits>) {
        while let Some(lower) = self.files.pop() {
            match lower {
                driver::FileLower::Request { request, deadline } => self.file_driver.request(request, deadline),
                driver::FileLower::Cancel { owner } => self.file_driver.cancel(owner),
            }
        }
        self.file_driver.progress(env.now);
        while let Some(event) = self.file_driver.take_event() {
            if event.owner() != skein_lib::Token::new(u64::MAX) {
                self.component.as_mut().expect("file event after start").filed(
                    env,
                    event,
                    &mut self.above,
                    &mut self.requests,
                    &mut self.files,
                );
            }
        }
        while let Some(submit) = self.file_driver.take_submit() {
            self.submissions.push(submit);
        }
    }
}
impl Host for Owner {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.now = now;
        let env = Env { now, wall, limits: self.limits };
        let io_env = Env { now, wall, limits: self.limits.io };
        if self.component.is_none() && self.address.is_some() {
            self.start(&env);
        }
        if self.component.is_none() {
            return;
        }
        if self.close_requested.take().is_some() && !self.closing {
            self.closing = true;
            self.child_event(now, wall, child::Event::Close);
        }
        if self.reject_requested.take().is_some() {
            self.child_event(now, wall, child::Event::Rejected { account: self.account, generation: 1 });
        }
        self.child_requests(&env);
        self.private_io(&env);
        for _ in 0..ROOM {
            if !self.io.is_ready() || self.events.room() < 3 || self.submissions.room() < 2 {
                break;
            }
            io::resume(&mut self.io, &io_env, &mut self.events, &mut self.submissions);
        }
        while let Some(complete) = self.completions.pop() {
            if files::Driver::owns(complete.op) {
                self.file_driver.up(complete);
            } else {
                io::up(&mut self.io, &io_env, complete, &mut self.events, &mut self.submissions);
            }
        }
        for _ in 0..ROOM {
            if !self.io.is_due(now) || self.events.room() < 3 || self.submissions.room() < 2 {
                break;
            }
            io::fire(&mut self.io, &io_env, &mut self.events, &mut self.submissions);
        }
        while let Some(event) = self.events.pop() {
            self.component.as_mut().expect("started").up(
                &env,
                event,
                &mut self.above,
                &mut self.requests,
                &mut self.files,
            );
        }
        let component = self.component.as_mut().expect("started");
        if component.has_work() || component.next_deadline().is_some_and(|due| due <= now) {
            component.fire(&env, &mut self.above, &mut self.requests, &mut self.files);
        }
        if self.child.is_due(now) && !self.closing {
            child::fire(&mut self.child, &Env { now, wall, limits: self.child_limits }, &mut self.child_out);
        }
        self.table.expire(now);
        self.observe(&env);
        self.child_requests(&env);
        for _ in 0..ROOM {
            if !self.io.takes() || self.submissions.room() < 2 {
                break;
            }
            let Some(request) = self.requests.pop() else { break };
            io::down(&mut self.io, &io_env, request, &mut self.submissions);
        }
        self.private_io(&env);
        self.io.reclaim();
        self.component.as_mut().expect("started").reclaim();
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.file_driver.has_work()
            || !self.files.is_empty()
            || self.io.is_ready()
            || !self.events.is_empty()
            || !self.requests.is_empty()
            || !self.above.is_empty()
            || !self.child_out.is_empty()
            || !self.completions.is_empty()
            || (self.component.is_none() && self.address.is_some())
            || (self.close_requested.is_some() && !self.closing)
            || self.reject_requested.is_some()
            || self.component.as_ref().is_some_and(|component| {
                component.has_work() || component.next_deadline().is_some_and(|due| due <= now)
            })
    }
    fn next_deadline(&self) -> Option<Time> {
        [self.io.next_deadline(), self.file_driver.next_deadline(), self.next_policy_deadline()]
            .into_iter()
            .flatten()
            .min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        [
            self.component.as_ref().and_then(driver::Component::next_deadline),
            (!self.closing).then(|| self.child.next_deadline()).flatten(),
            (!self.closing).then(|| self.table.next_expiry()).flatten(),
        ]
        .into_iter()
        .flatten()
        .min()
    }
    fn is_empty(&self) -> bool {
        self.closed
            && self.io.is_empty()
            && self.file_driver.is_empty()
            && self.events.is_empty()
            && self.requests.is_empty()
            && self.files.is_empty()
            && self.above.is_empty()
            && self.child_out.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
    }
    fn worst_case(&self) -> u64 {
        driver::worst_case(&self.limits).expect("driver bound")
            + io::worst_case(&self.limits.io).expect("io bound")
            + self.file_driver.worst_case(4)
            + child::worst_case(&self.child_limits).expect("child bound")
            + jig_protocol_accounts::table::worst_case(self.limits.accounts, self.limits.client.document.token_bytes)
                .expect("table bound")
            + 512_000
    }
    fn operations(&self) -> u32 {
        io::operations(&self.limits.io).expect("io operation bound") + 4
    }
}
