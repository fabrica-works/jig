#![expect(clippy::manual_map, reason = "step code carries optional owner facts without closures")]
//! Each original page's whole owner rows (protocol/client.md, 3).
use super::super::{activity, chips, history, inbox, live, objects, w};
use super::rows;
use alloc::boxed::Box;
use jig_domain_core::reads;
use skein_lib::List;
pub(super) fn activity(
    task: u64,
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::Path::ReadValueTag, reason: skein_codec::Reason::Tag };

    let mut rows = List::with_capacity(page.rows.len());
    for row in &page.rows {
        let (line, turn) = match row {
            reads::PageRow::Activity { line, turn } => (line, turn),
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Milestone(_)
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Conversation(_)
            | reads::PageRow::Line(_)
            | reads::PageRow::Goal(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => return Err(invalid),
        };
        rows.push(activity::page_turn(task, line, turn.as_ref(), limits)?).expect("selected rows fit");
    }
    Ok(w::ReadValue::Activity(w::ActivityPage::new(
        limits,
        w::ActivityPageParts {
            cut: page.cut,
            rows: rows::activity_pieces(
                &rows,
                limits.activity_page_rows.min(w::CEILINGS.activity_page_rows),
                w::Path::ActivityPageRows,
            )?,
            before: after,
            after: before,
        },
    )?))
}

pub(super) fn history(
    task: u64,
    subtree: bool,
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::Path::ReadValueTag, reason: skein_codec::Reason::Tag };

    let mut rows = List::with_capacity(page.rows.len());
    for row in &page.rows {
        rows.push(match row {
            reads::PageRow::History(value) if subtree || value.task == task => w::HistoryRow::Change(Box::new(
                w::HistoryRowChange::new(limits, w::HistoryRowChangeParts { value: history::line(value, limits)? })?,
            )),
            reads::PageRow::Milestone(value) if subtree || value.task == task => {
                w::HistoryRow::Milestone(w::HistoryRowMilestone::new(
                    limits,
                    w::HistoryRowMilestoneParts { value: objects::historical::milestone(value, limits)? },
                )?)
            }
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Milestone(_)
            | reads::PageRow::Activity { .. }
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Conversation(_)
            | reads::PageRow::Line(_)
            | reads::PageRow::Goal(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => return Err(invalid),
        })
        .expect("selected rows fit");
    }
    Ok(w::ReadValue::History(w::HistoryPage::new(
        limits,
        w::HistoryPageParts {
            cut: page.cut,
            rows: rows::history_pieces(
                &rows,
                limits.history_page_rows.min(w::CEILINGS.history_page_rows),
                w::Path::HistoryPageRows,
            )?,
            before,
            after,
        },
    )?))
}

pub(super) fn inbox(
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::Path::ReadValueTag, reason: skein_codec::Reason::Tag };

    let mut rows = List::with_capacity(page.rows.len());
    for row in &page.rows {
        let value = match row {
            reads::PageRow::Inbox(value) => value,
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Milestone(_)
            | reads::PageRow::Activity { .. }
            | reads::PageRow::Conversation(_)
            | reads::PageRow::Line(_)
            | reads::PageRow::Goal(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => return Err(invalid),
        };
        rows.push(inbox::entry(value, limits)?).expect("selected rows fit");
    }
    Ok(w::ReadValue::OwnerInbox(w::OwnerInboxPage::new(
        limits,
        w::OwnerInboxPageParts {
            cut: page.cut,
            rows: rows::inbox_pieces(
                &rows,
                limits.owner_inbox_page_rows.min(w::CEILINGS.owner_inbox_page_rows),
                w::Path::OwnerInboxPageRows,
            )?,
            before,
            after,
            selection: inbox::selection(
                page.inbox_selection.as_ref(),
                limits,
                limits.owner_inbox_page_selection_some,
                w::Path::OwnerInboxPageSelection,
            )?,
        },
    )?))
}

pub(super) fn conversations(
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::Path::ReadValueTag, reason: skein_codec::Reason::Tag };

    let mut rows = List::with_capacity(page.rows.len());
    for row in &page.rows {
        let value = match row {
            reads::PageRow::Conversation(value) => value,
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Milestone(_)
            | reads::PageRow::Activity { .. }
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Line(_)
            | reads::PageRow::Goal(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => return Err(invalid),
        };
        rows.push(live::conversation(value, limits)?).expect("selected rows fit");
    }
    Ok(w::ReadValue::Conversations(w::ConversationPage::new(
        limits,
        w::ConversationPageParts {
            cut: page.cut,
            rows: rows::conversation_pieces(
                &rows,
                limits.conversation_page_rows.min(w::CEILINGS.conversation_page_rows),
                w::Path::ConversationPageRows,
            )?,
            before,
            after,
        },
    )?))
}

