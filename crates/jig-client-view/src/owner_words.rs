//! Normal jig phrasebooks compose every exact owner fragment (view.md, 7).
//! Composition happens while languages load, never while a page renders.
use crate::phrases::{Language, PhraseShape, Phrasebook};
use alloc::boxed::Box;
use skein_lib::List;

#[must_use]
pub fn phrase_count() -> u32 {
    u32::try_from(crate::phrases::jig::ALL.len()).expect("normal phrase count fits")
        .checked_add(u32::try_from(crate::owner_authority_words::ALL.len()).expect("authority phrase count fits")).expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::owner_wake_words::ALL.len()).expect("wake phrase count fits")).expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::proposed_task_words::ALL.len()).expect("batch phrase count fits")).expect("composed phrase count fits")
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
    Some(largest)
}
