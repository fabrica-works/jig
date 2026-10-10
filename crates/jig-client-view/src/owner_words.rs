//! Normal jig phrasebooks compose every exact owner fragment (view.md, 7).
//! Composition happens while languages load, never while a page renders.
use crate::phrases::{Language, PhraseShape, Phrasebook};
use alloc::boxed::Box;
use skein_lib::List;

#[must_use]
pub fn phrase_count() -> u32 {
    u32::try_from(crate::phrases::jig::ALL.len())
        .expect("normal phrase count fits")
        .checked_add(u32::try_from(crate::owner_authority_words::ALL.len()).expect("authority phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::owner_wake_words::ALL.len()).expect("wake phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::proposed_task_words::ALL.len()).expect("batch phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::owner_result_words::ALL.len()).expect("result phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::owner_budget_words::ALL.len()).expect("budget phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::owner_outcome_words::ALL.len()).expect("outcome phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::owner_chip_words::ALL.len()).expect("chip phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::live_call_words::ALL.len()).expect("live call phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::owner_amendment_words::ALL.len()).expect("amendment phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::owner_activity_words::ALL.len()).expect("activity phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::owner_milestone_words::ALL.len()).expect("milestone phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::owner_history_words::ALL.len()).expect("history phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::owner_tree_words::ALL.len()).expect("tree phrase count fits"))
        .checked_add(u32::try_from(crate::owner_proposal_words::ALL.len()).expect("proposal phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::owner_inbox_words::ALL.len()).expect("Inbox phrase count fits"))
        .expect("composed phrase count fits")
}

#[must_use]
pub fn shapes(normal: Box<[PhraseShape]>) -> Box<[PhraseShape]> {
    let mut result = List::with_capacity(phrase_count());
    for shape in normal {
        result.push(shape).expect("normal shapes fit");
    }
    for shape in crate::owner_authority_words::shapes() {
        result.push(shape).expect("authority shapes fit");
    }
    for shape in crate::owner_wake_words::shapes() {
        result.push(shape).expect("wake shapes fit");
    }
    for shape in crate::proposed_task_words::shapes() {
        result.push(shape).expect("batch shapes fit");
    }
    for shape in crate::owner_result_words::shapes() {
        result.push(shape).expect("result shapes fit");
    }
    for shape in crate::owner_budget_words::shapes() {
        result.push(shape).expect("budget shapes fit");
    }
    for shape in crate::owner_outcome_words::shapes() {
        result.push(shape).expect("outcome shapes fit");
    }
    for shape in crate::owner_chip_words::shapes() {
        result.push(shape).expect("chip shapes fit");
    }
    for shape in crate::live_call_words::shapes() {
        result.push(shape).expect("live call shapes fit");
    }
    for shape in crate::owner_amendment_words::shapes() {
        result.push(shape).expect("amendment shapes fit");
    }
    for shape in crate::owner_activity_words::shapes() {
        result.push(shape).expect("activity shapes fit");
    }
    for shape in crate::owner_history_words::shapes() {
        result.push(shape).expect("history shapes fit");
    }
    for shape in crate::owner_milestone_words::shapes() {
        result.push(shape).expect("milestone shapes fit");
    }
    for shape in crate::owner_inbox_words::shapes() {
        result.push(shape).expect("Inbox shapes fit");
    }
    for shape in crate::owner_tree_words::shapes() {
        result.push(shape).expect("tree shapes fit");
    }
    for shape in crate::owner_proposal_words::shapes() {
        result.push(shape).expect("proposal shapes fit");
    }
    result.into_boxed()
}

#[must_use]
pub fn language(normal: Language) -> Language {
    let mut phrases = List::with_capacity(phrase_count());
    for phrase in normal.phrases {
        phrases.push(phrase).expect("normal words fit");
    }
    for phrase in crate::owner_authority_words::english().phrases {
        phrases.push(phrase).expect("authority words fit");
    }
    for phrase in crate::owner_wake_words::language().phrases {
        phrases.push(phrase).expect("wake words fit");
    }
    for phrase in crate::proposed_task_words::language().phrases {
        phrases.push(phrase).expect("batch words fit");
    }
    for phrase in crate::owner_result_words::language().phrases {
        phrases.push(phrase).expect("result words fit");
    }
    for phrase in crate::owner_budget_words::language().phrases {
        phrases.push(phrase).expect("budget words fit");
    }
    for phrase in crate::owner_outcome_words::english().phrases {
        phrases.push(phrase).expect("outcome words fit");
    }
    for phrase in crate::owner_chip_words::language().phrases {
        phrases.push(phrase).expect("chip words fit");
    }
    for phrase in crate::live_call_words::language().phrases {
        phrases.push(phrase).expect("live call words fit");
    }
    for phrase in crate::owner_amendment_words::language().phrases {
        phrases.push(phrase).expect("amendment words fit");
    }
    for phrase in crate::owner_activity_words::language().phrases {
        phrases.push(phrase).expect("activity words fit");
    }
    for phrase in crate::owner_history_words::language().phrases {
        phrases.push(phrase).expect("history words fit");
    }
    for phrase in crate::owner_milestone_words::language().phrases {
        phrases.push(phrase).expect("milestone words fit");
    }
    for phrase in crate::owner_inbox_words::language().phrases {
        phrases.push(phrase).expect("Inbox words fit");
    }
    for phrase in crate::owner_tree_words::language().phrases {
        phrases.push(phrase).expect("tree words fit");
    }
    for phrase in crate::owner_proposal_words::language().phrases {
        phrases.push(phrase).expect("tree words fit");
    }
    Language { plural: normal.plural, span_unit: normal.span_unit, wall: normal.wall, phrases: phrases.into_boxed() }
}

/// All loaded languages contribute to the saying envelope.
#[must_use]
pub fn phrase_bytes(book: &Phrasebook) -> Option<u32> {
    let mut largest = 0;
    for phrase in crate::owner_authority_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::owner_wake_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::proposed_task_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::owner_result_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::owner_budget_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::owner_outcome_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::owner_chip_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::live_call_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::owner_amendment_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::owner_activity_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::owner_history_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::owner_milestone_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::owner_inbox_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::owner_tree_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::owner_proposal_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    Some(largest)
}
