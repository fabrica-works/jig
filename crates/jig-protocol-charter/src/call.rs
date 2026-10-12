//! Smith's call name, read at the engine's protocol boundary
//! (protocol/channel.md, section 7).
use skein_lib::Reader;

/// The name's key; the routing checks its activation against the fence.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CallKey {
    pub activation: u64,
    pub completion: u32,
    pub position: u32,
}

#[must_use]
pub fn call_key(name: &[u8]) -> Option<CallKey> {
    let mut reader = Reader::new(name);
    let value = smith_channel::CallName::decode(&smith_channel::CEILINGS, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(CallKey { activation: value.activation(), completion: value.completion(), position: value.position() })
}
