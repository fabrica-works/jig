//! Three file drivers over the simulated crashing machine (protocol/store.md, 14).
use crate::{model::Model, root};
use jig_domain_core as core;
use jig_protocol_store::{
    Arrived, Below, Event, Get, Io, Load, Request, Restore, Root, Row, Secrets, Store, TranscriptRead, Write,
};
use skein_fake_machine::{Machine, serve};
use skein_io::{
    file,
    file_layer::{self, FileIo},
    kernel::{Complete, Submit},
};
use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_sim::{Config, Handle, Pid, Sim};
use std::collections::BTreeMap;
const OWNER: Token = Token::new(900);
const QUEUE: u32 = 128;

#[derive(Debug)]
pub struct Restored {
    pub last: u64,
    pub rows: BTreeMap<core::Key, core::Record>,
    pub application: BTreeMap<Box<[u8]>, Box<[u8]>>,
    pub transcripts: BTreeMap<u64, Vec<core::TurnRecord>>,
}

impl Restored {
    #[must_use]
    pub fn turns(&self, task: u64) -> &[core::TurnRecord] {
        self.transcripts.get(&task).map_or(&[], Vec::as_slice)
    }
}

pub struct World {
    machine: Machine,
    sim: Sim,
    pid: Pid,
    root: Root,
    io: [FileIo; 3],
    store: Store,
    env: Env<jig_protocol_store::Limits>,
    above: Queue<Event>,
    below: Below,
    subs: Queue<Submit>,
    completions: Queue<Complete>,
    cut: Option<u32>,
    operations: u32,
    transcript_sync: Option<u32>,
    next: u64,
    pub model: Model,
    /// The identifier the shell draws for the next start (configuration.md 8).
    pub drawn: [u8; 16],
    /// The installation's identifier the last open answered (store.md 4.4).
    pub installation: [u8; 16],
}

fn drivers(root: skein_io::kernel::Fd, limits: &jig_protocol_store::Limits) -> [FileIo; 3] {
    std::array::from_fn(|_| {
        let mut io = FileIo::new(8, limits.store.chunk, 32, limits.store.deadline).with_effective_user(0);
        assert_eq!(io.adopt_root(root), Some(Token::new(1)));
        io
    })
}

fn below() -> Below {
    Below {
        store: Queue::with_capacity(QUEUE),
        secrets: Queue::with_capacity(QUEUE),
        transcripts: Queue::with_capacity(QUEUE),
    }
}

impl World {
    #[must_use]
    pub fn new(seed: u64) -> World {
        Self::with_joins(seed, root::limits().joins)
    }

    #[must_use]
    pub fn with_joins(seed: u64, joins: u32) -> World {
        let mut machine = Machine::for_user(0);
        let handle = machine.lay(&[]);
        let mut sim = Sim::new(seed, Config::calm());
        let pid = sim.spawn_process();
        let fd = sim.root(pid, Handle::new(handle.raw()));
        let mut limits = root::limits();
        limits.joins = joins;
        let mut world = World {
            machine,
            sim,
            pid,
            root: Root { token: Token::new(1), fd },
            io: drivers(fd, &limits),
            store: Store::on_directory(&limits, root::APPLICATION, &[]),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            above: Queue::with_capacity(QUEUE),
            below: below(),
            subs: Queue::with_capacity(QUEUE),
            completions: Queue::with_capacity(QUEUE),
            cut: None,
            operations: 0,
            transcript_sync: None,
            next: 1,
            model: Model::default(),
            drawn: [1; 16],
            installation: [0; 16],
        };
        assert_eq!(world.open(), 0);
        world
    }

    pub fn cut_at(&mut self, operation: u32) {
        self.cut = Some(operation);
    }

    #[must_use]
    pub const fn operation_count(&self) -> u32 {
        self.operations
    }

    #[must_use]
    pub const fn transcript_sync(&self) -> Option<u32> {
        self.transcript_sync
    }

    pub fn commit(&mut self, writes: Box<[Write]>) -> Option<u64> {
        let number = self.next;
        self.model.stage(number, &writes);
        self.operations = 0;
        self.transcript_sync = None;
        let events = self.request(Request::Commit { number, writes });
        let mut acknowledged = None;
        for event in events {
            if let Event::Committed { number: found, .. } = event {
                assert_eq!(found, number);
                self.model.acknowledge(number);
                self.next = number + 1;
                acknowledged = Some(number);
            } else {
                panic!("commit terminal: {event:?}");
            }
        }
        acknowledged
    }

