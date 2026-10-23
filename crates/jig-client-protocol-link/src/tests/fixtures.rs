#[path = "fixtures/create.rs"]
mod create;
use super::*;
pub use create::*;
use jig_client_domain::Backoff;
use skein_lib::{Duration, RequestKey, Writer};
pub const ATTEMPT: Token = Token::new(17);
pub const KEY: RequestKey = [9; 16];
pub const SERVICE_CREDENTIAL: [u8; 40] = [7; 40];
const CLIENT: jig_client_domain::Limits = jig_client_domain::Limits {
    owners: &[jig_client_domain::Spoken { family: 19, version: 23 }],
    watches: 4,
    handles: 8,
    turn_bytes: 32,
    ask_bytes: 16384,
    requests: skein_lib::RequestLimits {
        requests: 4,
        bytes: 65536,
        out: 16,
        first: Duration::from_nanos(10),
        most: Duration::from_nanos(80),
        margin: Duration::from_secs(1),
    },
    fields: 4,
    reads: 4,
    parts: 8,
    delegates: 2,
    document_bytes: 65536,
    records: 2,
    kept: 4,
    projects: 2,
    text: 32,
    turn_text: 32,
    rows: 2,
    segments: 4,
    backoff: Backoff { first: Duration::from_nanos(10), most: Duration::from_nanos(80) },
    authority: 8,
    accounts: jig_client_domain::AccountCounts { accounts: 8, sources: 8, sign_ins: 8 },
    heartbeat: Duration::from_nanos(100),
};

