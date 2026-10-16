//! Read-owner continuations carried from bounded live documents
//! (`client/domain/contract.md`, section 6; `client/domain/client.md`, section 7).
//! These values name the next live slice or the stored range after it. They do
//! not say whether the store has rows, and using them is an authenticated read.

use super::{Cursor, Scope};
use crate::documents;

/// Continue conversations after this live window, entering stored conversations
/// when the window has no remaining live rows. The stored start can be replayed
/// to recover past rows displaced by a later live delivery.
#[must_use]
pub fn conversations_cursor(value: &documents::Conversations) -> Cursor {
    match value.next {
        Some(after) => Cursor::Live { scope: Scope::Conversations, after, store_after: None },
        None => Cursor::Store { scope: Scope::Conversations, after: None },
    }
}

use super::{Answer, Page, PageRow, Position, Range, Read, Row, Why};
use crate::{Core, Limits};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue};

#[expect(clippy::manual_map, reason = "the strict subset uses an exhaustive match rather than a closure")]
pub(super) fn live_prefix(read: &Read, value: &documents::Conversations) -> Option<Cursor> {
    match super::cursor(read) {
        Some(Cursor::Live { after: documents::Cursor::Number { number }, .. }) if *number != 0 => {
            Some(Cursor::ConversationsLiveBefore {
                before: match value.lines.get(0) {
                    Some(row) => Some(documents::Cursor::Number { number: row.chip.number }),
                    None => None,
                },
            })
        }
        Some(
            Cursor::Live { .. }
            | Cursor::Store { .. }
            | Cursor::ConversationsBefore { .. }
            | Cursor::ConversationsLiveBefore { .. }
            | Cursor::ConversationsStart,
        )
        | None => None,
    }
}

fn live_tail(core: &Core, limits: &Limits, party: u64) -> Option<Cursor> {
    match documents::make(
        core,
        limits,
        party,
        &documents::Subject::Conversations { party },
        &documents::Window { rows: 1, after: None },
    ) {
        Ok(documents::Document::Conversations(value)) => {
            if value.lines.is_empty() {
                None
            } else {
                Some(Cursor::ConversationsLiveBefore { before: None })
            }
        }
        Ok(
            documents::Document::Frame(_)
            | documents::Document::Chips(_)
            | documents::Document::Run(_)
            | documents::Document::Task(_)
            | documents::Document::Tree(_)
            | documents::Document::Proposal(_)
            | documents::Document::Escalation(_)
            | documents::Document::Question(_)
            | documents::Document::PersonTask(_)
            | documents::Document::Result(_)
            | documents::Document::Project(_)
            | documents::Document::Lines(_)
            | documents::Document::Tasks(_)
            | documents::Document::Goals(_)
            | documents::Document::Note(_)
            | documents::Document::Deployment(_)
            | documents::Document::Inbox(_),
        ) => unreachable!("conversation subject"),
        Err(documents::Missing::TooLarge) => Some(Cursor::ConversationsLiveBefore { before: None }),
        Err(documents::Missing::Unknown | documents::Missing::NotAllowed | documents::Missing::Ended) => None,
    }
}

#[expect(clippy::manual_map, reason = "the strict subset uses an exhaustive match rather than a closure")]
pub(super) fn stored_edges(
    core: &Core,
    limits: &Limits,
    range: &Range,
    raw: &[Row],
    next: Option<Position>,
) -> (Option<Cursor>, Option<Cursor>) {
    if range.scope != Scope::Conversations {
        let next = match next {
            Some(after) => Some(Cursor::Store { scope: range.scope.clone(), after: Some(after) }),
            None => None,
        };
        return (None, next);
    }
    let first = match raw.first() {
        Some(row) => super::row_key(row),
        None => None,
    };
    if !range.backwards {
        let before = match next {
            Some(before) => Some(Cursor::ConversationsBefore {
                before: super::indexed::conversation_position(&before, false).expect("conversation position"),
            }),
            None => live_tail(core, limits, range.party),
        };
        let first = match first {
            Some(first) => super::indexed::conversation_position(&first, true),
            None => None,
        };
        return (before, Some(Cursor::Store { scope: Scope::Conversations, after: first }));
    }
    let before = match &range.after {
        Some(_) => match first {
            Some(before) => Some(Cursor::ConversationsBefore {
                before: super::indexed::conversation_position(&before, false).expect("conversation position"),
            }),
            None => None,
        },
        None => live_tail(core, limits, range.party),
    };
    let next = match next {
        Some(after) => Some(Cursor::Store { scope: Scope::Conversations, after: Some(after) }),
        None => None,
    };
    (before, next)
}

