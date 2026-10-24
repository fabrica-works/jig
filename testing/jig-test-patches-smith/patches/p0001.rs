//! A turn's token count reads its replay and the frame's own key (codec.md 11.5).
use super::{Context, kept::fixture_turn::p0001::old, refusal::p0001::EMPTY};
use skein_codec::{
    patch::{Build, Halt, Old},
    value::Node,
};

/// Turn.tokens: replay bytes plus the frame name; an empty replay refuses.
pub fn turn_tokens(old: &Old, new: &mut Build, at: Node, cx: &mut Context) -> Result<(), Halt> {
    let replay = old.content(old.field(at, old::turn::REPLAY)?)?;
    if replay.is_empty() {
        return Err(new.refuse(EMPTY));
    }
    let tokens = replay.len().checked_add(cx.key.len()).ok_or(Halt::Room)?;
    new.put_uint(u64::try_from(tokens).map_err(|_| Halt::Room)?)
}