pub fn limits() -> Limits {
    Limits::new(CLIENT, 4).expect("valid client limits")
}
pub fn message() -> c::Ask {
    c::Ask::Message {
        project: 7,
        task: 8,
        words: c::Text::new(Box::from(b"hello".as_slice()), 32).expect("fixture text"),
    }
}
pub fn encoded_answer(value: &w::KeyedAnswer) -> Box<[u8]> {
    let mut out = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut out).unwrap();
    out.finish()
}
pub fn final_answer(value: w::Final) -> Box<[u8]> {
    let mut bytes = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut bytes).unwrap();
    let owned =
        w::Owned::new(&limits().wire, w::OwnedParts { owner: w::Owner::Jig, version: 15, record: bytes.finish() })
            .unwrap();
    encoded_answer(&w::KeyedAnswer::FinalAnswer(owned))
}
pub fn answer_done_chat(task: u64) -> Box<[u8]> {
    let lim = limits().wire;
    let mut tasks = skein_lib::List::with_capacity(1);
    tasks.push(task).unwrap();
    let made_tasks = w::MadeTasks::new(&lim, w::MadeTasksParts { tasks }).unwrap();
    let made = w::OutcomeMade::new(&lim, w::OutcomeMadeParts { what: w::Made::Tasks(made_tasks) }).unwrap();
    final_answer(w::Final::Done(w::Done::new(&lim, w::DoneParts { outcome: w::Outcome::Made(made) }).unwrap()))
}
pub fn answer_key_conflict() -> Box<[u8]> {
    final_answer(w::Final::Refused(
        w::Refused::new(&limits().wire, w::RefusedParts { at: None, refusal: w::Refusal::KeyConflict }).unwrap(),
    ))
}
pub fn answer_family(family: u16, version: u32) -> Box<[u8]> {
    let lim = limits().wire;
    let owner = w::Owner::Family(w::Family::new(&lim, w::FamilyParts { number: family }).unwrap());
    let record = w::Owned::new(&lim, w::OwnedParts { owner, version, record: family_read().bytes }).unwrap();
    encoded_answer(&w::KeyedAnswer::FinalAnswer(record))
}
pub fn answer_done_service(credential: &[u8]) -> Box<[u8]> {
    let lim = limits().wire;
    let row = w::MadeService::new(
        &lim,
        w::MadeServiceParts { party: 7, credential: skein_lib::Secret::new(Box::from(credential)) },
    )
    .unwrap();
    let made = w::OutcomeMade::new(&lim, w::OutcomeMadeParts { what: w::Made::Service(row) }).unwrap();
    final_answer(w::Final::Done(w::Done::new(&lim, w::DoneParts { outcome: w::Outcome::Made(made) }).unwrap()))
}
pub fn answer_busy() -> Box<[u8]> {
    encoded_answer(&w::KeyedAnswer::Busy)
}
pub fn answer_signed_out() -> Box<[u8]> {
    encoded_answer(&w::KeyedAnswer::SignedOut)
}
pub fn malformed_final() -> Box<[u8]> {
    let owned =
        w::Owned::new(&limits().wire, w::OwnedParts { owner: w::Owner::Jig, version: 15, record: Box::from([255]) })
            .unwrap();
    encoded_answer(&w::KeyedAnswer::FinalAnswer(owned))
}
pub struct Harness {
    pub link: Link,
    pub out: Queue<Request>,
}
impl Harness {
    pub fn new(limits: &Limits) -> Self {
        Self { link: Link::new(limits).unwrap(), out: Queue::with_capacity(32) }
    }
    pub fn send(&mut self, attempt: Token, key: RequestKey, ask: c::Ask) -> Token {
        self.link.step(
            Time::ZERO,
            Event::Send { attempt, key, ask: Ask::Jig(Box::new(ask)), value: None },
            &mut self.out,
        );
        match self.out.iter().next().expect("one exchange") {
            Request::Exchange { exchange, .. } => *exchange,
            other @ (Request::Answer { .. }
            | Request::NotKept { .. }
            | Request::Read { .. }
            | Request::OpenStream { .. }
            | Request::CloseStream
            | Request::Watch { .. }
            | Request::FamilyDelivery { .. }
            | Request::Ended { .. }
            | Request::Hello { .. }
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Kept { .. }
            | Request::Dropped
            | Request::Signing { .. }
            | Request::ServiceCredential { .. }
            | Request::Fact { .. }) => {
                panic!("expected exchange: {other:?}")
            }
        }
    }
    pub fn sending() -> (Self, Token) {
        let mut h = Self::new(&limits());
        let exchange = h.send(ATTEMPT, KEY, message());
        (h, exchange)
    }
    pub fn answer(&mut self, exchange: Token, bytes: &[u8]) {
        self.link.step(Time::ZERO, Event::Reply { exchange, message: Box::from(bytes) }, &mut self.out);
    }
    pub fn answered(&self, token: Token) -> Option<(RequestEnvelope, Option<&Answered>)> {
        for request in &self.out {
            match request {
                Request::Answer { attempt, envelope, answer, .. } if *attempt == token => {
                    return Some((*envelope, answer.as_ref()));
                }
                Request::Exchange { .. }
                | Request::Answer { .. }
                | Request::NotKept { .. }
                | Request::Read { .. }
                | Request::OpenStream { .. }
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Save { .. }
                | Request::Erase { .. }
                | Request::Kept { .. }
                | Request::Dropped
                | Request::Signing { .. }
                | Request::ServiceCredential { .. }
                | Request::Fact { .. } => {}
            }
        }
        None
    }
    pub fn has(&self, expected: &Request) -> bool {
        for value in &self.out {
            if value == expected {
                return true;
            }
        }
        false
    }
    pub fn terminals(&self) -> u32 {
        let mut count: u32 = 0;
        for value in &self.out {
            match value {
                Request::Answer { .. } | Request::NotKept { .. } => count = count.saturating_add(1),
                Request::Exchange { .. }
                | Request::Read { .. }
                | Request::OpenStream { .. }
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Save { .. }
                | Request::Erase { .. }
                | Request::Kept { .. }
                | Request::Dropped
                | Request::Signing { .. }
                | Request::ServiceCredential { .. }
                | Request::Fact { .. } => {}
            }
        }
        count
    }
    pub fn lost(&self) {
        assert_eq!(self.answered(ATTEMPT), Some((RequestEnvelope::Lost, None)));
    }
}

