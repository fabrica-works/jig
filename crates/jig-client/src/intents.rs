//! Intents remain until the root table gives their one durable answer.
//! Joining, progress and the terminal belong here; object policy is jig 21.
//! See client/domain/client.md, 6, and client/domain/root.md, 11.
use crate::contract::Ask;
use alloc::boxed::Box;
use skein_lib::Token;
/// What the root table says about an intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Progress {
    /// One attempt is waiting for its answer.
    InFlight,
    /// Waiting to send again.
    Retrying,
    /// Waiting for the link or its party.
    Parked,
    /// Retention ended without a known outcome.
    Unknown,
}

/// A program ask, with a caller token to receive its outcome.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ByValue {
    pub asked: Token,
    pub scope: u64,
    pub ask: Box<Ask>,
}

use crate::{
    boundary::Request,
    contract::{ActKind, Answer, Limit, ObjectKey},
    facts::Fact,
};
use skein_lib::{Id, Map, Queue, Slab};
/// The owner to which one terminal is applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Origin {
    /// An object's act; a second press joins by object and act.
    Card { object: ObjectKey, act: ActKind },
    /// A caller's by-value ask; its token receives the one answer.
    Program { asked: Token },
}
/// One intent, never the root table's key, saved record or retry state.
#[derive(Debug)]
pub struct Intent {
    pub scope: u64,
    pub origin: Origin,
    pub ask: Box<Ask>,
    pub progress: Progress,
    pub answer: Option<Answer>,
}
/// The owner and answer consumed at the durable terminal.
#[derive(Debug)]
pub struct Receipt {
    pub origin: Origin,
    pub answer: Answer,
}
/// Pending intents, with retired facts available until reclaim.
#[derive(Debug)]
pub struct Intents {
    slab: Slab<Intent>,
    ids: Map<Id<Intent>, ()>,
}
impl Intents {
    /// Allocates the configured pending obligations.
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { slab: Slab::with_capacity(capacity), ids: Map::with_capacity(capacity) }
    }
    /// Reads an intent, including a terminal awaiting reclaim.
    #[must_use]
    pub fn get(&self, id: Id<Intent>) -> Option<&Intent> {
        self.slab.get(id)
    }
    /// Joins a pending act or emits one new send under its intent token.
    pub fn ask(&mut self, scope: u64, origin: Origin, ask: Box<Ask>, out: &mut Queue<Request>) -> Option<Id<Intent>> {
        for (id, ()) in &self.ids {
            let intent = self.slab.get(*id).expect("registered intent exists");
            if intent.scope == scope && intent.origin == origin {
                return Some(*id);
            }
        }
        let intent = Intent { scope, origin, ask, progress: Progress::InFlight, answer: None };
        match self.slab.insert(intent) {
            Ok(id) => {
                let old = self.ids.insert(id, ()).expect("registry fits its paired slab");
                assert!(old.is_none(), "new intent is not registered");
                let ask = self.slab.get(id).expect("inserted intent exists").ask.clone();
                out.push(Request::Send { request: id.token(), scope, ask });
                Some(id)
            }
            Err(_) => {
                out.push(Request::Fact { fact: Fact::Refused { which: Limit::Requests } });
                None
            }
        }
    }
    /// Records only the root table's report of a pending request.
    pub fn progress(&mut self, token: Token, progress: Progress) -> Option<Origin> {
        let id = Id::from_token(token);
        self.ids.get(&id)?;
        let intent = self.slab.get_mut(id).expect("registered intent exists");
        intent.progress = progress;
        Some(intent.origin)
    }
    /// Applies a terminal once and retires the pending obligation.
    pub fn terminal(&mut self, token: Token, answer: Answer) -> Option<Receipt> {
        let id = Id::from_token(token);
        self.ids.remove(&id)?;
        let intent = self.slab.get_mut(id).expect("registered intent exists");
        intent.answer = Some(answer.clone());
        let receipt = Receipt { origin: intent.origin, answer };
        self.slab.retire(id);
        Some(receipt)
    }
    /// Frees slots only after their durable terminal was delivered.
    pub fn reclaim(&mut self) {
        self.slab.reclaim();
    }
}
/// Pending and retired intents, including an ask and its durable facts.
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    Slab::<Intent>::worst_case(limits.intents)?
        .checked_add(Map::<Id<Intent>, ()>::worst_case(limits.intents)?)?
        .checked_add(u64::from(limits.intents).checked_mul(limits.document_bytes)?.checked_mul(2)?)
}

