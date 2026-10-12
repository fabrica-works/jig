//! A conversation's attempts share one first attempt (domain/engine.md, 7.1).
use super::{Promise, Violation, require};
use std::collections::BTreeMap;

pub(super) fn check(
    conversations: &mut BTreeMap<u64, u64>,
    task: u64,
    attempt: u64,
    conversation: u64,
    resumed: bool,
) -> Result<(), Violation> {
    require(
        conversation != 0
            && conversation <= attempt
            && if resumed {
                conversation < attempt && conversations.get(&task).is_none_or(|first| *first == conversation)
            } else {
                conversation == attempt
            },
        Promise::Once,
        "a conversation's attempts must name one first attempt; a fresh start names its own",
    )?;
    conversations.insert(task, conversation);
    Ok(())
}
