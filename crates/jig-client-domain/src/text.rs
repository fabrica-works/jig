//! Bounded provisional text assembled by offset (client/domain/client.md, 8.3).
use crate::{
    Domain, Down, Handle, Latest,
    contract::{Delivery, RunText, Subject, Text},
    watches::Watch,
};
use skein_lib::Token;
use skein_lib::Writer;

pub(crate) fn merge(watch: &Watch, mut value: RunText, limit: u32) -> Option<RunText> {
    if value.attempt != value.turn.attempt
        || value.through.checked_sub(value.from) != u32::try_from(value.text.bytes().len()).ok()
    {
        return None;
    }
    let mut old: &[u8] = &[];
    let mut skip = 0;
    if let Some(held) = held(watch.latest.as_ref()) {
        if value.turn < held.turn {
            return None;
        }
        let replayed = watch.state == crate::State::Behind && value.from == 0;
        if value.turn == held.turn && !replayed && !(value.cut && value.through >= held.through) {
            if value.through <= held.through {
                let mut kept = held.clone();
                if value.through == held.through {
                    kept.began = value.began;
                    kept.call = value.call;
                }
                return Some(kept);
            }
            if value.from > held.through && !value.cut {
                return None;
            }
            skip = usize::try_from(held.through.saturating_sub(value.from)).ok()?;
            old = held.text.bytes();
            value.cut |= held.cut;
        }
    }
    let incoming = value.text.bytes().get(skip..)?;
    if let Some(byte) = incoming.first()
        && continuation(*byte)
    {
        return None;
    }
    let length = old.len().checked_add(incoming.len())?;
    value.cut |= value.from > 0 && old.is_empty() || length > usize::try_from(limit).ok()?;
    value.text = tail(old, incoming, limit);
    value.from = value.through.checked_sub(u32::try_from(value.text.bytes().len()).ok()?)?;
    Some(value)
}

fn tail(old: &[u8], incoming: &[u8], limit: u32) -> Text {
    let length = old.len().checked_add(incoming.len()).expect("validated text lengths");
    let kept = length.min(usize::try_from(limit).expect("text bound fits usize"));
    let dropped = length.saturating_sub(kept);
    let incoming_from = dropped.saturating_sub(old.len());
    let old = suffix(old, dropped.min(old.len()));
    let incoming = suffix(incoming, incoming_from);
    let mut writer = Writer::new(old.len().checked_add(incoming.len()).expect("bounded text allocation"));
    writer.put(old).expect("bounded old text");
    writer.put(incoming).expect("bounded new text");
    Text::new(writer.finish(), limit).expect("bounded text")
}

fn suffix(bytes: &[u8], mut from: usize) -> &[u8] {
    while let Some(byte) = bytes.get(from) {
        if !continuation(*byte) {
            break;
        }
        from = from.checked_add(1).expect("index within text");
    }
    bytes.get(from..).expect("text boundary")
}
fn continuation(byte: u8) -> bool {
    byte & 0xc0 == 0x80
}

fn held(latest: Option<&Latest>) -> Option<&RunText> {
    match latest? {
        Latest::Jig(Delivery::Run(text)) => Some(text),
        Latest::Jig(Delivery::RunState(state)) => state.writing.as_ref(),
        Latest::Jig(_) | Latest::Family(_) => None,
    }
}
impl Domain {
    #[must_use]
    pub fn text(&self, handle: Handle) -> Option<&RunText> {
        held(self.document(handle)?.latest)
    }
}
impl Watch {
    pub(crate) fn opening(&self, watch: Token) -> Down {
        if let crate::Watchable::Jig(Subject::Run { task, .. }) = self.subject
            && let Some(text) = held(self.latest.as_ref())
        {
            return Down::Continue {
                watch,
                subject: Subject::Run { task, turn: Some(text.turn), offset: text.through },
            };
        }
        Down::Open { watch, subject: self.subject.clone() }
    }
}
