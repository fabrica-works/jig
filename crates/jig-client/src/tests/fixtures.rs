use crate::*;
use alloc::boxed::Box;
use jig_client_protocol_wire::v8 as w;
use skein_lib::{Duration, List, Queue, Secret, Time, Token, Wall, Writer};
pub const PARTY: u64 = 7;
pub const SUBJECT_TASK: u64 = 9;
pub const PROJECT: u32 = 1;
pub const TASK: Handle = Handle { asker: Asker::JIG, number: 2 };
pub const ME: Handle = Handle { asker: Asker::JIG, number: 3 };
pub const STOP: Key = Key { asker: Asker::JIG, number: 1 };
pub fn domain_limits() -> DomainLimits {
    DomainLimits {
        // tests/client/src/test.rs:70-77, the concrete world's table profile.
        requests: skein_lib::RequestLimits {
            requests: 4,
            bytes: 65536,
            out: 16,
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
            margin: Duration::from_secs(1),
        },
        ask_bytes: 16384,
        rows: 2,
        text: 32,
        projects: 2,
        charters: 2,
        fields: 4,
        delegates: 2,
        records: 2,
        kept: 4,
        parts: 8,
        heartbeat: Duration::from_millis(100),
        handles: 8,
        turn_bytes: 32,        // tests/client/src/test.rs:42
        reads: 8,              // tests/client/src/test.rs:32
        watches: 8,            // tests/client/src/test.rs:23
        document_bytes: 65536, // tests/client/src/test.rs:38
        backoff: Backoff {
            // tests/client/src/test.rs:51-54
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
        },
    }
}
pub fn limits() -> Limits {
    Limits { domain: domain_limits(), exchanges: 8 }
}
pub fn task() -> Watchable {
    Watchable::Jig(contract::Subject::Task { task: SUBJECT_TASK })
}
pub fn stop() -> Ask {
    Ask::Jig(Box::new(contract::Ask::Stop { project: PROJECT, task: SUBJECT_TASK }))
}
pub fn hello(version: u32) -> Box<[u8]> {
    let served =
        w::Served::new(&w::CEILINGS, w::ServedParts { owner: w::Owner::Jig, first: version, last: version }).unwrap();
    let mut rows = List::with_capacity(1);
    rows.push(served).unwrap();
    event(&w::StreamEvent::Hello(
        w::Hello::new(
            &w::CEILINGS,
            w::HelloParts { stream: [7; 16], served: rows, heartbeat: Duration::from_millis(100) },
        )
        .unwrap(),
    ))
}
pub fn event(value: &w::StreamEvent) -> Box<[u8]> {
    let mut out = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut out).unwrap();
    out.finish()
}
pub fn frame(watch: u64) -> Box<[u8]> {
    let mut parts = jig_client_test_records::frame_record().into_parts();
    parts.retention = Duration::from_secs(60);
    let value = w::DeliveryValue::Frame(w::Frame::new(&w::CEILINGS, parts).unwrap());
    let mut bytes = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut bytes).unwrap();
    let document = w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: bytes.finish() },
    )
    .unwrap();
    event(&w::StreamEvent::Delivery(w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch, document }).unwrap()))
}
pub struct Client {
    pub machine: crate::Client,
    pub out: Queue<Request>,
    pub requests: List<Request>,
}
impl Client {
    pub fn new() -> Self {
        Self::with_kept(List::with_capacity(0))
    }
    pub fn with_kept(kept: List<Record>) -> Self {
        let limits = limits();
        let mut c = Self {
            machine: crate::Client::new(&limits, 7).unwrap(),
            out: Queue::with_capacity(max_out(&limits)),
            requests: List::with_capacity(max_out(&limits)),
        };
        c.step(Event::Start { kept });
        c
    }
    pub fn signed_in() -> Self {
        let mut c = Self::new();
        c.sign_in();
        c
    }
    pub fn sign_in(&mut self) {
        self.step(Event::SignIn { with: With::Held(Secret::new(Box::new([7; 40]))) });
        self.stream(hello(jig_client_protocol_wire::VERSION));
        let watch = self.opened_watch();
        self.stream(frame(watch));
        self.requests.clear();
    }
    pub fn step(&mut self, event: Event) {
        self.requests.clear();
        self.machine.step(Time::ZERO, Wall::EPOCH, event, &mut self.out);
        while let Some(request) = self.out.pop() {
            self.requests.push(request).unwrap();
        }
    }
    pub fn stream(&mut self, bytes: Box<[u8]>) {
        self.step(Event::Streamed { message: Streamed::Record(bytes) });
    }
    pub fn requests(&self) -> &[Request] {
        self.requests.as_slice()
    }
    pub fn opened_watch(&self) -> u64 {
        for request in &self.requests {
            if let Request::Exchange { message: Message::Watch(bytes), .. } = request {
                return w::Open::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).unwrap().watch();
            }
        }
        panic!("one watch opening")
    }
    pub fn exchange(&self) -> Token {
        for request in &self.requests {
            if let Request::Exchange { exchange, .. } = request {
                return *exchange;
            }
        }
        panic!("one exchange")
    }
}

pub fn keyed_count(requests: &[Request]) -> usize {
    let mut count = 0_usize;
    for request in requests {
        if let Request::Exchange { message: Message::Request(_), .. } = request {
            count = count.checked_add(1).expect("bounded requests");
        }
    }
    count
}
pub fn unknown_read() -> Box<[u8]> {
    let value = w::ReadAnswer::None(
        w::ReadAnswerNone::new(&w::CEILINGS, w::ReadAnswerNoneParts { value: w::ReadNone::Unknown }).unwrap(),
    );
    let mut out = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut out).unwrap();
    out.finish()
}

pub fn saved(requests: &[Request]) -> List<Record> {
    let mut records = List::with_capacity(max_out(&limits()));
    for request in requests {
        if let Request::Save { records: rows } = request {
            for record in rows {
                records.push(record.clone()).unwrap();
            }
        }
    }
    records
}
pub fn answer(value: &w::Answer) -> Box<[u8]> {
    let mut out = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut out).unwrap();
    out.finish()
}
pub fn other_store_version(record: &mut Record) {
    let value = w::Kept::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&record.bytes)).unwrap();
    let mut parts = value.into_parts();
    parts.store_version = parts.store_version.checked_add(1).unwrap();
    let value = w::Kept::new(&w::CEILINGS, parts).unwrap();
    let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut writer).unwrap();
    record.bytes = writer.finish();
}

pub fn answers(requests: &[Request]) -> List<(Key, Outcome)> {
    let mut answers = List::with_capacity(max_out(&limits()));
    for request in requests {
        if let Request::Told { told: Told::Answered { key, outcome } } = request {
            answers.push((*key, outcome.clone())).unwrap();
        }
    }
    answers
}
