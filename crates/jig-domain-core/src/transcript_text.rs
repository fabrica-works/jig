//! Retained protocol-rendered transcript text (domain/engine.md, section 9).
use crate::{Transcript, TurnRecord};
use alloc::boxed::Box;

/// A loaded stored turn and its protocol-rendered UTF-8 text; never stored together.
pub type TranscriptRow = (TurnRecord, Option<Box<[u8]>>);

/// Rendered bytes and the original text bytes they still show (excluding a cut marker).
#[derive(Debug)]
pub struct TranscriptText {
    pub bytes: Box<[u8]>,
    pub shown: u64,
}

/// Keeps a text's end without decoding it; the protocol supplied UTF-8.
pub(crate) fn cut_transcript_text(text: &[u8], bound: usize) -> TranscriptText {
    const MARKER: &[u8] = b"[earlier text cut]\n";
    if text.len() <= bound {
        return TranscriptText {
            bytes: Box::from(text),
            shown: u64::try_from(text.len()).expect("text length fits u64"),
        };
    }
    if bound < MARKER.len() {
        return TranscriptText { bytes: Box::new([]), shown: 0 };
    }
    let room = bound.checked_sub(MARKER.len()).expect("marker fits");
    let mut start = text.len().checked_sub(room).expect("oversized text");
    for _ in 0_u8..3 {
        match text.get(start) {
            Some(byte) if byte & 0xc0 == 0x80 => start = start.checked_add(1).expect("inside text"),
            Some(_) | None => break,
        }
    }
    let suffix = text.get(start..).expect("UTF-8 suffix boundary within text");
    let mut writer = skein_lib::Writer::new(MARKER.len().checked_add(suffix.len()).expect("bounded text"));
    writer.put(MARKER).expect("measured marker");
    writer.put(suffix).expect("measured suffix");
    TranscriptText { bytes: writer.finish(), shown: u64::try_from(suffix.len()).expect("bounded suffix") }
}

impl Transcript {
    pub(crate) fn append_text(&mut self, text: Option<Box<[u8]>>, bound: u32) {
        let text = match text {
            Some(text) => text,
            None => Box::from(&b"[a turn not readable as text]\n"[..]),
        };
        self.text_bytes = self
            .text_bytes
            .checked_add(u64::try_from(text.len()).expect("text length fits u64"))
            .expect("loaded text count");
        let kept = if text.len() <= usize::try_from(bound).expect("u32 fits usize") {
            TranscriptText { shown: u64::try_from(text.len()).expect("bounded text"), bytes: text }
        } else {
            cut_transcript_text(&text, usize::try_from(bound).expect("u32 fits usize"))
        };
        if kept.bytes.is_empty() {
            return;
        }
        let length = u64::try_from(kept.bytes.len()).expect("bounded text");
        for _ in 0..self.texts.len() {
            if self.text_kept.checked_add(length).expect("two bounded texts") <= u64::from(bound) {
                break;
            }
            let old = self.texts.pop().expect("overfull text queue has an oldest text");
            self.text_kept = self
                .text_kept
                .checked_sub(u64::try_from(old.bytes.len()).expect("bounded text"))
                .expect("counted text");
        }
        self.texts.push(kept);
        self.text_kept = self.text_kept.checked_add(length).expect("bounded text retention");
    }
}
