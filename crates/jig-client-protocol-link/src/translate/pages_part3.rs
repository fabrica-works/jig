//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Limits, List, Reader, Wall, budget_fields, c, chip_fields, cursor_fields, goal_line_piece, offered_piece,
    piece_problem, result_fields, too_large, w,
};
pub(crate) fn goal_page_fields(value: &w::GoalPage, limits: &Limits) -> Result<c::GoalPage, w::Problem> {
    let opt0 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::GoalPage {
        cut: value.cut(),
        rows: goal_line_piece(value.rows(), limits, limits.client.rows)?,
        before: opt0,
        after: opt1,
    })
}

pub(crate) fn pool_fields(value: &w::Pool, limits: &Limits) -> c::Pool {
    c::Pool { party: value.party(), period: value.period(), budget: budget_fields(value.budget(), limits) }
}

pub(crate) fn task_spend_fields(value: &w::TaskSpend, limits: &Limits) -> Result<c::TaskSpend, w::Problem> {
    Ok(c::TaskSpend { chip: chip_fields(value.chip(), limits)?, spent: value.spent() })
}

pub(crate) fn task_spend_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::TaskSpend>, w::Problem> {
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
        let row = w::TaskSpend::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = task_spend_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn spend_period_fields(value: &w::SpendPeriod, limits: &Limits) -> Result<c::SpendPeriod, w::Problem> {
    if value.pools().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.pools().len());
    for item in value.pools() {
        list0.push(pool_fields(item, limits)).expect("reserved supplied list");
    }
    Ok(c::SpendPeriod {
        period: value.period(),
        budget: budget_fields(value.budget(), limits),
        pools: list0,
        tasks: task_spend_piece(value.tasks(), limits, limits.client.rows)?,
    })
}

pub(crate) fn spend_period_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::SpendPeriod>, w::Problem> {
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
        let row = w::SpendPeriod::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = spend_period_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn spend_page_fields(value: &w::SpendPage, limits: &Limits) -> Result<c::SpendPage, w::Problem> {
    let opt0 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::SpendPage {
        cut: value.cut(),
        rows: spend_period_piece(value.rows(), limits, limits.client.rows)?,
        before: opt0,
        after: opt1,
    })
}

pub(crate) fn note_author_fields(value: &w::NoteAuthor, _limits: &Limits) -> c::NoteAuthor {
    match value {
        w::NoteAuthor::Party(row) => c::NoteAuthor::Party { party: row.party() },
        w::NoteAuthor::Task(row) => c::NoteAuthor::Task { task: row.task(), attempt: row.attempt() },
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn note_fields(value: &w::Note, limits: &Limits) -> Result<c::Note, w::Problem> {
    if value.tasks().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.tasks().len());
    for item in value.tasks() {
        list0.push(*item).expect("reserved supplied list");
    }
    let opt1 = match value.changed() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::Note {
        number: value.number(),
        revision: crate::translate::answers::revision(value.revision(), limits),
        scope: crate::translate::answers::note_scope(value.scope(), limits),
        description: crate::translate::facts::document_text(value.description(), limits)?,
        body: crate::translate::facts::document_text(value.body(), limits)?,
        writer: note_author_fields(value.writer(), limits),
        tasks: list0,
        changed: opt1,
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
        cut: value.cut(),
    })
}

pub(crate) fn note_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::Note>, w::Problem> {
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
        let row = w::Note::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = note_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn note_page_fields(value: &w::NotePage, limits: &Limits) -> Result<c::NotePage, w::Problem> {
    let opt0 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::NotePage {
        cut: value.cut(),
        rows: note_piece(value.rows(), limits, limits.client.rows)?,
        before: opt0,
        after: opt1,
    })
}

pub(crate) fn result_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::Result>, w::Problem> {
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
        let row = w::ResultDocument::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = result_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}
