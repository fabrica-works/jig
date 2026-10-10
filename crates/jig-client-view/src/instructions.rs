//! Supplied instruction facts, without default substitution (objects.md, 11; tasks.md, 7).
use crate::{Builder, phrases::jig::JigPhrase, words::Words};
use jig_client::{
    Limits,
    contract::{DocumentText, Text},
};

/// Show the supplied text, its own loss, known empty text or absence.
pub(crate) fn document(b: &mut Builder, value: &Option<DocumentText>, words: &Words) {
    if let Some(value) = value {
        supplied(b, value, words);
    } else {
        words.text(b, JigPhrase::InstructionNotSupplied, &[]);
    }
}
/// The amendment already shows complete text; retain its other supplied facts beside it.
pub(crate) fn before(b: &mut Builder, value: &Option<DocumentText>, words: &Words) {
    let Some(value) = value else {
        words.text(b, JigPhrase::InstructionNotSupplied, &[]);
        return;
    };
    if value.cut || value.bytes().is_empty() {
        supplied(b, value, words);
    }
}
fn supplied(b: &mut Builder, value: &DocumentText, words: &Words) {
    if value.bytes().is_empty() && !value.cut {
        words.text(b, JigPhrase::InstructionEmpty, &[]);
    }
    crate::document_text::markdown(b, value, words);
}
/// A whole replacement baseline is available only from complete supplied text.
pub(crate) fn whole(value: &Option<DocumentText>) -> Option<Text> {
    match value {
        Some(value) => value.whole(),
        None => None,
    }
}
/// Markdown and one independently supplied loss or presence explanation.
pub(crate) fn nodes(limits: &Limits) -> Option<u32> {
    crate::markdown::max_nodes(limits.text)?.checked_add(1)
}
