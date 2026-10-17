//! Whole result documents and exact task endings stay distinct (pages.md, 5.2).
use crate::{JigBindings, words::Words};
use skein_ui::Builder;

use crate::Borrowed;
use jig_client::contract::TaskResult;

pub(crate) fn render(
    b: &mut Builder,
    client: &Borrowed<'_>,
    value: &TaskResult,
    words: &Words,
    bindings: &mut JigBindings,
) {
    match value {
        TaskResult::Document(document) => crate::cards::result::facts(b, client, document, words, bindings),
        TaskResult::Ending(ending) => crate::owner_chip::ending(b, ending, client, words, bindings),
    }
}
/// The complete owner result plus the largest cancellation reason and scalar captions.
pub(crate) fn nodes(limits: &crate::Limits) -> Option<u32> {
    crate::result_nodes(limits)?.checked_add(skein_ui::markdown::max_nodes(limits.text)?)?.checked_add(8)
}
