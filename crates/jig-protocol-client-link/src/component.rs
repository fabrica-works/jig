//! One bounded exchange per request-table attempt (protocol/client.md, 5).
use crate::{Answered, Ask, Event, Fact, Family, Limits, Readable, Readout, Request, Unusable, translate};
use jig_protocol_client_wire::{self as wire, v3 as w};
use skein_lib::{Map, Queue, Reader, RequestEnvelope, RequestKey, Time, Token, Writer};
#[derive(Debug)]
pub(crate) struct Attempt {
    token: Token,
    owner: w::Owner,
    version: u32,
    until: Time,
    kind: ExchangeKind,
}
#[derive(Clone, Copy, Debug)]
enum ExchangeKind {
    Attempt,
    Read,
}
#[derive(Debug)]
pub struct Link {
    limits: Limits,
    attempts: Map<Token, Attempt>,
    next: u64,
}
impl Link {
    pub fn new(limits: &Limits) -> Result<Self, Unusable> {
        crate::worst_case(limits).ok_or(Unusable::Limits)?;
        Ok(Self { limits: *limits, attempts: Map::with_capacity(limits.exchanges), next: 1 })
    }
    #[must_use]
    pub fn max_out(&self) -> u32 {
        self.limits.exchanges.saturating_mul(2).max(2)
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut first = None;
        for (_, attempt) in &self.attempts {
            first = Some(match first {
                Some(value) => core::cmp::min(value, attempt.until),
                None => attempt.until,
            });
        }
        first
    }
    pub fn reclaim(&mut self) {}
    #[must_use]
    pub fn worst_case(limits: &Limits) -> Option<u64> {
        crate::worst_case(limits)
    }
    pub fn step(&mut self, now: Time, event: Event, out: &mut Queue<Request>) {
        match event {
            Event::Send { attempt, key, ask } => self.send(now, attempt, key, ask, out),
            Event::Read { read, what } => self.read(now, read, what, out),
            Event::Answered { exchange, status, body } => self.answered(exchange, status, &body, out),
            Event::Failed { exchange } => {
                if let Some(attempt) = self.attempts.remove(&exchange) {
                    failed(attempt, out);
                }
            }
        }
    }
    pub fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        let mut expired = skein_lib::List::with_capacity(self.limits.exchanges);
        for (exchange, attempt) in &self.attempts {
            if now >= attempt.until {
                expired.push(*exchange).expect("bounded attempts");
            }
        }
        for exchange in &expired {
            let attempt = self.attempts.remove(exchange).expect("collected live exchange");
            out.push(Request::Abandon { exchange: *exchange });
            failed(attempt, out);
        }
    }
    fn send(&mut self, now: Time, attempt: Token, key: RequestKey, ask: Ask, out: &mut Queue<Request>) {
        if self.attempts.len() == self.attempts.capacity() {
            lost(attempt, out);
            return;
        }
        let Some(next) = self.next.checked_add(1) else {
            lost(attempt, out);
            return;
        };
        let Ok(record) = keyed(key, ask, &self.limits.wire) else {
            lost(attempt, out);
            return;
        };
        let exchange = Token::new(self.next);
        self.next = next;
        let state = Attempt {
            token: attempt,
            owner: record.owner().clone(),
            version: record.version(),
            until: now.saturating_add(self.limits.deadline),
            kind: ExchangeKind::Attempt,
        };
        self.attempts.insert(exchange, state).expect("checked exchange capacity");
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("u32 fits usize"));
        record.encode(&mut writer).expect("measured envelope");
        out.push(Request::Exchange { exchange, path: b"/jig/v1/request", body: writer.finish() });
    }
    fn read(&mut self, now: Time, read: Token, what: Readable, out: &mut Queue<Request>) {
        let owner = match &what {
            Readable::Jig(_) => w::Owner::Jig,
            Readable::Family(value) => w::Owner::Family(
                w::Family::new(&self.limits.wire, w::FamilyParts { number: value.family })
                    .expect("unbounded family number"),
            ),
        };
        if self.attempts.len() == self.attempts.capacity() {
            read_none(read, &owner, jig_domain_client::contract::ReadNone::Unreachable, out);
            return;
        }
        let Some(next) = self.next.checked_add(1) else {
            read_none(read, &owner, jig_domain_client::contract::ReadNone::Unreachable, out);
            return;
        };
        let Ok(record) = translate::reads::owned(what, &self.limits.wire) else {
            read_none(read, &owner, jig_domain_client::contract::ReadNone::Unreachable, out);
            return;
        };
        let exchange = Token::new(self.next);
        self.next = next;
        let state = Attempt {
            token: read,
            owner,
            version: record.version(),
            until: now.saturating_add(self.limits.deadline),
            kind: ExchangeKind::Read,
        };
        self.attempts.insert(exchange, state).expect("checked exchange capacity");
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("u32 fits usize"));
        record.encode(&mut writer).expect("measured read envelope");
        out.push(Request::Exchange { exchange, path: b"/jig/v1/read", body: writer.finish() });
    }
    fn answered(&mut self, exchange: Token, status: u16, body: &[u8], out: &mut Queue<Request>) {
        let Some(attempt) = self.attempts.remove(&exchange) else { return };
        match attempt.kind {
            ExchangeKind::Attempt => {}
            ExchangeKind::Read => {
                self.read_answer(attempt, status, body, out);
                return;
            }
        }
        if status != 200 {
            lost(attempt.token, out);
            return;
        }
        let Ok(record) = w::Answer::decode(&self.limits.wire, &mut Reader::new(body)) else {
            lost(attempt.token, out);
            return;
        };
        let envelope = match record {
            w::Answer::Busy | w::Answer::NotReady => RequestEnvelope::Again,
            w::Answer::SignedOut => RequestEnvelope::SignedOut,
            w::Answer::FinalAnswer(record) => {
                let answer = if record.owner() != &attempt.owner || record.version() != attempt.version {
                    Err(translate::Undecodable::Version)
                } else {
                    match record.owner() {
                        w::Owner::Jig => match translate::answers::answer(w::Answer::FinalAnswer(record), &self.limits)
                        {
                            Ok(answer) => Ok(Answered::Jig(answer)),
                            Err(problem) => Err(problem),
                        },
                        w::Owner::Family(owner) => Ok(Answered::Family(Family {
                            family: owner.number(),
                            version: record.version(),
                            bytes: skein_lib::bytes::copy_of(record.record()),
                        })),
                    }
                };
                match answer {
                    Ok(answer) => {
                        out.push(Request::Answer {
                            attempt: attempt.token,
                            envelope: RequestEnvelope::Final,
                            answer: Some(answer),
                        });
                        return;
                    }
                    Err(_) => {
                        out.push(Request::Fact { fact: Fact::UndecodableAnswer });
                        RequestEnvelope::Lost
                    }
                }
            }
        };
        out.push(Request::Answer { attempt: attempt.token, envelope, answer: None });
    }
    fn read_answer(&self, attempt: Attempt, status: u16, body: &[u8], out: &mut Queue<Request>) {
        if status != 200 {
            failed(attempt, out);
            return;
        }
        let result = match w::ReadAnswer::decode(&self.limits.wire, &mut Reader::new(body)) {
            Ok(w::ReadAnswer::None(record)) => {
                read_none(attempt.token, &attempt.owner, translate::reads::none(record.value()), out);
                return;
            }
            Ok(w::ReadAnswer::Value(record)) => family_read_value(&attempt, record),
            Err(_) => Err(translate::Undecodable::Malformed),
        };
        match result {
            Ok(result) => out.push(Request::Read { read: attempt.token, result }),
            Err(_) => {
                out.push(Request::Fact { fact: Fact::UndecodableDocument });
                failed(attempt, out);
            }
        }
    }
}
fn family_read_value(attempt: &Attempt, record: w::Owned) -> Result<Readout, translate::Undecodable> {
    if record.owner() != &attempt.owner || record.version() != attempt.version {
        return Err(translate::Undecodable::Version);
    }
    match record.owner() {
        // Ruling 208: jig read values require 30.s1's discriminated wire.
        w::Owner::Jig => Err(translate::Undecodable::Version),
        w::Owner::Family(owner) => {
            let family = owner.number();
            Ok(Readout::Family { family, record: Some(record.into_parts().record) })
        }
    }
}
fn read_none(read: Token, owner: &w::Owner, why: jig_domain_client::contract::ReadNone, out: &mut Queue<Request>) {
    let result = match owner {
        w::Owner::Jig => Readout::Jig(jig_domain_client::contract::ReadResult::None(why)),
        w::Owner::Family(owner) => Readout::Family { family: owner.number(), record: None },
    };
    out.push(Request::Read { read, result });
}
fn failed(attempt: Attempt, out: &mut Queue<Request>) {
    match attempt.kind {
        ExchangeKind::Attempt => lost(attempt.token, out),
        ExchangeKind::Read => {
            read_none(attempt.token, &attempt.owner, jig_domain_client::contract::ReadNone::Unreachable, out);
        }
    }
}
fn lost(attempt: Token, out: &mut Queue<Request>) {
    out.push(Request::Answer { attempt, envelope: RequestEnvelope::Lost, answer: None });
}
fn keyed(key: RequestKey, ask: Ask, limits: &w::Limits) -> Result<w::Keyed, w::Problem> {
    let (owner, version, request) = match ask {
        Ask::Jig(ask) => {
            let record = translate::asks::ask(&ask, limits)?;
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("u32 fits usize"));
            record.encode(&mut writer).expect("measured ask");
            (w::Owner::Jig, wire::VERSION, writer.finish())
        }
        Ask::Family(record) => {
            let owner = w::Family::new(limits, w::FamilyParts { number: record.family })?;
            (w::Owner::Family(owner), record.version, record.bytes)
        }
    };
    w::Keyed::new(limits, w::KeyedParts { key, owner, version, request })
}
