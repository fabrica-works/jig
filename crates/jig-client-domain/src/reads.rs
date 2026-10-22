//! One terminal page per handle's current read (client/domain/client.md, 7).
use crate::{
    Domain, Down, Handle, Limits, Readable, Readout, Request, Told,
    domain::{down, tell},
};
use skein_lib::{Env, Queue, Token};

#[derive(Debug)]
pub(crate) struct PendingRead {
    pub token: Token,
    pub family: Option<u16>,
}

impl Domain {
    pub(crate) fn read(&mut self, env: &Env<Limits>, handle: Handle, what: Readable, out: &mut Queue<Request>) {
        self.reads.remove(&handle);
        let refused = match &what {
            Readable::Family { record, .. } => {
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
        let family = match &what {
            Readable::Family { record, .. } => Some(record.family),
            Readable::Jig(_) => None,
        };
        self.reads.insert(handle, PendingRead { token: read, family }).expect("read admitted");
        down(out, Down::Read { read, what });
    }

    pub(crate) fn end_reads(&mut self, out: &mut Queue<Request>) {
        for _ in 0..self.reads.len() {
            let (handle, _) = (&self.reads).into_iter().next().expect("pending read");
            let handle = *handle;
            let pending = self.reads.remove(&handle).expect("pending read exists");
            let page = match pending.family {
                Some(family) => Readout::Family { family, record: None },
                None => Readout::Jig(crate::contract::ReadResult::None(crate::contract::ReadNone::SignedOut)),
            };
            tell(out, Told::Page { handle, page });
        }
    }

    pub(crate) fn read_done(&mut self, read: Token, page: Readout, out: &mut Queue<Request>) {
        let mut owner = None;
        for (handle, pending) in &self.reads {
            if pending.token == read {
                owner = Some(*handle);
            }
        }
        if let Some(handle) = owner {
            self.reads.remove(&handle);
            tell(out, Told::Page { handle, page });
        }
    }
}
