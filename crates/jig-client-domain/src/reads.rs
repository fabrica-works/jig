//! One terminal page per handle's current read (client/domain/client.md, 7).
use crate::{
    Domain, Down, Handle, Limits, Readable, Readout, Request, Told,
    domain::{down, tell},
};
use skein_lib::{Env, Queue, Token};

impl Domain {
    pub(crate) fn read(&mut self, env: &Env<Limits>, handle: Handle, what: Readable, out: &mut Queue<Request>) {
        self.reads.remove(&handle);
        let refused = match &what {
            Readable::Family(record) => {
                let owner = self.family(record);
                (!owner.served).then_some(Readout::NotServed { family: Some(record.family), served: owner.versions })
            }
            Readable::Jig(_) => None,
        };
        let refused = if self.link == crate::Link::OutOfDate {
            Some(Readout::NotServed { family: None, served: self.jig_versions })
        } else {
            refused
        };
        if let Some(page) = refused {
            tell(out, Told::Page { handle, page });
            return;
        }
        if self.reads.len() == env.limits.reads {
            tell(out, Told::Page { handle, page: Readout::Limit });
            return;
        }
        let read = self.token();
        self.reads.insert(handle, read).expect("read admitted");
        down(out, Down::Read { read, what });
    }

    pub(crate) fn read_done(&mut self, read: Token, page: Readout, out: &mut Queue<Request>) {
        let mut owner = None;
        for (handle, pending) in &self.reads {
            if *pending == read {
                owner = Some(*handle);
            }
        }
        if let Some(handle) = owner {
            self.reads.remove(&handle);
            tell(out, Told::Page { handle, page });
        }
    }
}
