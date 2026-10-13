//! Ordered authentication, payload and capacity checks (protocol/channel.md, 4.1).
use crate::Component;
use jig_protocol_worker_wire::{Credential, Reason};
use skein_io::digest::{Digest, digest};
use skein_lib::{Reader, Token};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Verdict {
    Accept { worker: u32 },
    Refuse(Reason),
    Replace { worker: u32, older: Token },
}
impl Component {
    pub(crate) fn judge(&self, credential: &[u8]) -> Verdict {
        let Ok(credential) = Credential::decode(&self.codec, &mut Reader::new(credential)) else {
            return Verdict::Refuse(Reason::Unauthorized);
        };
        let supplied = digest(credential.secret().expose());
        let mut found = None;
        for (index, worker) in self.workers.iter().enumerate() {
            if worker.name.as_ref() == credential.name() && same(&worker.digest, &supplied) {
                found = Some(u32::try_from(index).expect("bounded workers"));
            }
        }
        let Some(worker) = found else { return Verdict::Refuse(Reason::Unauthorized) };
        if credential.payload() != self.payload {
            return Verdict::Refuse(Reason::Application);
        }
        let mut open = 0_u32;
        for (token, connection) in &self.connections {
            if connection.worker == Some(worker) {
                return Verdict::Replace { worker, older: *token };
            }
            if connection.worker.is_some() {
                open = open.saturating_add(1);
            }
        }
        if open >= self.limits.workers { Verdict::Refuse(Reason::Busy) } else { Verdict::Accept { worker } }
    }
}
pub(crate) fn same(left: &Digest, right: &Digest) -> bool {
    let mut different = 0_u64;
    for index in 0..4 {
        let left = left.0.get(index).expect("four digest words");
        let right = right.0.get(index).expect("four digest words");
        different |= left ^ right;
    }
    different == 0
}