pub(super) fn live_before(core: &Core, env: &Env<Limits>, party: u64, before: Option<&documents::Cursor>) -> Answer {
    let before = match before {
        Some(documents::Cursor::Number { number }) => Some(*number),
        Some(documents::Cursor::Inbox { .. }) => return Answer::None(Why::Unknown),
        None => None,
    };
    let mut tail = Queue::with_capacity(env.limits.documents.rows);
    let mut after = None;
    let mut discarded = false;
    for _ in 0..env.limits.tasks.tasks {
        let document = match documents::make_with_clock(
            core,
            &env.limits,
            party,
            &documents::Subject::Conversations { party },
            &documents::Window { rows: env.limits.documents.rows, after },
            documents::DocumentClock { now: env.now, wall: env.wall },
        ) {
            Ok(document) => document,
            Err(documents::Missing::TooLarge) => return Answer::None(Why::Busy),
            Err(documents::Missing::Unknown | documents::Missing::NotAllowed | documents::Missing::Ended) => {
                return Answer::None(Why::Unknown);
            }
        };
        let value = match document {
            documents::Document::Conversations(value) => value,
            documents::Document::Frame(_)
            | documents::Document::Chips(_)
            | documents::Document::Run(_)
            | documents::Document::Task(_)
            | documents::Document::Tree(_)
            | documents::Document::Proposal(_)
            | documents::Document::Escalation(_)
            | documents::Document::Question(_)
            | documents::Document::PersonTask(_)
            | documents::Document::Result(_)
            | documents::Document::Project(_)
            | documents::Document::Lines(_)
            | documents::Document::Tasks(_)
            | documents::Document::Goals(_)
            | documents::Document::Note(_)
            | documents::Document::Deployment(_)
            | documents::Document::Inbox(_) => unreachable!("conversation subject"),
        };
        after = value.next;
        let mut reached = false;
        for row in value.lines.into_boxed() {
            if match before {
                Some(number) => row.chip.number >= number,
                None => false,
            } {
                reached = true;
                break;
            }
            if tail.room() == 0 {
                tail.pop().expect("nonempty bounded tail");
                discarded = true;
            }
            tail.push(PageRow::Conversation(row));
        }
        if reached || after.is_none() {
            break;
        }
    }
    let mut rows = List::with_capacity(env.limits.documents.rows);
    for _ in 0..env.limits.documents.rows {
        let Some(row) = tail.pop() else { break };
        rows.push(row).expect("bounded live tail");
    }
    let mut first = None;
    let mut last = None;
    for row in &rows {
        match row {
            PageRow::Conversation(value) => {
                if first.is_none() {
                    first = Some(documents::Cursor::Number { number: value.chip.number });
                }
                last = Some(documents::Cursor::Number { number: value.chip.number });
            }
            PageRow::EndedDelegate(_)
            | PageRow::History(_)
            | PageRow::Milestone(_)
            | PageRow::Activity { .. }
            | PageRow::Inbox(_)
            | PageRow::Line(_)
            | PageRow::Goal(_)
            | PageRow::Note(_)
            | PageRow::Spend(_) => unreachable!("live tail row"),
        }
    }
    let next = match before {
        Some(_) => match last {
            Some(after) => Cursor::Live { scope: Scope::Conversations, after, store_after: None },
            None => Cursor::ConversationsStart,
        },
        None => Cursor::Store { scope: Scope::Conversations, after: None },
    };
    let before = if discarded { Some(Cursor::ConversationsLiveBefore { before: first }) } else { None };
    Answer::Page(Box::new(Page { rows, before, next: Some(next), cut: false, inbox_selection: None }))
}
