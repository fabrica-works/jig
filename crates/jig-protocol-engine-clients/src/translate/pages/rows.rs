#![expect(clippy::manual_map, reason = "step code carries optional owner facts without closures")]
//! Bounded owner page pieces (protocol/client.md, 3).
use super::super::{owner, pieces, task, w};
use alloc::boxed::Box;
use jig_domain_core::documents as d;
use jig_domain_tasks as t;
use skein_lib::{List, Writer};

fn charge(size: u32, measure: u32, bound: u32, path: w::Path) -> Result<u32, w::Problem> {
    let invalid = w::Problem { path, reason: skein_codec::Reason::Bound };
    let size = size.checked_add(4).ok_or(invalid)?.checked_add(measure).ok_or(invalid)?;
    if size > bound {
        return Err(invalid);
    }
    Ok(size)
}
pub(super) fn activity_pieces(
    rows: &List<w::ActivityLine>,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut size = 0;
    for row in rows {
        size = charge(size, row.measure(), bound, path)?;
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        row.encode(&mut out).or(Err(w::Problem { path, reason: skein_codec::Reason::Bound }))?;
        encoded.push(out.finish()).expect("selected rows fit");
    }
    pieces(&encoded, bound, path)
}
pub(super) fn history_pieces(rows: &List<w::HistoryRow>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut size = 0;
    for row in rows {
        size = charge(size, row.measure(), bound, path)?;
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        row.encode(&mut out).or(Err(w::Problem { path, reason: skein_codec::Reason::Bound }))?;
        encoded.push(out.finish()).expect("selected rows fit");
    }
    pieces(&encoded, bound, path)
}
pub(super) fn inbox_pieces(
    rows: &List<w::OwnerInboxEntry>,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut size = 0;
    for row in rows {
        size = charge(size, row.measure(), bound, path)?;
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        row.encode(&mut out).or(Err(w::Problem { path, reason: skein_codec::Reason::Bound }))?;
        encoded.push(out.finish()).expect("selected rows fit");
    }
    pieces(&encoded, bound, path)
}
pub(super) fn conversation_pieces(
    rows: &List<w::ConversationLine>,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut size = 0;
    for row in rows {
        size = charge(size, row.measure(), bound, path)?;
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        row.encode(&mut out).or(Err(w::Problem { path, reason: skein_codec::Reason::Bound }))?;
        encoded.push(out.finish()).expect("selected rows fit");
    }
    pieces(&encoded, bound, path)
}
pub(super) fn goal_pieces(rows: &List<w::GoalLine>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut size = 0;
    for row in rows {
        size = charge(size, row.measure(), bound, path)?;
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        row.encode(&mut out).or(Err(w::Problem { path, reason: skein_codec::Reason::Bound }))?;
        encoded.push(out.finish()).expect("selected rows fit");
    }
    pieces(&encoded, bound, path)
}
pub(super) fn ended_pieces(
    rows: &List<w::OwnerEndedDelegate>,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut size = 0;
    for row in rows {
        size = charge(size, row.measure(), bound, path)?;
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        row.encode(&mut out).or(Err(w::Problem { path, reason: skein_codec::Reason::Bound }))?;
        encoded.push(out.finish()).expect("selected rows fit");
    }
    pieces(&encoded, bound, path)
}
pub(super) fn note_pieces(rows: &List<w::Note>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut size = 0;
    for row in rows {
        size = charge(size, row.measure(), bound, path)?;
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        row.encode(&mut out).or(Err(w::Problem { path, reason: skein_codec::Reason::Bound }))?;
        encoded.push(out.finish()).expect("selected rows fit");
    }
    pieces(&encoded, bound, path)
}
pub(super) fn funding(value: &t::FundingRecord, limits: &w::Limits) -> Result<w::OwnerFundingRecord, w::Problem> {
    w::OwnerFundingRecord::new(
        limits,
        w::OwnerFundingRecordParts {
            period: match value.period {
                Some(value) => Some(w::OwnerPeriodTimes::new(
                    limits,
                    w::OwnerPeriodTimesParts {
                        began: value.began.as_nanos(),
                        next: match value.next {
                            Some(value) => Some(value.as_nanos()),
                            None => None,
                        },
                    },
                )?),
                None => None,
            },
            funder: owner::funder(value.funder, limits)?,
            parent: match value.parent {
                Some(value) => Some(owner::funder(value, limits)?),
                None => None,
            },
            numbers: owner::numbers(value.numbers, limits)?,
            made: value.made,
            closed: value.closed,
        },
    )
}
pub(super) fn note(value: &d::Note, limits: &w::Limits) -> Result<w::Note, w::Problem> {
    let scope = match &value.scope {
        d::NoteLocation::Deployment => w::NoteScope::Deployment,
        d::NoteLocation::Project(value) => {
            w::NoteScope::Project(w::NoteScopeProject::new(limits, w::NoteScopeProjectParts { value: *value })?)
        }
        d::NoteLocation::Goal { project, task } => {
            w::NoteScope::Goal(w::NoteScopeGoal::new(limits, w::NoteScopeGoalParts { project: *project, goal: *task })?)
        }
        d::NoteLocation::Resources { project, scope } => w::NoteScope::Resource(w::NoteScopeResource::new(
            limits,
            w::NoteScopeResourceParts {
                project: *project,
                connector: scope.connector,
                pattern: owner::pattern(&scope.segments, &scope.last, limits)?,
            },
        )?),
    };
    let writer = match value.writer {
        jig_domain_notes::Author::Party { party } => {
            w::NoteAuthor::Party(w::NoteAuthorParty::new(limits, w::NoteAuthorPartyParts { party })?)
        }
        jig_domain_notes::Author::Task { task, attempt } => {
            w::NoteAuthor::Task(w::NoteAuthorTask::new(limits, w::NoteAuthorTaskParts { task, attempt })?)
        }
    };
    if value.tasks.capacity() > limits.note_tasks.min(w::CEILINGS.note_tasks) {
        return Err(w::Problem { path: w::Path::NoteTasks, reason: skein_codec::Reason::Bound });
    }
    let mut tasks = List::with_capacity(value.tasks.capacity());
    for value in &value.tasks {
        tasks.push(*value).expect("selected references fit");
    }
    w::Note::new(
        limits,
        w::NoteParts {
            number: value.number,
            revision: w::Revision::new(limits, w::RevisionParts { value: u64::from(value.revision) })?,
            scope,
            description: owner::text(&value.description, limits)?,
            body: owner::text(&value.body, limits)?,
            writer,
            tasks,
            changed: match value.changed {
                Some(value) => Some(value.as_nanos()),
                None => None,
            },
            acts: task::offered(&value.acts, limits, limits.note_acts, w::Path::NoteActs)?,
            cut: value.cut,
        },
    )
}
