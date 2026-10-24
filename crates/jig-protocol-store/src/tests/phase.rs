//! The patch phase on a simulated disk (store.md 10.2 and 10.4).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "disk step tests use ordinary Rust (programming-model.md 10.2)"
)]
use super::fixtures::{
    self,
    disk::{Disk, Seen},
};
use crate::directory::{OWNER, Opening};
use crate::patching::phase::{Patching, Pending};
use crate::{Arrived, Event, Get, Io, OpenRefusal, Phase, Request, Root, Row, Store, Write};
use skein_fake_machine::{How, Is, Item, Machine, Opened};
use skein_io::{
    file,
    file_layer::{self, FileIo},
    kernel::{Complete, Error, Kind, Submit},
};
use skein_lib::{Queue, Token};
use skein_sim::{Answer, Ask, Config, Handle, Pid, Sim};

const CHUNK: u32 = 4096;
const KEY: &[u8] = b"row";
const VALUE: &[u8] = b"committed";
fn assert_opened(events: &[Event]) {
    let [Event::Opened { owner, last: 1, .. }] = events else { panic!("one opened: {events:?}") };
    assert_eq!(*owner, Token::new(1));
}
fn baseline() -> Vec<Item> {
    baseline_with_proof(false)
}
fn baseline_with_proof(proof: bool) -> Vec<Item> {
    let mut disk = Disk::new(&[]);
    let mut store = disk.store_with_chunk(CHUNK);
    let events = disk.open(&mut store);
    assert!({ if let [Event::Opened { last: 0, .. }] = events.as_ref() { true } else { false } });
    let mut writes = vec![fixtures::header(1), Write::Put { key: Box::from(KEY), value: Box::from(VALUE) }];
    if proof {
        writes.push(fixtures::save(fixtures::run_proof(1)));
    }
    let events = disk.request(&mut store, fixtures::commit(1, writes.into_boxed_slice()));
    assert!({ if let [Event::Committed { number: 1, .. }] = events.as_ref() { true } else { false } });
    disk.items()
}
fn read_row(disk: &mut Disk, store: &mut Store) {
    let events = disk.request(store, Request::Get { owner: Token::new(2), key: Get::Application(Box::from(KEY)) });
    let [Event::Got { row: Some(Row::Application { key, value }), .. }] = events.as_ref() else {
        panic!("committed application row: {events:?}")
    };
    assert_eq!(key.as_ref(), KEY);
    assert_eq!(value.as_ref(), VALUE);
}
fn patch(disk: &mut Disk) -> Store {
    let mut store = disk.store_with_chunk(CHUNK);
    // No level triggers B0 yet: install what task B's checks will leave,
    // before open reaches 10.1 step 7 (after checks, sweep and transcripts make).
    store.directory.patch = Some(Box::new(Pending {}));
    assert_opened(&disk.open(&mut store));
    store
}
#[test]
fn a_patch_opens_the_copy_with_the_committed_row_and_keeps_the_previous_whole() {
    let mut disk = Disk::new(&baseline());
    let mut store = patch(&mut disk);
    assert_eq!(disk.bytes(b"active").as_ref(), b"2\n");
    assert_eq!(disk.bytes(b"generations/2/manifest").as_ref(), b"2\n");
    let contents = disk.contents();
    for (path, kind, _, bytes) in contents.iter().filter(|(p, ..)| p.starts_with(b"generations/1/")) {
        let target = [b"generations/2/", path.strip_prefix(b"generations/1/").expect("prefix")].concat();
        let (_, copied_kind, _, copied_bytes) =
            contents.iter().find(|(p, ..)| p.as_ref() == target).expect("whole copy");
        assert_eq!(copied_kind, kind);
        assert_eq!(copied_bytes, bytes);
    }
    read_row(&mut disk, &mut store);
    disk.crash(17);
    let mut store = disk.store_with_chunk(CHUNK);
    assert_opened(&disk.open(&mut store));
    assert_eq!(store.directory.active.expect("active read").number, 2);
    assert!(
        !disk
            .requests()
            .iter()
            .any(|(driver, r)| *driver == 2 && { if let Seen::Remove { .. } = r { true } else { false } })
    );
    read_row(&mut disk, &mut store);
}
#[test]
fn the_next_patch_drops_one_keeps_two_and_opens_three() {
    let mut first = Disk::new(&baseline());
    patch(&mut first);
    let mut disk = Disk::new(&first.items());
    let mut store = patch(&mut disk);
    assert_eq!(disk.bytes(b"active").as_ref(), b"3\n");
    assert_eq!(disk.bytes(b"generations/3/manifest").as_ref(), b"3\n");
    let tree = disk.tree();
    assert!(!tree.iter().any(|(p, ..)| p.as_ref() == b"generations/1"));
    for generation in [2_u64, 3] {
        for map in ["store", "secrets"] {
            let path = format!("generations/{generation}/{map}");
            assert!(
                tree.iter()
                    .any(|(p, kind, mode)| p.as_ref() == path.as_bytes() && *kind == Kind::Directory && *mode == 0o700)
            );
        }
    }
    read_row(&mut disk, &mut store);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Other,
    Closing,
    Dropping,
    Copying,
    Opening,
    Manifest,
    Leaving,
    Switching,
}
fn stage(store: &Store) -> Stage {
    match &store.directory.opening {
        Opening::Patching(phase) => match **phase {
            Patching::Closing { .. } => Stage::Closing,
            Patching::Dropping(_) => Stage::Dropping,
            Patching::Copying(_) => Stage::Copying,
            Patching::Mapping { .. } | Patching::Recovering { .. } => Stage::Opening,
            Patching::Manifest(_) => Stage::Manifest,
            Patching::Leaving { .. } => Stage::Leaving,
            Patching::Switching(_) => Stage::Switching,
        },
        _ => Stage::Other,
    }
}
#[derive(Debug, PartialEq, Eq)]
enum Trace {
    KvClose(usize),
    MapClose(usize),
    MapOpen(usize, Box<[u8]>),
    KvOpen(usize),
    Store(Box<[u8]>),
    Load(Box<[u8]>),
    Cut,
    Other,
}
fn trace(driver: usize, request: &file::Request) -> Trace {
    match request {
        file::Request::Close { owner, .. } if *owner == OWNER => Trace::MapClose(driver),
        file::Request::Close { .. } => Trace::KvClose(driver),
        file::Request::OpenDirectory { name, .. } => Trace::MapOpen(driver, name.clone()),
        file::Request::OpenPrivate { path, .. } => Trace::MapOpen(driver, path.clone()),
        file::Request::List { .. } => Trace::KvOpen(driver),
        file::Request::Store { path, .. } => Trace::Store(path.clone()),
        file::Request::Load { path, .. } => Trace::Load(path.clone()),
        file::Request::Cut { .. } => Trace::Cut,
        _ => Trace::Other,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Change {
    Create,
    Write,
    Sync,
    Rename,
    Remove,
    Cut,
    Make,
    Other,
}
fn change(ask: &Ask) -> Change {
    match ask {
        Ask::Open {
            how: skein_io::kernel::OpenHow::Create { .. } | skein_io::kernel::OpenHow::CreateNoFollow { .. },
            ..
        } => Change::Create,
        Ask::Write { .. } | Ask::Append { .. } => Change::Write,
        Ask::Remove { .. } => Change::Remove,
        Ask::Truncate { .. } => Change::Cut,
        Ask::MakeDirectory { .. } => Change::Make,
        Ask::Sync { .. } => Change::Sync,
        Ask::Rename { .. } => Change::Rename,
        _ => Change::Other,
    }
}
/// Same three real file drivers as `fixtures::disk`, with a seam inside Store's
/// durable replace. Cuts and faults act on the simulator's machine calls.
struct Harness {
    machine: Machine,
    handle: Opened,
    sim: Sim,
    pid: Pid,
    root: Root,
    io: [FileIo; 3],
    seeded: crate::Seeded,
    trace: Vec<(Stage, Trace)>,
    changes: Vec<(Stage, Change)>,
    cut: Option<(usize, bool)>,
    fault_write: bool,
    halted: bool,
    source: Option<Vec<Item>>,
}
impl Harness {
    fn new(items: &[Item]) -> Self {
        let mut machine = Machine::for_user(0);
        let handle = machine.lay(items);
        Self::over(machine, handle)
    }
    fn over(machine: Machine, handle: Opened) -> Self {
        let mut sim = Sim::new(1, Config::calm());
        let pid = sim.spawn_process();
        let fd = sim.root(pid, Handle::new(handle.raw()));
        let mut io = core::array::from_fn(|_| {
            let limits = fixtures::store_limits();
            let mut io = FileIo::new(8, limits.store.chunk, 32, limits.store.deadline).with_effective_user(0);
            assert_eq!(io.adopt_root(fd), Some(Token::new(1)));
            io
        });
        let [public, private, transcripts] = &mut io;
        let seeded = crate::seed([public, private, transcripts], 1);
        Self {
            machine,
            handle,
            sim,
            pid,
            root: Root { token: Token::new(1), fd },
            io,
            seeded,
            trace: Vec::new(),
            changes: Vec::new(),
            cut: None,
            fault_write: false,
            halted: false,
            source: None,
        }
    }
    fn store(&self, patch: bool) -> Store {
        let mut limits = fixtures::store_limits();
        limits.store.chunk = CHUNK;
        limits.store.value =
            CHUNK.checked_sub(limits.store.key).expect("key fits").checked_sub(20).expect("header fits");
        limits.secrets.chunk = CHUNK;
        limits.secrets.value =
            CHUNK.checked_sub(limits.secrets.key).expect("key fits").checked_sub(20).expect("header fits");
        let mut store = Store::on_directory(&limits, fixtures::APPLICATION, &[], self.seeded);
        if patch {
            store.directory.patch = Some(Box::new(Pending {}));
        }
        store
    }
    fn open(&mut self, store: &mut Store) -> Box<[Event]> {
        let mut above = Queue::with_capacity(128);
        let mut below = fixtures::below();
        crate::down(
            store,
            &fixtures::env(),
            Request::Open { owner: Token::new(1), root: self.root, installation: fixtures::INSTALLATION },
            &mut above,
            &mut below,
        );
        for _ in 0_u32..10_000 {
            crate::fire(store, &fixtures::env(), &mut above, &mut below);
            let next = below
                .store
                .pop()
                .map(|r| (0, r))
                .or_else(|| below.secrets.pop().map(|r| (1, r)))
                .or_else(|| below.transcripts.pop().map(|r| (2, r)));
            let Some((driver, request)) = next else {
                return fixtures::events(&mut above);
            };
            let stage = stage(store);
            if stage == Stage::Closing && self.source.is_none() {
                self.source = Some(self.source_items(store.directory.active.expect("active read").number));
            }
            self.trace.push((stage, trace(driver, &request)));
            let Some(arrived) = self.file(driver, request, stage) else {
                return fixtures::events(&mut above);
            };
            let io = match driver {
                0 => Io::Store(arrived),
                1 => Io::Secrets(arrived),
                2 => Io::Transcripts(arrived),
                _ => unreachable!(),
            };
            crate::up(store, &fixtures::env(), io, &mut above, &mut below);
        }
        panic!("phase did not settle");
    }
    fn file(&mut self, driver: usize, request: file::Request, stage: Stage) -> Option<Arrived> {
        let mut events = Queue::with_capacity(128);
        let mut subs: Queue<Submit> = Queue::with_capacity(128);
        let mut completions: Queue<Complete> = Queue::with_capacity(128);
        file_layer::down(&mut self.io[driver], fixtures::env().now, request, &mut events, &mut subs);
        for _ in 0_u32..100 {
            self.sim.submit(self.pid, &mut subs);
            let mut calls = Queue::with_capacity(128);
            let mut answers = Queue::with_capacity(128);
            self.sim.calls(&mut calls);
            while let Some(call) = calls.pop() {
                let changing = change(&call.ask);
                let targeted = (stage == Stage::Manifest || stage == Stage::Opening) && changing != Change::Other;
                let ordinal = self.changes.len();
                if targeted {
                    self.changes.push((stage, changing));
                    if self.cut == Some((ordinal, false)) {
                        self.halted = true;
                        return None;
                    }
                }
                if self.fault_write && stage == Stage::Manifest && changing == Change::Write {
                    self.fault_write = false;
                    answers.push(Answer { ticket: call.ticket, result: Err(Error::NoSpace) });
                } else {
                    skein_fake_machine::step(&mut self.machine, call, &mut answers);
                }
                self.sim.answer(&mut answers);
                if targeted && self.cut == Some((ordinal, true)) {
                    self.halted = true;
                    return None;
                }
            }
            self.sim.reap(self.pid, &mut completions);
            while let Some(complete) = completions.pop() {
                file_layer::up(&mut self.io[driver], complete, &mut events, &mut subs);
            }
            if self.io[driver].takes() {
                let event = events.pop().expect("file terminal");
                let fd = if let file::Event::Opened { file, .. } = &event {
                    self.io[driver].descriptor(*file)
                } else {
                    None
                };
                return Some(Arrived { event, fd });
            }
        }
        panic!("file did not settle");
    }
    fn bytes(&mut self, path: &[u8]) -> Box<[u8]> {
        let file = self.machine.open(self.handle, path, How::ReadNoFollow).expect("file stands");
        let bytes = self.machine.read(file, 0, u32::MAX).expect("file read");
        self.machine.close(file);
        bytes.into_boxed_slice()
    }
    fn items(&mut self) -> Vec<Item> {
        let mut items = Vec::new();
        self.walk(b"", &mut items);
        items
    }
    fn walk(&mut self, prefix: &[u8], items: &mut Vec<Item>) {
        let directory = self
            .machine
            .open(self.handle, if prefix.is_empty() { b"." } else { prefix }, How::Directory)
            .expect("directory");
        loop {
            let entries = self.machine.list(directory, 32, 8192).expect("list");
            if entries.is_empty() {
                break;
            }
            for (kind, name) in entries {
                let path = if prefix.is_empty() { name.to_vec() } else { [prefix, b"/", &name].concat() };
                if kind == Is::Directory {
                    items.push(Item::directory(&path).mode(0o700));
                    self.walk(&path, items);
                } else {
                    items.push(Item::file(&path, &self.bytes(&path)).mode(0o600));
                }
            }
        }
        self.machine.close(directory);
    }
    fn source_items(&mut self, number: u64) -> Vec<Item> {
        let prefix = format!("generations/{number}/");
        self.items().into_iter().filter(|item| item.path.starts_with(prefix.as_bytes())).collect()
    }
    fn assert_source_unchanged(&mut self, number: u64) {
        assert_eq!(self.source_items(number), *self.source.as_ref().expect("captured before closing A"));
    }
    fn crash(self, seed: u64) -> Self {
        let mut machine = self.machine;
        machine.crash(seed);
        let handle = machine.reopen_root(0);
        Self::over(machine, handle)
    }
}
#[test]
fn the_phase_closes_drops_copies_opens_manifests_closes_switches_then_reopens() {
    let mut first = Disk::new(&baseline());
    patch(&mut first);
    let mut disk = Harness::new(&first.items());
    let mut store = disk.store(true);
    assert_opened(&disk.open(&mut store));
    let stages: Vec<_> = disk.trace.iter().map(|(stage, _)| *stage).filter(|s| *s != Stage::Other).fold(
        Vec::new(),
        |mut stages, stage| {
            if stages.last() != Some(&stage) {
                stages.push(stage);
            }
            stages
        },
    );
    assert_eq!(
        stages,
        [
            Stage::Closing,
            Stage::Dropping,
            Stage::Copying,
            Stage::Opening,
            Stage::Manifest,
            Stage::Leaving,
            Stage::Switching
        ]
    );
    for stage in [Stage::Closing, Stage::Leaving] {
        let closes: Vec<_> = disk.trace.iter().filter(|(s, _)| *s == stage).map(|(_, request)| request).collect();
        // Both KV log closes must finish before either map directory closes.
        assert_eq!(closes, [&Trace::KvClose(0), &Trace::KvClose(1), &Trace::MapClose(0), &Trace::MapClose(1)]);
    }
    let opens: Vec<_> = disk
        .trace
        .iter()
        .filter(|(s, _)| *s == Stage::Opening)
        .map(|(_, r)| r)
        .filter(|r| **r != Trace::Other)
        .collect();
    assert_eq!(opens.first(), Some(&&Trace::MapOpen(0, Box::from(b"generations/3/store".as_slice()))));
    assert_eq!(opens.get(1), Some(&&Trace::MapOpen(1, Box::from(b"generations/3/secrets".as_slice()))));
    assert!(opens.contains(&&Trace::KvOpen(0)) && opens.contains(&&Trace::KvOpen(1)));
    let switch = disk
        .trace
        .iter()
        .position(|(s, r)| *s == Stage::Switching && *r == Trace::Store(Box::from(b"active".as_slice())))
        .expect("switch");
    assert_eq!(disk.trace.get(switch + 1), Some(&(Stage::Other, Trace::Load(Box::from(b"active".as_slice())))));
    assert_eq!(
        disk.trace
            .iter()
            .filter(|(_, r)| { if let Trace::Load(p) = r { p.as_ref() == b"active" } else { false } })
            .count(),
        2
    );
}
#[test]
fn cuts_at_the_manifests_write_syncs_and_rename_and_copy_recovery_keep_a_whole_generation() {
    let items = baseline();
    let mut completed = Harness::new(&items);
    let mut store = completed.store(true);
    assert_opened(&completed.open(&mut store));
    let changes = completed.changes.clone();
    for change in [Change::Write, Change::Sync, Change::Rename] {
        assert!(changes.contains(&(Stage::Manifest, change)), "manifest {change:?}");
    }
    assert!(changes.contains(&(Stage::Opening, Change::Sync)), "copy recovery reaches the simulated disk");
    for (cut, target) in changes.iter().enumerate() {
        for after in [false, true] {
            let mut disk = Harness::new(&items);
            disk.cut = Some((cut, after));
            let mut store = disk.store(true);
            assert!(disk.open(&mut store).is_empty(), "cut {cut} {target:?}");
            assert!(disk.halted, "cut reached");
            disk.assert_source_unchanged(1);
            let mut disk = disk.crash(u64::try_from(cut).expect("seed"));
            let mut store = disk.store(false);
            assert_opened(&disk.open(&mut store));
            assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
            let mut recovered = Disk::new(&disk.items());
            let mut store = recovered.store_with_chunk(CHUNK);
            assert_opened(&recovered.open(&mut store));
            read_row(&mut recovered, &mut store);
            assert!(!recovered.tree().iter().any(|(p, ..)| p.starts_with(b"generations/2")), "cut {cut} {target:?}");
        }
    }
}
#[test]
fn a_manifest_write_fault_refuses_failed_and_the_next_start_removes_the_copy() {
    let mut disk = Harness::new(&baseline());
    disk.fault_write = true;
    let mut store = disk.store(true);
    let events = disk.open(&mut store);
    assert!({ if let [Event::OpenRefused { why: OpenRefusal::Failed, .. }] = events.as_ref() { true } else { false } });
    assert_eq!(store.phase, Phase::Stopped);
    assert!(!disk.fault_write, "fault injected into the manifest's write");
    disk.assert_source_unchanged(1);
    assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
    let mut disk = disk.crash(19);
    let mut store = disk.store(false);
    assert_opened(&disk.open(&mut store));
    let mut recovered = Disk::new(&disk.items());
    let mut store = recovered.store_with_chunk(CHUNK);
    assert_opened(&recovered.open(&mut store));
    read_row(&mut recovered, &mut store);
    assert!(!recovered.tree().iter().any(|(p, ..)| p.starts_with(b"generations/2")));
}

#[test]
fn the_run_proofs_transcript_cut_waits_until_after_the_switch() {
    let mut items = baseline_with_proof(true);
    items.push(Item::directory(b"transcripts/0000000000000000"));
    items.push(Item::file(b"transcripts/0000000000000000/0000000000000001", b"uncommitted tail"));
    let mut disk = Harness::new(&items);
    let mut store = disk.store(true);
    assert_opened(&disk.open(&mut store));
    let switch = disk.trace.iter().position(|(s, _)| *s == Stage::Switching).expect("switch");
    let cuts: Vec<_> = disk
        .trace
        .iter()
        .enumerate()
        .filter(|(_, (_, r))| *r == Trace::Cut)
        .map(|(index, (stage, _))| (index, *stage))
        .collect();
    assert_eq!(cuts.len(), 1);
    assert!(cuts.iter().all(|(index, stage)| *index > switch && *stage == Stage::Other));
    assert!(disk.bytes(b"transcripts/0000000000000000/0000000000000001").is_empty());
    disk.assert_source_unchanged(1);
}

#[test]
fn the_manifest_write_refuses_to_replace_a_standing_manifest() {
    let mut disk = Disk::new(&[
        Item::directory(b"generations"),
        Item::directory(b"generations/2"),
        Item::file(b"generations/2/manifest", b"standing"),
    ]);
    let mut below = fixtures::below();
    let mut writing = crate::generations::store_manifest(disk.root(), 2, Box::from(b"2\n".as_slice()), &mut below);
    let mut progress = crate::generations::Progress::Pending;
    disk.drive(&mut below, |io, _| {
        let Io::Transcripts(arrived) = io else {
            panic!("manifest uses driver 2");
        };
        progress = crate::generations::manifested(&mut writing, arrived);
    });
    assert_eq!(progress, crate::generations::Progress::Failed);
    assert_eq!(disk.bytes(b"generations/2/manifest").as_ref(), b"standing");
}
