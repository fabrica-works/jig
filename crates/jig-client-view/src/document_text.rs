//! Present each supplied text loss fact (client/domain/contract.md, 5.2; view.md, 10).
use crate::{phrases::jig::JigPhrase, words::Words};
use skein_ui::{Builder, Run};

use jig_client::contract::DocumentText;

/// Show the exact supplied bytes and that value's own loss marker.
pub fn plain(b: &mut Builder, value: &DocumentText, words: &Words) {
    b.text(Run::Plain, value.bytes());
    loss(b, value, words);
}
/// Show structured words and that value's own loss marker.
pub fn markdown(b: &mut Builder, value: &DocumentText, words: &Words) {
    skein_ui::markdown::read(value.bytes(), 2, b);
    loss(b, value, words);
}
/// Show only the independently supplied loss fact next to existing content.
pub fn loss(b: &mut Builder, value: &DocumentText, words: &Words) {
    if value.cut {
        words.text(b, JigPhrase::DocumentTextCut, &[]);
    }
}
