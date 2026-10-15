//! Whole specification presentation (client/domain/contract.md, 5.2; view.md, 10).
use crate::ui::{Builder, Run};
use crate::{
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};

use jig_domain_client::{
    Limits,
    contract::{Parameter, ParameterDocument, Specification, SpecificationDocument},
};

/// Show every supplied parameter and input without interpreting opaque values.
pub fn specification(b: &mut Builder, value: &Specification, words: &Words) {
    crate::ui::markdown::read(value.words.bytes(), 2, b);
    for parameter in &value.parameters {
        match parameter {
            Parameter::Number { name, value } => {
                words.text(b, JigPhrase::SpecificationNumber, &[Hole::Number(u64::from(*name)), Hole::Number(*value)]);
            }
            Parameter::Bytes { name, value } => {
                words.text(b, JigPhrase::SpecificationBytes, &[Hole::Number(u64::from(*name))]);
                b.text(Run::Plain, value.bytes());
            }
            Parameter::Resource { name, connector, resource } => words.text(
                b,
                JigPhrase::SpecificationResource,
                &[Hole::Number(u64::from(*name)), Hole::Number(u64::from(*connector)), Hole::Number(*resource)],
            ),
        }
    }
    for task in &value.inputs {
        words.text(b, JigPhrase::SpecificationInput, &[Hole::Number(*task)]);
    }
}
/// Retain the owner's loss marker separately from the supplied values.
pub fn document(b: &mut Builder, value: &SpecificationDocument, words: &Words) {
    crate::document_text::markdown(b, &value.specification.words, words);
    for parameter in &value.specification.parameters {
        match parameter {
            ParameterDocument::Number { name, value } => {
                words.text(b, JigPhrase::SpecificationNumber, &[Hole::Number(u64::from(*name)), Hole::Number(*value)]);
            }
            ParameterDocument::Bytes { name, value } => {
                words.text(b, JigPhrase::SpecificationBytes, &[Hole::Number(u64::from(*name))]);
                crate::document_text::plain(b, value, words);
            }
            ParameterDocument::Resource { name, connector, resource } => words.text(
                b,
                JigPhrase::SpecificationResource,
                &[Hole::Number(u64::from(*name)), Hole::Number(u64::from(*connector)), Hole::Number(*resource)],
            ),
        }
    }
    for task in &value.specification.inputs {
        words.text(b, JigPhrase::SpecificationInput, &[Hole::Number(*task)]);
    }
    if value.cut {
        words.text(b, JigPhrase::SpecificationCut, &[]);
    }
}
/// Maximum nodes for whole words, all parameter variants, input rows and loss marker.
#[must_use]
pub fn nodes(limits: &Limits) -> Option<u32> {
    crate::ui::markdown::max_nodes(limits.text)?.checked_add(limits.rows.checked_mul(4)?)?.checked_add(2)
}
