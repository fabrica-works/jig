//! Whole result documents and exact task endings stay distinct (pages.md, 5.2).
use crate::{Builder, JigBindings, words::Words};
use jig_client::{Domain, contract::TaskResult};

pub(crate) fn render(b: &mut Builder, client: &Domain, value: &TaskResult, words: &Words, bindings: &mut JigBindings) {
    match value {
        TaskResult::Document(document) => crate::cards::result::facts(b, client, document, words, bindings),
        TaskResult::Ending(ending) => crate::owner_chip::ending(b, ending, client, words, bindings),
    }
}
/// The complete owner result plus the largest cancellation reason and scalar captions.
pub(crate) fn nodes(limits: &jig_client::Limits) -> Option<u32> {
    crate::result_nodes(limits)?.checked_add(crate::markdown::max_nodes(limits.text)?)?.checked_add(8)
}
