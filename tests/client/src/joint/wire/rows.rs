//! Whole current and retained list rows (contract.md, 5.5–5.6).
//! A row without runtime or changed time remains without those facts; joined
//! chip metadata and cached replies never substitute for them.
use super::{chips, owner};
use jig_domain_client::{Limits, contract as client};
use jig_domain_core::documents;

/// # Errors
/// Refuses a whole chip or source text allocation above the receiving bounds.
pub fn conversation(
    value: &documents::ConversationLine,
    limits: &Limits,
) -> Result<client::ConversationLine, owner::Invalid> {
    Ok(client::ConversationLine {
        chip: chips::chip(&value.chip, limits)?,
        project: value.project,
        first_words: owner::text(&value.first_words, limits)?,
        unread: value.replies_unread.map(|value| match value {
            documents::Count::Known(value) => client::Count { value, limited: false },
            documents::Count::AtLeast(value) => client::Count { value, limited: true },
        }),
        cached_replies: value.cached_replies,
        seen: value.seen,
        last_reply: value.last_reply,
        delegates: value.delegates,
        budget: owner::budget(&value.budget),
        runtime: None,
        changed: None,
    })
}

#[must_use]
pub fn phases(value: documents::PhaseCounts) -> client::OwnerPhaseCounts {
    client::OwnerPhaseCounts {
        waiting: value.waiting,
        idle: value.idle,
        starting: value.starting,
        running: value.running,
        backing_off: value.backing_off,
        closing: value.closing,
        held: value.held,
        done: value.done,
        failed: value.failed,
        cancelled: value.cancelled,
    }
}

/// # Errors
/// Refuses a whole source chip allocation above the receiving bounds.
pub fn goal(value: &documents::GoalLine, limits: &Limits) -> Result<client::GoalLine, owner::Invalid> {
    Ok(client::GoalLine {
        chip: chips::chip(&value.chip, limits)?,
        priority: value.priority,
        progress: phases(value.phases),
        budget: owner::budget(&value.budget),
        held_below: value.held_below,
        changed: None,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    const UTF8_TEXTS: [&[u8]; 1] = [b"~\0"];

    use super::*;
    use skein_lib::Wall;

    pub(crate) fn limits() -> Limits {
        Limits { rows: 4, text: 200, document_bytes: 100_000, ..crate::test::limits().jig }
    }

    pub(crate) fn conversation_source() -> documents::ConversationLine {
        let chip = chips::tests::source_chip();
        documents::ConversationLine {
            budget: chip.details.budget,
            chip,
            project: u32::MAX,
            first_words: documents::Text { bytes: Box::from(UTF8_TEXTS[0]), cut: true },
            seen: u64::MAX,
            last_reply: 0,
            replies_unread: None,
            cached_replies: u32::MAX,
            delegates: 17,
        }
    }
    pub(crate) fn goal_source() -> documents::GoalLine {
        let chip = chips::tests::source_chip();
        documents::GoalLine {
            budget: chip.details.budget,
            chip,
            priority: u32::MAX,
            phases: documents::PhaseCounts {
                waiting: 1,
                idle: 2,
                starting: 3,
                running: 4,
                backing_off: 5,
                closing: 6,
                held: 7,
                done: 8,
                failed: 9,
                cancelled: u32::MAX,
            },
            held_below: 23,
        }
    }
    #[test]
    fn conversation_rows_keep_missing_counts_and_clocks_separate_from_cached_replies_and_whole_chip_facts() {
        let mut source = conversation_source();
        let limits = limits();
        let value = conversation(&source, &limits).expect("whole supplied row");
        assert_eq!(value.runtime, None);
        assert_eq!(value.changed, None);
        assert_eq!(value.unread, None);
        assert_eq!(value.cached_replies, u32::MAX);
        assert_eq!((value.project, value.seen, value.last_reply, value.delegates), (u32::MAX, u64::MAX, 0, 17));
        assert_eq!(value.first_words.bytes.bytes(), UTF8_TEXTS[0]);
        assert!(value.first_words.cut);
        assert_eq!(value.budget, owner::budget(&source.budget));
        let client::ChipPhase::Owner { details, .. } = value.chip.details.as_ref() else { panic!("whole chip") };
        assert_eq!(details.ended_at, Some(Wall::EPOCH));
        for (count, expected) in [
            (documents::Count::Known(0), client::Count { value: 0, limited: false }),
            (documents::Count::AtLeast(u32::MAX), client::Count { value: u32::MAX, limited: true }),
        ] {
            source.replies_unread = Some(count);
            assert_eq!(conversation(&source, &limits).expect("exact supplied count").unread, Some(expected));
        }
    }
    #[test]
    fn goal_rows_copy_all_ten_owner_phase_counts_and_whole_budget_without_a_changed_time() {
        let source = goal_source();
        let value = goal(&source, &limits()).expect("whole supplied goal");
        assert_eq!(value.changed, None);
        assert_eq!((value.priority, value.held_below), (u32::MAX, 23));
        assert_eq!(
            value.progress,
            client::OwnerPhaseCounts {
                waiting: 1,
                idle: 2,
                starting: 3,
                running: 4,
                backing_off: 5,
                closing: 6,
                held: 7,
                done: 8,
                failed: 9,
                cancelled: u32::MAX,
            }
        );
        assert_eq!(value.budget, owner::budget(&source.budget));
        assert_eq!(value.chip.task, u64::MAX);
    }
}
