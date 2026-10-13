use super::*;
use jig_domain_client::limits::Backoff;
use skein_lib::{Duration, RequestKey, Writer};
pub const ATTEMPT: Token = Token::new(17);
pub const KEY: RequestKey = [9; 16];
const CLIENT: jig_domain_client::Limits = jig_domain_client::Limits {
    watches: 4,
    notices: 2,
    notice_life: Duration::from_secs(5),
    linger: Duration::from_secs(5),
    save_delay: Duration::from_secs(1),
    intents: 2,
    fields: 4,
    providers: 2,
    objects: 2,
    sources: 8,
    reads: 4,
    parts: 8,
    delegates: 2,
    document_bytes: 65536,
    records: 2,
    kept: 4,
    drafts: 1,
    projects: 2,
    roles: 2,
    charters: 2,
    text: 32,
    rows: 2,
    conversation_words: 2,
    navigation_entries: 8,
    updates_count: 99,
    backoff: Backoff { first: Duration::from_nanos(10), most: Duration::from_nanos(80) },
    heartbeat: Duration::from_nanos(100),
};

pub fn limits() -> Limits {
    Limits::new(CLIENT, 4, Duration::from_secs(10)).expect("valid client limits")
}
pub fn start_chat() -> c::Ask {
    c::Ask::StartChat { project: 7, words: c::Text::new(Box::from(b"hello".as_slice()), 32).expect("fixture text") }
}
pub fn encoded_answer(value: &w::Answer) -> Box<[u8]> {
    let mut out = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut out).unwrap();
    out.finish()
}
pub fn final_answer(value: w::Final) -> Box<[u8]> {
    let mut bytes = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut bytes).unwrap();
    let owned =
        w::Owned::new(&limits().wire, w::OwnedParts { owner: w::Owner::Jig, version: 2, record: bytes.finish() })
            .unwrap();
    encoded_answer(&w::Answer::FinalAnswer(owned))
}
pub fn answer_done_chat(task: u64) -> Box<[u8]> {
    let lim = limits().wire;
    let chat = w::MadeChat::new(&lim, w::MadeChatParts { task }).unwrap();
    let made = w::OutcomeMade::new(&lim, w::OutcomeMadeParts { what: w::Made::Chat(chat) }).unwrap();
    final_answer(w::Final::Done(w::Done::new(&lim, w::DoneParts { outcome: w::Outcome::Made(made) }).unwrap()))
}
pub fn answer_busy() -> Box<[u8]> {
    encoded_answer(&w::Answer::Busy)
}
pub fn answer_signed_out() -> Box<[u8]> {
    encoded_answer(&w::Answer::SignedOut)
}
pub fn malformed_final() -> Box<[u8]> {
    let owned =
        w::Owned::new(&limits().wire, w::OwnedParts { owner: w::Owner::Jig, version: 2, record: Box::from([255]) })
            .unwrap();
    encoded_answer(&w::Answer::FinalAnswer(owned))
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
        self.link.step(Time::ZERO, Event::Send { attempt, key, ask: Ask::Jig(Box::new(ask)) }, &mut self.out);
        match self.out.iter().next().expect("one exchange") {
            Request::Exchange { exchange, .. } => *exchange,
            other @ (Request::Abandon { .. } | Request::Answer { .. } | Request::Fact { .. }) => {
                panic!("expected exchange: {other:?}")
            }
        }
    }
    pub fn sending() -> (Self, Token) {
        let mut h = Self::new(&limits());
        let exchange = h.send(ATTEMPT, KEY, start_chat());
        (h, exchange)
    }
    pub fn answer(&mut self, exchange: Token, status: u16, bytes: &[u8]) {
        self.link.step(Time::ZERO, Event::Answered { exchange, status, body: Box::from(bytes) }, &mut self.out);
    }
    pub fn answered(&self, token: Token) -> Option<(RequestEnvelope, Option<&Answered>)> {
        for request in &self.out {
            match request {
                Request::Answer { attempt, envelope, answer } if *attempt == token => {
                    return Some((*envelope, answer.as_ref()));
                }
                Request::Exchange { .. } | Request::Abandon { .. } | Request::Answer { .. } | Request::Fact { .. } => {}
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
                Request::Answer { .. } => count = count.saturating_add(1),
                Request::Exchange { .. } | Request::Abandon { .. } | Request::Fact { .. } => {}
            }
        }
        count
    }
    pub fn lost(&self) {
        assert_eq!(self.answered(ATTEMPT), Some((RequestEnvelope::Lost, None)));
    }
}

pub fn fact_bytes(value: &w::Fact) -> Box<[u8]> {
    let mut row = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut row).unwrap();
    let mut piece = Writer::new(usize::try_from(value.measure().checked_add(4).unwrap()).unwrap());
    skein_codec::piece::write(&mut piece, &row.finish()).unwrap();
    piece.finish()
}
pub fn refusal_with_hold(rows: Box<[u8]>) -> Box<[u8]> {
    let lim = limits().wire;
    let reason = w::HoldReason::new(&lim, w::HoldReasonParts { rows }).unwrap();
    let reason = w::CurrentHoldDocument::new(&lim, w::CurrentHoldDocumentParts { value: reason }).unwrap();
    let what = w::MovedHold::new(&lim, w::MovedHoldParts { reason: Some(w::CurrentHold::Document(reason)) }).unwrap();
    let now = w::Revision::new(&lim, w::RevisionParts { value: u64::MAX }).unwrap();
    let value = w::RefusalMoved::new(&lim, w::RefusalMovedParts { now, what: w::Moved::Hold(what) }).unwrap();
    let value = w::Refused::new(&lim, w::RefusedParts { refusal: w::Refusal::Moved(value) }).unwrap();
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