pub fn fact_bytes(value: &w::Fact) -> Box<[u8]> {
    jig_client_test_records::fact_bytes(value)
}
pub fn refusal_with_hold(rows: Box<[u8]>) -> Box<[u8]> {
    let lim = limits().wire;
    let reason = w::HoldReason::new(&lim, w::HoldReasonParts { rows }).unwrap();
    let reason = w::CurrentHoldDocument::new(&lim, w::CurrentHoldDocumentParts { value: reason }).unwrap();
    let what = w::MovedHold::new(&lim, w::MovedHoldParts { reason: Some(w::CurrentHold::Document(reason)) }).unwrap();
    let now = w::Revision::new(&lim, w::RevisionParts { value: u64::MAX }).unwrap();
    let value = w::RefusalMoved::new(&lim, w::RefusalMovedParts { now, what: w::Moved::Hold(what) }).unwrap();
    let value = w::Refused::new(&lim, w::RefusedParts { at: None, refusal: w::Refusal::Moved(value) }).unwrap();
    final_answer(w::Final::Refused(value))
}
pub fn hold_budget_fact() -> w::Fact {
    let lim = limits().wire;
    let value =
        w::HoldReasonBudgetFields::new(&lim, w::HoldReasonBudgetFieldsParts { spent: u64::MAX, limit: 16 }).unwrap();
    w::Fact::HoldReason(
        w::HoldReasonRow::new(&lim, w::HoldReasonRowParts { parent: None, value: w::HoldReasonFields::Budget(value) })
            .unwrap(),
    )
}

mod documents;
pub use documents::*;
mod phases;
pub use phases::*;
mod owners;
pub use owners::*;

mod reads;
pub use reads::*;

mod turns;
pub use turns::*;

mod link_reads;
pub use link_reads::*;

mod link_stream;
pub use link_stream::*;

mod link_deliveries;
pub use link_deliveries::*;

mod link_store;
pub use link_store::*;

impl Harness {
    pub fn sign_in(&mut self, with: With) -> Option<Token> {
        self.link.step(Time::ZERO, Event::SignIn { with }, &mut self.out);
        for request in &self.out {
            if let Request::Exchange { exchange, .. } = request {
                return Some(*exchange);
            }
        }
        None
    }
    pub fn credentials(&self) -> skein_lib::List<Option<skein_lib::Secret>> {
        let mut values = skein_lib::List::with_capacity(32);
        for request in &self.out {
            if let Request::Exchange { credential, .. } = request {
                values.push(credential.clone()).expect("output bound");
            }
        }
        values
    }
    pub fn sign_out(&mut self) -> Token {
        self.out = Queue::with_capacity(32);
        self.link.step(Time::ZERO, Event::SignOut, &mut self.out);
        for request in &self.out {
            if let Request::Exchange { exchange, .. } = request {
                return *exchange;
            }
        }
        unreachable!("sign-out exchange")
    }
}
pub fn signed_in(party: u64, token: &[u8; 40]) -> Box<[u8]> {
    let record = w::SignInAnswer::SignedIn(
        w::SignedIn::new(
            &limits().wire,
            w::SignedInParts { token: Some(skein_lib::Secret::new(Box::from(*token))), party, expires: 10 },
        )
        .unwrap(),
    );
    let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut writer).unwrap();
    writer.finish()
}

impl Harness {
    pub fn sending_family() -> (Self, Token) {
        let mut harness = Self::new(&limits());
        let ask = Ask::Family { scope: c::Scope::Own, record: family_read() };
        harness.link.step(Time::ZERO, Event::Send { attempt: ATTEMPT, key: KEY, ask, value: None }, &mut harness.out);
        let Some(Request::Exchange { exchange, .. }) = harness.out.iter().next() else { panic!("exchange") };
        let exchange = *exchange;
        (harness, exchange)
    }
}
