//! A party's seen turns never return unread while their owner is retained.
use super::{Promise, Snapshot, Violation, require};
use std::collections::BTreeMap;

pub(super) fn check(before: &BTreeMap<(u64, u64), u64>, after: &Snapshot) -> Result<(), Violation> {
    for ((party, task), through) in before {
        if after.tasks.contains_key(task) {
            require(
                after.read_positions.get(&(*party, *task)).is_some_and(|position| position >= through),
                Promise::Lost,
                "a retained task lost or moved back its party's read position",
            )?;
        }
    }
    Ok(())
}
