//! Whole retained read pages with original query identity and opaque continuation
//! (client/domain/contract.md, 6). The supplied page loss remains independent of row and cursor facts.
use super::{activity, cursor, history, inbox, notes, owner, rows};
use jig_client::{Limits, contract as client};
use jig_domain_core::reads;
use jig_scripted_engine as admission;
use skein_lib::List;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    Activity(activity::Invalid),
    Owner(owner::Invalid),
    Identity,
    Cursor,
    Payload,
}
fn bounds(limits: &Limits) -> admission::Bounds {
    admission::Bounds {
        text: limits.text,
        rows: limits.rows,
        document_bytes: limits.document_bytes,
        ..admission::Bounds::default()
    }
}

fn continuation(
    page: &reads::Page,
    expected: &reads::Scope,
    limits: &Limits,
) -> Result<Option<client::Cursor>, Invalid> {
    if page.before.is_some() && expected != &reads::Scope::Conversations {
        return Err(Invalid::Identity);
    }
    encode_continuation(page.next.as_ref(), expected, limits)
}
fn encode_continuation(
    value: Option<&reads::Cursor>,
    expected: &reads::Scope,
    limits: &Limits,
) -> Result<Option<client::Cursor>, Invalid> {
    value
        .map(|value| {
            if !cursor::belongs_to(value, expected) {
                return Err(Invalid::Identity);
            }
            cursor::encode(value, limits.text).map_err(|_| Invalid::Cursor)
        })
        .transpose()
}

/// # Errors
/// Refuses another query/row kind or a whole reserved envelope above bounds.
pub fn conversations_page(page: &reads::Page, limits: &Limits) -> Result<client::ReadValue, Invalid> {
    let after = continuation(page, &reads::Scope::Conversations, limits)?;
    let before = encode_continuation(page.before.as_ref(), &reads::Scope::Conversations, limits)?;
    if page.rows.capacity() > limits.rows {
        return Err(Invalid::Owner(owner::Invalid::TooLarge));
    }
    let mut result = List::with_capacity(page.rows.capacity());
    for value in &page.rows {
        let reads::PageRow::Conversation(value) = value else {
            return Err(Invalid::Identity);
        };
        result
            .push(rows::conversation(value, limits).map_err(Invalid::Owner)?)
            .map_err(|_| Invalid::Owner(owner::Invalid::TooLarge))?;
    }
    let value = client::ReadValue::Conversations(Box::new(client::ConversationPage {
        rows: result,
        before,
        after,
        cut: page.cut,
    }));
    admission::admit_read_value(&value, &bounds(limits)).map_err(|_| Invalid::Payload)?;
    Ok(value)
}

/// # Errors
/// Refuses another project/query/row kind or a whole reserved envelope above bounds.
pub fn goals_page(project: u32, page: &reads::Page, limits: &Limits) -> Result<client::ReadValue, Invalid> {
    let after = continuation(page, &reads::Scope::Goals { project }, limits)?;
    if page.rows.capacity() > limits.rows {
        return Err(Invalid::Owner(owner::Invalid::TooLarge));
    }
    let mut result = List::with_capacity(page.rows.capacity());
    for value in &page.rows {
        let reads::PageRow::Goal(value) = value else {
            return Err(Invalid::Identity);
        };
        if value.chip.details.project != project {
            return Err(Invalid::Identity);
        }
        result
            .push(rows::goal(value, limits).map_err(Invalid::Owner)?)
            .map_err(|_| Invalid::Owner(owner::Invalid::TooLarge))?;
    }
    let value =
        client::ReadValue::Goals(Box::new(client::GoalPage { rows: result, before: None, after, cut: page.cut }));
    admission::admit_read_value(&value, &bounds(limits)).map_err(|_| Invalid::Payload)?;
    Ok(value)
}

