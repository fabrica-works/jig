//! Fixed owner-written bytes and ordinary readable children (view.md, section 3).
use crate::words::{Word, Words};
use jig_view::Owner;
use jig_view::ui::phrases::Hole;
use jig_view::ui::{Builder, DrawingKind};
pub const BYTES: u32 = 8;
pub const KIND: DrawingKind =
    DrawingKind { owner: (Owner::Application { number: jig_test_domain_client::FIRST }).opaque(), kind: 1 };
pub fn value(b: &mut Builder, words: &Words, value: u64) {
    let bytes = value.to_le_bytes();
    b.drawing(KIND, &words.say(Word::Drawing, &[]), &bytes, BYTES);
    words.status(b, Word::Value, &[Hole::Number(value)]);
    b.close();
}
