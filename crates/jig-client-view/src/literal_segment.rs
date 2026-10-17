//! Resource name segments are display bytes, never joined or parsed (authority.md, 4).
#![expect(
    clippy::disallowed_methods,
    reason = "Q39 permits UTF-8 classification for display only; supplied literal bytes remain opaque"
)]
use alloc::boxed::Box;
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

/// The words beside a displayed path share its finite node-text allowance.
pub(crate) struct PathDisplay {
    pub text: u32,
    pub reserved: u32,
    pub keep_last: bool,
}
pub(crate) trait Segment {
    fn bytes(&self) -> &[u8];
}
impl Segment for Box<[u8]> {
    fn bytes(&self) -> &[u8] {
        self
    }
}
impl Segment for jig_client::contract::Bytes {
    fn bytes(&self) -> &[u8] {
        self.bytes()
    }
}
pub(crate) fn path<S: Segment>(b: &mut Builder, path: &[S], words: &crate::words::Words, space: PathDisplay) {
    let mut total = 0_u32;
    for segment in path {
        total =
            total.checked_add(display_bytes(segment.bytes()).expect("bounded segment")).expect("bounded path bytes");
    }
    let minimum = max_bytes(space.text)
        .expect("checked segment bytes")
        .checked_mul(2)
        .expect("checked node text")
        .saturating_sub(space.reserved);
    let bound = if total <= minimum { minimum } else { words.node_text(space.text).saturating_sub(space.reserved) };
    let mut first = 0;
    if total > bound {
        let mark = words
            .book
            .say(words.language, crate::owner_authority_words::AuthorityPhrase::OmittedSegments.id(), &[])
            .expect("checked path mark")
            .text;
        first = path.len();
        let mut bytes = u32::try_from(mark.len()).expect("bounded mark");
        for segment in path.iter().rev() {
            let next =
                bytes.checked_add(display_bytes(segment.bytes()).expect("bounded segment")).expect("bounded path");
            if next > bound && !(space.keep_last && first == path.len()) {
                break;
            }
            bytes = next;
            first = first.checked_sub(1).expect("one preceding segment remains");
        }
        b.text(Run::Plain, &mark);
    }
    for segment in path.get(first..).expect("suffix lies inside path") {
        render(b, segment.bytes());
    }
}
