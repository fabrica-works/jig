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
use super::patching::{BASE, SETS, decoded, label, levels, shelf};
use crate::directory::{OWNER, Opening};
use crate::patching::phase::{Patching, Pending};
use crate::{Arrived, Event, Get, Io, OpenRefusal, Phase, Request, Root, Row, Store, Write};
use jig_test_patches::patches as app;
use jig_test_patches_jig::patches as jig;
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
fn empty_pending() -> Pending {
    let sets = crate::PatchSets::NONE;
    let found = crate::patching::plan::binary(&sets);
    let plan = crate::patching::plan::plan(&sets, &found, &found).expect("empty plan");
    let machines = crate::patching::machines::make(
        &sets,
        &plan,
        &found,
        crate::patching::machines::Room { bytes: 128, nodes: 16 },
    )
    .expect("empty machines");
    Pending { plan, machines, found }
}
fn assert_opened(events: &[Event], expected: u64) {
    let [Event::Opened { owner, last, .. }] = events else { panic!("one opened: {events:?}") };
    assert_eq!(*owner, Token::new(1));
    assert_eq!(*last, expected);
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
    store.directory.patch = Some(Box::new(empty_pending()));
    let events = disk.open(&mut store);
    assert_opened(&events, store.directory.active.expect("switched").number);
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
        if path.starts_with(b"generations/1/secrets/") {
            assert_eq!(copied_bytes, bytes);
        }
    }
    read_row(&mut disk, &mut store);
    disk.crash(17);
    let mut store = disk.store_with_chunk(CHUNK);
    assert_opened(&disk.open(&mut store), 2);
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
    Walking,
    Sealing,
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
            Patching::Walking(_) => Stage::Walking,
            Patching::Sealing { .. } => Stage::Sealing,
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
#[derive(Clone, Copy, PartialEq, Eq)]
enum SiteAnswer {
    Written,
    NoSite,
    WrongOwner,
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
    fault_at: Option<Stage>,
    halted: bool,
    source: Option<Vec<Item>>,
    sites: Vec<(crate::Owner, Box<[u8]>)>,
    site_answer: SiteAnswer,
    site_commit_limit: Option<u32>,
    page_header: Option<u64>,
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
            fault_at: None,
            halted: false,
            source: None,
            sites: Vec::new(),
            site_answer: SiteAnswer::Written,
            site_commit_limit: None,
            page_header: None,
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
            store.directory.patch = Some(Box::new(empty_pending()));
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
            let mut retained = Vec::new();
            while let Some(event) = above.pop() {
                if let Event::Patched { of: crate::Owner::Application, patch: 2, .. } = &event {
                    // Tests can lower the page limit after recovery, at the
                    // first written site, and restore it before sealing.
                    if self.site_commit_limit.is_some() {
                        store.limits.store.commit = fixtures::store_limits().store.commit;
                    }
                }
                if let Event::Site { owner, of, key, site, at, mut patcher } = event {
                    self.sites.push((of, key.clone()));
                    if let Some(limit) = self.site_commit_limit {
                        store.limits.store.commit = limit;
                    }
                    let result = if self.site_answer == SiteAnswer::NoSite {
                        Err(skein_codec::patch::Halt::NoSite(site))
                    } else {
                        let mut cx = jig_test_patches::patches::Context { key };
                        let (old, new) = patcher.parts();
                        jig_test_patches::patches::written(site, old, new, at, &mut cx)
                    };
                    crate::down(
                        store,
                        &fixtures::env(),
                        Request::Site {
                            owner: if self.site_answer == SiteAnswer::WrongOwner { Token::new(99) } else { owner },
                            patcher,
                            result,
                        },
                        &mut above,
                        &mut below,
                    );
                } else {
                    retained.push(event);
                }
            }
            for event in retained {
                above.push(event);
            }
            crate::fire(store, &fixtures::env(), &mut above, &mut below);