pub(super) fn goals(
    project: u32,
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::Path::ReadValueTag, reason: skein_codec::Reason::Tag };

    let mut rows = List::with_capacity(page.rows.len());
    for row in &page.rows {
        let value = match row {
            reads::PageRow::Goal(value) => value,
            reads::PageRow::Line(_)
            | reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Milestone(_)
            | reads::PageRow::Activity { .. }
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Conversation(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => return Err(invalid),
        };
        if value.chip.details.project != project {
            return Err(invalid);
        }
        rows.push(live::goal(value, limits)?).expect("selected rows fit");
    }
    Ok(w::ReadValue::Goals(w::GoalPage::new(
        limits,
        w::GoalPageParts {
            cut: page.cut,
            rows: rows::goal_pieces(
                &rows,
                limits.goal_page_rows.min(w::CEILINGS.goal_page_rows),
                w::Path::GoalPageRows,
            )?,
            before,
            after,
        },
    )?))
}

pub(super) fn ended(
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::Path::ReadValueTag, reason: skein_codec::Reason::Tag };

    let mut rows = List::with_capacity(page.rows.len());
    for row in &page.rows {
        let value = match row {
            reads::PageRow::EndedDelegate(value) => value,
            reads::PageRow::History(_)
            | reads::PageRow::Milestone(_)
            | reads::PageRow::Activity { .. }
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Conversation(_)
            | reads::PageRow::Line(_)
            | reads::PageRow::Goal(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => return Err(invalid),
        };
        rows.push(w::OwnerEndedDelegate::new(
            limits,
            w::OwnerEndedDelegateParts {
                chip: chips::chip(&value.chip, limits)?,
                ended: chips::ending(&value.ended, limits)?,
                at: match value.at {
                    Some(value) => Some(value.as_nanos()),
                    None => None,
                },
            },
        )?)
        .expect("selected rows fit");
    }
    Ok(w::ReadValue::OwnerEnded(w::OwnerEndedPage::new(
        limits,
        w::OwnerEndedPageParts {
            cut: page.cut,
            rows: rows::ended_pieces(
                &rows,
                limits.owner_ended_page_rows.min(w::CEILINGS.owner_ended_page_rows),
                w::Path::OwnerEndedPageRows,
            )?,
            before,
            after,
        },
    )?))
}

pub(super) fn spend(
    project: u32,
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::Path::ReadValueTag, reason: skein_codec::Reason::Tag };

    if page.rows.capacity() > limits.owner_spend_page_rows.min(w::CEILINGS.owner_spend_page_rows) {
        return Err(w::Problem { path: w::Path::OwnerSpendPageRows, reason: skein_codec::Reason::Bound });
    }
    let mut rows = List::with_capacity(page.rows.capacity());
    for row in &page.rows {
        let value = match row {
            reads::PageRow::Spend(value) => value,
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Milestone(_)
            | reads::PageRow::Activity { .. }
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Conversation(_)
            | reads::PageRow::Line(_)
            | reads::PageRow::Goal(_)
            | reads::PageRow::Note(_) => return Err(invalid),
        };
        let same_project = match value.funder {
            jig_domain_tasks::Funder::Period { project: owner, .. } => owner == project,
            jig_domain_tasks::Funder::Task(_)
            | jig_domain_tasks::Funder::Pool { .. }
            | jig_domain_tasks::Funder::Recurring { .. } => false,
        };
        if !same_project {
            return Err(invalid);
        }
        rows.push(rows::funding(value, limits)?).expect("selected rows fit");
    }
    Ok(w::ReadValue::OwnerSpend(w::OwnerSpendPage::new(
        limits,
        w::OwnerSpendPageParts { cut: page.cut, rows, before, after },
    )?))
}

pub(super) fn notes(
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::Path::ReadValueTag, reason: skein_codec::Reason::Tag };

    let mut rows = List::with_capacity(page.rows.len());
    for row in &page.rows {
        let value = match row {
            reads::PageRow::Note(value) => value,
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Milestone(_)
            | reads::PageRow::Activity { .. }
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Conversation(_)
            | reads::PageRow::Line(_)
            | reads::PageRow::Goal(_)
            | reads::PageRow::Spend(_) => return Err(invalid),
        };
        rows.push(rows::note(value, limits)?).expect("selected rows fit");
    }
    Ok(w::ReadValue::Notes(w::NotePage::new(
        limits,
        w::NotePageParts {
            cut: page.cut,
            rows: rows::note_pieces(
                &rows,
                limits.note_page_rows.min(w::CEILINGS.note_page_rows),
                w::Path::NotePageRows,
            )?,
            before,
            after,
        },
    )?))
}
