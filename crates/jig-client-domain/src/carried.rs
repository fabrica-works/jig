//! Requests carrying a value, beside the table (client/domain/client.md, 6):
//! sent once under the name their caller gives, never kept nor sent again.
use crate::{
    Ask, Domain, Down, Key, Limits, Outcome, Pending, Progress, Request, Told,
    domain::{down, tell},
    requests::{declared, pending},
};
use skein_lib::{Env, Queue, RequestEnvelope, RequestKey, Secret, Token};
/// One request carrying a value, in flight; its value's digest alone stays,
/// to tell a joined `Carry` from a key conflict.
#[derive(Debug)]
pub(crate) struct Carrying {
    attempt: Token,
    name: RequestKey,
    request: Ask,
    party: u64,
    digest: Option<[u8; 32]>,
}
impl Carrying {
    pub(crate) fn pending(&self) -> Pending {
        Pending { progress: Progress::InFlight, ..pending(self.party, &self.request) }
    }
}
impl Domain {
    #[expect(clippy::manual_map, reason = "step code matches optional values without closures")]
    pub(crate) fn carry(
        &mut self,
        env: &Env<Limits>,
        key: Key,
        name: RequestKey,
        request: Ask,
        value: Option<Secret>,
        out: &mut Queue<Request>,
    ) {
        if let Ask::Family { record, .. } = &request {
            self.family(record);
        }
        let digest = match &value {
            Some(value) => Some(skein_lib::sha256::digest(value.expose())),
            None => None,
        };
        if let Some(held) = self.carried.get(&key) {
            assert!(held.request == request && held.name == name && held.digest == digest, "request key conflict");
            return;
        }
        assert!(!self.requests.contains_key(&key), "request key conflict");
        let outcome = if self.table.owner(name).is_some() || self.carrying(name) { Some(Outcome::Held) } else { None };
        let party = match self.identity() {
            Some((party, _, true)) if self.served(&request) => party,
            Some(_) | None => {
                tell(out, Told::Answered { key, outcome: outcome.unwrap_or(Outcome::Unsent) });
                return;
            }
        };
        if let Some(outcome) = outcome {
            tell(out, Told::Answered { key, outcome });
            return;
        }
        if !self.room(&env.limits, &request, value.as_ref()) {
            tell(out, Told::Answered { key, outcome: Outcome::Limit });
            return;
        }
        let attempt = Token::new(self.next_carried);
        self.next_carried = self.next_carried.checked_add(1).expect("carried attempts never wrap");
        down(out, Down::Send { attempt, key: name, request: request.clone(), value });
        self.carried
            .insert(key, Carrying { attempt, name, request, party, digest })
            .expect("room for a carried request");
        tell(out, Told::Pending { key });
    }
    /// Whether a request carrying a value holds this engine key.
    pub(crate) fn carrying(&self, name: RequestKey) -> bool {
        for (_, carrying) in &self.carried {
            if carrying.name == name {
                return true;
            }
        }
        false
    }
    pub(crate) fn carried_by(&self, attempt: Token) -> Option<Key> {
        for (key, carrying) in &self.carried {
            if carrying.attempt == attempt {
                return Some(*key);
            }
        }
        None
    }
    /// Its one answer: the engine's, or the client's own unsent or lost.
    pub(crate) fn carried_answered(
        &mut self,
        key: Key,
        envelope: RequestEnvelope,
        answer: Option<crate::Answered>,
        out: &mut Queue<Request>,
    ) {
        self.carried.remove(&key);
        let outcome = match (envelope, answer) {
            (
                RequestEnvelope::Final,
                Some(crate::Answered::Jig(crate::contract::Answer::Done(crate::contract::Outcome::Made {
                    what: crate::contract::Made::MovingOut { record, .. },
                }))),
            ) if record.is_empty() => Outcome::Lost,
            (RequestEnvelope::Final, Some(answer)) => Outcome::Answered(answer),
            (RequestEnvelope::Again | RequestEnvelope::SignedOut, _) => Outcome::Unsent,
            (RequestEnvelope::Final | RequestEnvelope::Lost, _) => Outcome::Lost,
        };
        tell(out, Told::Answered { key, outcome });
    }
    fn served(&self, request: &Ask) -> bool {
        let Ask::Family { record, .. } = request else { return true };
        self.family(record).served
    }
    /// Within the client's bounds: requests carrying a value are counted
    /// and their asks' bytes summed apart from the table's, and a value is
    /// at most the wire's bound.
    fn room(&self, limits: &Limits, request: &Ask, value: Option<&Secret>) -> bool {
        let value_bytes = match value {
            Some(value) => value.len(),
            None => 0,
        };
        if self.carried.len() == limits.requests.requests || value_bytes > crate::VALUE_BYTES {
            return false;
        }
        let mut bytes = declared(limits, request);
        for (_, carrying) in &self.carried {
            bytes = bytes.saturating_add(declared(limits, &carrying.request));
        }
        bytes <= limits.requests.bytes
    }
}
