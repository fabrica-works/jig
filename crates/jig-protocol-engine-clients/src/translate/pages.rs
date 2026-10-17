//! Whole selected page rows and their owner continuations (protocol/client.md, 3 and 6).
use super::{cursor, w};
use alloc::boxed::Box;
use jig_domain_core::reads;
mod documents;
mod rows;

pub fn page(read: &reads::Read, page: &reads::Page, limits: &w::Limits) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::Path::ReadValueTag, reason: skein_codec::Reason::Tag };
    check_edges(read, page)?;
    let before = match &page.before {
        Some(value) => Some(wire_cursor(value, limits)?),
        None => None,
    };
    let after = match &page.next {
        Some(value) => Some(wire_cursor(value, limits)?),
        None => None,
    };
    Ok(match read {
        reads::Read::Activity { task, .. } => documents::activity(*task, page, before, after, limits)?,
        reads::Read::History { task, subtree, .. } => documents::history(*task, *subtree, page, before, after, limits)?,
        reads::Read::Inbox { .. } => documents::inbox(page, before, after, limits)?,
        reads::Read::Conversations { .. } => documents::conversations(page, before, after, limits)?,
        reads::Read::Goals { project, .. } => documents::goals(*project, page, before, after, limits)?,
        reads::Read::EndedDelegates { .. } => documents::ended(page, before, after, limits)?,
        reads::Read::Spend { project, .. } => documents::spend(*project, page, before, after, limits)?,
        reads::Read::Notes { .. } | reads::Read::ScopedNotes { .. } => documents::notes(page, before, after, limits)?,
        reads::Read::Tasks { .. }
        | reads::Read::Application { .. }
        | reads::Read::Frame
        | reads::Read::Task { .. }
        | reads::Read::Proposal { .. }
        | reads::Read::Escalation { .. }
        | reads::Read::Question { .. }
        | reads::Read::PersonTask { .. }
        | reads::Read::Result { .. }
        | reads::Read::Reply { .. }
        | reads::Read::Turn { .. }
        | reads::Read::Project { .. }
        | reads::Read::Deployment => return Err(invalid),
    })
}

fn check_edges(read: &reads::Read, page: &reads::Page) -> Result<(), w::Problem> {
    let invalid = w::Problem { path: w::Path::CursorValue, reason: skein_codec::Reason::Tag };
    for cursor in [&page.before, &page.next] {
        let Some(cursor) = cursor else { continue };
        let matches = match cursor {
            reads::Cursor::Live { scope, .. } | reads::Cursor::Store { scope, .. } => {
                let expected = match read {
                    reads::Read::Activity { task, .. } => reads::Scope::Activity { task: *task },
                    reads::Read::History { task, subtree, .. } => {
                        reads::Scope::History { task: *task, subtree: *subtree }
                    }
                    reads::Read::EndedDelegates { task, .. } => reads::Scope::EndedDelegates { task: *task },
                    reads::Read::Inbox { group, filter, .. } => reads::Scope::Inbox { group: *group, filter: *filter },
                    reads::Read::Conversations { .. } => reads::Scope::Conversations,
                    reads::Read::Tasks { query, .. } => reads::Scope::Tasks { query: *query },
                    reads::Read::Goals { project, .. } => reads::Scope::Goals { project: *project },
                    reads::Read::Spend { project, .. } => reads::Scope::Spend { project: *project },
                    reads::Read::Notes { project, .. } => reads::Scope::Notes { project: *project },
                    reads::Read::ScopedNotes { scope, description, .. } => {
                        reads::Scope::ScopedNotes { scope: scope.clone(), description: description.clone() }
                    }
                    reads::Read::Application { .. }
                    | reads::Read::Frame
                    | reads::Read::Task { .. }
                    | reads::Read::Proposal { .. }
                    | reads::Read::Escalation { .. }
                    | reads::Read::Question { .. }
                    | reads::Read::PersonTask { .. }
                    | reads::Read::Result { .. }
                    | reads::Read::Reply { .. }
                    | reads::Read::Turn { .. }
                    | reads::Read::Project { .. }
                    | reads::Read::Deployment => return Err(invalid),
                };
                *scope == expected
            }
            reads::Cursor::ConversationsBefore { .. }
            | reads::Cursor::ConversationsLiveBefore { .. }
            | reads::Cursor::ConversationsStart => match read {
                reads::Read::Conversations { .. } => true,
                reads::Read::Application { .. }
                | reads::Read::Frame
                | reads::Read::Task { .. }
                | reads::Read::Proposal { .. }
                | reads::Read::Escalation { .. }
                | reads::Read::Question { .. }
                | reads::Read::PersonTask { .. }
                | reads::Read::Result { .. }
                | reads::Read::Reply { .. }
                | reads::Read::Turn { .. }
                | reads::Read::EndedDelegates { .. }
                | reads::Read::History { .. }
                | reads::Read::Activity { .. }
                | reads::Read::Inbox { .. }
                | reads::Read::Tasks { .. }
                | reads::Read::Goals { .. }
                | reads::Read::Spend { .. }
                | reads::Read::Project { .. }
                | reads::Read::Notes { .. }
                | reads::Read::ScopedNotes { .. }
                | reads::Read::Deployment => false,
            },
        };
        if !matches {
            return Err(invalid);
        }
    }
    Ok(())
}

fn wire_cursor(value: &reads::Cursor, limits: &w::Limits) -> Result<w::Cursor, w::Problem> {
    let bytes = cursor::encode(value, limits.cursor_value.min(w::CEILINGS.cursor_value))
        .ok_or(w::Problem { path: w::Path::CursorValue, reason: skein_codec::Reason::Bound })?;
    w::Cursor::new(limits, w::CursorParts { value: bytes })
}

pub(super) fn tasks(page: &reads::TaskPage, limits: &w::Limits) -> Result<w::ReadValue, w::Problem> {
    let ended = match &page.ended {
        reads::EndedGroup::None => w::EndedGroup::None,
        reads::EndedGroup::Rows { rows, next, cut } => w::EndedGroup::Rows(w::EndedRows::new(
            limits,
            w::EndedRowsParts {
                rows: task_pieces(rows, limits, limits.ended_rows_rows, w::Path::EndedRowsRows)?,
                after: match next {
                    Some(v) => Some(wire_cursor(v, limits)?),
                    None => None,
                },
                cut: *cut,
            },
        )?),
    };
    Ok(w::ReadValue::Tasks(w::TaskPage::new(
        limits,
        w::TaskPageParts {
            live: task_pieces(&page.live, limits, limits.task_page_live, w::Path::TaskPageLive)?,
            live_after: match &page.live_next {
                Some(v) => Some(wire_cursor(v, limits)?),
                None => None,
            },
            ended,
        },
    )?))
}
fn task_pieces(
    lines: &skein_lib::List<jig_domain_core::documents::TaskLine>,
    limits: &w::Limits,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut pieces = skein_lib::List::with_capacity(lines.len());
    for line in lines {
        let line = super::queries::line(line, limits)?;
        let mut writer = skein_lib::Writer::new(usize::try_from(line.measure()).expect("measure fits"));
        line.encode(&mut writer).expect("measured line");
        pieces.push(writer.finish()).expect("same line count");
    }
    super::pieces(&pieces, bound, path)
}
