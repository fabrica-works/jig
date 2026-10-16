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
    pub(crate) fn signed_in(&mut self, out: &mut Queue<Request>) {
        self.set_link(Link::Starting, out);
        for (_, watch) in &self.watches {
            if watch.own {
                return;
            }
        }
        self.open(Watchable::Jig(contract::Subject::Frame), true, out);
    }
}