/// A card policy's one send, joined by object and act. Jig 21 calls this after
/// validating its confirmation and completes the durable card transition.
pub fn send_card(
    domain: &mut crate::Domain,
    object: ObjectKey,
    act: ActKind,
    ask: Box<Ask>,
    out: &mut Queue<Request>,
) -> Option<Token> {
    let scope = domain.confirmed?;
    let object_id = domain.objects.find(object)?;
    let origin = Origin::Card { object, act };
    match domain.intents.ask(scope, origin, ask, out) {
        Some(id) => {
            let progress = domain.intents.get(id).expect("joined intent exists").progress;
            domain.objects.get_mut(object_id).expect("named card exists").card =
                crate::objects::Card::Deciding { request: id.token(), progress };
            domain.changed();
            Some(id.token())
        }
        None => {
            domain.objects.get_mut(object_id).expect("named card exists").card =
                crate::objects::Card::Refused { refusal: crate::contract::Refusal::Limit { which: Limit::Requests } };
            domain.changed();
            None
        }
    }
}
pub(crate) fn by_value(domain: &mut crate::Domain, value: ByValue, out: &mut Queue<Request>) {
    if domain.blocked {
        out.push(Request::Answered {
            asked: value.asked,
            answer: Answer::Refused(crate::contract::Refusal::NotOffered),
        });
        return;
    }
    let origin = Origin::Program { asked: value.asked };
    if domain.intents.ask(value.scope, origin, value.ask, out).is_none() {
        out.push(Request::Answered {
            asked: value.asked,
            answer: Answer::Refused(crate::contract::Refusal::Limit { which: Limit::Requests }),
        });
    }
}
pub(crate) fn progress(domain: &mut crate::Domain, request: Token, progress: Progress) {
    let Some(origin) = domain.intents.progress(request, progress) else {
        return;
    };
    match origin {
        Origin::Card { object, .. } => {
            if let Some(id) = domain.objects.find(object) {
                domain.objects.get_mut(id).expect("named card exists").card =
                    crate::objects::Card::Deciding { request, progress };
                domain.changed();
            }
        }
        Origin::Program { .. } => {}
    }
}
pub(crate) fn answered(domain: &mut crate::Domain, request: Token, answer: Answer, out: &mut Queue<Request>) {
    let Some(receipt) = domain.intents.terminal(request, answer) else {
        return;
    };
    match receipt.origin {
        Origin::Program { asked } => out.push(Request::Answered { asked, answer: receipt.answer }),
        Origin::Card { object, .. } => {
            if let Some(id) = domain.objects.find(object) {
                if !domain.objects.get(id).expect("named card exists").shown {
                    crate::notices::show(
                        domain,
                        crate::notices::Kind::Answer { object, answer: receipt.answer.clone() },
                    );
                }
                match receipt.answer {
                    Answer::Refused(refusal) => {
                        domain.objects.get_mut(id).expect("named card exists").card =
                            crate::objects::Card::Refused { refusal };
                    }
                    Answer::Done(outcome) => {
                        if crate::objects::stops_waiting(object, &outcome) {
                            let left = domain.leave_answer(id, outcome);
                            assert!(left, "answered card exists");
                        } else {
                            domain.objects.get_mut(id).expect("named card exists").card = crate::objects::Card::Open;
                        }
                    }
                }
                domain.objects.prune();
                domain.changed();
            } else {
                crate::notices::show(domain, crate::notices::Kind::Answer { object, answer: receipt.answer });
            }
        }
    }
}
