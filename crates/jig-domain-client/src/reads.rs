//! Reads wait for one terminal and retain abandoned obligations until it arrives.
//! A full slab refuses in place. A receipt from an old generation is discarded,
//! never routed into a new page. See client/domain/client.md, 7.
use crate::{boundary::Request, contract::Read, facts::Fact, objects::Source};
use skein_lib::{Id, Map, Queue, Slab, Token};

/// One read obligation, whose document is expected by its source.
#[derive(Debug)]
pub struct Reading {
    pub what: Read,
    pub generation: u32,
    pub source: Source,
    pub abandoned: bool,
    pub terminal: bool,
}
/// The routing facts consumed with the one read terminal.
#[derive(Debug)]
pub struct Receipt {
    pub what: Read,
    pub generation: u32,
    pub source: Source,
    pub abandoned: bool,
}
/// A bounded slab and registry of read obligations.
#[derive(Debug)]
pub struct Reads {
    pub(crate) slab: Slab<Reading>,
    pub(crate) ids: Map<Id<Reading>, ()>,
}
impl Reads {
    /// Allocates one configured read slot per obligation.
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { slab: Slab::with_capacity(capacity), ids: Map::with_capacity(capacity) }
    }
    /// Starts one read, or reports refusal without retaining a partial obligation.
    pub fn ask(
        &mut self,
        generation: u32,
        source: Source,
        what: Read,
        out: &mut Queue<Request>,
    ) -> Option<Id<Reading>> {
        let reading = Reading { what, generation, source, abandoned: false, terminal: false };
        match self.slab.insert(reading) {
            Ok(id) => {
                let previous = self.ids.insert(id, ()).expect("registry fits its paired slab");
                assert!(previous.is_none(), "new read is not registered");
                let what = self.slab.get(id).expect("inserted read exists").what.clone();
                out.push(Request::Read { read: id.token(), what });
                Some(id)
            }
            Err(_) => {
                out.push(Request::Fact { fact: Fact::Refused { which: crate::contract::Limit::Reads } });
                None
            }
        }
    }
    /// Whether this page already has the same read in flight.
    #[must_use]
    pub fn pending(&self, what: &Read) -> bool {
        for (id, ()) in &self.ids {
            let reading = self.slab.get(*id).expect("registered read exists");
            if !reading.abandoned && reading.what == *what {
                return true;
            }
        }
        false
    }
    /// Marks old-page reads abandoned; no terminal is invented by leaving a page.
    pub fn abandon(&mut self, generation: u32) {
        for (id, ()) in &self.ids {
            let reading = self.slab.get_mut(*id).expect("registered read exists");
            if reading.generation == generation {
                reading.abandoned = true;
            }
        }
    }
    /// Discards pre-change facts while retaining each read until its terminal arrives.
    pub(crate) fn abandon_matching(&mut self, generation: u32, what: &Read) {
        for (id, ()) in &self.ids {
            let reading = self.slab.get_mut(*id).expect("registered read exists");
            if reading.generation == generation && reading.what == *what {
                reading.abandoned = true;
            }
        }
    }
    /// Takes one routing receipt and retires its obligation, ignoring duplicates.
    pub fn terminal(&mut self, token: Token) -> Option<Receipt> {
        let id = Id::from_token(token);
        let reading = self.slab.get_mut(id)?;
        if reading.terminal {
            return None;
        }
        reading.terminal = true;
        let what = core::mem::replace(&mut reading.what, Read::Frame);
        let receipt =
            Receipt { what, generation: reading.generation, source: reading.source, abandoned: reading.abandoned };
        let old = self.ids.remove(&id);
        assert!(old.is_some(), "terminal read was registered");
        self.slab.retire(id);
        Some(receipt)
    }
    /// Reclaims only reads whose terminals arrived.
    pub fn reclaim(&mut self) {
        self.slab.reclaim();
    }
}