/// Preserve already selected owner Inbox rows and their original query/context.
/// # Errors
/// Refuses foreign cursors or any complete allocated receiving envelope.
pub fn inbox_page(original: &client::Read, page: &reads::Page, limits: &Limits) -> Result<client::ReadValue, Invalid> {
    let expected = inbox::scope(original, limits.text).ok_or(Invalid::Identity)?;
    let after = continuation(page, &expected, limits)?;
    if page.rows.capacity() > limits.rows {
        return Err(Invalid::Owner(owner::Invalid::TooLarge));
    }
    let mut rows = List::with_capacity(page.rows.capacity());
    for row in &page.rows {
        let reads::PageRow::Inbox(value) = row else { return Err(Invalid::Identity) };
        rows.push(inbox::entry(value, limits).map_err(Invalid::Owner)?)
            .map_err(|_| Invalid::Owner(owner::Invalid::TooLarge))?;
    }
    let value = client::ReadValue::OwnerInbox(Box::new(client::OwnerInboxPage {
        rows,
        before: None,
        after,
        cut: page.cut,
        selection: inbox::selection(page.inbox_selection.as_ref(), limits).map_err(Invalid::Owner)?,
    }));
    admission::admit_read_value(&value, &bounds(limits)).map_err(|_| Invalid::Payload)?;
    Ok(value)
}

