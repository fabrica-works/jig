//! File requests answered by skein's own crashing machine.
use jig_protocol_store::{Arrived, Root};
use skein_fake_machine::{How, Machine, Opened, Refusal, kernel_error};
use skein_io::{file, kernel::Fd};
use skein_lib::Token;
use std::collections::BTreeMap;

/// The transcripts' filesystem with its durable file lengths.
pub struct Files {
    machine: Machine,
    root: Opened,
    handles: BTreeMap<Token, Opened>,
}
impl Default for Files {
    fn default() -> Self {
        Self::new()
    }
}
impl Files {
    #[must_use]
    pub fn new() -> Files {
        let mut machine = Machine::new();
        let root = machine.lay(&[]);
        Files { machine, root, handles: BTreeMap::from([(Token::new(root.raw()), root)]) }
    }
    #[must_use]
    pub fn root(&mut self) -> Root {
        Root { token: Token::new(self.root.raw()), fd: fd(self.root) }
    }
    pub fn crash(&mut self, seed: u64) {
        self.machine.crash(seed);
        self.root = self.machine.reopen_root(0);
        self.handles.clear();
        self.handles.insert(Token::new(self.root.raw()), self.root);
    }
    /// Answer every operation the store's persistent file face reaches.
    #[must_use]
    pub fn answer(&mut self, request: file::Request) -> Arrived {
        let mut descriptor = None;
        let event = match request {
            file::Request::MakeDirectory { owner, root, path, mode } => {
                let result = self.machine.make_directory(self.handle(root), &path, mode);
                terminal(owner, result, file::Event::Made { owner })
            }
            file::Request::Create { owner, root, name, mode } => {
                self.open(owner, root, &name, How::Create { mode }, &mut descriptor)
            }
            file::Request::CreateNoFollow { owner, root, name, mode } => {
                self.open(owner, root, &name, How::CreateNoFollow { mode }, &mut descriptor)
            }
            file::Request::OpenRead { owner, root, name } => self.open(owner, root, &name, How::Read, &mut descriptor),
            file::Request::OpenReadNoFollow { owner, root, name } => {
                self.open(owner, root, &name, How::ReadNoFollow, &mut descriptor)
            }
            file::Request::OpenWrite { owner, root, name } => {
                self.open(owner, root, &name, How::Write, &mut descriptor)
            }
            file::Request::OpenWriteNoFollow { owner, root, name } => {
                self.open(owner, root, &name, How::WriteNoFollow, &mut descriptor)
            }
            file::Request::OpenDirectory { owner, root, name, no_follow } => {
                let how = if no_follow { How::DirectoryNoFollow } else { How::Directory };
                self.open(owner, root, &name, how, &mut descriptor)
            }
            file::Request::WriteAt { owner, file, offset, bytes } => {
                let result = self.machine.write(self.handle(file), offset, &bytes);
                terminal(owner, result, file::Event::Written { owner })
            }
            file::Request::ReadAt { owner, file, offset, max } => {
                match self.machine.read(self.handle(file), offset, max) {
                    Ok(bytes) => file::Event::Read { owner, bytes: bytes.into_boxed_slice() },
                    Err(error) => failed(owner, error),
                }
            }
            file::Request::Cut { owner, file, length } => {
                let result = self.machine.truncate(self.handle(file), length);
                terminal(owner, result, file::Event::Cut { owner })
            }
            file::Request::Sync { owner, file } => {
                self.machine.sync(self.handle(file));
                file::Event::Synced { owner }
            }
            file::Request::SyncDirectory { owner, root } => {
                self.machine.sync(opened(root));
                file::Event::Synced { owner }
            }
            file::Request::Close { owner, file } => {
                let handle = self.handles.remove(&file).expect("close one open file");
                self.machine.close(handle);
                file::Event::Closed { owner }
            }
            file::Request::Remove { owner, root, name } => {
                let result = self.machine.remove(opened(root), &name, false);
                terminal(owner, result, file::Event::Removed { owner })
            }
            file::Request::Rename { owner, root, from, to } => {
                let root = opened(root);
                let result = self.machine.rename(root, &from, root, &to);
                terminal(owner, result, file::Event::Renamed { owner })
            }
            file::Request::OpenPrivate { .. }
            | file::Request::Load { .. }
            | file::Request::Scan { .. }
            | file::Request::Store { .. }
            | file::Request::Stat { .. }
            | file::Request::List { .. } => {
                panic!("the store uses positioned operations and derived paths")
            }
        };
        Arrived { event, fd: descriptor }
    }
    fn handle(&self, token: Token) -> Opened {
        *self.handles.get(&token).expect("operation owns an open file")
    }
    fn open(&mut self, owner: Token, root: Fd, name: &[u8], how: How, descriptor: &mut Option<Fd>) -> file::Event {
        match self.machine.open(opened(root), name, how) {
            Ok(handle) => {
                let token = Token::new(handle.raw());
                self.handles.insert(token, handle);
                *descriptor = Some(fd(handle));
                file::Event::Opened { owner, file: token, len: self.machine.stat(handle).size }
            }
            Err(error) => failed(owner, error),
        }
    }
}
fn opened(fd: Fd) -> Opened {
    Opened::new(u64::try_from(fd.raw()).expect("fake descriptor is a handle"))
}
fn fd(handle: Opened) -> Fd {
    Fd::new(i32::try_from(handle.raw()).expect("bounded world handles fit a descriptor"))
}
fn failed(owner: Token, refusal: Refusal) -> file::Event {
    file::Event::Failed { owner, error: kernel_error(refusal), committed: false, residue: None }
}
fn terminal(owner: Token, result: Result<(), Refusal>, success: file::Event) -> file::Event {
    match result {
        Ok(()) => success,
        Err(error) => failed(owner, error),
    }
}
