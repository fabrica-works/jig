//! A page's joined transcript identity belongs to its immutable line (store.md, 7.3).
use super::{PageRow, PageTurn, Row, UnjoinedTurn};
use crate::{CoreRecord, Record, documents::Missing};
use jig_domain_tasks as tasks;

pub(crate) fn row(line: &tasks::ActivityLine, row: &Row) -> Result<PageRow, Missing> {
    let mut supplied = None;
    if let tasks::Line::Turn { attempt, turn, .. } = line.line {
        supplied = Some(match &row.related {
            Some(Record::Core(CoreRecord::Turn(value))) => {
                if (value.task, value.attempt, value.turn) != (line.task, attempt, turn) || row.unjoined.is_some() {
                    return Err(Missing::Unknown);
                }
                PageTurn::Carried(value.clone())
            }
            Some(_) => return Err(Missing::Unknown),
            None => match &row.unjoined {
                Some(UnjoinedTurn::TooLarge { task, attempt: owner, turn: number, spent, read, at, bytes }) => {
                    if (*task, *owner, *number) != (line.task, attempt, turn) {
                        return Err(Missing::Unknown);
                    }
                    PageTurn::TooLarge { spent: *spent, read: *read, at: *at, bytes: *bytes }
                }
                Some(UnjoinedTurn::Unread { task, attempt: owner, turn: number }) => {
                    if (*task, *owner, *number) != (line.task, attempt, turn) {
                        return Err(Missing::Unknown);
                    }
                    PageTurn::Unread
                }
                None => PageTurn::Unread,
            },
        });
    }
    Ok(PageRow::Activity { line: line.clone(), turn: supplied })
}
