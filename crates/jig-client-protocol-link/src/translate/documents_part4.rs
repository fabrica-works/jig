//! Field-for-field document values (client/domain/contract.md, 5–7).
use super::{
    Box, Limits, List, Reader, Wall, bytes, c, chip_fields, chip_piece, count_fields, needs_fields, piece_problem,
    runtime_fields, too_large, w,
};
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn conversation_line_fields(
    value: &w::ConversationLine,
    limits: &Limits,
) -> Result<c::ConversationLine, w::Problem> {
    let opt0 = match value.runtime() {
        Some(item) => Some(runtime_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.unread() {
        Some(item) => Some(count_fields(item, limits)),
        None => None,
    };
    let opt2 = match value.changed() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::ConversationLine {
        runtime: opt0,
        delegates: value.delegates(),
        budget: crate::translate::facts::owner_budget(value.budget(), limits),
        chip: chip_fields(value.chip(), limits)?,
        project: value.project(),
        first_words: crate::translate::facts::document_text(value.first_words(), limits)?,
        unread: opt1,
        cached_replies: value.cached_replies(),
        seen: value.seen(),
        last_reply: value.last_reply(),
        changed: opt2,
    })
}

pub(crate) fn conversation_line_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ConversationLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::ConversationLine::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = conversation_line_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn cursor_fields(value: &w::Cursor, limits: &Limits) -> Result<c::Cursor, w::Problem> {
    match c::Cursor::new(bytes::copy_of(value.value()), limits.wire.cursor_value) {
        Ok(value) => Ok(value),
        Err(_) => Err(too_large()),
    }
}

pub(crate) fn conversations_fields(value: &w::Conversations, limits: &Limits) -> Result<c::Conversations, w::Problem> {
    Ok(c::Conversations {
        rows: conversation_line_piece(value.rows(), limits, limits.client.rows)?,
        more: value.more(),
        after: cursor_fields(value.after(), limits)?,
    })
}

pub(crate) fn object_key_fields(value: &w::ObjectKey, limits: &Limits) -> c::ObjectKey {
    match value {
        w::ObjectKey::Task(row) => c::ObjectKey::Task(row.value()),
        w::ObjectKey::Proposal(row) => {
            c::ObjectKey::Proposal(crate::translate::answers::proposal_key(row.value(), limits))
        }
        w::ObjectKey::Escalation(row) => c::ObjectKey::Escalation(row.value()),
        w::ObjectKey::Question(row) => c::ObjectKey::Question { task: row.task(), message: row.message() },
        w::ObjectKey::PersonTask(row) => c::ObjectKey::PersonTask(row.value()),
        w::ObjectKey::Result(row) => c::ObjectKey::Result(row.value()),
        w::ObjectKey::Note(row) => c::ObjectKey::Note { project: row.project(), note: row.note() },
    }
}

pub(crate) fn inbox_group_fields(value: &w::InboxGroup, _limits: &Limits) -> c::InboxGroup {
    match value {
        w::InboxGroup::Questions => c::InboxGroup::Questions,
        w::InboxGroup::Proposals => c::InboxGroup::Proposals,
        w::InboxGroup::Escalations => c::InboxGroup::Escalations,
        w::InboxGroup::PersonTasks => c::InboxGroup::PersonTasks,
        w::InboxGroup::Results => c::InboxGroup::Results,
        w::InboxGroup::Replies => c::InboxGroup::Replies,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn inbox_entry_fields(value: &w::InboxEntry, limits: &Limits) -> Result<c::InboxEntry, w::Problem> {
    let opt0 = match value.reply() {
        Some(item) => Some(Box::new(conversation_line_fields(item, limits)?)),
        None => None,
    };
    let opt1 = match value.goal() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt2 = match value.recipient() {
        Some(item) => Some(crate::translate::facts::recipient(item, limits)),
        None => None,
    };
    let opt3 = match value.stall() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::InboxEntry {
        reply: opt0,
        lineage: chip_piece(value.lineage(), limits, limits.client.rows)?,
        goal: opt1,
        recipient: opt2,
        stall: opt3,
        object: object_key_fields(value.object(), limits),
        project: value.project(),
        group: inbox_group_fields(value.group(), limits),
        position: value.position(),
        at: Wall::from_nanos(value.at()),
    })
}

pub(crate) fn inbox_entry_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::InboxEntry>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::InboxEntry::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = inbox_entry_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn reply_position_fields(value: &w::ReplyPosition, _limits: &Limits) -> c::ReplyPosition {
    c::ReplyPosition { chat: value.chat(), through: value.through() }
}

pub(crate) fn inbox_fields(value: &w::Inbox, limits: &Limits) -> Result<c::Inbox, w::Problem> {
    if value.replies_seen().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.replies_seen().len());
    for item in value.replies_seen() {
        list0.push(reply_position_fields(item, limits)).expect("reserved supplied list");
    }
    Ok(c::Inbox {
        entries: inbox_entry_piece(value.entries(), limits, limits.client.rows)?,
        needs: needs_fields(value.needs(), limits),
        updates: count_fields(value.updates(), limits),
        results_seen: value.results_seen(),
        replies_seen: list0,
        more: value.more(),
    })
}

pub(crate) fn owner_phase_counts_fields(value: &w::OwnerPhaseCounts, _limits: &Limits) -> c::OwnerPhaseCounts {
    c::OwnerPhaseCounts {
        waiting: value.waiting(),
        idle: value.idle(),
        starting: value.starting(),
        running: value.running(),
        backing_off: value.backing_off(),
        closing: value.closing(),
        held: value.held(),
        done: value.done(),
        failed: value.failed(),
        cancelled: value.cancelled(),
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn goal_line_fields(value: &w::GoalLine, limits: &Limits) -> Result<c::GoalLine, w::Problem> {
    let opt0 = match value.changed() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::GoalLine {
        changed: opt0,
        chip: chip_fields(value.chip(), limits)?,
        priority: value.priority(),
        progress: owner_phase_counts_fields(value.progress(), limits),
        budget: crate::translate::facts::owner_budget(value.budget(), limits),
        held_below: value.held_below(),
    })
}

pub(crate) fn goal_line_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::GoalLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::GoalLine::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = goal_line_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn goals_fields(value: &w::Goals, limits: &Limits) -> Result<c::Goals, w::Problem> {
    Ok(c::Goals {
        project: value.project(),
        rows: goal_line_piece(value.rows(), limits, limits.client.rows)?,
        more: value.more(),
    })
}

pub(crate) fn task_line_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::TaskLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::TaskLine::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = crate::translate::queries::line(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn tasks_fields(value: &w::Tasks, limits: &Limits) -> Result<c::Tasks, w::Problem> {
    Ok(c::Tasks { rows: task_line_piece(value.rows(), limits, limits.client.rows)?, more: value.more() })
}
