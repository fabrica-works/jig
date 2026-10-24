//! The component behind the domain worlds' durable store face.
use crate::Files;
use jig_protocol_store::{self as protocol, Below, Event, Io, Limits, Request, Stores};
use jig_test_domain as root;
use jig_test_store::{StoreOut, Up, route};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use std::collections::{BTreeMap, VecDeque};
use std::fmt;

/// The installation's identifier these worlds draw: an empty store keeps it,
/// and a reopened one answers the one it kept (configuration.md, 8).
const INSTALLATION: [u8; 16] = [0; 16];

/// Faults between the root and the component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Hold { commits: u32 },
    Fail { commit: u64 },
    SlowPages { by: u32 },
}
/// A commit held before it reaches the component.
#[derive(Clone, Debug)]
pub struct Pending {
    pub number: u64,
    pub writes: Box<[root::Write]>,
    pub delay: u32,
}
struct Delayed {
    event: Event,
    delay: u32,
}
/// The real component, with a typed mirror of its durable writes.
pub struct Store {
    pub applied: u64,
    pub rows: BTreeMap<root::Key, root::Record>,
    pub pending: VecDeque<Pending>,
    pub held: VecDeque<u64>,
    component: Option<protocol::Store>,
    files: Files,
    routes: route::Routes,
    limits: root::Limits,
    component_limits: Limits,
    /// The configured models the component names a role's models by.
    models: Box<[jig_domain_core::Model]>,
    inflight: BTreeMap<u64, Box<[root::Write]>>,
    durable: Vec<(u64, Box<[root::Write]>)>,
    replies: Vec<Up>,
    delayed: VecDeque<Delayed>,
    hold_next: u32,
    holding: bool,
    fail_at: Option<u64>,
    page_delay: u32,
    pages_paused: bool,
    failed: bool,
}
impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store")
            .field("applied", &self.applied)
            .field("rows", &self.rows)
            .field("pending", &self.pending)
            .field("held", &self.held)
            .finish_non_exhaustive()
    }
}
impl Store {
    /// The component under a configuration's `models`, in its order.
    #[must_use]
    pub fn new(limits: &root::Limits, models: &[jig_domain_core::Model]) -> Store {
        let component_limits = component_limits(limits);
        let stores = Stores {
            store: skein_kv::Store::memory(&component_limits.store),
            secrets: skein_kv::Store::memory(&component_limits.secrets),
        };
        let component =
            protocol::Store::over(&component_limits, jig_test_store::application(&limits.connector), models, stores);
        let mut routes = route::Routes::new(limits.routes);
        routes.configure(limits);
        let mut store = Store {
            applied: 0,
            rows: BTreeMap::new(),
            pending: VecDeque::new(),
            held: VecDeque::new(),
            component: Some(component),
            files: Files::new(),
            routes,
            limits: *limits,
            component_limits,
            models: models.into(),
            inflight: BTreeMap::new(),
            durable: Vec::new(),
            replies: Vec::new(),
            delayed: VecDeque::new(),
            hold_next: 0,
            holding: false,
            fail_at: None,
            page_delay: 0,
            pages_paused: false,
            failed: false,
        };
        store.open();
        store
    }
    /// Make a real transcript read unread without altering its typed mirror.
    pub fn unread_transcript(&mut self, task: u64) {
        self.files.unread_transcript(task);
    }
    pub fn fault(&mut self, fault: Fault) {
        match fault {
            Fault::Hold { commits } => self.hold_next = commits,
            Fault::Fail { commit } => self.fail_at = Some(commit),
            Fault::SlowPages { by } => self.page_delay = by,
        }
    }
    pub fn submit(&mut self, number: u64, writes: Box<[root::Write]>, delay: u32) {
        assert!(!self.failed, "a failed world must restart before writing");
        let previous = self.pending.back().map_or(self.applied, |pending| pending.number);
        assert_eq!(number, previous.checked_add(1).expect("bounded commit number"));
        self.pending.push_back(Pending { number, writes, delay });
    }
    /// Release one commit, or hold its durable acknowledgement as the fault asks.
    ///
    /// # Errors
    /// Returns the selected or component-refused commit's number.
    pub fn tick(&mut self, hold: bool) -> Result<Option<u64>, u64> {
        self.tick_pages();
        if self.failed {
            return Ok(None);
        }
        let Some(front) = self.pending.front_mut() else {
            return Ok(None);
        };
        if front.delay != 0 {
            front.delay -= 1;
            return Ok(None);
        }
        let pending = self.pending.pop_front().expect("checked front");
        if self.fail_at == Some(pending.number) {
            self.failed = true;
            self.pending.clear();
            return Err(pending.number);
        }
        self.holding = hold || self.hold_next != 0 || !self.held.is_empty();
        self.hold_next = self.hold_next.saturating_sub(1);
        self.inflight.insert(pending.number, pending.writes.clone());
        self.request(StoreOut::Commit { number: pending.number, writes: pending.writes });
        if self.failed {
            return Err(pending.number);
        }
        assert_eq!(self.applied, pending.number, "the synchronous stand-in settles the commit");
        if self.holding { Ok(None) } else { Ok(Some(pending.number)) }
    }
    pub fn release_held(&mut self) -> Option<u64> {
        if self.failed { None } else { self.held.pop_front() }
    }
    /// Crash and reopen under a configuration whose models may differ from the last.
    pub fn reconfigure(&mut self, seed: u64, models: &[jig_domain_core::Model]) {
        self.models = models.into();
        self.crash(seed);
    }
    pub fn crash(&mut self, seed: u64) {
        let stores = self.component.take().expect("component exists").into_stores();
        self.files.crash(seed);
        self.component = Some(protocol::Store::over(
            &self.component_limits,
            jig_test_store::application(&self.limits.connector),
            &self.models,
            stores,
        ));
        self.routes = route::Routes::new(self.limits.routes);
        self.routes.configure(&self.limits);
        self.pending.clear();
        self.held.clear();
        self.inflight.clear();
        self.replies.clear();
        self.delayed.clear();
        self.holding = false;
        self.failed = false;
        self.hold_next = 0;
        self.fail_at = None;
        self.open();
    }
    /// Replay durable commits into fresh stores and a fresh filesystem.
    #[must_use]
    pub fn fork(&self) -> Store {
        let mut fresh = Store::new(&self.limits, &self.models);
        for (number, writes) in &self.durable {
            fresh.submit(*number, writes.clone(), 0);
            assert_eq!(fresh.tick(false), Ok(Some(*number)));
        }
        fresh.page_delay = self.page_delay;
        fresh
    }
    /// Save fixture rows in one numbered transaction with the deployment header.
    pub fn seed(&mut self, rows: Box<[root::Record]>) {
        assert!(self.pending.is_empty(), "seed only between root commits");
        let key = root::Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment));
        let root::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(mut header))) =
            self.rows.get(&key).expect("fixture seed follows the first root commit").clone()
        else {
            panic!("the header has its deployment key");
        };
        let number = self.applied.checked_add(1).expect("bounded fixture commit");
        header.commits = number;
        let mut writes: Vec<_> = rows.into_vec().into_iter().map(root::Write::Save).collect();
        writes.push(root::Write::Save(root::Record::Core(jig_domain_core::Record::Core(
            jig_domain_core::CoreRecord::Deployment(header),
        ))));
        self.submit(number, writes.into_boxed_slice(), 0);
        assert_eq!(self.tick(false), Ok(Some(number)), "seed reaches the real component");
    }
    pub fn ask(&mut self, out: StoreOut) {
        match out {
            StoreOut::Commit { number, writes } => self.submit(number, writes, 0),
            out @ (StoreOut::Restart { .. }
            | StoreOut::ReadLoad { .. }
            | StoreOut::NotesLoad { .. }
            | StoreOut::Now(_)) => self.request(out),
        }
    }
    fn request(&mut self, out: StoreOut) {
        let request = route::down(&mut self.routes, out).expect("root has room for its store request");
        self.drive(VecDeque::from([request]), Queue::with_capacity(protocol::MAX_OUT_DOWN));
    }
    pub fn answers(&mut self) -> Vec<Up> {
        std::mem::take(&mut self.replies)
    }
    #[must_use]
    pub fn loading(&self) -> bool {
        !self.delayed.is_empty() || !self.routes.idle()
    }
    fn open(&mut self) {
        let root = self.files.root();
        self.drive(
            VecDeque::from([Request::Open { owner: Token::new(0), root, installation: INSTALLATION }]),
            Queue::with_capacity(protocol::MAX_OUT_DOWN),
        );
        assert!(!self.failed, "durable stores reopen after their files are cut");
    }
    fn env(&self) -> Env<Limits> {
        Env { now: Time::ZERO, wall: Wall::EPOCH, limits: self.component_limits }
    }
    fn drive(&mut self, mut requests: VecDeque<Request>, mut above: Queue<Event>) {
        let mut below = Below {
            store: Queue::with_capacity(skein_kv::MAX_OUT.below),
            secrets: Queue::with_capacity(skein_kv::MAX_OUT.below),
            transcripts: Queue::with_capacity(skein_kv::MAX_OUT.below),
        };
        loop {
            if let Some(event) = above.pop() {
                self.event(event, &mut requests);
                continue;
            }
            if let Some(request) = requests.pop_front() {
                let env = self.env();
                protocol::down(
                    self.component.as_mut().expect("component exists"),
                    &env,
                    request,
                    &mut above,
                    &mut below,
                );
                continue;
            }
            let io = if let Some(request) = below.transcripts.pop() {
                Io::Transcripts(self.files.answer(request))
            } else if let Some(request) = below.store.pop() {
                Io::Store(self.files.answer(request))
            } else if let Some(request) = below.secrets.pop() {
                Io::Secrets(self.files.answer(request))
            } else {
                break;
            };
            let env = self.env();
            protocol::up(self.component.as_mut().expect("component exists"), &env, io, &mut above, &mut below);
        }
    }
    fn event(&mut self, event: Event, requests: &mut VecDeque<Request>) {
        match event {
            Event::Opened { last, .. } => {
                self.applied = last;
            }
            Event::Committed { number, .. } => {
                let writes = self.inflight.remove(&number).expect("commit belongs to the root");
                for write in &writes {
                    match write {
                        root::Write::CopySettingsRevision(number) => {
                            use jig_domain_core as core;
                            let key = root::Key::Core(core::Key::Core(core::CoreKey::Settings(
                                core::settings::Key::Held(*number),
                            )));
                            let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(
                                core::settings::Record::Revision { revision, .. },
                            ))) = self.rows.get(&key).expect("held revision").clone()
                            else {
                                panic!("revision row")
                            };
                            let row = root::Record::Core(core::Record::Core(core::CoreRecord::Settings(
                                core::settings::Record::Revision { history: true, revision },
                            )));
                            self.rows.insert(row.key(), row);
                        }
                        root::Write::Save(row) => {
                            self.rows.insert(row.key(), row.clone());
                        }
                        root::Write::Erase(key) => {
                            self.rows.remove(key);
                        }
                    }
                }
                self.applied = number;
                self.durable.push((number, writes));
                if self.holding {
                    self.held.push_back(number);
                }
            }
            Event::Failed { .. } | Event::OpenRefused { .. } => self.failed = true,
            event @ (Event::Restored { .. } | Event::Loaded { .. } | Event::Read { .. })
                if self.page_delay != 0 || self.pages_paused =>
            {
                self.delayed.push_back(Delayed { event, delay: self.page_delay });
            }
            event @ Event::Corrupt { .. } => {
                self.failed = true;
                self.route(event, requests);
            }
            event @ (Event::Restored { .. }
            | Event::Loaded { .. }
            | Event::Read { .. }
            | Event::Got { .. }
            | Event::Turns { .. }
            | Event::Unread { .. }
            | Event::Kept { .. }
            | Event::Carried { .. }
            | Event::Stale { .. }
            | Event::Erased { .. }
            | Event::Secret { .. }
            | Event::SecretsLoaded { .. }) => self.route(event, requests),
        }
    }
    fn route(&mut self, event: Event, requests: &mut VecDeque<Request>) {
        let mut outputs = Queue::with_capacity(8);
        route::up(&mut self.routes, event, &mut outputs);
        while let Some(output) = outputs.pop() {
            match output {
                Up::Ask(request) => requests.push_back(request),
                output @ (Up::Restore(_) | Up::Root(_)) => self.replies.push(output),
            }
        }
    }
    /// Suspend captured pages without losing their original store flights.
    pub fn pause_pages(&mut self, paused: bool) {
        self.pages_paused = paused;
    }
    fn tick_pages(&mut self) {
        if self.pages_paused {
            return;
        }
        let mut ready = Queue::with_capacity(u32::try_from(self.delayed.len()).expect("bounded flight table"));
        let count = self.delayed.len();
        for _ in 0..count {
            let mut delayed = self.delayed.pop_front().expect("counted pages");
            delayed.delay = delayed.delay.saturating_sub(1);
            if delayed.delay == 0 {
                ready.push(delayed.event);
            } else {
                self.delayed.push_back(delayed);
            }
        }
        // Released pages route directly; continuations alone receive the next delay.
        let mut requests = VecDeque::new();
        while let Some(event) = ready.pop() {
            self.route(event, &mut requests);
        }
        self.drive(requests, Queue::with_capacity(protocol::MAX_OUT_DOWN));
    }
}
/// Derive admission from the caller's actual budgets, never from fixture counts.
#[must_use]
pub fn component_limits(root: &root::Limits) -> Limits {
    let mut kv = skein_kv::Limits {
        key: u32::MAX,
        value: u32::MAX,
        ops: u32::MAX,
        commit: u32::MAX,
        queued: 8,
        queued_bytes: u64::MAX,
        budget: 4 * 1024 * 1024,
        segment: 1,
        snapshot_after: 1,
        chunk: 1,
        page: skein_kv::Page { rows: 2, bytes: u32::MAX },
        deadline: Duration::from_secs(1),
    };
    let sizes =
        protocol::sizes::check(&root.core, &jig_test_store::application(&root.connector), &root.journal, &kv, 1)
            .expect("root's actual journal and map rows fit the store");
    kv.key = sizes.key;
    kv.value = sizes.value;
    kv.ops = sizes.ops;
    kv.commit = sizes.commit;
    kv.queued_bytes = u64::from(sizes.commit) * u64::from(kv.queued);
    kv.segment = u64::from(sizes.commit) * 2;
    kv.snapshot_after = u64::from(sizes.commit);
    kv.chunk = sizes.key.checked_add(sizes.value).and_then(|sum| sum.checked_add(20)).expect("bounded largest row");
    kv.page.bytes = sizes.key.checked_add(sizes.value).expect("one largest row fits");
    Limits {
        core: root.core,
        store: kv,
        secrets: kv,
        journal: root.journal,
        joins: 128,
        queue: 8,
        files: 8,
        reads: root.core.load_slots.checked_add(root.core.tasks.tasks).expect("bounded readers"),
        orphan_erases: 1,
    }
}
