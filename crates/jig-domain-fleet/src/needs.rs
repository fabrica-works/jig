//! Agent admission and model eligibility for configured charters (domain/engine.md, section 8).
use crate::{Bounds, HostKind, Kinds};
use alloc::boxed::Box;
use skein_lib::{Duration, List};

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
    Turns { charter: u32 },
    Spend,
    Time { charter: u32 },
    Waiting { charter: u32 },
    Checks { charter: u32 },
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

/// The first agent bound that cannot hold this host kind's configured charters.
///
/// Checks applicable charters in their configured order, then deployment-wide
/// bounds if any charter applies. Missing models narrow placement eligibility;
/// only models the host declares have their window and output checked here.
/// This is the same decision used for hello admission (domain/engine.md, section 8).
#[must_use]
pub fn shortfall(needs: &Needs, bounds: &Bounds, kind: HostKind) -> Option<AgentShortfall> {
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
            let Some(held) = held else { continue };
            if held.window < need.window {
                return Some(AgentShortfall::Window { charter: charter.charter, model: number });
            }
            if held.output < need.output {
                return Some(AgentShortfall::Output { charter: charter.charter, model: number });
            }
        }
        if bounds.turns < charter.turns {
            return Some(AgentShortfall::Turns { charter: charter.charter });
        }
        if bounds.time < charter.time {
            return Some(AgentShortfall::Time { charter: charter.charter });
        }
        if bounds.waiting < charter.waiting {
            return Some(AgentShortfall::Waiting { charter: charter.charter });
        }
        if let Some(reserve) = charter.change_reserve
            && bounds.checks > reserve
        {
            return Some(AgentShortfall::Checks { charter: charter.charter });
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

/// One eligibility bit per configured charter, bounded by `Needs`' owned bytes.
/// The parent admits charter numbers; an unconfigured number has no needs here.
pub(crate) fn eligibility(needs: &Needs, models: &[crate::ModelBound]) -> Box<[bool]> {
    let mut eligible = List::with_capacity(u32::try_from(needs.charters.len()).expect("configured charter count"));
    for charter in &needs.charters {
        let mut all = true;
        for need in &charter.models {
            let mut declared = false;
            for model in models {
                if model.name == need.name {
                    declared = true;
                    break;
                }
            }
            if !declared {
                all = false;
                break;
            }
        }
        eligible.push(all).expect("one bit per configured charter");
    }
    eligible.into_boxed()
}

pub(crate) fn allows(needs: &Needs, eligible: &[bool], charter: u32) -> bool {
    for (index, need) in needs.charters.iter().enumerate() {
        if need.charter == charter {
            return *eligible.get(index).expect("one bit per configured charter");
        }
    }
    true
}

/// Declared models in the configured needs' order, without retaining peer-owned names.
pub(crate) fn declarations(needs: &Needs, models: &[crate::ModelBound]) -> Box<[bool]> {
    let mut count = 0_u32;
    for charter in &needs.charters {
        count = count.checked_add(u32::try_from(charter.models.len()).expect("model count")).expect("model count");
    }
    let mut result = List::with_capacity(count);
    for charter in &needs.charters {
        for need in &charter.models {
            let mut declared = false;
            for model in models {
                if model.name == need.name && model.window >= need.window && model.output >= need.output {
                    declared = true;
                }
            }
            result.push(declared).expect("one bit per model need");
        }
    }
    result.into_boxed()
}

pub(crate) fn allows_models(needs: &Needs, declarations: &[bool], models: &[ModelNeed]) -> bool {
    for model in models {
        let mut at = 0_usize;
        let mut found = false;
        for charter in &needs.charters {
            for configured in &charter.models {
                if configured.name == model.name
                    && configured.window >= model.window
                    && configured.output >= model.output
                    && declarations.get(at) == Some(&true)
                {
                    found = true;
                }
                at = at.checked_add(1).expect("bounded configured model index");
            }
        }
        if !found {
            return false;
        }
    }
    true
}
