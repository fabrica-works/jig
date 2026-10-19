//! Single-turn failure evidence, authenticated before revealing payload facts.
use super::{Answer, Range, Scope, TurnFailure, Why};
use crate::Core;
use jig_domain_tasks::TaskRecord;

pub(super) fn failure(
    core: &Core,
    range: &Range,
    task: u64,
    attempt: u64,
    turn: u32,
    owner: &TaskRecord,
    why: TurnFailure,
) -> Answer {
    if range.scope != (Scope::Turn { task, attempt, turn })
        || owner.number != task
        || core.people.role(range.party, owner.project).is_none()
    {
        return Answer::None(Why::Unknown);
    }
    let Some(bytes) = TurnFailure::owned_bytes(owner) else { return Answer::None(Why::Busy) };
    if bytes > range.bytes {
        return Answer::None(Why::Busy);
    }
    Answer::None(match why {
        TurnFailure::TooLarge { bytes } => Why::TooLarge { bytes },
        TurnFailure::Malformed => Why::Malformed,
    })
}
