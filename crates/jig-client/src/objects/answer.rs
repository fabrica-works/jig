//! Durable card answers and local retention expiry, kept as distinct facts.
//! Refusals preserve the engine's judgment; a moved object is read afresh,
//! while an unknown request is never sent automatically again. See client.md,
//! 6, objects.md, 9, and contract.md, 4.
//!
//! | Card event | State | Effect |
//! |---|---|---|
//! | done, still waiting | open with answer | none |
//! | done, stopped waiting | leaving with outcome | linger |
//! | refused moved | refused with facts | fresh read |
//! | refused otherwise | refused with facts | none |
//! | retention expired | unknown, inspect address | notice if hidden |
use crate::{
    Domain, Request,
    contract::{ActKind, Answer, Limit, ObjectKey, Refusal},
    objects::{Card, Object, Source},
};
use skein_lib::{Id, Queue};
/// The useful next actions from a factual refusal, never an authority judgment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Recovery {
    pub look_again: bool,
    pub change_ask: bool,
}
/// Whether the party should inspect fresh facts or change the request they made.
#[must_use]
pub fn recovery(object: &Object) -> Recovery {
    let mut recovery = Recovery { look_again: false, change_ask: false };
    match &object.card {
        Card::Unknown { .. } => recovery.look_again = true,
        Card::Refused { refusal } => match refusal {
            Refusal::Moved { .. } => recovery.look_again = true,
            Refusal::Authority { .. } | Refusal::Limit { .. } => recovery.change_ask = true,
            Refusal::Role { .. }
            | Refusal::Ended { .. }
            | Refusal::Unknown
            | Refusal::NotOffered
            | Refusal::KeyConflict => {}
        },
        Card::Open | Card::Deciding { .. } | Card::Leaving { .. } => {}
    }
    recovery
}
/// Reads the current object, joining a read already in flight and preserving its refusal.
pub fn look_again(domain: &mut Domain, id: Id<Object>, out: &mut Queue<Request>) {
    let Some(object) = domain.objects.get(id) else {
        return;
    };
    if !object.shown || !recovery(object).look_again {
        return;
    }
    let what = super::document_read(object.key);
    if domain.reads.pending(&what) {
        return;
    }
    if domain.reads.ask(domain.generation, Source::Card(id), what, out).is_none() {
        crate::notices::show(domain, crate::notices::Kind::Refused(Limit::Reads));
    }
    domain.changed();
}
pub(crate) fn answered(domain: &mut Domain, key: ObjectKey, act: ActKind, answer: Answer, out: &mut Queue<Request>) {
    let Some(id) = domain.objects.find(key) else {
        crate::notices::show(domain, crate::notices::Kind::Answer { object: key, answer });
        return;
    };
    if !domain.objects.get(id).expect("named card exists").shown {
        crate::notices::show(domain, crate::notices::Kind::Answer { object: key, answer: answer.clone() });
    }
    domain.objects.get_mut(id).expect("named card exists").answer = Some(answer.clone());
    match answer {
        Answer::Refused(refusal) => {
            domain.objects.get_mut(id).expect("named card exists").card = Card::Refused { refusal };
            if recovery(domain.objects.get(id).expect("named card exists")).look_again {
                look_again(domain, id, out);
            }
        }
        Answer::Done(outcome) => {
            if stops_waiting(key, act, &outcome) {
                let left = domain.leave_answer(id, outcome);
                assert!(left, "answered card exists");
            } else {
                domain.objects.get_mut(id).expect("named card exists").card = Card::Open;
            }
        }
    }
    domain.objects.prune();
    domain.changed();
}
fn stops_waiting(key: ObjectKey, act: ActKind, outcome: &crate::contract::Outcome) -> bool {
    if act == ActKind::Take || act == ActKind::HandBack {
        return false;
    }
    if act == ActKind::DeleteNote {
        return true;
    }
    super::stops_waiting(key, outcome)
}
pub(crate) fn unknown(domain: &mut Domain, key: ObjectKey, act: ActKind) {
    let Some(id) = domain.objects.find(key) else {
        crate::notices::show(domain, crate::notices::Kind::Unknown { object: key, act });
        return;
    };
    if !domain.objects.get(id).expect("named card exists").shown {
        crate::notices::show(domain, crate::notices::Kind::Unknown { object: key, act });
    }
    let object = domain.objects.get_mut(id).expect("named card exists");
    object.card = Card::Unknown { act, look: super::document_read(key) };
    object.answer = None;
    domain.objects.prune();
    domain.changed();
}
