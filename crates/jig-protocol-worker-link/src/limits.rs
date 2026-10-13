//! Bounds supplied by the layer (protocol/channel.md, 11).
use crate::{Component, Unusable};
use core::mem::size_of;
use jig_protocol_worker_wire::{
    self as wire,
    sizes::{self, Sizes},
};
use skein_channel::{self as channel, Role};
use skein_lib::{Duration, Queue};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub sizes: Sizes,
    pub chunk: u32,
    pub handshake: Duration,
    pub ping: Duration,
    pub silence: Duration,
    pub stall: Duration,
    pub backoff: Duration,
    pub ceiling: Duration,
    pub proved: Duration,
    pub grace: Duration,
    pub busy: Duration,
}
pub const MAX_OUT_DOWN: u32 = 24;
pub const MAX_OUT_FIRE: u32 = 12;
pub fn channel_limits(limits: &Limits) -> Result<channel::Limits, Unusable> {
    let bodies = sizes::limits(&limits.sizes).ok_or(Unusable::Limits)?;
    let cap = sizes::worker_cap(&limits.sizes).ok_or(Unusable::Limits)?;
    let mut skip = 0;
    for rule in wire::KINDS {
        let largest = sizes::largest_body(&limits.sizes, rule.body).ok_or(Unusable::Limits)?;
        skip = skip.max(largest);
    }
    let credential = wire::Credential::worst_case_bytes(&bodies).ok_or(Unusable::Limits)?;
    let Ok(credential) = u32::try_from(credential) else { return Err(Unusable::Limits) };
    Ok(channel::Limits {
        chunk: limits.chunk,
        credential,
        skip,
        output_bytes: cap.cap.bytes,
        output_frames: cap.cap.frames,
        kinds: 15,
    })
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.backoff == Duration::ZERO
        || limits.ceiling < limits.backoff
        || limits.handshake == Duration::ZERO
        || limits.proved == Duration::ZERO
    {
        return None;
    }
    let bodies = sizes::limits(&limits.sizes)?;
    let schema = wire::schema(&bodies).ok()?;
    let channel = channel_limits(limits).ok()?;
    let bound = channel::worst_case(&schema, Role::Initiator, &channel)?;
    let state = u64::try_from(size_of::<Component>()).ok()?;
    bound
        .checked_add(state)?
        .checked_add(u64::from(limits.sizes.name))?
        .checked_add(u64::from(limits.sizes.secret))?
        .checked_add(Queue::<channel::Event>::worst_case(channel::MAX_UP.checked_mul(3)?)?)?
        .checked_add(Queue::<channel::Lower>::worst_case(channel::MAX_DOWN.checked_mul(3)?)?)
}