/// Copy both source History kinds in original order for the exact task.
/// Revision and activity position remain independent. Subtree rows have already
/// been selected by the owner; their membership is never reconstructed here.
/// # Errors
/// Refuses another query/row kind or a whole reserved envelope above bounds.
pub fn history_page(
    task: u64,
    subtree: bool,
    page: &reads::Page,
    limits: &Limits,
) -> Result<client::ReadValue, Invalid> {
    let after = continuation(page, &reads::Scope::History { task, subtree }, limits)?;
    if page.rows.capacity() > limits.rows {
        return Err(Invalid::Owner(owner::Invalid::TooLarge));
    }
    let mut result = List::with_capacity(page.rows.capacity());
    for row in &page.rows {
        let copied = match row {
            reads::PageRow::History(value) if subtree || value.task == task => {
                client::HistoryRow::Change(Box::new(history::line(value, limits).map_err(Invalid::Owner)?))
            }
            reads::PageRow::Milestone(value) if subtree || value.task == task => {
                client::HistoryRow::Milestone(Box::new(history::milestone(value, limits).map_err(Invalid::Owner)?))
            }
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Milestone(_)
            | reads::PageRow::Activity(_)
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Conversation(_)
            | reads::PageRow::Goal(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => return Err(Invalid::Identity),
        };
        result.push(copied).map_err(|_| Invalid::Owner(owner::Invalid::TooLarge))?;
    }
    let value =
        client::ReadValue::History(Box::new(client::HistoryPage { rows: result, before: None, after, cut: page.cut }));
    admission::admit_read_value(&value, &bounds(limits)).map_err(|_| Invalid::Payload)?;
    Ok(value)
}

/// Copy a whole newest-first activity page for its exact original task.
/// Core supplies only the next older continuation, never a fabricated reverse cursor.
/// # Errors
/// Refuses another query or row identity, a cursor
/// above its whole bound or a complete allocated envelope that cannot fit.
pub fn activity_page(task: u64, page: &reads::Page, limits: &Limits) -> Result<client::ReadValue, Invalid> {
    let before = continuation(page, &reads::Scope::Activity { task }, limits)?;
    if page.rows.capacity() > limits.rows {
        return Err(Invalid::Owner(owner::Invalid::TooLarge));
    }
    let mut rows = List::with_capacity(page.rows.capacity());
    for row in &page.rows {
        let value = match row {
            reads::PageRow::Activity(value) => activity::line(task, value, limits).map_err(Invalid::Activity)?,
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Milestone(_)
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Conversation(_)
            | reads::PageRow::Goal(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => return Err(Invalid::Identity),
        };
        rows.push(value).map_err(|_| Invalid::Owner(owner::Invalid::TooLarge))?;
    }
    let value =
        client::ReadValue::Activity(Box::new(client::ActivityPage { rows, before, after: None, cut: page.cut }));
    admission::admit_read_value(&value, &bounds(limits)).map_err(|_| Invalid::Payload)?;
    Ok(value)
}

/// Copy every supplied scoped note and its independent page continuation/loss.
/// The owner has already selected rows; this layer never re-filters a bounded answer.
/// # Errors
/// Refuses another cursor scope, row kind, reserved allocation or whole payload.
pub fn notes_page(
    scope: &client::NoteScope,
    description: Option<&client::Text>,
    page: &reads::Page,
    limits: &Limits,
) -> Result<client::ReadValue, Invalid> {
    let expected = reads::Scope::ScopedNotes {
        scope: super::reads::note_scope(scope),
        description: description.map(|value| Box::from(value.bytes())),
    };
    let after = continuation(page, &expected, limits)?;
    if page.rows.capacity() > limits.rows {
        return Err(Invalid::Owner(owner::Invalid::TooLarge));
    }
    let mut rows = List::with_capacity(page.rows.capacity());
    for row in &page.rows {
        let value = match row {
            reads::PageRow::Note(value) => notes::note(value, limits).map_err(Invalid::Owner)?,
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Milestone(_)
            | reads::PageRow::Activity(_)
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Conversation(_)
            | reads::PageRow::Goal(_)
            | reads::PageRow::Spend(_) => return Err(Invalid::Identity),
        };
        rows.push(value).map_err(|_| Invalid::Owner(owner::Invalid::TooLarge))?;
    }
    let value = client::ReadValue::Notes(Box::new(client::NotePage { rows, before: None, after, cut: page.cut }));
    admission::admit_read_value(&value, &bounds(limits)).map_err(|_| Invalid::Payload)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_domain_tasks as tasks;
    use skein_lib::Wall;
    fn limits() -> Limits {
        Limits { text: 200, rows: 4, document_bytes: 100_000, ..crate::test::limits().jig }
    }
    #[test]
    fn inbox_pages_retain_original_query_context_cut_and_opaque_next_without_refiltering_owner_rows() {
        let source = inbox::tests::source();
        let original = client::Read::Inbox {
            group: client::InboxReadGroup::Updates,
            project: Some(0),
            kind: Some(client::InboxGroup::Results),
            goal: Some(17),
            after: None,
        };
        let expected = inbox::scope(&original, limits().text).expect("original owner query");
        let mut page = reads::Page {
            rows: List::with_capacity(8),
            before: None,
            next: Some(reads::Cursor::Store { scope: expected, after: None }),
            cut: true,
            inbox_selection: source.selection,
        };
        // The bounded answer is already selected by its owner; no row-local goal is invented.
        page.rows
            .push(reads::PageRow::Inbox(source.needs.get(0).expect("stored reference").clone()))
            .expect("owner row");
        let original_source = page.clone();
        let client::ReadValue::OwnerInbox(value) =
            inbox_page(&original, &page, &inbox::tests::limits()).expect("whole original answer")
        else {
            panic!("owner inbox page")
        };
        assert!(value.cut);
        assert_eq!((value.rows.capacity(), value.rows.len()), (8, 1));
        assert_eq!(value.before, None);
        assert_eq!(cursor::decode(value.after.as_ref().expect("original query cursor"), 200), page.next);
        assert_eq!(value.rows.get(0).expect("row").project, None);
        assert_eq!(
            value.selection.as_ref().expect("original retained contexts").get(0).expect("roots").goals.capacity(),
            12
        );
        assert_eq!(page, original_source);
        assert_eq!(
            inbox_page(&original, &page, &Limits { document_bytes: 100, ..inbox::tests::limits() }),
            Err(Invalid::Payload)
        );
        page.next = Some(reads::Cursor::Store {
            scope: reads::Scope::Inbox {
                group: reads::Group::Results,
                filter: reads::Filter { project: Some(0), goal: Some(0) },
            },
            after: None,
        });
        assert_eq!(inbox_page(&original, &page, &inbox::tests::limits()), Err(Invalid::Identity));
        page.next = Some(reads::Cursor::ConversationsStart);
        assert_eq!(inbox_page(&original, &page, &inbox::tests::limits()), Err(Invalid::Identity));
    }

    #[test]
    fn mixed_history_pages_keep_change_revision_and_milestone_position_in_source_order_with_true_cut_and_exact_cursor()
    {
        let task = u64::MAX;
        let mut source = List::with_capacity(6);
        source
            .push(reads::PageRow::Milestone(jig_domain_core::documents::MilestoneLine {
                task,
                position: 0,
                phase: jig_domain_core::documents::MilestonePhase::Held {
                    before: jig_domain_core::documents::PriorPhase::Closing(tasks::Stage::Run { attempt: u64::MAX }),
                    why: tasks::Hold::Failures(tasks::Class::Invalid),
                    source: None,
                    ending: None,
                },
                at: Some(Wall::EPOCH),
            }))
            .expect("milestone first");
        source.push(reads::PageRow::History(history::tests::source())).expect("change second");
        let mut page = reads::Page {
            before: None,
            inbox_selection: None,
            rows: source,
            next: Some(reads::Cursor::Store { scope: reads::Scope::History { task, subtree: false }, after: None }),
            cut: true,
        };
        let limits = history::tests::limits();
        let client::ReadValue::History(copied) = history_page(task, false, &page, &limits).expect("whole mixed page")
        else {
            panic!("history")
        };
        assert_eq!((copied.rows.capacity(), copied.rows.len()), (6, 2));
        assert!(copied.cut);
        assert_eq!(copied.before, None);
        assert_eq!(cursor::decode(copied.after.as_ref().expect("original continuation"), limits.text), page.next);
        assert!(
            matches!(copied.rows.get(0), Some(client::HistoryRow::Milestone(value)) if value.position == 0 && value.at == Some(Wall::EPOCH) && matches!(value.phase, client::OwnerMilestonePhase::Held { before: client::OwnerPriorPhase::Closing(client::ClosingStage::Run { attempt: u64::MAX }), why: client::HoldCategory::Failures { class: client::HoldFailure::Invalid }, source: None, ending: None }))
        );
        assert!(
            matches!(copied.rows.get(1), Some(client::HistoryRow::Change(value)) if value.revision == 0 && value.at.is_none())
        );
        assert_eq!(
            history_page(task, false, &page, &Limits { rows: 5, ..limits }),
            Err(Invalid::Owner(owner::Invalid::TooLarge))
        );
        assert_eq!(history_page(task, false, &page, &Limits { document_bytes: 0, ..limits }), Err(Invalid::Payload));
        assert_eq!(history_page(0, false, &page, &limits), Err(Invalid::Identity));
        page.next = None;
        assert_eq!(history_page(0, false, &page, &limits), Err(Invalid::Identity));
        assert!(matches!(page.rows.get(0), Some(reads::PageRow::Milestone(value)) if value.position == 0));
        assert!(
            history_page(0, true, &page, &limits).is_ok(),
            "actual owner-selected subtree rows need no invented ancestry"
        );
    }

    #[test]
    fn conversation_pages_keep_independent_owner_edges_and_refuse_foreign_prefixes_without_consuming_rows() {
        let before = reads::Cursor::ConversationsLiveBefore {
            before: Some(jig_domain_core::documents::Cursor::Number { number: u64::MAX }),
        };
        let after = reads::Cursor::Store { scope: reads::Scope::Conversations, after: None };
        let mut page = reads::Page {
            rows: List::with_capacity(3),
            before: Some(before.clone()),
            next: Some(after.clone()),
            cut: true,
            inbox_selection: None,
        };
        page.rows.push(reads::PageRow::Conversation(rows::tests::conversation_source())).expect("whole source row");
        let original = page.clone();
        let client::ReadValue::Conversations(value) = conversations_page(&page, &limits()).expect("both owner edges")
        else {
            panic!("conversation page")
        };
        assert_eq!(cursor::decode(value.before.as_ref().expect("prefix"), limits().text), Some(before));
        assert_eq!(cursor::decode(value.after.as_ref().expect("suffix"), limits().text), Some(after));
        assert_eq!((value.rows.capacity(), value.rows.len(), value.cut), (3, 1, true));
        assert_eq!(page, original);
        page.before = Some(reads::Cursor::Store { scope: reads::Scope::Goals { project: 0 }, after: None });
        assert_eq!(conversations_page(&page, &limits()), Err(Invalid::Identity));
        page.before = original.before;
        assert_eq!(conversations_page(&page, &Limits { text: 1, ..limits() }), Err(Invalid::Cursor));
        assert_eq!(conversations_page(&page, &Limits { document_bytes: 0, ..limits() }), Err(Invalid::Payload));
        page.next = Some(reads::Cursor::ConversationsStart);
        let client::ReadValue::Conversations(value) = conversations_page(&page, &limits()).expect("owner live restart")
        else {
            panic!("conversation restart")
        };
        assert_eq!(cursor::decode(value.after.as_ref().expect("restart"), limits().text), page.next);
        assert_eq!(goals_page(0, &page, &limits()), Err(Invalid::Identity));
    }

    #[test]
    fn conversation_and_goal_pages_keep_owner_row_order_exact_continuations_and_true_page_loss() {
        let mut values = List::with_capacity(3);
        for task in [u64::MAX, 0] {
            let mut row = rows::tests::conversation_source();
            row.chip.number = task;
            values.push(reads::PageRow::Conversation(row)).expect("source row");
        }
        let mut page = reads::Page {
            before: None,
            inbox_selection: None,
            rows: values,
            next: Some(reads::Cursor::Store { scope: reads::Scope::Conversations, after: None }),
            cut: true,
        };
        let client::ReadValue::Conversations(value) = conversations_page(&page, &limits()).expect("whole page") else {
            panic!("conversations")
        };
        assert!(value.cut);
        assert_eq!((value.rows.capacity(), value.rows.len()), (3, 2));
        assert_eq!(
            (value.rows.get(0).expect("first").chip.task, value.rows.get(1).expect("second").chip.task),
            (u64::MAX, 0)
        );
        assert_eq!(value.before, None);
        assert_eq!(cursor::decode(value.after.as_ref().expect("continuation"), limits().text), page.next);
        assert_eq!(
            conversations_page(&page, &Limits { rows: 2, ..limits() }),
            Err(Invalid::Owner(owner::Invalid::TooLarge))
        );
        assert_eq!(conversations_page(&page, &Limits { document_bytes: 0, ..limits() }), Err(Invalid::Payload));
        page.next = Some(reads::Cursor::Store { scope: reads::Scope::Goals { project: u32::MAX }, after: None });
        assert_eq!(conversations_page(&page, &limits()), Err(Invalid::Identity));
        let mut goal_rows = List::with_capacity(3);
        goal_rows.push(reads::PageRow::Goal(rows::tests::goal_source())).expect("source goal");
        page.rows = goal_rows;
        let client::ReadValue::Goals(value) = goals_page(u32::MAX, &page, &limits()).expect("whole goal page") else {
            panic!("goals")
        };
        assert!(value.cut);
        assert_eq!((value.rows.capacity(), value.rows.len()), (3, 1));
        assert_eq!(value.rows.get(0).expect("goal").progress.cancelled, u32::MAX);
        assert_eq!(value.rows.get(0).expect("goal").changed, None);
        assert_eq!(cursor::decode(value.after.as_ref().expect("continuation"), limits().text), page.next);
        assert_eq!(goals_page(0, &page, &limits()), Err(Invalid::Identity));
        page.next = None;
        assert_eq!(goals_page(0, &page, &limits()), Err(Invalid::Identity), "row project is independently checked");
        assert_eq!(goals_page(u32::MAX, &page, &Limits { document_bytes: 0, ..limits() }), Err(Invalid::Payload));
        assert_eq!(conversations_page(&page, &limits()), Err(Invalid::Identity), "wrong row kind");
    }
    fn source() -> reads::Page {
        let mut rows = List::with_capacity(3);
        for position in [u64::MAX, 0] {
            rows.push(reads::PageRow::Activity(tasks::ActivityLine {
                task: u64::MAX,
                position,
                at: Wall::EPOCH,
                line: tasks::Line::Turn { reply: u64::MAX, attempt: u64::MAX, turn: u32::MAX, read: Some(0) },
            }))
            .expect("source row");
        }
        reads::Page {
            before: None,
            inbox_selection: None,
            rows,
            next: Some(reads::Cursor::Store { scope: reads::Scope::Activity { task: u64::MAX }, after: None }),
            cut: false,
        }
    }
    #[test]
    fn whole_activity_page_keeps_allocated_rows_exact_archive_references_and_opaque_older_cursor() {
        let page = source();
        let client::ReadValue::Activity(value) = activity_page(u64::MAX, &page, &limits()).expect("whole page") else {
            panic!("activity page");
        };
        assert_eq!(value.rows.capacity(), 3);
        assert_eq!(value.rows.len(), 2);
        assert_eq!(
            (value.rows.get(0).expect("first").position, value.rows.get(1).expect("second").position),
            (u64::MAX, 0)
        );
        assert!(value.after.is_none());
        assert_eq!(cursor::decode(value.before.as_ref().expect("older"), limits().text), page.next);
        for row in &value.rows {
            let client::Activity::Owner { source, decoded } = &row.event else {
                panic!("owner row");
            };
            assert!(decoded.is_none());
            assert_eq!(source.task, u64::MAX);
            assert_eq!(
                source.line,
                client::OwnerActivity::Turn { reply: u64::MAX, attempt: u64::MAX, turn: u32::MAX, read: Some(0) }
            );
        }
        let mut ended = page.clone();
        ended.next = None;
        let client::ReadValue::Activity(ended) = activity_page(u64::MAX, &ended, &limits()).expect("end") else {
            panic!("page");
        };
        assert!(ended.before.is_none());
    }
    #[test]
    fn activity_page_refuses_another_task_or_cursor_and_preserves_true_page_loss_independently() {
        let mut page = source();
        let original = page.clone();
        assert_eq!(activity_page(0, &page, &limits()), Err(Invalid::Identity));
        page.next = Some(reads::Cursor::Store { scope: reads::Scope::Activity { task: 0 }, after: None });
        assert_eq!(activity_page(u64::MAX, &page, &limits()), Err(Invalid::Identity));
        page = original.clone();
        page.cut = true;
        let client::ReadValue::Activity(copied) =
            activity_page(u64::MAX, &page, &limits()).expect("actual true source loss")
        else {
            panic!("whole activity page");
        };
        assert!(copied.cut);
        assert_eq!(copied.rows.capacity(), original.rows.capacity());
        assert_eq!(
            cursor::decode(copied.before.as_ref().expect("independent continuation"), limits().text),
            original.next
        );
        assert_eq!(copied.after, None);
        assert!(page.cut);
        assert_eq!(page.rows, original.rows);
        page = original.clone();
        if let reads::PageRow::Activity(value) = page.rows.get_mut(0).expect("source row") {
            value.task = 0;
        }
        assert_eq!(activity_page(u64::MAX, &page, &limits()), Err(Invalid::Activity(activity::Invalid::Identity)));
        assert_eq!(original.rows.capacity(), 3);
    }
    #[test]
    fn activity_page_whole_admission_refuses_reserved_capacity_cursor_bytes_and_complete_envelope() {
        let page = source();
        let original = page.clone();
        let mut small = limits();
        small.rows = 2;
        assert_eq!(activity_page(u64::MAX, &page, &small), Err(Invalid::Owner(owner::Invalid::TooLarge)));
        small = limits();
        small.text = 1;
        assert_eq!(activity_page(u64::MAX, &page, &small), Err(Invalid::Cursor));
        small = limits();
        let mut refused = 0;
        let mut admitted = small.document_bytes;
        while admitted - refused > 1 {
            small.document_bytes = refused + (admitted - refused) / 2;
            if activity_page(u64::MAX, &page, &small).is_ok() {
                admitted = small.document_bytes;
            } else {
                refused = small.document_bytes;
            }
        }
        small.document_bytes = admitted;
        assert!(activity_page(u64::MAX, &page, &small).is_ok());
        small.document_bytes = refused;
        for row in &page.rows {
            let reads::PageRow::Activity(row) = row else {
                panic!("source activity");
            };
            assert!(activity::line(u64::MAX, row, &small).is_ok(), "individual rows still fit");
        }
        assert_eq!(activity_page(u64::MAX, &page, &small), Err(Invalid::Payload));
        assert_eq!(page, original);
    }
    fn note_source() -> reads::Page {
        let mut rows = List::with_capacity(3);
        rows.push(reads::PageRow::Note(jig_domain_core::documents::Note {
            acts: List::with_capacity(2),
            number: u64::MAX,
            scope: jig_domain_core::documents::NoteLocation::Goal { project: u32::MAX, task: u64::MAX },
            description: jig_domain_core::documents::Text { bytes: Box::new([]), cut: false },
            body: jig_domain_core::documents::Text { bytes: Box::from([255, 0]), cut: true },
            writer: jig_domain_notes::Author::Task { task: u64::MAX, attempt: u64::MAX },
            tasks: List::with_capacity(4),
            changed: None,
            revision: u32::MAX,
            cut: true,
        }))
        .expect("whole source note");
        reads::Page {
            before: None,
            inbox_selection: None,
            rows,
            next: Some(reads::Cursor::Store {
                scope: reads::Scope::ScopedNotes {
                    scope: jig_domain_notes::Scope::Goal { project: u32::MAX, goal: u64::MAX },
                    description: Some(Box::new([])),
                },
                after: None,
            }),
            cut: true,
        }
    }
    #[test]
    fn a_whole_scoped_note_page_keeps_empty_search_exact_owner_fields_and_independent_cursor_and_loss() {
        let source = note_source();
        let scope = client::NoteScope::Goal { project: u32::MAX, goal: u64::MAX };
        let empty = client::Text::new(Box::new([]), limits().text).expect("explicit empty search");
        let client::ReadValue::Notes(page) =
            notes_page(&scope, Some(&empty), &source, &limits()).expect("whole note page")
        else {
            panic!("Notes page")
        };
        assert_eq!(page.rows.capacity(), 3);
        assert_eq!(page.rows.len(), 1);
        assert!(page.cut);
        assert_eq!(page.before, None);
        assert_eq!(cursor::decode(page.after.as_ref().expect("actual continuation"), limits().text), source.next);
        let row = page.rows.get(0).expect("actual supplied note");
        assert_eq!(row.scope, scope);
        assert_eq!(row.number, u64::MAX);
        assert_eq!(row.revision.value, u64::from(u32::MAX));
        assert_eq!(row.body.bytes.bytes(), &[255, 0]);
        assert!(row.body.cut && row.cut);
        assert_eq!(row.changed, None);
        assert_eq!(row.writer, client::NoteAuthor::Task { task: u64::MAX, attempt: u64::MAX });
        assert_eq!(row.tasks.capacity(), 4);
        assert_eq!(row.acts.capacity(), 2);
        assert_eq!(notes_page(&scope, None, &source, &limits()), Err(Invalid::Identity));
        assert_eq!(
            notes_page(&client::NoteScope::Goal { project: 0, goal: u64::MAX }, Some(&empty), &source, &limits()),
            Err(Invalid::Identity)
        );
    }
    #[test]
    fn scoped_note_pages_refuse_wrong_rows_reserved_capacity_and_whole_envelope_without_altering_owner_data() {
        let source = note_source();
        let before = source.clone();
        let scope = client::NoteScope::Goal { project: u32::MAX, goal: u64::MAX };
        let empty = client::Text::new(Box::new([]), limits().text).expect("empty search");
        assert_eq!(
            notes_page(&scope, Some(&empty), &source, &Limits { rows: 2, ..limits() }),
            Err(Invalid::Owner(owner::Invalid::TooLarge))
        );
        assert_eq!(
            notes_page(&scope, Some(&empty), &source, &Limits { document_bytes: 0, ..limits() }),
            Err(Invalid::Payload)
        );
        let mut wrong = source.clone();
        *wrong.rows.get_mut(0).expect("whole row") = reads::PageRow::Activity(tasks::ActivityLine {
            task: u64::MAX,
            position: 0,
            at: Wall::EPOCH,
            line: tasks::Line::Turn { reply: 0, attempt: 0, turn: 0, read: None },
        });
        assert_eq!(notes_page(&scope, Some(&empty), &wrong, &limits()), Err(Invalid::Identity));
        assert_eq!(source, before);
    }
}
