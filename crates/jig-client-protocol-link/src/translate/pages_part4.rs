//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, List, Reader, Wall, c, chip_fields, cursor_fields, owner_chip_ending_fields, owner_task_link_fields,
    piece_problem, result_piece, too_large, w,
};
pub(crate) fn object_page_fields(value: &w::ObjectPage, limits: &Limits) -> Result<c::ObjectPage, w::Problem> {
    let opt0 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::ObjectPage {
        cut: value.cut(),
        rows: result_piece(value.rows(), limits, limits.client.rows)?,
        before: opt0,
        after: opt1,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_ended_delegate_fields(
    value: &w::OwnerEndedDelegate,
    limits: &Limits,
) -> Result<c::OwnerEndedDelegate, w::Problem> {
    let opt0 = match value.at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::OwnerEndedDelegate {
        chip: chip_fields(value.chip(), limits)?,
        ended: owner_chip_ending_fields(value.ended(), limits)?,
        at: opt0,
    })
}

pub(crate) fn owner_ended_delegate_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::OwnerEndedDelegate>, w::Problem> {
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
        let row = w::OwnerEndedDelegate::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = owner_ended_delegate_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn owner_ended_page_fields(
    value: &w::OwnerEndedPage,
    limits: &Limits,
) -> Result<c::OwnerEndedPage, w::Problem> {
    let opt0 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::OwnerEndedPage {
        rows: owner_ended_delegate_piece(value.rows(), limits, limits.client.rows)?,
        before: opt0,
        after: opt1,
        cut: value.cut(),
    })
}

pub(crate) fn owner_inbox_object_fields(value: &w::OwnerInboxObject, limits: &Limits) -> c::OwnerInboxObject {
    match value {
        w::OwnerInboxObject::Question(row) => c::OwnerInboxObject::Question { message: row.message() },
        w::OwnerInboxObject::Proposal(row) => c::OwnerInboxObject::Proposal {
            proposer: crate::translate::answers::proposer(row.proposer(), limits),
            number: row.number(),
        },
        w::OwnerInboxObject::Escalation(row) => c::OwnerInboxObject::Escalation { revision: row.revision() },
        w::OwnerInboxObject::PersonTask => c::OwnerInboxObject::PersonTask,
        w::OwnerInboxObject::Turn(row) => c::OwnerInboxObject::Reply { message: row.message() },
        w::OwnerInboxObject::Result(row) => c::OwnerInboxObject::Result { position: row.position() },
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_inbox_cursor_fields(value: &w::OwnerInboxCursor, limits: &Limits) -> c::OwnerInboxCursor {
    match value {
        w::OwnerInboxCursor::Number(row) => c::OwnerInboxCursor::Number { number: row.number() },
        w::OwnerInboxCursor::Inbox(row) => {
            let opt0 = match row.at() {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            c::OwnerInboxCursor::Inbox {
                task: row.task(),
                object: owner_inbox_object_fields(row.object(), limits),
                at: opt0,
            }
        }
    }
}

pub(crate) fn owner_inbox_selection_fields(
    value: &w::OwnerInboxSelection,
    limits: &Limits,
) -> Result<c::OwnerInboxSelection, w::Problem> {
    if value.goals().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.goals().len());
    for item in value.goals() {
        list0.push(*item).expect("reserved supplied list");
    }
    Ok(c::OwnerInboxSelection { cursor: owner_inbox_cursor_fields(value.cursor(), limits), goals: list0 })
}

pub(crate) fn owner_inbox_selection_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::OwnerInboxSelection>, w::Problem> {
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
        let row = w::OwnerInboxSelection::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = owner_inbox_selection_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_inbox_entry_fields(
    value: &w::OwnerInboxEntry,
    limits: &Limits,
) -> Result<c::OwnerInboxEntry, w::Problem> {
    let opt0 = match value.reply() {
        Some(item) => Some(Box::new(crate::translate::queries::line(item, limits)?)),
        None => None,
    };
    let opt1 = match value.project() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt2 = match value.activity() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt3 = match value.at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt4 = match value.unread() {
        Some(item) => Some(*item),
        None => None,
    };
    Ok(c::OwnerInboxEntry {
        reply: opt0,
        task: owner_task_link_fields(value.task(), limits)?,
        project: opt1,
        object: owner_inbox_object_fields(value.object(), limits),
        activity: opt2,
        at: opt3,
        unread: opt4,
        cursor: owner_inbox_cursor_fields(value.cursor(), limits),
    })
}

pub(crate) fn owner_inbox_entry_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::OwnerInboxEntry>, w::Problem> {
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
        let row = w::OwnerInboxEntry::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = owner_inbox_entry_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn owner_inbox_page_fields(
    value: &w::OwnerInboxPage,
    limits: &Limits,
) -> Result<c::OwnerInboxPage, w::Problem> {
    let opt0 = match value.selection() {
        Some(item) => Some(owner_inbox_selection_piece(item, limits, limits.client.rows)?),
        None => None,
    };
    let opt1 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt2 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::OwnerInboxPage {
        selection: opt0,
        cut: value.cut(),
        rows: owner_inbox_entry_piece(value.rows(), limits, limits.client.rows)?,
        before: opt1,
        after: opt2,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_period_times_fields(value: &w::OwnerPeriodTimes, _limits: &Limits) -> c::OwnerPeriodTimes {
    let opt0 = match value.next() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    c::OwnerPeriodTimes { began: Wall::from_nanos(value.began()), next: opt0 }
}
