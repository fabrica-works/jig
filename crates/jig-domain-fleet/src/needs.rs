//! Agent admission for the configured charters (domain/engine.md, section 8).
use crate::{Bounds, HostKind, Kinds};
use alloc::boxed::Box;
use skein_lib::Duration;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Needs {
    pub charters: Box<[CharterNeeds]>,
    pub spend: u64,
    pub message_bytes: u32,
    pub start_messages: u32,
    pub unacknowledged: u32,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CharterNeeds {
    pub charter: u32,
    pub kinds: Kinds,
    pub models: Box<[ModelNeed]>,
    pub turns: u32,
    pub time: Duration,
    pub waiting: Duration,
    pub change_reserve: Option<Duration>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ModelNeed {
    pub name: Box<[u8]>,
    pub window: u32,
    pub output: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AgentShortfall {
    Model { charter: u32, model: u32 },
    Window { charter: u32, model: u32 },
    Output { charter: u32, model: u32 },
    Turns,
    Spend,
    Time,
    Waiting,
    Checks,
    Message,
    Inbox,
    Unacknowledged,
}

pub(crate) fn owned_bytes(needs: &Needs) -> Option<u64> {
    let mut bytes =
        u64::try_from(needs.charters.len()).ok()?.checked_mul(u64::try_from(size_of::<CharterNeeds>()).ok()?)?;
    for charter in &needs.charters {
        bytes = bytes.checked_add(
            u64::try_from(charter.models.len()).ok()?.checked_mul(u64::try_from(size_of::<ModelNeed>()).ok()?)?,
        )?;
        for model in &charter.models {
            bytes = bytes.checked_add(u64::try_from(model.name.len()).ok()?)?;
        }
    }
    Some(bytes)
}

pub(crate) fn shortfall(needs: &Needs, bounds: &Bounds, kind: HostKind) -> Option<AgentShortfall> {
    let mut applicable = false;
    for charter in &needs.charters {
        if !charter.kinds.allows(kind) {
            continue;
        }
        applicable = true;
        for (index, need) in charter.models.iter().enumerate() {
            let number = u32::try_from(index).expect("configured model count");
            let mut held = None;
            for model in &bounds.models {
                if model.name == need.name {
                    held = Some(model);
                    break;
                }
            }
            let Some(held) = held else {
                return Some(AgentShortfall::Model { charter: charter.charter, model: number });
            };
            if held.window < need.window {
                return Some(AgentShortfall::Window { charter: charter.charter, model: number });
            }
            if held.output < need.output {
                return Some(AgentShortfall::Output { charter: charter.charter, model: number });
            }
        }
        if bounds.turns < charter.turns {
            return Some(AgentShortfall::Turns);
        }
        if bounds.time < charter.time {
            return Some(AgentShortfall::Time);
        }
        if bounds.waiting < charter.waiting {
            return Some(AgentShortfall::Waiting);
        }
        if let Some(reserve) = charter.change_reserve
            && bounds.checks > reserve
        {
            return Some(AgentShortfall::Checks);
        }
    }
    if applicable {
        if bounds.spend < needs.spend {
            return Some(AgentShortfall::Spend);
        }
        if bounds.message_bytes < needs.message_bytes {
            return Some(AgentShortfall::Message);
        }
        if bounds.inbox < needs.start_messages {
            return Some(AgentShortfall::Inbox);
        }
        if bounds.unacknowledged < needs.unacknowledged {
            return Some(AgentShortfall::Unacknowledged);
        }
    }
    None
}
