//! One process composing the real child, translation, table and driver,
//! and the secrets' store that keeps the driver's records (llm.md 5 and 8;
//! store.md 9) and the installation's identifier (store.md 4.4 item 9).
use crate::fixtures::{self, ACCOUNT_CHAT, ROOM};
use crate::world::{Installation, Story};
use jig_domain_accounts as child;
use jig_domain_core as core;
use jig_protocol_accounts::{
    handed_in, installation,
    records::{self, Records},
    table::Table,
    translate::{self, Up},
};
use jig_protocol_store as store;
use skein_io::{self as io, kernel};
use skein_lib::{Env, List, Queue, Secret, Time, Token, Wall, bytes, sha256};
use skein_oauth_accounts as driver;
use skein_world::Host;
use std::net::SocketAddr;

/// A store component over empty memory stores.
fn memory(limits: &store::Limits) -> store::Store {
    store::Store::over(
        limits,
        fixtures::APPLICATION,
        &[],
        store::Stores {
            store: skein_kv::Store::memory(&limits.store),
            secrets: skein_kv::Store::memory(&limits.secrets),
        },
    )
}

/// The store's answers, by what was asked; a replace is answered under its
/// source's number.
const OPEN: Token = Token::new(1);
const LOAD: Token = Token::new(2);
const GET: Token = Token::new(3);
const SOURCE: u64 = 1 << 32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Heard {
    Granted { account: u32, generation: u64 },
    Expiring { account: u32, generation: u64 },
    Failed { account: u32, failure: child::Failure },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trace {
    Requested(child::ToComponent),
    /// The store held this generation of the source's record at start.
    Loaded {
        account: u32,
        generation: u64,
    },
    /// The store answered that this generation of the record is durable.
    Kept {
        account: u32,
        generation: u64,
    },
    Heard(Heard),
    /// The driver asked for a sign-in's visit.
    Visit {
        account: u32,
    },
    /// The driver ended a sign-in, cancelled.
    SignInCancelled {
        account: u32,
    },
    Closed,
}
/// A value the table lent a run, by its digest.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Lent {
    pub account: u32,
    pub generation: u64,
    pub digest: [u8; 32],
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
    /// The owner keeper files nothing; the driver's calls take the queue.
    files: Queue<driver::FileLower>,
    above: Queue<driver::Event>,
    store: store::Store,
    store_limits: store::Limits,
    store_above: Queue<store::Event>,
    below: store::Below,
    records: Records,
    lent: Vec<Lent>,
    last_kept: Option<Secret>,
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
    /// The installation's identifier the store answered at open.
    installation: [u8; 16],
    /// The URL the driver asked to be visited.
    visit: Option<Box<[u8]>>,
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
    /// The owner over an open memory store holding `held` as its source's
    /// record, as an earlier run left it, and `installation.kept` as the
    /// installation's identifier if an earlier start kept one.
    pub(crate) fn new(story: Story, held: Option<&[u8]>, installation: Installation) -> Self {
        let limits = fixtures::limits();
        let child_limits = fixtures::child_limits();
        let account = if matches!(story, Story::HandedIn { .. } | Story::HandedInRejected) {
            fixtures::ACCOUNT_HANDED
        } else {
            ACCOUNT_CHAT
        };
        let store_limits = fixtures::store_limits();
        let mut owner = Self {
            io: io::Io::new(&limits.io),
            component: None,
            child: child::Domain::new(
                &child_limits,
                child::Providers::none(),
                Box::new([child::Configured { number: account, provider: None }]),
            ),
            child_out: Queue::with_capacity(ROOM),
            child_limits,
            table: Table::new(limits.accounts, limits.client.document.token_bytes),
            events: Queue::with_capacity(ROOM),
            requests: Queue::with_capacity(ROOM),
            files: Queue::with_capacity(ROOM),
            above: Queue::with_capacity(ROOM),
            completions: Queue::with_capacity(ROOM),
            submissions: Queue::with_capacity(ROOM),
            store: memory(&store_limits),
            store_limits,
            store_above: Queue::with_capacity(store::MAX_OUT_DOWN),
            below: store::Below {
                store: Queue::with_capacity(skein_kv::MAX_OUT.below),
                secrets: Queue::with_capacity(skein_kv::MAX_OUT.below),
                transcripts: Queue::with_capacity(skein_kv::MAX_OUT.below),
            },
            records: Records::new(limits.accounts),
            lent: Vec::with_capacity(8),
            last_kept: None,
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
            installation: [0; 16],
            visit: None,
        };
        if let Some(kept) = installation.kept {
            // An earlier start kept its draw with its first commit; this
            // start opens the same stores again.
            assert_eq!(owner.open(kept), kept, "an empty store answers this start's draw");
            let stores = std::mem::replace(&mut owner.store, memory(&owner.store_limits)).into_stores();
            owner.store = store::Store::over(&owner.store_limits, fixtures::APPLICATION, &[], stores);
        }
        owner.installation = owner.open(installation.drawn);
        if let Some(held) = held {
            let value = Secret::new(bytes::copy_of(held));
            owner.secrets(Time::ZERO, Wall::EPOCH, source_token(account), replace(account, value, None));
            let kept = owner.store_above.pop();
            assert!(matches!(kept, Some(store::Event::Kept { .. })), "the store keeps the record held before the run");
        }
        owner
    }
    /// Opens the store, as this start drew `drawn`, and answers the
    /// installation's identifier it keeps. At an empty store the owner, as
    /// the core's stand-in, commits the deployment's header first, which
    /// keeps the identifier.
    fn open(&mut self, drawn: [u8; 16]) -> [u8; 16] {
        // Memory stores open no file: the root is never read.
        let root = store::Root { token: OPEN, fd: kernel::Fd::new(3) };
        self.ask(Time::ZERO, Wall::EPOCH, store::Request::Open { owner: OPEN, root, installation: drawn });
        let Some(store::Event::Opened { owner: OPEN, last, installation }) = self.store_above.pop() else {
            panic!("the memory store opens at once");
        };
        if last == 0 {
            let header = core::Record::Core(core::CoreRecord::Deployment(core::Deployment {
                id: fixtures::DEPLOYMENT,
                tasks: 0,
                people: 0,
                sign_ins: 0,
                messages: 0,
                runs: 0,
                calls: 0,
                connector_rows: 0,
                applications: 0,
                commits: 1,
            }));
            self.ask(
                Time::ZERO,
                Wall::EPOCH,
                store::Request::Commit { number: 1, writes: Box::new([store::Write::Save(header)]) },
            );
            assert!(
                matches!(self.store_above.pop(), Some(store::Event::Committed { number: 1, .. })),
                "the memory store commits at once"
            );
        }
        installation
    }
    /// The installation's identifier the store answered at this start.
    #[must_use]
    pub fn installation(&self) -> [u8; 16] {
        self.installation
    }
    /// The URL the driver asked to be visited for a sign-in.
    #[must_use]
    pub fn visit(&self) -> Option<&[u8]> {
        self.visit.as_deref()
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
    /// Each value the table lent, in order.
    #[must_use]
    pub fn lent(&self) -> &[Lent] {
        &self.lent
    }
    /// The record the store last answered as kept, read back from it.
    #[must_use]
    pub fn last_kept(&self) -> Option<&[u8]> {
        self.last_kept.as_ref().map(Secret::expose)
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
    /// The driver starts from the store's records (llm.md 5.1); a handed-in
    /// record is kept before it is handed in, unless the store holds it.
    fn start(&mut self, env: &Env<driver::Limits>) {
        let handed = fixtures::handed_lapse(self.story);
        let mut configured = List::with_capacity(1);
        assert!(
            configured
                .push(if handed.is_some() {
                    driver::Account::HandedIn
                } else {
                    driver::Account::SignIn {
                        shared: false,
                        origins: List::with_capacity(0),
                        registration: fixtures::registration(installation::terms(&self.installation)),
                        endpoint: driver::Endpoint {
                            address: self.address.expect("issuer address"),
                            transport: driver::Transport::Plaintext,
                        },
                        keeper: driver::Keeper::Owner { kept: None },
                    }
                })
                .is_ok()
        );
        self.load(env, &mut configured);
        let crypto = skein_lib::CryptoRng::new(&skein_world::crypto_seed(17), 0).expect("world crypto seed");
        self.component = Some(driver::Component::new(configured, &self.limits, crypto).expect("configured driver"));
        if let Some(lapses_in) = handed {
            // A handed-in file states when its token lapses, the same at
            // every start.
            let record = handed_in::record(
                self.account,
                Secret::new(bytes::copy_of(fixtures::TOKEN_FIRST)),
                Wall::from_nanos(Wall::EPOCH.as_nanos().saturating_add(lapses_in.as_nanos())),
                None,
                &self.limits.client.document,
            )
            .expect("handed-in fixture");
            match self.records.hand_in(self.account, record, &self.limits.client.document).expect("handed-in fixture") {
                records::Next::Driver(request) => self.driver(env, request),
                records::Next::Store(keep) => self.replace(env, keep),
            }
        }
        if self.story == Story::SignIn {
            // The owner asks the driver itself: the child wants no source
            // in this story.
            self.driver(env, driver::Request::SignIn { account: self.account, subject: None });
        } else {
            self.child_event(env.now, env.wall, child::Event::Want);
        }
    }
    /// Reads the store's records, page by page, into the configured sources.
    fn load(&mut self, env: &Env<driver::Limits>, configured: &mut List<driver::Account>) {
        let mut after = None;
        loop {
            let load = store::Secrets::Load { owner: store::SecretOwner::Source, after };
            self.secrets(env.now, env.wall, LOAD, load);
            let Some(store::Event::SecretsLoaded { owner: LOAD, rows, next }) = self.store_above.pop() else {
                panic!("a memory store answers a load at once");
            };
            for row in rows {
                let account = u32::try_from(row.number).expect("a source's number");
                let Some(record) = self.records.load(account, row.value.expose(), &self.limits.client.document) else {
                    continue;
                };
                self.trace.push(Trace::Loaded { account, generation: record.generation });
                if let Some(driver::Account::SignIn { keeper, .. }) = configured.get_mut(account) {
                    *keeper = driver::Keeper::Owner { kept: Some(record) };
                }
            }
            match next {
                Some(next) => after = Some(next),
                None => return,
            }
        }
    }
    fn driver(&mut self, env: &Env<driver::Limits>, request: driver::Request) {
        self.component.as_mut().expect("started").down(
            env,
            request,
            &mut self.above,
            &mut self.requests,
            &mut self.files,
        );
    }
    fn ask(&mut self, now: Time, wall: Wall, request: store::Request) {
        let env = Env { now, wall, limits: self.store_limits };
        store::down(&mut self.store, &env, request, &mut self.store_above, &mut self.below);
        assert!(
            self.below.store.is_empty() && self.below.secrets.is_empty() && self.below.transcripts.is_empty(),
            "a memory store files nothing"
        );
    }
    fn secrets(&mut self, now: Time, wall: Wall, owner: Token, request: store::Secrets) {
        self.ask(now, wall, store::Request::Secrets { owner, request });
    }
    /// Asks the store to keep a record in place of the one it holds, and
    /// passes its answer on.
    fn replace(&mut self, env: &Env<driver::Limits>, keep: records::Keep) {
        let records::Keep { account, value, replaces } = keep;
        self.secrets(env.now, env.wall, source_token(account), replace(account, value, replaces));
        self.answers(env);
    }
    /// The store's answers to replaces (store.md 9): the driver hears `Kept`,
    /// or a handed-in record is handed in, only once the record is durable.
    fn answers(&mut self, env: &Env<driver::Limits>) {
        while let Some(event) = self.store_above.pop() {
            let request = match event {
                store::Event::Kept { owner } => {
                    let account = source(owner);
                    let Some(request) = self.records.kept(account) else { continue };
                    self.trace.push(Trace::Kept { account, generation: generation(&request) });
                    let get = store::Secrets::Get { owner: store::SecretOwner::Source, number: u64::from(account) };
                    self.secrets(env.now, env.wall, GET, get);
                    let Some(store::Event::Secret { owner: GET, value: Some(value) }) = self.store_above.pop() else {
                        panic!("a kept record reads back");
                    };
                    self.last_kept = Some(value);
                    request
                }
                store::Event::Stale { owner } => {
                    let Some(request) = self.records.stale(source(owner)) else { continue };
                    request
                }
                store::Event::Opened { .. }
                | store::Event::OpenRefused { .. }
                | store::Event::Committed { .. }
                | store::Event::Failed { .. }
                | store::Event::Restored { .. }
                | store::Event::Loaded { .. }
                | store::Event::Read { .. }
                | store::Event::Got { .. }
                | store::Event::Turns { .. }
                | store::Event::Unread { .. }
                | store::Event::Erased { .. }
                | store::Event::Carried { .. }
                | store::Event::Secret { .. }
                | store::Event::SecretsLoaded { .. }
                | store::Event::Corrupt { .. } => unreachable!("a replace is answered kept or stale"),
            };
            self.driver(env, request);
        }
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
                // No party signs in or out here: JP6 carries those to the component.
                child::Request::Granted { grant: _ }
                | child::Request::Save { .. }
                | child::Request::Erase { .. }
                | child::Request::Signing(_)
                | child::Request::Answer { .. } => (),
                child::Request::Closed => {
                    self.closed = true;
                    self.trace.push(Trace::Closed);
                }
            }
        }
    }
    fn observe(&mut self, env: &Env<driver::Limits>) {
        while let Some(event) = self.above.pop() {
            // A sign-in's events stop here: the child asks for none.
            let event = match event {
                driver::Event::Visit { account, url } => {
                    self.trace.push(Trace::Visit { account });
                    self.visit = Some(url);
                    continue;
                }
                driver::Event::Failed {
                    account,
                    ends: driver::Ends::SignIn,
                    failure: driver::Failure::Exchange(skein_oauth::Failure::Cancelled),
                } => {
                    self.trace.push(Trace::SignInCancelled { account });
                    continue;
                }
                driver::Event::Failed { ends: driver::Ends::SignIn, .. }
                | driver::Event::Refused { asked: driver::Asked::SignIn, .. }
                | driver::Event::SignedIn { .. } => panic!("a sign-in ends only cancelled here"),
                other @ (driver::Event::Removed { .. }
                | driver::Event::SignedOut { .. }
                | driver::Event::Granted { .. }
                | driver::Event::Expiring { .. }
                | driver::Event::Keep { .. }
                | driver::Event::Failed { .. }
                | driver::Event::Refused { .. }
                | driver::Event::Lock { .. }
                | driver::Event::Unlock { .. }
                | driver::Event::Closed) => other,
            };
            let event = match self.records.up(event, env.now) {
                Up::Granted { event, account, generation, value, expires } => {
                    self.lent.push(Lent { account, generation, digest: sha256::digest(value.token.expose()) });
                    self.table.granted(account, generation, value, expires).expect("configured value fits");
                    event
                }
                Up::Child(event) => event,
                Up::Keep { account, record } => {
                    match self.records.keep(account, record.0, &self.limits.client.document) {
                        records::Next::Driver(request) => self.driver(env, request),
                        records::Next::Store(keep) => self.replace(env, keep),
                    }
                    continue;
                }
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
}
fn source_token(account: u32) -> Token {
    Token::new(SOURCE | u64::from(account))
}
fn source(owner: Token) -> u32 {
    u32::try_from(owner.raw() - SOURCE).expect("a source's number")
}
fn replace(account: u32, value: Secret, replaces: Option<[u8; 32]>) -> store::Secrets {
    store::Secrets::Replace { owner: store::SecretOwner::Source, number: u64::from(account), value, replaces }
}
/// The generation the store's answer makes durable.
fn generation(request: &driver::Request) -> u64 {
    if let driver::Request::Kept { generation, .. } = request {
        *generation
    } else if let driver::Request::HandIn { record, .. } = request {
        record.generation
    } else {
        unreachable!("the store answers a candidate or a handed-in record")
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
            if self.story == Story::SignIn {
                self.driver(&env, driver::Request::Cancel { account: self.account });
            }
            self.child_event(now, wall, child::Event::Close);
        }
        if self.reject_requested.take().is_some() {
            self.child_event(now, wall, child::Event::Unauthorized { account: self.account, generation: 1 });
        }
        self.child_requests(&env);
        for _ in 0..ROOM {
            if !self.io.is_ready() || self.events.room() < 3 || self.submissions.room() < 2 {
                break;
            }
            io::resume(&mut self.io, &io_env, &mut self.events, &mut self.submissions);
        }
        while let Some(complete) = self.completions.pop() {
            io::up(&mut self.io, &io_env, complete, &mut self.events, &mut self.submissions);
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
        assert!(self.files.is_empty(), "the owner keeper files nothing");
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
        self.io.is_ready()
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
        [self.io.next_deadline(), self.next_policy_deadline()].into_iter().flatten().min()
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
            && !self.records.keeping()
            && self.store_above.is_empty()
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
            + store::worst_case(&self.store_limits).expect("store bound")
            + Queue::<store::Event>::worst_case(store::MAX_OUT_DOWN).expect("store answers bound")
            + 3 * Queue::<skein_io::file::Request>::worst_case(skein_kv::MAX_OUT.below).expect("store files bound")
            + records::worst_case(self.limits.accounts, &self.limits.client.document).expect("records bound")
            + child::worst_case(&self.child_limits).expect("child bound")
            + jig_protocol_accounts::table::worst_case(self.limits.accounts, self.limits.client.document.token_bytes)
                .expect("table bound")
            + 512_000
    }
    fn operations(&self) -> u32 {
        io::operations(&self.limits.io).expect("io operation bound")
    }
}
