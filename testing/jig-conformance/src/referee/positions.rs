//! A party's seen turns never return unread while their owner is retained.
use super::{Promise, Snapshot, Source, Violation, require};

pub(super) fn check(before: &Snapshot, after: &Snapshot) -> Result<(), Violation> {
    for ((party, task), through) in &before.read_positions {
        let moved = before.tasks.get(task).zip(after.tasks.get(task)).is_some_and(|(old, new)| {
            old.source == Source::Person(*party) && matches!(new.source, Source::Person(to) if to != *party)
        });
        if after.tasks.contains_key(task) && !moved {
            require(
                after.read_positions.get(&(*party, *task)).is_some_and(|position| position >= through),
                Promise::Lost,
                "a retained task lost or moved back its party's read position",
            )?;
        }
    }
    Ok(())
}
