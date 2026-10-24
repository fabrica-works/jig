//! A calm disk, three real file drivers and request-boundary power cuts.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::arithmetic_side_effects,
    reason = "the simulated disk fixture is ordinary Rust (skein's programming-model.md, 10.2)"
)]
use super::{APPLICATION, INSTALLATION, below, env, store_limits};
use crate::{Arrived, Below, Event, Io, Request, Root, Store};
use skein_fake_machine::{How, Is, Item, Machine, Opened, serve};
use skein_io::{
    file,
    file_layer::{self, FileIo},
    kernel::{Complete, Kind, Submit},
};
use skein_lib::{Queue, Token};
use skein_sim::{Config, Handle, Pid, Sim};

const QUEUE: u32 = 128;
const SEED: u64 = 1;
type Contents = Vec<(Box<[u8]>, Kind, u32, Box<[u8]>)>;
#[derive(Debug, PartialEq, Eq)]
pub enum Seen {
    Create { name: Box<[u8]> },
    Read { max: u32 },
    Write { bytes: u32 },
    SyncFile,
    Rename { from: Box<[u8]>, to: Box<[u8]> },
    Lock,
    Load,
    Make(Box<[u8]>),
    Private(Box<[u8]>),
    Open,
    Store { path: Box<[u8]>, bytes: Box<[u8]>, expected: file::Expect },
    Remove { name: Box<[u8]>, directory: bool },
    Sync,
    Close,
    Other,
}
fn seen(request: &file::Request) -> Seen {
    match request {
        file::Request::CreateNoFollow { name, .. } => Seen::Create { name: name.clone() },
        file::Request::ReadAt { max, .. } => Seen::Read { max: *max },
        file::Request::WriteAt { bytes, .. } => {
            Seen::Write { bytes: u32::try_from(bytes.len()).expect("bounded write") }
        }
        file::Request::Sync { .. } => Seen::SyncFile,
        file::Request::Rename { from, to, .. } => Seen::Rename { from: from.clone(), to: to.clone() },
        file::Request::Lock { .. } => Seen::Lock,
        file::Request::Load { .. } => Seen::Load,
        file::Request::MakeDirectory { path, .. } => Seen::Make(path.clone()),
        file::Request::OpenPrivate { path, .. } => Seen::Private(path.clone()),
        file::Request::OpenDirectory { .. } => Seen::Open,
        file::Request::Store { path, bytes, expected, .. } => {
            Seen::Store { path: path.clone(), bytes: bytes.clone(), expected: *expected }
        }
        file::Request::Remove { name, directory, .. } => Seen::Remove { name: name.clone(), directory: *directory },
        file::Request::SyncDirectory { .. } => Seen::Sync,
        file::Request::Close { .. } => Seen::Close,
        file::Request::Create { .. }
        | file::Request::OpenRead { .. }
        | file::Request::OpenReadNoFollow { .. }
        | file::Request::OpenWrite { .. }
        | file::Request::OpenWriteNoFollow { .. }
        | file::Request::Scan { .. }
        | file::Request::Stat { .. }
        | file::Request::Cut { .. }
        | file::Request::List { .. }
        | file::Request::ListPage { .. } => Seen::Other,
    }
}
pub struct Disk {
    machine: Machine,
    handle: Opened,
    sim: Sim,
    pid: Pid,
    root: Root,
    io: [FileIo; 3],
    seeded: crate::Seeded,
    cut: Option<u32>,
    operations: u32,
    starts: u64,
    requests: Vec<(usize, Seen)>,
}
fn drivers(fd: skein_io::kernel::Fd) -> [FileIo; 3] {
    core::array::from_fn(|_| {
        let limits = store_limits();
        let mut io = FileIo::new(8, limits.store.chunk, 32, limits.store.deadline).with_effective_user(0);
        assert_eq!(io.adopt_root(fd), Some(Token::new(1)));
        io
    })
}
impl Disk {
    pub fn new(items: &[Item]) -> Disk {
        let mut machine = Machine::for_user(0);
        let handle = machine.lay(items);
        let mut sim = Sim::new(SEED, Config::calm());
        let pid = sim.spawn_process();
        let fd = sim.root(pid, Handle::new(handle.raw()));
        let mut io = drivers(fd);
        let [store, secrets, transcripts] = &mut io;
        let seeded = crate::seed([store, secrets, transcripts], SEED);
        Disk {
            machine,
            handle,
            sim,
            pid,
            root: Root { token: Token::new(1), fd },
            io,
            seeded,
            cut: None,
            operations: 0,
            starts: 0,
            requests: Vec::new(),
        }
    }
    pub fn root(&self) -> Root {
        self.root
    }
    pub fn store(&self) -> Store {
        Store::on_directory(&store_limits(), APPLICATION, &[], self.seeded)
    }
    /// Bound recovery reads for small generation-operation scenes too.
    pub fn store_with_chunk(&self, chunk: u32) -> Store {
        let mut limits = store_limits();
        // kv requires room for the largest key, value and 20-byte frame header
        // in one chunk (skein's kv Limits::is_usable).
        let value = chunk
            .checked_sub(limits.store.key)
            .expect("chunk holds the key")
            .checked_sub(20)
            .expect("chunk holds the frame header");
        limits.store.chunk = chunk;
        limits.store.value = value;
        limits.secrets.chunk = chunk;
        limits.secrets.value = value;
        Store::on_directory(&limits, APPLICATION, &[], self.seeded)
    }
    pub fn open(&mut self, store: &mut Store) -> Box<[Event]> {
        self.request(store, Request::Open { owner: Token::new(1), root: self.root, installation: INSTALLATION })
    }
    pub fn request(&mut self, store: &mut Store, request: Request) -> Box<[Event]> {
        let mut above = Queue::with_capacity(QUEUE);
        let mut below = below();
        crate::down(store, &env(), request, &mut above, &mut below);
        for _ in 0_u32..10_000 {
            crate::fire(store, &env(), &mut above, &mut below);
            if !self.drive_one(&mut below, |io, below| crate::up(store, &env(), io, &mut above, below)) {
                break;
            }
        }
        super::events(&mut above)
    }
    /// Drive sub-machines too, without routing their terminals through Store.
    pub fn drive(&mut self, below: &mut Below, mut deliver: impl FnMut(Io, &mut Below)) {
        for _ in 0_u32..10_000 {
            if !self.drive_one(below, &mut deliver) {
                return;
            }
        }
        panic!("disk did not settle");
    }
    fn drive_one(&mut self, below: &mut Below, mut deliver: impl FnMut(Io, &mut Below)) -> bool {
        if self.cut.is_some_and(|cut| self.operations >= cut) {
            return false;
        }
        let next = below
            .store
            .pop()
            .map(|r| (0, r))
            .or_else(|| below.secrets.pop().map(|r| (1, r)))
            .or_else(|| below.transcripts.pop().map(|r| (2, r)));
        let Some((driver, request)) = next else {
            return false;
        };
        let arrived = self.file(driver, request);
        self.operations += 1;
        let io = match driver {
            0 => Io::Store(arrived),
            1 => Io::Secrets(arrived),
            2 => Io::Transcripts(arrived),
            _ => unreachable!(),
        };
        deliver(io, below);
        true
    }
    pub fn file(&mut self, driver: usize, request: file::Request) -> Arrived {
        self.requests.push((driver, seen(&request)));
        let io = &mut self.io[driver];
        assert!(io.takes());
        let mut events = Queue::with_capacity(QUEUE);
        let mut subs: Queue<Submit> = Queue::with_capacity(QUEUE);
        let mut completions: Queue<Complete> = Queue::with_capacity(QUEUE);
        file_layer::down(io, env().now, request, &mut events, &mut subs);
        for _ in 0_u32..100 {
            if !subs.is_empty() {
                self.sim.submit(self.pid, &mut subs);
                serve(&mut self.machine, &mut self.sim);
            }
            self.sim.reap(self.pid, &mut completions);
            while let Some(complete) = completions.pop() {
                file_layer::up(io, complete, &mut events, &mut subs);
            }
            if io.takes() {
                break;
            }
        }
        assert!(io.takes(), "calm operation settled");
        let event = events.pop().expect("one file terminal");
        assert!(events.is_empty());
        let fd = if let file::Event::Opened { file, .. } = &event { io.descriptor(*file) } else { None };
        Arrived { event, fd }
    }
    /// Another descriptor of the same deployment, with its own driver token.
    pub fn second_root(&mut self) -> Root {
        let handle = self.machine.open(self.handle, b".", How::Directory).expect("another open");
        let fd = self.sim.root(self.pid, Handle::new(handle.raw()));
        let token = self.io[0].adopt_root(fd).expect("second root fits");
        Root { token, fd }
    }
    pub fn cut_at(&mut self, operation: u32) {
        self.cut = Some(operation);
    }
    pub fn operations(&self) -> u32 {
        self.operations
    }
    pub fn requests(&self) -> &[(usize, Seen)] {
        &self.requests
    }
    pub fn crash(&mut self, seed: u64) {
        self.machine.crash(seed);
        self.handle = self.machine.reopen_root(0);
        self.sim = Sim::new(seed, Config::calm());
        self.pid = self.sim.spawn_process();
        let fd = self.sim.root(self.pid, Handle::new(self.handle.raw()));
        self.root = Root { token: Token::new(1), fd };
        self.starts += 1;
        self.io = drivers(fd);
        let [store, secrets, transcripts] = &mut self.io;
        self.seeded =
            crate::seed([store, secrets, transcripts], seed ^ 0xd1b5_4a32_d192_ed03_u64.wrapping_mul(self.starts));
        self.cut = None;
        self.operations = 0;
        self.requests.clear();
    }
    pub fn bytes(&mut self, path: &[u8]) -> Box<[u8]> {
        let file = self.machine.open(self.handle, path, How::ReadNoFollow).expect("file stands");
        let bytes = self.machine.read(file, 0, u32::MAX).expect("read file");
        self.machine.close(file);
        bytes.into_boxed_slice()
    }
    pub fn tree(&mut self) -> Vec<(Box<[u8]>, Kind, u32)> {
        let mut tree = Vec::new();
        self.walk(b"", &mut tree);
        tree.sort();
        tree
    }
    fn walk(&mut self, path: &[u8], tree: &mut Vec<(Box<[u8]>, Kind, u32)>) {
        let directory = self
            .machine
            .open(self.handle, if path.is_empty() { b"." } else { path }, How::DirectoryNoFollow)
            .expect("directory stands");
        loop {
            let entries = self.machine.list(directory, 32, 8192).expect("list directory");
            if entries.is_empty() {
                break;
            }
            for (is, name) in entries {
                let name = if path.is_empty() { name.to_vec() } else { [path, b"/", &name].concat() };
                let file = self.machine.open(self.handle, &name, How::PathNoFollow).expect("entry stands");
                let mode = self.machine.stat(file).mode;
                self.machine.close(file);
                let kind = match is {
                    Is::Directory => Kind::Directory,
                    Is::File => Kind::File,
                    Is::Link => Kind::Symlink,
                    Is::Fifo | Is::Device => Kind::Other,
                };
                tree.push((name.clone().into_boxed_slice(), kind, mode));
                if is == Is::Directory {
                    self.walk(&name, tree);
                }
            }
        }
        self.machine.close(directory);
    }
    /// Capture once, then lay a fresh machine for each independent power cut.
    pub fn items(&mut self) -> Vec<Item> {
        self.contents()
            .into_iter()
            .map(|(path, kind, mode, bytes)| match kind {
                Kind::Directory => Item::directory(&path).mode(mode),
                Kind::File => Item::file(&path, &bytes).mode(mode),
                Kind::Symlink | Kind::Other => panic!("generation fixture holds only directories and files"),
            })
            .collect()
    }
    /// Include bytes when proving a refusal preserved everything.
    pub fn contents(&mut self) -> Contents {
        self.tree()
            .into_iter()
            .map(|(path, kind, mode)| {
                let bytes = if kind == Kind::File { self.bytes(&path) } else { Box::new([]) };
                (path, kind, mode, bytes)
            })
            .collect()
    }
}
