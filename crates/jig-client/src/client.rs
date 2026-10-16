//! Exhaustive routing between the domain and link (client/domain/client.md, 3).
use crate::{
    Ask, Document, DomainLimits, Handle, Key, Limits, Link, Message, Pending, Readable, Record, Streamed, Told,
    Unusable, Watchable, With,
};
use alloc::boxed::Box;
use jig_client_domain::{self as domain, Down, Up};
use jig_client_protocol_link as link;
use skein_lib::{Env, List, Queue, Secret, Time, Token, Wall};
#[derive(Debug)]
pub enum Event {
    Start { kept: List<Record> },
    SignIn { with: With },
    SignOut,
    Watch { handle: Handle, subject: Watchable },
    Unwatch { handle: Handle },
    Read { handle: Handle, read: Readable },
    Send { key: Key, request: Ask },
    Reply { exchange: Token, message: Box<[u8]> },
    Lost { exchange: Token },
    Streamed { message: Streamed },
    StreamGone,
    Stored,
    NotStored,
}
#[derive(Debug)]
pub enum Request {
    Exchange { exchange: Token, credential: Option<Secret>, message: Message },
    OpenStream { credential: Option<Secret> },
    CloseStream,
    Save { records: List<Record> },
    Erase { records: List<Box<[u8]>> },
    Told { told: Told },
}
#[derive(Debug)]
pub struct LinkView<'a> {
    pub state: Link,
    pub party: Option<u64>,
    pub not_served: &'a [u16],
    pub sign_out_lost: bool,
    pub credential: Option<&'a Secret>,
}
#[derive(Debug)]
pub struct Client {
    domain: domain::Domain,
    link: link::Link,
    limits: Limits,
    domain_out: Queue<domain::Request>,
    link_out: Queue<link::Request>,
    store: crate::store::Store,
    saves: List<Record>,
    erases: List<Box<[u8]>>,
}
impl Client {
    /// # Errors
    /// Refuses limits whose queues or receiving bounds cannot be represented.
    pub fn new(limits: &Limits, seed: u64) -> Result<Self, Unusable> {
        crate::worst_case(limits).ok_or(Unusable::Limits)?;
        let Ok(bounds) = link::Limits::new(limits.domain, limits.exchanges) else {
            return Err(Unusable::Limits);
        };
        let Ok(link) = link::Link::new(&bounds) else {
            return Err(Unusable::Limits);
        };
        let bound = crate::max_out(limits);
        Ok(Self {
            domain: domain::Domain::new(&limits.domain, seed),
            link,
            limits: *limits,
            domain_out: Queue::with_capacity(bound),
            link_out: Queue::with_capacity(bound),
            store: crate::store::Store::new(limits.domain.requests.requests),
            saves: List::with_capacity(bound),
            erases: List::with_capacity(bound),
        })
    }
    pub fn step(&mut self, now: Time, wall: Wall, event: Event, out: &mut Queue<Request>) {
        let env = Env { now, wall, limits: self.limits.domain };
        match event {
            Event::Start { kept } => {
                domain::step(
                    &mut self.domain,
                    &env,
                    domain::Event::Start { kept: List::with_capacity(0) },
                    &mut self.domain_out,
                );
                let mut versions = List::with_capacity(1);
                versions
                    .push(link::StoreVersion { keeper: link::Keeper::Table, version: domain::TABLE_VERSION })
                    .expect("one keeper");
                self.link.step(now, link::Event::Start { kept, versions }, &mut self.link_out);
            }
            Event::Watch { handle, subject } => self.domain_step(&env, domain::Event::Watch { handle, subject }),
            Event::Unwatch { handle } => self.domain_step(&env, domain::Event::Unwatch { handle }),
            Event::Read { handle, read } => self.domain_step(&env, domain::Event::Read { handle, read }),
            Event::Send { key, request } => self.domain_step(&env, domain::Event::Send { key, request }),
            Event::SignIn { with } => {
                let held = match &with {
                    With::Held(_) => true,
                    With::Link(_) => false,
                };
                self.link.step(now, link::Event::SignIn { with }, &mut self.link_out);
                if held {
                    self.up(&env, Up::SignedIn);
                }
            }
            Event::SignOut => self.link.step(now, link::Event::SignOut, &mut self.link_out),
            Event::Reply { exchange, message } => {
                self.link.step(now, link::Event::Reply { exchange, message }, &mut self.link_out);
            }
            Event::Lost { exchange } => self.link.step(now, link::Event::Lost { exchange }, &mut self.link_out),
            Event::Streamed { message } => self.link.step(now, link::Event::Streamed { message }, &mut self.link_out),
            Event::StreamGone => self.link.step(now, link::Event::StreamGone, &mut self.link_out),
            Event::Stored => self.store_answer(&env, true),
            Event::NotStored => self.store_answer(&env, false),
        }
        self.drain(&env, out);
    }
    pub fn fire(&mut self, now: Time, wall: Wall, out: &mut Queue<Request>) {
        let env = Env { now, wall, limits: self.limits.domain };
        self.link.fire(now, &mut self.link_out);
        self.drain(&env, out);
        if self.domain.link() != Link::OutOfDate {
            domain::fire(&mut self.domain, &env, &mut self.domain_out);
        }
        self.drain(&env, out);
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if self.domain.link() == Link::OutOfDate {
            return None;
        }
        domain::next_deadline(&self.domain).into_iter().chain(self.link.next_deadline()).min()
    }
    pub fn reclaim(&mut self) {
        domain::reclaim(&mut self.domain);
        self.link.reclaim();
    }
    #[must_use]
    pub fn document(&self, handle: Handle) -> Option<Document<'_>> {
        self.domain.document(handle)
    }
    #[must_use]
    pub fn text(&self, handle: Handle) -> Option<&domain::contract::RunText> {
        self.domain.text(handle)
    }
    #[must_use]
    pub fn pending(&self, key: Key) -> Option<Pending> {
        self.domain.pending(key)
    }
    #[must_use]
    pub fn link(&self) -> LinkView<'_> {
        LinkView {
            state: self.domain.link(),
            party: self.domain.party(),
            not_served: &[],
            sign_out_lost: false,
            credential: self.link.credential(),
        }
    }
    fn store_answer(&mut self, env: &Env<DomainLimits>, stored: bool) {
        if let Some(write) = self.store.answer() {
            while let Some(key) = self.store.take(write) {
                if !stored {
                    self.up(env, Up::NotKept { key });
                }
            }
        }
        if !stored {
            self.link.step(env.now, link::Event::NotStored, &mut self.link_out);
        } else if !self.store.pending() {
            self.link.step(env.now, link::Event::Stored, &mut self.link_out);
        }
        if self.store.pending() {
            self.link.wait_for_store();
        }
    }
    fn domain_step(&mut self, env: &Env<DomainLimits>, event: domain::Event) {
        domain::step(&mut self.domain, env, event, &mut self.domain_out);
    }
    fn up(&mut self, env: &Env<DomainLimits>, up: Up) {
        self.domain_step(env, domain::Event::Up { up });
    }
    fn drain(&mut self, env: &Env<DomainLimits>, out: &mut Queue<Request>) {
        for _ in 0..crate::max_out(&self.limits) {
            if let Some(request) = self.link_out.pop() {
                self.route_link(env, request, out);
            } else if let Some(request) = self.domain_out.pop() {
                match request {
                    domain::Request::Told { told } => out.push(Request::Told { told }),
                    domain::Request::Down { down } => {
                        if self.domain.link() != Link::OutOfDate {
                            match &down {
                                Down::Save { record } => self.store.save(record.key),
                                Down::Erase { key } => self.store.erase(*key),
                                Down::Send { .. }
                                | Down::Open { .. }
                                | Down::Continue { .. }
                                | Down::Close { .. }
                                | Down::Read { .. } => {}
                            }
                            self.link.step(env.now, down_event(down), &mut self.link_out);
                        }
                    }
                }
            } else {
                break;
            }
        }
        assert!(self.link_out.is_empty() && self.domain_out.is_empty(), "bounded composition settles");
        if !self.saves.is_empty() {
            self.store.write();
            out.push(Request::Save {
                records: core::mem::replace(&mut self.saves, List::with_capacity(crate::max_out(&self.limits))),
            });
        }
        if !self.erases.is_empty() {
            self.store.write();
            out.push(Request::Erase {
                records: core::mem::replace(&mut self.erases, List::with_capacity(crate::max_out(&self.limits))),
            });
        }
    }
    fn route_link(&mut self, env: &Env<DomainLimits>, request: link::Request, out: &mut Queue<Request>) {
        match request {
            link::Request::Exchange { exchange, credential, message } => {
                if self.domain.link() != Link::OutOfDate {
                    out.push(Request::Exchange { exchange, credential, message });
                }
            }
            link::Request::OpenStream { credential } => {
                if self.domain.link() != Link::OutOfDate {
                    out.push(Request::OpenStream { credential });
                }
            }
            link::Request::CloseStream => out.push(Request::CloseStream),
            link::Request::Save { record } => self.saves.push(record).expect("bounded step saves"),
            link::Request::Erase { key } => self.erases.push(key).expect("bounded step erases"),
            link::Request::Answer { attempt, envelope, answer, why } => {
                self.up(env, Up::Answer { attempt, envelope, answer, why });
            }
            link::Request::NotKept { attempt } => {
                self.up(
                    env,
                    Up::Answer { attempt, envelope: skein_lib::RequestEnvelope::Lost, answer: None, why: None },
                );
            }
            link::Request::Watch { watch, event } => self.up(env, Up::Watched { watch, event }),
            link::Request::FamilyDelivery { watch, record } => self.up(env, Up::FamilyDelivered { watch, record }),
            link::Request::Ended { watch, end } => self.up(env, Up::WatchEnded { watch, end }),
            link::Request::Read { read, result } => self.up(env, Up::ReadDone { read, result }),
            link::Request::Hello { family: None, versions } => {
                self.up(env, Up::Served { served: link::serves(versions) });
            }
            link::Request::Signing { signing: link::Signing::SignedIn { .. } } => self.up(env, Up::SignedIn),
            link::Request::Kept { record: link::Saved::Table { record, .. } } => {
                let mut kept = List::with_capacity(1);
                kept.push(record).expect("one restored record");
                self.domain.restore_record(env, kept, &mut self.domain_out);
            }
            link::Request::Dropped { .. } => out.push(Request::Told { told: Told::Dropped { count: 1 } }),
            link::Request::Hello { family: Some(_), .. }
            | link::Request::Fact { .. }
            | link::Request::ServiceCredential { .. }
            | link::Request::Kept { record: link::Saved::Jig(_) | link::Saved::Family { .. } }
            | link::Request::Signing {
                signing: link::Signing::Again | link::Signing::Refused | link::Signing::Lost | link::Signing::SignedOut,
            } => {}
        }
    }
}
fn down_event(down: Down) -> link::Event {
    match down {
        Down::Save { record } => {
            link::Event::Save { record: link::Saved::Table { version: domain::TABLE_VERSION, record } }
        }
        Down::Erase { key } => link::Event::Erase { key: link::Erased::Table(key) },
        Down::Send { attempt, key, request } => link::Event::Send { attempt, key, ask: request },
        Down::Open { watch, subject } => link::Event::Open { watch, subject },
        Down::Continue { watch, subject } => link::Event::Continue { watch, subject },
        Down::Close { watch } => link::Event::Close { watch },
        Down::Read { read, what } => link::Event::Read { read, what },
    }
}
