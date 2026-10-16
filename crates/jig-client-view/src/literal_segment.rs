//! Resource name segments are display bytes, never joined or parsed (authority.md, 4).
#![expect(
    clippy::disallowed_methods,
    reason = "Q39 permits UTF-8 classification for display only; supplied literal bytes remain opaque"
)]
use skein_lib::List;
use skein_ui::{Builder, Run};

/// Display byte count without changing the supplied segment.
#[must_use]
pub fn display_bytes(bytes: &[u8]) -> Option<u32> {
    let count = u32::try_from(bytes.len()).ok()?;
    if core::str::from_utf8(bytes).is_ok() { Some(count) } else { count.checked_mul(4) }
}

/// Byte envelope for a single segment, including non-UTF-8 escaping.
#[must_use]
pub fn max_bytes(text: u32) -> Option<u32> {
    text.checked_mul(4)
}

/// Preserve UTF-8 whole; escape a non-UTF-8 segment's bytes for display only.
pub fn render(b: &mut Builder, bytes: &[u8]) {
    if core::str::from_utf8(bytes).is_ok() {
        b.text(Run::Code, bytes);
        return;
    }
    let capacity = display_bytes(bytes).expect("escaped segment fits");
    let mut display = List::with_capacity(capacity);
    let digits = b"0123456789ABCDEF";
    for byte in bytes {
        for escaped in [
            b'\\',
            b'x',
            *digits.get(usize::from(byte / 16)).expect("high nibble is below sixteen"),
            *digits.get(usize::from(byte % 16)).expect("low nibble is below sixteen"),
        ] {
            display.push(escaped).expect("four bytes per input byte fit");
        }
    }
    b.text(Run::Code, display.as_slice());
}