            let next = below
                .store
                .pop()
                .map(|r| (0, r))
                .or_else(|| below.secrets.pop().map(|r| (1, r)))
                .or_else(|| below.transcripts.pop().map(|r| (2, r)));
            let Some((driver, request)) = next else {
                if stage(store) == Stage::Walking {
                    continue;
                }
                return fixtures::events(&mut above);
            };
            let stage = stage(store);
            if stage == Stage::Walking
                && let Some(last) = self.page_header
            {
                let key = crate::keys::encode(
                    &jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment),
                    store.limits.store.key,
                )
                .expect("header key");
                let bytes = fixtures::get_kv(store, &key).expect("page keeps header");
                let record = crate::codec::decode(
                    &jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment),
                    &bytes,
                    &store.codec,
                    &store.names,
                )
                .expect("header decodes");
                assert_eq!(record, fixtures::header_record(last), "page commits never write the header");
            }
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
                let targeted = (stage == Stage::Manifest
                    || stage == Stage::Opening
                    || stage == Stage::Walking
                    || stage == Stage::Sealing)
                    && changing != Change::Other;
                let ordinal = self.changes.len();
                if targeted {
                    self.changes.push((stage, changing));
                    if self.cut == Some((ordinal, false)) {
                        self.halted = true;
                        return None;
                    }
                }
                if changing == Change::Write
                    && ((self.fault_write && stage == Stage::Manifest) || self.fault_at == Some(stage))
                {
                    self.fault_write = false;
                    self.fault_at = None;
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
    assert_opened(&disk.open(&mut store), 3);
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
            Stage::Sealing,
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
    assert_opened(&completed.open(&mut store), 2);
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
            assert_opened(&disk.open(&mut store), 1);
            assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
            let mut recovered = Disk::new(&disk.items());
            let mut store = recovered.store_with_chunk(CHUNK);
            assert_opened(&recovered.open(&mut store), 1);
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
    assert_opened(&disk.open(&mut store), 1);
    let mut recovered = Disk::new(&disk.items());
    let mut store = recovered.store_with_chunk(CHUNK);
    assert_opened(&recovered.open(&mut store), 1);
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
    assert_opened(&disk.open(&mut store), 2);
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

const FIXTURE_LAST: u64 = 3;
const SHELVES: u32 = 5;
fn fixture_key(prefix: u8, number: u32) -> Box<[u8]> {
    format!("{}/{number}", char::from(prefix)).into_bytes().into_boxed_slice()
}
fn fixture_baseline(bad: Option<(u32, u16, u32)>) -> Vec<Item> {
    let mut disk = Disk::new(&[]);
    let mut store = disk.store_with_chunk(CHUNK);
    assert_opened(&disk.open(&mut store), 0);
    let mut rows = vec![
        Write::Put { key: fixture_key(b'l', 0), value: label(1) },
        Write::Put { key: fixture_key(b'l', 1), value: label(1) },
        // keys::application adds exactly tag 2, so this lands inside the
        // fixture-jig family's whole-map range [02 6e, 02 6f).
        Write::Put { key: Box::from(b"n/0".as_slice()), value: label(1) },
    ];
    let note_key = crate::keys::application(b"n/0", store.limits.store.key).expect("tag");
    assert!(note_key.as_ref() >= jig_test_patches_jig::keys::NOTES_FROM);
    assert!(note_key.as_ref() < jig_test_patches_jig::keys::NOTES_TO);
    for number in 0..SHELVES {
        let (version, books) = match bad {
            Some((index, version, books)) if index == number => (version, books),
            _ => (1, number.checked_add(1).expect("books")),
        };
        rows.push(Write::Put { key: fixture_key(b's', number), value: shelf(version, books, &label(1)) });
    }
    // Three rows and the header fit the fixture's four-write journal.
    let mut rows = rows.into_iter();
    for number in 1..=FIXTURE_LAST {
        let mut writes: Vec<_> = rows.by_ref().take(3).collect();
        writes.push(fixtures::header(number));
        let events = disk.request(&mut store, fixtures::commit(number, writes.into_boxed_slice()));
        assert!(
            match events.as_ref() {
                [Event::Committed { number: found, .. }] => *found == number,
                _ => false,
            },
            "{events:?}"
        );
    }
    assert_eq!(store.stores.store.last(), FIXTURE_LAST);
    disk.items()
}
fn fixture_pending(store: &Store, wanted: crate::patching::plan::Levels) -> Pending {
    // Patch one alone represents a binary with only that patch landed.
    const ONE: crate::PatchSets = crate::PatchSets {
        sets: &[
            crate::PatchSet { patches: &[app::PATCHES[0]], ..SETS.sets[0] },
            crate::PatchSet { patches: &[], ..SETS.sets[1] },
        ],
    };
    let found = levels(0, 0);
    let sets = if wanted == levels(1, 0) { ONE } else { SETS };
    let plan = crate::patching::plan::plan(&sets, &found, &wanted).expect("fixture plan");
    let room = crate::patching::machines::Room { bytes: store.limits.store.value, nodes: 16 };
    let machines = crate::patching::machines::make(&sets, &plan, &found, room).expect("fixture machines");
    Pending { plan, machines, found }
}
fn fixture_store(disk: &Harness, writes: u32, commit: u32, wanted: crate::patching::plan::Levels) -> Store {
    let mut store = disk.store(false);
    store.limits.journal.writes = writes;
    store.limits.store.commit = commit;
    store.patch_sets(SETS);
    store.directory.patch = Some(Box::new(fixture_pending(&store, wanted)));
    store
}
fn patched(events: &[Event]) -> Vec<(crate::Owner, u32, crate::Counts)> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Patched { owner, of, patch, counts } => {
                assert_eq!(*owner, Token::new(1));
                Some((*of, *patch, *counts))
            }
            Event::Opened { .. } | Event::OpenRefused { .. } => None,
            _ => panic!("unexpected open answer: {event:?}"),
        })
        .collect()
}
fn assert_fixture_opened(events: &[Event], page_commits: u64) {
    assert_opened(
        events.get(events.len().checked_sub(1).expect("opened")..).expect("last event"),
        FIXTURE_LAST.checked_add(page_commits).expect("pages").checked_add(1).expect("seal"),
    );
}
fn fixture_rows(store: &mut Store, shelf_version: u16, note_version: u16) -> Vec<Box<[u8]>> {
    let mut rows = Vec::new();
    for number in 0..2 {
        let key = crate::keys::application(&fixture_key(b'l', number), store.limits.store.key).expect("key");
        let bytes = fixtures::get_kv(store, &key).expect("label");
        drop(decoded(app::kept::fixture_label::TEXTS[1], "Label", &bytes));
        assert_eq!(bytes, label(2));
        rows.push(bytes);
    }
    for number in 0..SHELVES {
        let relative = fixture_key(b's', number);
        let key = crate::keys::application(&relative, store.limits.store.key).expect("key");
        let bytes = fixtures::get_kv(store, &key).expect("shelf");
        let text = app::kept::fixture_shelf::TEXTS
            .get(usize::from(shelf_version.checked_sub(1).expect("version")))
            .expect("text");
        let (_, value, root) = decoded(text, "Shelf", &bytes);
        let held = value
            .content(value.field(root, app::kept::fixture_shelf::v1::shelf::LABEL).expect("label"))
            .expect("bytes");
        assert_eq!(held, label(2).as_ref());
        if shelf_version == 2 {
            let weight = value
                .uint(value.field(root, app::kept::fixture_shelf::v2::shelf::WEIGHT).expect("weight"))
                .expect("u32");
            let expected = u64::from(number.checked_add(1).expect("books"))
                .checked_mul(10)
                .expect("weight")
                .checked_add(u64::try_from(relative.len()).expect("key bytes"))
                .expect("weight");
            assert_eq!(weight, expected);
        }
        rows.push(bytes);
    }
    let key = crate::keys::application(b"n/0", store.limits.store.key).expect("key");
    let bytes = fixtures::get_kv(store, &key).expect("note");
    let text =
        jig::kept::fixture_note::TEXTS.get(usize::from(note_version.checked_sub(1).expect("version"))).expect("text");
    let (_, value, root) = decoded(text, "Note", &bytes);
    if note_version == 2 {
        assert!(
            !value
                .boolean(value.field(root, jig::kept::fixture_note::v2::note::PINNED).expect("pinned"))
                .expect("bool")
        );
    }
    rows.push(bytes);
    rows
}
fn assert_sealed(store: &mut Store, last: u64) {
    let header_key =
        crate::keys::encode(&jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment), store.limits.store.key)
            .expect("header key");
    let header = fixtures::get_kv(store, &header_key).expect("sealed header");
    let expected = fixtures::header_put(fixtures::DEPLOYMENT, last);
    let skein_kv::Op::Put { value, .. } = expected else { panic!("header put") };
    assert_eq!(header, value);
    let key = crate::keys::encode_own(&crate::keys::Own::Format, store.limits.store.key).expect("format key");
    assert_eq!(
        fixtures::get_kv(store, &key).expect("format"),
        crate::format::current(store.application.version, fixtures::DEPLOYMENT).encode().expect("format bytes")
    );
}
#[test]
fn fixture_patches_walk_in_plan_order_answer_relative_keys_and_seal_both_rows() {
    let mut disk = Harness::new(&fixture_baseline(None));
    let mut store = fixture_store(&disk, 8, fixtures::store_limits().store.commit, levels(2, 1));
    disk.page_header = Some(FIXTURE_LAST);
    let events = disk.open(&mut store);
    assert_eq!(
        patched(&events),
        [
            (crate::Owner::Application, 1, crate::Counts { records: 7, commits: 2 }),
            (crate::Owner::Jig, 1, crate::Counts { records: 1, commits: 1 }),
            (crate::Owner::Application, 2, crate::Counts { records: 5, commits: 1 }),
        ]
    );
    let expected_sites: Vec<_> = (0..SHELVES).map(|n| (crate::Owner::Application, fixture_key(b's', n))).collect();
    assert_eq!(disk.sites, expected_sites);
    assert_fixture_opened(&events, 4);
    fixture_rows(&mut store, 2, 2);
    assert_sealed(&mut store, FIXTURE_LAST.checked_add(5).expect("last"));
    assert_eq!(disk.bytes(b"active").as_ref(), b"2\n");
    disk.assert_source_unchanged(1);
    // A plain restart reads the format and header from the durable seal too.
    let mut disk = disk.crash(29);
    let mut store = disk.store(false);
    assert_opened(&disk.open(&mut store), FIXTURE_LAST.checked_add(5).expect("last"));
    fixture_rows(&mut store, 2, 2);
    assert_sealed(&mut store, FIXTURE_LAST.checked_add(5).expect("last"));
}
#[test]
fn patch_one_alone_rewrites_held_labels_but_keeps_shelves_at_version_one() {
    let mut disk = Harness::new(&fixture_baseline(None));
    let mut store = fixture_store(&disk, 8, fixtures::store_limits().store.commit, levels(1, 0));
    let events = disk.open(&mut store);
    assert_eq!(patched(&events), [(crate::Owner::Application, 1, crate::Counts { records: 7, commits: 2 })]);
    assert_fixture_opened(&events, 2);
    assert!(disk.sites.is_empty());
    fixture_rows(&mut store, 1, 1);
}
#[test]
fn two_put_pages_take_three_shelf_commits_and_match_the_one_page_records() {
    let items = fixture_baseline(None);
    let mut whole = Harness::new(&items);
    let mut store = fixture_store(&whole, 8, fixtures::store_limits().store.commit, levels(2, 1));
    whole.open(&mut store);
    let expected = fixture_rows(&mut store, 2, 2);
    let mut paged = Harness::new(&items);
    let mut store = fixture_store(&paged, 2, fixtures::store_limits().store.commit, levels(2, 1));
    let events = paged.open(&mut store);
    assert_eq!(
        patched(&events),
        [
            (crate::Owner::Application, 1, crate::Counts { records: 7, commits: 4 }),
            (crate::Owner::Jig, 1, crate::Counts { records: 1, commits: 1 }),
            (crate::Owner::Application, 2, crate::Counts { records: 5, commits: 3 }),
        ]
    );
    assert_fixture_opened(&events, 8);
    assert_eq!(fixture_rows(&mut store, 2, 2), expected);
}
fn assert_refused(events: &[Event], store: &Store) {
    assert!(
        match events.last() {
            Some(Event::OpenRefused { owner, why: OpenRefusal::Failed }) => *owner == Token::new(1),
            _ => false,
        },
        "{events:?}"
    );
    assert_eq!(store.phase, Phase::Stopped);
}
fn assert_abandoned(mut disk: Harness) {
    disk.assert_source_unchanged(1);
    assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
    let source = disk.source_items(1);
    let mut disk = disk.crash(31);
    let mut store = disk.store(false);
    assert_opened(&disk.open(&mut store), FIXTURE_LAST);
    assert!(!disk.items().iter().any(|item| item.path.starts_with(b"generations/2")));
    assert_eq!(disk.source_items(1), source);
}
#[test]
fn a_shell_without_the_written_site_refuses_and_the_next_start_removes_the_copy() {
    let mut disk = Harness::new(&fixture_baseline(None));
    disk.site_answer = SiteAnswer::NoSite;
    let mut store = fixture_store(&disk, 8, fixtures::store_limits().store.commit, levels(2, 1));
    assert_refused(&disk.open(&mut store), &store);
    assert_eq!(disk.sites.len(), 1);
    assert_abandoned(disk);
}
#[test]
fn an_empty_shelf_refuses_before_any_put_of_its_page_commits() {
    let mut disk = Harness::new(&fixture_baseline(Some((1, 1, 0))));
    let mut store = fixture_store(&disk, 8, fixtures::store_limits().store.commit, levels(2, 1));
    let events = disk.open(&mut store);
    assert_refused(&events, &store);
    assert_eq!(disk.sites.len(), 2, "one successful record then Refused at the next site");
    let key = crate::keys::application(&fixture_key(b's', 0), store.limits.store.key).expect("key");
    assert_eq!(fixtures::get_kv(&mut store, &key).expect("uncommitted page"), shelf(1, 1, &label(2)));
    assert_eq!(patched(&events).len(), 2, "the failed patch never reports Patched");
    assert_abandoned(disk);
}
#[test]
fn a_shelf_at_another_version_stops_the_start_without_changing_the_active_generation() {
    let mut disk = Harness::new(&fixture_baseline(Some((1, 2, 2))));
    let mut store = fixture_store(&disk, 8, fixtures::store_limits().store.commit, levels(2, 1));
    let events = disk.open(&mut store);
    assert_refused(&events, &store);
    assert!(disk.sites.is_empty(), "identity decoding refuses the version before patch two");
    assert_abandoned(disk);
}
#[test]
fn a_site_answer_from_another_owner_refuses_the_open() {
    let mut disk = Harness::new(&fixture_baseline(None));
    disk.site_answer = SiteAnswer::WrongOwner;
    let mut store = fixture_store(&disk, 8, fixtures::store_limits().store.commit, levels(2, 1));
    assert_refused(&disk.open(&mut store), &store);
    assert_abandoned(disk);
}
#[test]
fn a_site_request_without_a_waiting_site_refuses_failed() {
    let mut disk = Disk::new(&baseline());
    let mut store = disk.store_with_chunk(CHUNK);
    assert_opened(&disk.open(&mut store), 1);
    let patcher = Box::new(skein_codec::patch::Patcher::new(skein_codec::patch::Capacity {
        versions: 0,
        patches: 0,
        text: 0,
        bytes: 0,
        nodes: 0,
    }));
    let events = disk.request(&mut store, Request::Site { owner: Token::new(1), patcher, result: Ok(()) });
    assert_refused(&events, &store);
}
#[test]
fn cuts_at_each_page_and_sealing_write_and_sync_reopen_the_unchanged_source() {
    let items = fixture_baseline(None);
    let mut completed = Harness::new(&items);
    let mut store = fixture_store(&completed, 2, fixtures::store_limits().store.commit, levels(2, 1));
    assert_fixture_opened(&completed.open(&mut store), 8);
    let changes = completed.changes.clone();
    for stage in [Stage::Walking, Stage::Sealing] {
        for change in [Change::Write, Change::Sync] {
            assert!(changes.contains(&(stage, change)), "{stage:?} {change:?}");
        }
    }
    for (cut, (stage, change)) in changes.iter().enumerate().filter(|(_, (stage, change))| {
        (match stage {
            Stage::Walking | Stage::Sealing => true,
            _ => false,
        }) && match change {
            Change::Write | Change::Sync => true,
            _ => false,
        }
    }) {
        for after in [false, true] {
            let mut disk = Harness::new(&items);
            disk.cut = Some((cut, after));
            let mut store = fixture_store(&disk, 2, fixtures::store_limits().store.commit, levels(2, 1));
            let events = disk.open(&mut store);
            assert!(disk.halted, "cut {cut} {stage:?} {change:?} after={after}");
            assert!(!events.iter().any(|e| match e {
                Event::Opened { .. } | Event::OpenRefused { .. } => true,
                _ => false,
            }));
            assert_abandoned(disk);
        }
    }
}

