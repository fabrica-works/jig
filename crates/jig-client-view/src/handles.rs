//! Presentation handles and the local place each result belongs to (view.md, 18.2).
//! The client owns transport obligations, reopening and abandoned reads.
use crate::{Request, objects::Source};
use jig_client::{
    Asker, Event, Handle, Readable, Watchable,
    contract::{Read, Subject},
};
use skein_lib::{Map, Queue};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Owner {
    Frame,
    Page,
    Application,
    Sections,
}
#[derive(Debug)]
pub(crate) struct Watched {
    pub subject: Subject,
    pub owner: Owner,
}
#[derive(Clone, Debug)]
pub(crate) struct Reading {
    pub what: Read,
    pub source: Source,
}
#[derive(Debug)]
pub(crate) struct Handles {
    pub watches: Map<Handle, Watched>,
    pub reads: Map<Handle, Reading>,
    pub next: u64,
    pub anchor: Option<Handle>,
}
impl Handles {
    pub fn new(watches: u32, reads: u32) -> Self {
        Self { watches: Map::with_capacity(watches), reads: Map::with_capacity(reads), next: 0, anchor: None }
    }
    fn fresh(&mut self) -> Option<Handle> {
        let number = self.next;
        self.next = number.checked_add(1)?;
        Some(Handle { asker: Asker::JIG, number })
    }
    pub fn watch(&mut self, subject: Subject, owner: Owner, out: &mut Queue<Request>) -> Option<Handle> {
        if self.watches.len() == self.watches.capacity() {
            return None;
        }
        let handle = self.fresh()?;
        if owner != Owner::Frame {
            self.anchor.get_or_insert(handle);
        }
        out.push(Request::Client { asked: Event::Watch { handle, subject: Watchable::Jig(subject.clone()) } });
        self.watches.insert(handle, Watched { subject, owner }).expect("watch admission checked");
        Some(handle)
    }
    pub fn read(&mut self, source: Source, what: Read, out: &mut Queue<Request>) -> Option<Handle> {
        if self.reads.len() == self.reads.capacity() {
            return None;
        }
        let handle = self.fresh()?;
        self.anchor.get_or_insert(handle);
        out.push(Request::Client { asked: Event::Read { handle, read: Readable::Jig(what.clone()) } });
        self.reads.insert(handle, Reading { what, source }).expect("read admission checked");
        Some(handle)
    }
    pub fn received(&mut self, handle: Handle) -> Option<Reading> {
        self.reads.remove(&handle)
    }
    pub fn pending(&self, what: &Read) -> bool {
        for (_, read) in &self.reads {
            if read.what == *what {
                return true;
            }
        }
        false
    }
    pub fn forget_read(&mut self, what: &Read, out: &mut Queue<Request>) {
        for _ in 0..self.reads.len() {
            let mut found = None;
            for (handle, read) in &self.reads {
                if read.what == *what {
                    found = Some(*handle);
                    break;
                }
            }
            let Some(handle) = found else {
                break;
            };
            self.reads.remove(&handle);
            out.push(Request::Client { asked: Event::Unwatch { handle } });
        }
    }
    pub fn leave_reads(&mut self, out: &mut Queue<Request>) {
        for (handle, _) in &self.reads {
            out.push(Request::Client { asked: Event::Unwatch { handle: *handle } });
        }
        for _ in 0..self.reads.len() {
            let (handle, _) = self.reads.first().expect("counted read exists");
            let handle = *handle;
            self.reads.remove(&handle);
        }
    }
}

pub(crate) fn handle(token: skein_lib::Token) -> Handle {
    Handle { asker: Asker::JIG, number: token.raw() }
}