    pub fn crash(&mut self, seed: u64) {
        self.machine.crash(seed);
        let handle = self.machine.reopen_root(0);
        self.sim = Sim::new(seed, Config::calm());
        self.pid = self.sim.spawn_process();
        let fd = self.sim.root(self.pid, Handle::new(handle.raw()));
        self.root = Root { token: Token::new(1), fd };
        self.io = drivers(fd, &self.env.limits);
        self.store = Store::on_directory(&self.env.limits, root::APPLICATION, &[]);
        self.above = Queue::with_capacity(QUEUE);
        self.below = below();
        self.subs = Queue::with_capacity(QUEUE);
        self.completions = Queue::with_capacity(QUEUE);
        self.cut = None;
        self.operations = 0;
        self.transcript_sync = None;
    }

    pub fn reopen(&mut self) -> Restored {
        let last = self.open();
        self.next = last + 1;
        let mut restored = Restored {
            last,
            rows: BTreeMap::new(),
            application: self.application(false),
            transcripts: BTreeMap::new(),
        };
        let mut after = None;
        loop {
            let events = self.request(Request::Restore {
                owner: OWNER,
                range: Restore::Jig(core::RestoreRange::LIVE.into()),
                after: after.take(),
                rows: 2,
                bytes: u64::from(self.env.limits.store.page.bytes),
            });
            let [Event::Restored { rows, next, .. }] = events.as_slice() else {
                panic!("restore terminal: {events:?}")
            };
            for row in rows {
                let Row::Jig(record) = row else { panic!("jig restore") };
                assert!(restored.rows.insert(record.key(), record.clone()).is_none(), "no repeated page row");
            }
            after.clone_from(next);
            if after.is_none() {
                break;
            }
        }
        for record in restored.rows.values() {
            if let core::Record::Core(core::CoreRecord::RunProof(proof)) = record {
                let turns = match proof.window {
                    Some((attempt, turn)) => {
                        self.transcript(TranscriptRead::Window { task: proof.task, attempt, turn, after: None })
                    }
                    None => Vec::new(),
                };
                restored.transcripts.insert(proof.task, turns);
            }
        }
        restored
    }

    pub fn accept_recovery(&mut self, last: u64) {
        self.model.recovered(last);
    }

    pub fn request(&mut self, request: Request) -> Vec<Event> {
        jig_protocol_store::down(&mut self.store, &self.env, request, &mut self.above, &mut self.below);
        self.drive();
        let mut events = Vec::new();
        while let Some(event) = self.above.pop() {
            events.push(event);
        }
        events
    }

    fn open(&mut self) -> u64 {
        let events = self.request(Request::Open { owner: OWNER, root: self.root, installation: self.drawn });
        let [Event::Opened { last, installation, .. }] = events.as_slice() else { panic!("open terminal: {events:?}") };
        self.installation = *installation;
        *last
    }

    fn drive(&mut self) {
        for _ in 0..10_000 {
            jig_protocol_store::fire(&mut self.store, &self.env, &mut self.above, &mut self.below);
            if self.cut.is_some_and(|cut| self.operations >= cut) {
                return;
            }
            let next = self
                .below
                .store
                .pop()
                .map(|request| (0, request))
                .or_else(|| self.below.secrets.pop().map(|request| (1, request)))
                .or_else(|| self.below.transcripts.pop().map(|request| (2, request)));
            let Some((driver, request)) = next else {
                return;
            };
            let synced = driver == 2 && matches!(&request, file::Request::Sync { .. });
            self.file(driver, request);
            self.operations += 1;
            if synced {
                self.transcript_sync = Some(self.operations);
            }
        }
        panic!("store did not quiesce");
    }

    fn file(&mut self, driver: usize, request: file::Request) {
        let io = self.io.get_mut(driver).expect("three drivers");
        assert!(io.takes());
        let mut events = Queue::with_capacity(QUEUE);
        file_layer::down(io, self.env.now, request, &mut events, &mut self.subs);
        for _ in 0..100 {
            if !self.subs.is_empty() {
                self.sim.submit(self.pid, &mut self.subs);
                serve(&mut self.machine, &mut self.sim);
            }
            self.sim.reap(self.pid, &mut self.completions);
            while let Some(complete) = self.completions.pop() {
                file_layer::up(io, complete, &mut events, &mut self.subs);
            }
            if io.takes() {
                break;
            }
        }
        assert!(io.takes(), "a calm file operation settles");
        assert!(!events.is_empty(), "one file terminal");
        while let Some(event) = events.pop() {
            let fd = if let file::Event::Opened { file, .. } = &event { io.descriptor(*file) } else { None };
            let arrived = Arrived { event, fd };
            let event = match driver {
                0 => Io::Store(arrived),
                1 => Io::Secrets(arrived),
                2 => Io::Transcripts(arrived),
                _ => panic!("three drivers"),
            };
            jig_protocol_store::up(&mut self.store, &self.env, event, &mut self.above, &mut self.below);
        }
    }

