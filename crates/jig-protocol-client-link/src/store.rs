//! Durability before exchanges, and owner-versioned restoration (protocol/client.md, 10).
use crate::{Erased, Fact, Keeper, Limits, Link, Request, Saved, StoreRead, StoreVersion, translate};
use alloc::boxed::Box;
use jig_domain_client::store::StoreKey;
use jig_protocol_client_wire::v5 as w;
use skein_lib::{List, Queue, Reader, Token, Writer, bytes};
#[derive(Debug)]
struct Held {
    exchange: Token,
    path: &'static [u8],
    body: Box<[u8]>,
    tracked: bool,
}
#[derive(Debug)]
pub(super) struct Store {
    waiting: bool,
    failed: bool,
    held: Queue<Held>,
}
impl Store {
    pub(super) fn new(limits: &Limits) -> Self {
        Self {
            waiting: false,
            failed: false,
            held: Queue::with_capacity(limits.exchanges.checked_add(1).expect("validated store queue")),
        }
    }
}
pub(super) fn worst_case(limits: &Limits) -> Option<u64> {
    let capacity = limits.exchanges.checked_add(1)?;
    let body = core::cmp::max(w::Keyed::worst_case_bytes(&limits.wire)?, w::Open::worst_case_bytes(&limits.wire)?);
    let body = core::cmp::max(body, w::Owned::worst_case_bytes(&limits.wire)?);
    Queue::<Held>::worst_case(capacity)?.checked_add(body.checked_mul(u64::from(capacity))?)
}
pub(super) fn records(limits: &Limits) -> Option<u32> {
    limits.client.records.checked_add(limits.client.kept)?.checked_add(limits.client.parts)
}
fn keeper(value: &Saved) -> Keeper {
    match value {
        Saved::Jig(_) => Keeper::Jig,
        Saved::Table { .. } => Keeper::Table,
        Saved::Family { family, .. } => Keeper::Family(*family),
    }
}
fn wire_keeper(value: Keeper) -> w::Keeper {
    match value {
        Keeper::Jig => w::Keeper::Jig,
        Keeper::Table => w::Keeper::Table,
        Keeper::Family(number) => w::Keeper::Family(
            w::Family::new(&w::CEILINGS, w::FamilyParts { number }).expect("family identifier is unbounded"),
        ),
    }
}
fn read_keeper(value: &w::Keeper) -> Keeper {
    match value {
        w::Keeper::Jig => Keeper::Jig,
        w::Keeper::Table => Keeper::Table,
        w::Keeper::Family(value) => Keeper::Family(value.number()),
    }
}
fn storage_keeper(key: &[u8], limits: &Limits) -> Result<(Keeper, usize), w::Problem> {
    let length = match key.first() {
        Some(0 | 1) => 1,
        Some(2) => 3,
        Some(_) => return Err(translate::piece_problem(skein_codec::Reason::Tag)),
        None => return Err(translate::piece_problem(skein_codec::Reason::Short)),
    };
    let prefix = key.get(..length).ok_or(translate::piece_problem(skein_codec::Reason::Short))?;
    let owner = w::Keeper::decode(&limits.wire, &mut Reader::new(prefix))?;
    Ok((read_keeper(&owner), length))
}
fn keyed(owner: Keeper, value: &[u8]) -> Box<[u8]> {
    let owner = wire_keeper(owner);
    let length = usize::try_from(owner.measure())
        .expect("keeper fits usize")
        .checked_add(value.len())
        .expect("admitted key fits usize");
    let mut writer = Writer::new(length);
    owner.encode(&mut writer).expect("measured keeper");
    writer.put(value).expect("measured key");
    writer.finish()
}
fn jig_key(value: StoreKey) -> Box<[u8]> {
    match value {
        StoreKey::Place => keyed(Keeper::Jig, &[]),
        StoreKey::Draft { task } => keyed(Keeper::Jig, &task.to_be_bytes()),
    }
}
pub(super) fn key(record: &Saved) -> Box<[u8]> {
    match record {
        Saved::Jig(value) => jig_key(value.key()),
        Saved::Table { record, .. } => keyed(Keeper::Table, &record.key),
        Saved::Family { family, key, .. } => keyed(Keeper::Family(*family), key),
    }
}
fn erased(value: Erased) -> Box<[u8]> {
    match value {
        Erased::Jig(value) => jig_key(value),
        Erased::Table(value) => keyed(Keeper::Table, &value),
        Erased::Family { family, key } => keyed(Keeper::Family(family), &key),
    }
}
fn encoded(value: &Saved, limits: &Limits) -> Result<Box<[u8]>, w::Problem> {
    let (store_version, record) = match value {
        Saved::Jig(value) => {
            let store_version = match value {
                jig_domain_client::store::Record::Place { version, .. }
                | jig_domain_client::store::Record::Draft { version, .. } => *version,
            };
            let record = translate::store::jig(value, limits)?;
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("record fits usize"));
            record.encode(&mut writer).expect("measured jig record");
            (store_version, writer.finish())
        }
        Saved::Table { version, record } => {
            let record = translate::store::table(record, limits)?;
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("record fits usize"));
            record.encode(&mut writer).expect("measured table record");
            (*version, writer.finish())
        }
        Saved::Family { version, bytes, .. } => (*version, bytes::copy_of(bytes)),
    };
    let record =
        w::Kept::new(&limits.wire, w::KeptParts { keeper: wire_keeper(keeper(value)), store_version, record })?;
    if u64::from(record.measure()) > limits.client.document_bytes {
        return Err(translate::too_large());
    }
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("record fits usize"));
    record.encode(&mut writer).expect("measured kept record");
    Ok(writer.finish())
}
fn reads(keeper: Keeper, version: u16, versions: &List<StoreVersion>) -> bool {
    for value in versions {
        if value.keeper == keeper {
            return value.version == version;
        }
    }
    false
}
fn restore(value: w::Kept, storage_key: &[u8], limits: &Limits) -> Result<Saved, w::Problem> {
    let version = value.store_version();
    let mut reader = Reader::new(value.record());
    let saved = match value.keeper() {
        w::Keeper::Jig => Saved::Jig(translate::store::restore_jig(
            w::JigRecord::decode(&limits.wire, &mut reader)?,
            version,
            limits,
        )?),
        w::Keeper::Table => Saved::Table {
            version,
            record: translate::store::restore_table(w::TableEntry::decode(&limits.wire, &mut reader)?, limits)?,
        },
        w::Keeper::Family(family) => {
            let (owner, length) = storage_keeper(storage_key, limits)?;
            if owner != Keeper::Family(family.number()) {
                return Err(translate::piece_problem(skein_codec::Reason::Malformed));
            }
            let key = bytes::copy_of(storage_key.get(length..).expect("decoded storage keeper prefix"));
            return Ok(Saved::Family { family: family.number(), version, key, bytes: bytes::copy_of(value.record()) });
        }
    };
    if reader.remaining() != 0 || key(&saved).as_ref() != storage_key {
        return Err(translate::piece_problem(skein_codec::Reason::Malformed));
    }
    Ok(saved)
}
impl Link {
    pub(super) fn exchange(&mut self, exchange: Token, path: &'static [u8], body: Box<[u8]>, out: &mut Queue<Request>) {
        if !self.attempts.contains_key(&exchange)
            && (path == b"/jig/v1/request" || path == b"/jig/v1/read" || path == b"/jig/v1/watch")
        {
            return;
        }
        if self.store.failed {
            if let Some(attempt) = self.attempts.remove(&exchange) {
                self.failure(attempt, out);
            }
        } else if self.store.waiting {
            if self.store.held.len() == self.store.held.capacity() {
                if let Some(attempt) = self.attempts.remove(&exchange) {
                    self.failure(attempt, out);
                }
            } else {
                self.store.held.push(Held { exchange, path, body, tracked: self.attempts.contains_key(&exchange) });
            }
        } else {
            out.push(Request::Exchange { exchange, path, body });
        }
    }
    fn gather(&mut self, out: &mut Queue<Request>) {
        self.store.waiting = true;
        let count = out.len();
        for _ in 0..count {
            match out.pop().expect("counted step outputs") {
                Request::Exchange { exchange, path, body } => self.exchange(exchange, path, body, out),
                value @ (Request::Write { .. }
                | Request::Remove { .. }
                | Request::Kept { .. }
                | Request::Dropped { .. }
                | Request::Abandon { .. }
                | Request::Answer { .. }
                | Request::OpenStream
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Read { .. }
                | Request::ServiceCredential { .. }
                | Request::Fact { .. }) => out.push(value),
            }
        }
    }
    pub(super) fn save(&mut self, record: Saved, out: &mut Queue<Request>) {
        self.gather(out);
        let key_fits = match &record {
            Saved::Family { key, .. } => {
                u64::try_from(key.len()).expect("key fits u64") <= self.limits.client.document_bytes
            }
            Saved::Jig(_) | Saved::Table { .. } => true,
        };
        if !key_fits {
            self.write_failed(out);
            self.store.failed = true;
            out.push(Request::Fact { fact: Fact::UndecodableStore });
            return;
        }
        let key = key(&record);
        match encoded(&record, &self.limits) {
            Ok(record) => out.push(Request::Write { key, record }),
            Err(_) => {
                self.write_failed(out);
                self.store.failed = true;
                out.push(Request::Fact { fact: Fact::UndecodableStore });
            }
        }
    }
    pub(super) fn erase(&mut self, key: Erased, out: &mut Queue<Request>) {
        self.gather(out);
        let key_fits = match &key {
            Erased::Family { key, .. } => {
                u64::try_from(key.len()).expect("key fits u64") <= self.limits.client.document_bytes
            }
            Erased::Jig(_) | Erased::Table(_) => true,
        };
        if !key_fits {
            self.write_failed(out);
            self.store.failed = true;
            out.push(Request::Fact { fact: Fact::UndecodableStore });
            return;
        }
        out.push(Request::Remove { key: erased(key) });
    }
    pub(super) fn written(&mut self, out: &mut Queue<Request>) {
        self.store.waiting = false;
        self.store.failed = false;
        for _ in 0..self.store.held.capacity() {
            let Some(value) = self.store.held.pop() else { break };
            if !value.tracked || self.attempts.contains_key(&value.exchange) {
                out.push(Request::Exchange { exchange: value.exchange, path: value.path, body: value.body });
            }
        }
    }
    pub(super) fn write_failed(&mut self, out: &mut Queue<Request>) {
        self.store.waiting = false;
        self.store.failed = false;
        for _ in 0..self.store.held.capacity() {
            let Some(value) = self.store.held.pop() else { break };
            if let Some(attempt) = self.attempts.remove(&value.exchange) {
                self.failure(attempt, out);
            }
        }
    }
    pub(super) fn start_store(
        &mut self,
        versions: List<StoreVersion>,
        kept: List<StoreRead>,
        out: &mut Queue<Request>,
    ) {
        let bound = records(&self.limits).expect("validated store input bound");
        if kept.len() > bound || versions.len() > bound.checked_add(2).expect("validated keeper input bound") {
            out.push(Request::Fact { fact: Fact::UndecodableStore });
            return;
        }
        for entry in kept.into_boxed() {
            let Ok((owner, prefix)) = storage_keeper(&entry.key, &self.limits) else {
                out.push(Request::Fact { fact: Fact::UndecodableStore });
                continue;
            };
            if u64::try_from(entry.key.len().checked_sub(prefix).expect("decoded keeper prefix")).expect("key fits u64")
                > self.limits.client.document_bytes
            {
                out.push(Request::Dropped { keeper: owner });
                out.push(Request::Fact { fact: Fact::UndecodableStore });
                continue;
            }
            let mut reader = Reader::new(&entry.record);
            let result = if u64::try_from(entry.record.len()).expect("bounded bytes fit u64")
                > self.limits.client.document_bytes
            {
                Err(translate::too_large())
            } else {
                w::Kept::decode(&self.limits.wire, &mut reader)
            };
            let Ok(value) = result else {
                out.push(Request::Dropped { keeper: owner });
                out.push(Request::Fact { fact: Fact::UndecodableStore });
                continue;
            };
            if reader.remaining() != 0
                || read_keeper(value.keeper()) != owner
                || !reads(owner, value.store_version(), &versions)
            {
                out.push(Request::Dropped { keeper: owner });
                continue;
            }
            match restore(value, &entry.key, &self.limits) {
                Ok(record) => out.push(Request::Kept { record }),
                Err(_) => {
                    out.push(Request::Dropped { keeper: owner });
                    out.push(Request::Fact { fact: Fact::UndecodableStore });
                }
            }
        }
    }
}
