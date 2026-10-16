//! Keyed, durable requests and their display state (client/domain/client.md, 6 and 11).
use crate::{
    Ask, Domain, Down, Held, Key, Limits, Outcome, Pending, Progress, Request, Retry, State, Told, contract,
    domain::{down, tell},
};
use skein_lib::{
    Env, List, Queue, RequestAnswered, RequestEnvelope, RequestOut, RequestProgress, RequestRecord, Token,
};
#[derive(Debug)]
pub(crate) struct PendingRequest {
    request: Ask,
    pending: Pending,
    attempt: Option<Token>,
}
impl Domain {
    #[must_use]
    pub fn pending(&self, key: Key) -> Option<Pending> {
        Some(self.requests.get(&key)?.pending)
    }
    fn identity(&self) -> Option<(u64, skein_lib::Duration, bool)> {
        for (_, w) in &self.watches {
            if w.own {
                let Some(crate::Latest::Jig(contract::Delivery::Frame(frame))) = &w.latest else {
                    continue;
                };
                return Some((
                    frame.party.number,
                    frame.retention,
                    w.state == State::Live && self.link == crate::Link::Live,
                ));
            }
        }
        None
    }
    pub(crate) fn send(&mut self, env: &Env<Limits>, key: Key, request: Ask, out: &mut Queue<Request>) {
        if let Some(held) = self.requests.get(&key) {
            assert!(held.request == request, "request key conflict");
            return;
        }
        let Some((party, _, _)) = self.identity() else {
            tell(out, Told::Answered { key, outcome: Outcome::Unconfirmed });
            return;
        };
        let size = declared(&env.limits, &request);
        let pending = pending(party, &request);
        let held = Held { key, request: request.clone() };
        match self.table.ask(key, party, size, held) {
            Ok(_) => {
                self.requests
                    .insert(key, PendingRequest { request, pending, attempt: None })
                    .expect("table admitted request");
            }
            Err(_) => tell(out, Told::Answered { key, outcome: Outcome::Limit }),
        }
    }
    /// Restores decoded records from the small store (client/domain/client.md, 11).
    pub fn restore_record(&mut self, env: &Env<Limits>, kept: List<RequestRecord<Held>>, out: &mut Queue<Request>) {
        let mut dropped = 0_u32;
        for record in kept.into_boxed() {
            let key = record.request.key;
            let request = record.request.request.clone();
            let pending = pending(record.scope, &request);
            if self.requests.contains_key(&key)
                || self.table.restore(key, declared(&env.limits, &request), record).is_err()
            {
                dropped = dropped.checked_add(1).expect("restored list bounded");
            } else {
                self.requests
                    .insert(key, PendingRequest { request: request.clone(), pending, attempt: None })
                    .expect("table admitted record");
                tell(out, Told::Kept { key, request });
            }
        }
        if dropped != 0 {
            tell(out, Told::Dropped { count: dropped });
        }
    }
    pub(crate) fn answer(
        &mut self,
        env: &Env<Limits>,
        attempt: Token,
        envelope: RequestEnvelope,
        answer: Option<crate::Answered>,
        why: Option<Retry>,
        out: &mut Queue<Request>,
    ) {
        let envelope =
            if answer.is_none() && envelope == RequestEnvelope::Final { RequestEnvelope::Lost } else { envelope };
        match self.table.answered(env.now, attempt, envelope) {
            RequestAnswered::Final(key) => {
                self.requests.remove(&key);
                tell(out, Told::Answered { key, outcome: Outcome::Answered(answer.expect("decoded final answer")) });
            }
            RequestAnswered::Pending => {
                let mut found = None;
                for (key, held) in &self.requests {
                    if held.attempt == Some(attempt) {
                        found = Some(*key);
                    }
                }
                if let Some(key) = found {
                    let held = self.requests.get_mut(&key).expect("attempt request");
                    held.pending.progress = Progress::Retrying { why: why.unwrap_or(Retry::Lost) };
                }
            }
            RequestAnswered::Stale => {}
        }
    }
    pub(crate) fn flush(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) {
        match self.identity() {
            Some((party, retention, live)) => {
                self.table.confirmed(Some(party));
                self.table.retention(retention);
                self.table.link(live);
            }
            None => {
                self.table.link(false);
                self.table.confirmed(None);
            }
        }
        {
            let mut cursor = self.table.take(env.now, env.wall);
            for _ in 0..env.limits.requests.out {
                match cursor.next_out() {
                    Some(RequestOut::Save(record)) => down(out, Down::Save { record: record.clone() }),
                    Some(RequestOut::Erase(key)) => down(out, Down::Erase { key }),
                    Some(RequestOut::Send { attempt, key, request }) => {
                        self.requests.get_mut(&request.key).expect("sending request").attempt = Some(attempt);
                        down(out, Down::Send { attempt, key, request: request.request.clone() });
                    }
                    None => break,
                }
            }
        }
        for _ in 0..env.limits.requests.requests {
            let Some((key, progress)) = self.table.progress() else {
                break;
            };
            let held = self.requests.get_mut(&key).expect("progress names held request");
            held.pending.progress = match progress {
                RequestProgress::InFlight => Progress::InFlight,
                RequestProgress::Parked => Progress::Parked,
                RequestProgress::Unknown => Progress::Unknown,
                RequestProgress::Retrying => match held.pending.progress {
                    Progress::Retrying { why } => Progress::Retrying { why },
                    Progress::InFlight | Progress::Parked | Progress::Unknown => {
                        Progress::Retrying { why: Retry::Lost }
                    }
                },
            };
            if progress == RequestProgress::Unknown {
                self.requests.remove(&key);
                tell(out, Told::Answered { key, outcome: Outcome::Unknown });
            } else {
                tell(out, Told::Pending { key });
            }
        }
    }
}
fn declared(limits: &Limits, request: &Ask) -> u64 {
    match request {
        Ask::Jig(_) => limits.ask_bytes,
        Ask::Family(f) => u64::try_from(f.bytes.len()).expect("family bytes fit"),
    }
}
fn pending(party: u64, request: &Ask) -> Pending {
    let owner = match request {
        Ask::Jig(_) => None,
        Ask::Family(f) => Some(f.family),
    };
    Pending { party, owner, progress: Progress::Parked }
}
