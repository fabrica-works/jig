//! Sized writing of framed items (codec.md, 3).
use alloc::boxed::Box;
use skein_codec::Reason;
use skein_lib::{List, Writer};
pub(super) fn join(items: List<Box<[u8]>>, bound: u32) -> Result<Box<[u8]>, Reason> {
    let mut length = 0_usize;
    for item in &items {
        length = match length.checked_add(size_of::<u32>()) {
            Some(value) => value,
            None => return Err(Reason::Bound),
        };
        length = match length.checked_add(item.len()) {
            Some(value) => value,
            None => return Err(Reason::Bound),
        };
    }
    if length > usize::try_from(bound).expect("bound fits usize") {
        return Err(Reason::Bound);
    }
    let mut out = Writer::new(length);
    for item in items.into_boxed() {
        skein_codec::piece::write(&mut out, &item).expect("measured piece");
    }
    Ok(out.finish())
}
