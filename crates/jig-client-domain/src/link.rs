//! Stream state, confirmed through the own watch (client/domain/client.md, 8.2).
use crate::{Domain, Link, Request, Told, Watchable, contract, domain::tell};
use skein_lib::Queue;
impl Domain {
    #[must_use]
    pub fn link(&self) -> Link {
        self.link
    }
    #[must_use]
    pub fn party(&self) -> Option<u64> {
        for (_, watch) in &self.watches {
            if watch.own
                && let Some(crate::Latest::Jig(contract::Delivery::Frame(frame))) = &watch.latest
            {
                return Some(frame.party.number);
            }
        }
        None
    }
    pub(crate) fn set_link(&mut self, link: Link, out: &mut Queue<Request>) {
        if self.link != Link::OutOfDate && self.link != link {
            self.link = link;
            tell(out, Told::Link { link });
        }
    }
    pub(crate) fn signed_out(&mut self, out: &mut Queue<Request>) {
        self.scratch.clear();
        for (handle, held) in &self.handles {
            if held.state != crate::State::Ended {
                self.scratch.push(*handle).expect("bounded handles");
            }
        }
        for index in 0..self.scratch.len() {
            let handle = *self.scratch.get(index).expect("selected handle");
            let held = self.handles.get_mut(&handle).expect("selected handle exists");
            held.state = crate::State::Ended;
            if let Some(watch) = held.watch.take() {
                held.previous.clone_from(&self.watches.get(&watch).expect("handle's watch").latest);
            }
            tell(out, Told::Ended { handle, why: crate::Why::SignedOut });
        }
        self.scratch.clear();
        while let Some((key, _)) = (&self.watches).into_iter().next() {
            let key = *key;
            self.watches.remove(&key);
        }
        while let Some((key, _)) = (&self.reads).into_iter().next() {
            let key = *key;
            self.reads.remove(&key);
        }
        self.table.link(false);
        self.table.confirmed(None);
        self.set_link(Link::SignedOut, out);
    }
    pub(crate) fn signed_in(&mut self, out: &mut Queue<Request>) {
        if self.link != Link::SignedOut {
            self.signed_out(out);
        }
        self.set_link(Link::Starting, out);
        for (_, watch) in &self.watches {
            if watch.own {
                return;
            }
        }
        self.open(Watchable::Jig(contract::Subject::Frame), true, out);
    }
}
