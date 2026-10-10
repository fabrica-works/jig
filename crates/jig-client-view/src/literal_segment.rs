//! Resource name segments are display bytes, never joined or parsed (authority.md, 4).
use crate::{Builder, Run};
use skein_lib::List;

/// Preserve UTF-8 whole; escape a non-UTF-8 segment's bytes for display only.
pub fn render(b: &mut Builder, bytes: &[u8]) {
    if core::str::from_utf8(bytes).is_ok() {
        b.text(Run::Code, bytes);
        return;
    }
    let capacity =
        u32::try_from(bytes.len()).expect("bounded segment length fits").checked_mul(4).expect("escaped segment fits");
    let mut display = List::with_capacity(capacity);
    let digits = b"0123456789ABCDEF";
    for byte in bytes {
        for escaped in [b'\\', b'x', digits[usize::from(byte / 16)], digits[usize::from(byte % 16)]] {
            display.push(escaped).expect("four bytes per input byte fit");
        }
    }
    b.text(Run::Code, display.as_slice());
}
