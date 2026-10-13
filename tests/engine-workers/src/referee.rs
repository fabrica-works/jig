//! The root hears each channel's life once, and the component terminal last.
use jig_protocol_engine_workers::Request;
use skein_lib::Token;
use std::collections::BTreeMap;

/// Checks the ordering of the root's typed channel observations.
///
/// # Errors
/// Returns the first channel lifetime or terminal ordering violated.
pub fn judge(root: &[Request]) -> Result<(), &'static str> {
    let mut channels = BTreeMap::<Token, bool>::new();
    let mut closed = false;
    for request in root {
        if closed {
            return Err("Closed is last");
        }
        match request {
            Request::Hello { channel, .. } => {
                if channels.insert(*channel, true).is_some() {
                    return Err("Hello once");
                }
            }
            Request::Lost { channel } => {
                let Some(live) = channels.get_mut(channel) else {
                    return Err("Lost follows Hello");
                };
                if !*live {
                    return Err("Lost once");
                }
                *live = false;
            }
            Request::Up { channel, .. } => {
                if channels.get(channel) != Some(&true) {
                    return Err("Up only between Hello and Lost");
                }
            }
            Request::Closed => {
                if channels.values().any(|live| *live) {
                    return Err("Closed follows every Lost");
                }
                closed = true;
            }
            Request::Take { .. }
            | Request::TurnAway { .. }
            | Request::Lower { .. }
            | Request::Close { .. }
            | Request::Abort { .. } => return Err("only root outputs"),
        }
    }
    Ok(())
}
