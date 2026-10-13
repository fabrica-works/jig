//! Expectations from both scripted neighbours, never component state.
use crate::{Trace, World};
use skein_lib::Duration;
use std::collections::{BTreeMap, BTreeSet};
/// Check once-only relay endings, busy jitter, grace epochs, and hello ordering.
///
/// # Errors
/// Returns the first violated observation contract.
pub fn judge(world: &World) -> Result<(), &'static str> {
    let mut relays = BTreeSet::new();
    let mut busy = BTreeMap::new();
    let mut lost = false;
    let mut grace = false;
    let mut closed = false;
    for trace in world.trace() {
        if closed {
            return Err("Closed is last");
        }
        match trace {
            Trace::RelayStarted(run, attempt, call) => {
                relays.insert((*run, *attempt, call.clone()));
            }
            Trace::RelayEnded(run, attempt, call) => {
                if !relays.remove(&(*run, *attempt, call.clone())) {
                    return Err("relay ends once");
                }
            }
            Trace::Busy(at, run, attempt, turn) => {
                busy.insert((*run, *attempt, *turn), *at);
            }
            Trace::Kept(run, attempt, turn) => {
                busy.remove(&(*run, *attempt, *turn));
            }
            Trace::Acknowledged(run, attempt) => busy.retain(|(r, a, _), _| r != run || a != attempt),
            Trace::Resend(at, run, attempt, turn) => {
                let Some(start) = busy.remove(&(*run, *attempt, *turn)) else {
                    return Err("resend follows busy");
                };
                let span = at.saturating_since(start);
                if span < Duration::from_millis(100) || span > Duration::from_millis(200) {
                    return Err("busy resend within jitter");
                }
            }
            Trace::Ready => {
                lost = false;
                grace = false;
            }
            Trace::Lost => {
                if lost {
                    return Err("Lost once per channel");
                }
                lost = true;
                busy.clear();
            }
            Trace::GracePassed => {
                if !lost || grace {
                    return Err("grace once per loss");
                }
                grace = true;
            }
            Trace::Closed => {
                if !relays.is_empty() {
                    return Err("every relay ended before Stop");
                }
                closed = true;
            }
        }
    }
    for channel in world.records() {
        let first = channel.iter().find(|record| record.kind >= 256);
        if let Some(first) = first
            && first.kind != 0x0100
        {
            return Err("hello first on every channel");
        }
        if channel.iter().filter(|record| record.kind == 0x0100).count() > 1 {
            return Err("hello once per channel");
        }
    }
    if closed && world.next_deadline().is_some() {
        return Err("no timer after Closed");
    }
    Ok(())
}
