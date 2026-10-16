//! Fixed owner-written bytes and ordinary readable children (view.md, section 3).
use crate::words::{Word, Words};
use jig_client_view::Owner;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, DrawingKind};
pub const BYTES: u32 = 8;
pub const KIND: DrawingKind =
    DrawingKind { owner: (Owner::Application { number: jig_client_test_domain::FIRST }).opaque(), kind: 1 };
pub fn value(b: &mut Builder, words: &Words, value: u64) {
    let bytes = value.to_le_bytes();
    b.drawing(KIND, &words.say(Word::Drawing, &[]), &bytes, BYTES);
    words.status(b, Word::Value, &[Hole::Number(value)]);
    b.close();
}