#[test]
fn patching_peak_grows_with_kept_texts_and_record_room() {
    let mut limits = fixtures::store_limits();
    limits.store.value = 128;
    let none = crate::patching::phase::worst_case(&limits, &crate::PatchSets::NONE).expect("empty peak");
    let baseline = crate::patching::phase::worst_case(&limits, &BASE).expect("baseline texts");
    let kept = crate::patching::phase::worst_case(&limits, &SETS).expect("kept peak");
    assert!(baseline > none);
    assert!(kept > baseline, "the families' kept texts, plans and machines contribute");
    limits.store.value = 256;
    assert!(crate::patching::phase::worst_case(&limits, &SETS).expect("more room") > kept);
}
#[test]
fn the_store_prices_the_larger_peak_and_worst_case_prices_none() {
    let mut limits = fixtures::store_limits();
    // A large decoded page makes serving dominate; a tiny page and large
    // record machines make patching dominate. Shared maps are counted once.
    for patching_larger in [false, true] {
        limits.store.page.bytes = if patching_larger { 1 } else { 16_778_240 };
        limits.store.value = if patching_larger { 16_777_216 } else { 128 };
        limits.joins = if patching_larger { 0 } else { 32 };
        let (common, serving) = crate::serving_peak(&limits).expect("serving peak");
        let patching = crate::patching::phase::worst_case(&limits, &SETS).expect("patching peak");
        assert_eq!(patching > serving, patching_larger, "both sides of the maximum");
        let total = crate::worst_case_with_patches(&limits, &SETS).expect("total");
        assert_eq!(total.checked_sub(common), Some(if patching_larger { patching } else { serving }));
        assert!(total < common.checked_add(serving).expect("serving").checked_add(patching).expect("both"));
        assert_eq!(crate::worst_case(&limits), crate::worst_case_with_patches(&limits, &crate::PatchSets::NONE));
    }
}

