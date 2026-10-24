//! Written shelf weight (codec.md 11.5).
use super::Context;
use super::kept::fixture_shelf::p0002::old;
use super::refusal::p0002::EMPTY;
use skein_codec::patch::{Build, Halt, Old};
use skein_codec::value::Node;

/// A shelf weighs ten per book plus its record key's bytes; empty refuses.
pub fn shelf_weight(old: &Old, new: &mut Build, at: Node, cx: &mut Context) -> Result<(), Halt> {
    let books = old.uint(old.field(at, old::shelf::BOOKS)?)?;
    if books == 0 {
        return Err(new.refuse(EMPTY));
    }
    let key = u32::try_from(cx.key.len()).unwrap_or(u32::MAX);
    let weight = u32::try_from(books).expect("books is a u32").saturating_mul(10).saturating_add(key);
    new.put_uint(u64::from(weight))
}