    pub fn application(&mut self, load: bool) -> BTreeMap<Box<[u8]>, Box<[u8]>> {
        let mut result = BTreeMap::new();
        let mut after = None;
        loop {
            let ranges = Box::new([skein_kv::Range { start: Box::new([]), end: None }]);
            let request = if load {
                Request::Load {
                    owner: OWNER,
                    range: Load::Application(ranges),
                    after: after.take(),
                    rows: 2,
                    bytes: u64::from(self.env.limits.store.page.bytes),
                }
            } else {
                Request::Restore {
                    owner: OWNER,
                    range: Restore::Application(ranges),
                    after: after.take(),
                    rows: 2,
                    bytes: u64::from(self.env.limits.store.page.bytes),
                }
            };
            let events = self.request(request);
            let [Event::Restored { rows, next, .. } | Event::Loaded { rows, next, .. }] = events.as_slice() else {
                panic!("application page: {events:?}")
            };
            for row in rows {
                let Row::Application { key, value } = row else { panic!("application row") };
                assert!(result.insert(key.clone(), value.clone()).is_none(), "page boundary has no duplicate");
            }
            after.clone_from(next);
            if after.is_none() {
                break;
            }
        }
        result
    }

    pub fn transcript(&mut self, mut read: TranscriptRead) -> Vec<core::TurnRecord> {
        let mut result = Vec::new();
        loop {
            let events = self.request(Request::Transcript {
                owner: OWNER,
                read,
                rows: 2,
                bytes: u64::from(self.env.limits.store.page.bytes),
            });
            if let [Event::Unread { .. }] = events.as_slice() {
                return result;
            }
            let [Event::Turns { turns, next, .. }] = events.as_slice() else {
                panic!("transcript terminal: {events:?}")
            };
            result.extend(turns.iter().cloned());
            let Some(after) = next else {
                return result;
            };
            read = match read {
                TranscriptRead::Window { task, attempt, turn, .. } => {
                    TranscriptRead::Window { task, attempt, turn, after: Some(*after) }
                }
                TranscriptRead::Page { task, backwards, .. } => {
                    TranscriptRead::Page { task, backwards, after: Some(*after) }
                }
                TranscriptRead::Turn { .. } => panic!("a single turn has no next page"),
            };
        }
    }

    pub fn get(&mut self, key: core::Key) -> core::Record {
        let events = self.request(Request::Get { owner: OWNER, key: Get::Jig(key) });
        let [Event::Got { row: Some(Row::Jig(row)), .. }] = events.as_slice() else {
            panic!("get terminal: {events:?}")
        };
        row.clone()
    }

    pub fn read_task(&mut self, task: u64) -> Vec<core::Record> {
        let range = core::reads::Range {
            party: 1,
            scope: core::reads::Scope::Task { task },
            after: None,
            backwards: false,
            rows: 2,
            bytes: u64::from(self.env.limits.store.page.bytes),
        };
        let events = self.request(Request::Read { owner: OWNER, range });
        let [Event::Read { result: core::reads::Loaded::Page { rows, next: None }, .. }] = events.as_slice() else {
            panic!("task read terminal: {events:?}")
        };
        rows.iter().map(|row| row.value.clone()).collect()
    }

    /// One secrets' request, its file operations counted from zero.
    pub fn secret(&mut self, request: Secrets) -> Vec<Event> {
        self.secrets(vec![request])
    }

    /// Secrets' requests asked together, before any of their files is
    /// written; their operations counted from zero.
    pub fn secrets(&mut self, requests: Vec<Secrets>) -> Vec<Event> {
        self.operations = 0;
        for request in requests {
            jig_protocol_store::down(
                &mut self.store,
                &self.env,
                Request::Secrets { owner: OWNER, request },
                &mut self.above,
                &mut self.below,
            );
        }
        self.drive();
        let mut events = Vec::new();
        while let Some(event) = self.above.pop() {
            events.push(event);
        }
        events
    }
}