#[test]
fn a_plan_without_any_matching_records_reports_zero_counts_and_still_seals() {
    let mut disk = Harness::new(&baseline());
    let mut store = fixture_store(&disk, 8, fixtures::store_limits().store.commit, levels(2, 1));
    let events = disk.open(&mut store);
    let zero = crate::Counts { records: 0, commits: 0 };
    assert_eq!(
        patched(&events),
        [(crate::Owner::Application, 1, zero), (crate::Owner::Jig, 1, zero), (crate::Owner::Application, 2, zero)]
    );
    assert_opened(events.get(3..).expect("open after patches"), 2);
    assert_sealed(&mut store, 2);
}
#[test]
fn one_put_past_the_commit_byte_bound_stops_before_submitting_its_page() {
    let mut disk = Harness::new(&fixture_baseline(None));
    let mut store = fixture_store(&disk, 8, fixtures::store_limits().store.commit, levels(2, 1));
    disk.site_commit_limit = Some(65); // 26 + (tagged key 4 + shelf 27 + put overhead 9) - 1.
    let events = disk.open(&mut store);
    assert_refused(&events, &store);
    assert_eq!(patched(&events).len(), 2, "patch two never commits its first page");
    assert_eq!(store.stores.store.last(), FIXTURE_LAST.checked_add(3).expect("prior patches"));
    assert_abandoned(disk);
}
#[test]
fn page_and_sealing_write_faults_refuse_and_leave_the_source_active() {
    let items = fixture_baseline(None);
    for stage in [Stage::Walking, Stage::Sealing] {
        let mut disk = Harness::new(&items);
        disk.fault_at = Some(stage);
        let mut store = fixture_store(&disk, 8, fixtures::store_limits().store.commit, levels(2, 1));
        let events = disk.open(&mut store);
        assert_refused(&events, &store);
        assert_eq!(disk.fault_at, None, "fault reached {stage:?}");
        assert_abandoned(disk);
    }
}

#[test]
fn a_full_byte_page_defers_the_next_finished_record_without_repeating_its_site() {
    let items = fixture_baseline(None);
    let mut whole = Harness::new(&items);
    let mut store = fixture_store(&whole, 8, fixtures::store_limits().store.commit, levels(2, 1));
    whole.open(&mut store);
    let expected = fixture_rows(&mut store, 2, 2);
    let mut disk = Harness::new(&items);
    // Exactly three shelf puts: 26 + 3 * (tagged key 4 + value 27 + 9).
    disk.site_commit_limit = Some(146);
    let mut store = fixture_store(&disk, 8, fixtures::store_limits().store.commit, levels(2, 1));
    let events = disk.open(&mut store);
    assert_eq!(
        patched(&events).last(),
        Some(&(crate::Owner::Application, 2, crate::Counts { records: 5, commits: 2 }))
    );
    assert_fixture_opened(&events, 5);
    assert_eq!(disk.sites.len(), usize::try_from(SHELVES).expect("shelves"));
    assert_eq!(fixture_rows(&mut store, 2, 2), expected);
}
